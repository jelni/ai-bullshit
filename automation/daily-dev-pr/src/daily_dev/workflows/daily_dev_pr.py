from datetime import timedelta

from pydantic import BaseModel, Field

import mistralai.workflows as workflows
from mistralai.workflows import workflow
from mistralai.workflows.plugins.mistralai import Agent, Runner, RemoteSession, TextChunk

from daily_dev.activities import (
    delete_file,
    finish_development,
    list_files,
    push_and_open_pr,
    read_file,
    repo_overview,
    run_command,
    setup_workspace,
    verify_changes,
    write_file,
)
from daily_dev.prompts import (
    SYSTEM_PROMPT,
    build_commit_message,
    build_pr_body,
    build_pr_title,
    build_task_prompt,
)


class DailyDevPrInput(BaseModel):
    repo: str = Field(default="jelni/ai-bullshit", description='GitHub repository in "owner/name" form')
    base_branch: str = Field(default="main", description="Branch the pull request targets")
    model: str = Field(default="mistral-large-4", description="Mistral model driving the development agent")
    max_turns: int = Field(default=40, ge=5, le=100, description="Maximum agent turns per run")


class DailyDevPrResult(BaseModel):
    status: str = Field(description='"pr_opened", "no_changes" or "verification_failed"')
    pr_url: str | None = None
    pr_number: int | None = None
    branch: str | None = None
    summary: str = ""
    verification_log_tail: str = ""


@workflow.define(
    name="daily-dev-pr",
    workflow_display_name="Daily development PR",
    workflow_description=(
        "Analyzes the target repository, picks and implements the single best next major "
        "development, verifies tests and clippy, then opens a pull request."
    ),
    execution_timeout=timedelta(hours=8),
)
class DailyDevPrWorkflow:
    @workflow.entrypoint
    async def run(self, input: DailyDevPrInput) -> DailyDevPrResult:
        setup = await setup_workspace(input.repo, input.base_branch)
        overview = await repo_overview()
        task = build_task_prompt(input.repo, input.base_branch, overview)

        agent = Agent(
            model=input.model,
            name="daily-dev-pr",
            description="Autonomous developer that ships one focused improvement per run",
            instructions=SYSTEM_PROMPT,
            tools=[list_files, read_file, write_file, delete_file, run_command, finish_development],
        )
        outputs = await Runner.run(
            agent=agent,
            inputs=task,
            session=RemoteSession(raise_on_tool_fail=False),
            max_turns=input.max_turns,
        )
        summary = "\n".join(getattr(o, "text", "") for o in outputs if isinstance(o, TextChunk)).strip()

        branch = f"auto/daily-dev-{setup['date']}"
        verification = await verify_changes()
        if not verification["ok"]:
            return DailyDevPrResult(
                status="verification_failed",
                branch=branch,
                summary=summary,
                verification_log_tail=verification["log_tail"],
            )

        pr = await push_and_open_pr(
            input.repo,
            input.base_branch,
            branch,
            build_commit_message(summary),
            build_pr_title(summary, branch),
            build_pr_body(summary, verification["log_tail"]),
        )
        if not pr.get("pr_url"):
            return DailyDevPrResult(status="no_changes", branch=branch, summary=summary)
        return DailyDevPrResult(
            status="pr_opened",
            pr_url=pr["pr_url"],
            pr_number=pr.get("pr_number"),
            branch=branch,
            summary=summary,
        )
