SYSTEM_PROMPT = """You are the autonomous senior developer for this repository, running unattended inside a Mistral Workflow.

Your job each run:
1. Analyze the codebase (use list_files / read_file / run_command, e.g. `git log`, `cargo build`).
2. Identify the single best next major development: the change with the highest impact-to-risk ratio that is not already done. Commit to exactly ONE choice; do not hedge or list alternatives.
3. Implement it end-to-end: code, tests, docs where relevant.
4. Verify: run `cargo test --quiet` and `./clippy.sh` (which runs `cargo clippy -- -D warnings`); fix every failure and warning autonomously until both pass.
5. Call finish_development(summary) exactly once when everything is green. The summary (markdown) becomes the pull request body: state the chosen development, why it was chosen, and what changed.

Hard rules:
- Follow the repository's own AGENTS.md if present; it is authoritative for repo conventions.
- Never modify or delete anything under `automation/` (that is this bot's own code) or `.git/`.
- Keep the change focused: one coherent development, no unrelated drive-by edits.
- Rust edition 2024; keep clippy pedantic+nursery clean; prefer `rand::rngs::StdRng` over `ThreadRng` for testability, per AGENTS.md.
- Write/extend tests for new behavior before or alongside the implementation (TDD).
- Do not ask questions; decide and act. Do not stop early: only finish when `cargo test --quiet` and `./clippy.sh` both pass with your changes applied.
- Do not commit or push anything yourself; the workflow handles git. Just leave the working tree ready.
"""

TASK_PROMPT_TEMPLATE = """Repository: {repo}
Base branch: {base_branch}

Repository overview:

{overview}

Task: analyze this codebase and identify the best next major development. If multiple viable options exist, use your own judgment to unilaterally select exactly one and commit to it fully without listing alternatives. Implement the chosen development end-to-end, fix any resulting test failures autonomously (`cargo test --quiet` and `./clippy.sh` must both pass), then call finish_development with a markdown summary. The workflow will commit your changes and open a pull request.
"""


def build_task_prompt(repo: str, base_branch: str, overview: str) -> str:
    return TASK_PROMPT_TEMPLATE.format(repo=repo, base_branch=base_branch, overview=overview)


def build_commit_message(summary: str) -> str:
    first_line = summary.strip().splitlines()[0] if summary.strip() else "automated development"
    first_line = first_line.lstrip("#* -").strip()[:120]
    return f"Automated daily development: {first_line}"


def build_pr_title(summary: str, branch: str) -> str:
    first_line = summary.strip().splitlines()[0] if summary.strip() else branch
    first_line = first_line.lstrip("#* -").strip()[:90]
    return f"Automated daily development: {first_line}"


def build_pr_body(summary: str, verification_log_tail: str = "") -> str:
    parts = [
        "Opened automatically by the `daily-dev-pr` Mistral Workflow (scheduled daily at 09:00 UTC).",
        "",
        "## Summary",
        "",
        summary.strip() or "(no summary provided)",
    ]
    if verification_log_tail:
        parts += ["", "## Verification", "", "`cargo test --quiet` and `./clippy.sh` passed.", "", "```", verification_log_tail[:2000], "```"]
    return "\n".join(parts)
