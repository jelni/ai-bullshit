import shutil
import subprocess

import mistralai.workflows as workflows
from mistralai.workflows.plugins.mistralai import TextChunk

import daily_dev.activities as acts
from daily_dev.workflows import daily_dev_pr as mod
from daily_dev.workflows.daily_dev_pr import DailyDevPrInput, DailyDevPrWorkflow

WORKDIR = acts.WORKDIR


def _git(*args, cwd):
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True)


async def fake_setup_workspace(repo: str, base_branch: str) -> dict:
    if WORKDIR.parent.exists():
        shutil.rmtree(WORKDIR.parent, ignore_errors=True)
    (WORKDIR / "src").mkdir(parents=True)
    (WORKDIR / "src" / "lib.rs").write_text("pub fn hello() -> i32 { 42 }\n")
    (WORKDIR / "README.md").write_text("# test repo\n")
    _git("init", "-b", "main", cwd=WORKDIR)
    _git("config", "user.name", "test", cwd=WORKDIR)
    _git("config", "user.email", "test@example.com", cwd=WORKDIR)
    _git("add", "-A", cwd=WORKDIR)
    _git("commit", "-m", "init", cwd=WORKDIR)
    return {"repo": repo, "base_branch": base_branch, "head": "abc123", "log": "init", "date": "2026-11-23"}


async def fake_verify_ok() -> dict:
    return {"ok": True, "log_tail": "cargo test: ok\nclippy: ok"}


async def fake_verify_fail() -> dict:
    return {"ok": False, "log_tail": "cargo test: FAILED"}


async def fake_push_and_open_pr(repo: str, base_branch: str, branch: str, commit_message: str, pr_title: str, pr_body: str) -> dict:
    sha = acts.commit_branch(branch, commit_message)
    assert sha, "expected the agent's changes to produce a commit"
    assert "Automated daily development" in commit_message
    assert pr_body
    return {"pr_url": "https://github.com/example/repo/pull/1", "pr_number": 1, "branch": branch, "reason": "created"}


class FakeRunner:
    summary = "## Chose the portals feature\n\nImplemented it with tests."

    @staticmethod
    async def run(agent, inputs, max_turns=10, session=None):
        assert "jelni/ai-bullshit" in inputs
        overview = await mod.repo_overview()
        assert "src/lib.rs" in overview
        await mod.write_file("src/lib.rs", "pub fn hello() -> i32 { 43 }\n")
        listing = await mod.list_files("src/*")
        assert "src/lib.rs" in listing
        content = await mod.read_file("src/lib.rs")
        assert "43" in content
        cmd = await mod.run_command("echo hello-from-agent")
        assert "hello-from-agent" in cmd
        await mod.finish_development(FakeRunner.summary)
        return [TextChunk(text=FakeRunner.summary)]


async def _run_workflow(monkeypatch, verify_activity) -> dict:
    monkeypatch.setattr(mod, "setup_workspace", fake_setup_workspace)
    monkeypatch.setattr(mod, "verify_changes", verify_activity)
    monkeypatch.setattr(mod, "push_and_open_pr", fake_push_and_open_pr)
    monkeypatch.setattr(mod, "Runner", FakeRunner)
    result = await workflows.execute_workflow(
        DailyDevPrWorkflow,
        params=DailyDevPrInput(repo="jelni/ai-bullshit"),
    )
    return result if isinstance(result, dict) else result.model_dump()


async def test_daily_dev_pr_opens_pr(monkeypatch):
    result = await _run_workflow(monkeypatch, fake_verify_ok)
    assert result["status"] == "pr_opened"
    assert result["pr_url"] == "https://github.com/example/repo/pull/1"
    assert result["pr_number"] == 1
    assert result["branch"].startswith("auto/daily-dev-")
    assert "portals feature" in result["summary"]


async def test_daily_dev_pr_fails_when_verification_fails(monkeypatch):
    result = await _run_workflow(monkeypatch, fake_verify_fail)
    assert result["status"] == "verification_failed"
    assert result["pr_url"] is None
    assert "FAILED" in result["verification_log_tail"]
