import json
import os
import shutil
import subprocess
import urllib.request
from datetime import datetime, timedelta, timezone
from pathlib import Path

import mistralai.workflows as workflows

WORKDIR = Path("/tmp/daily-dev-pr/repo")
MAX_OUTPUT_CHARS = 12000
MAX_FILE_CONTENT_CHARS = 200000
FORBIDDEN_TOP_DIRS = {"automation", ".git"}


def _token() -> str:
    token = os.environ.get("GITHUB_CLEANUP_PAT") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise RuntimeError(
            "GitHub token missing: bind the workspace secret GITHUB_CLEANUP_PAT on the deployment"
        )
    return token


def clone_url(repo: str) -> str:
    return f"https://x-access-token:{_token()}@github.com/{repo}.git"


def _safe_path(path: str) -> Path:
    root = WORKDIR.resolve()
    candidate = (root / path).resolve()
    if root != candidate and root not in candidate.parents:
        raise ValueError(f"path escapes the repository: {path}")
    if candidate.relative_to(root).parts and candidate.relative_to(root).parts[0] in FORBIDDEN_TOP_DIRS:
        raise ValueError(f"path is off-limits for automated changes: {path}")
    return candidate


def _truncate(text: str, limit: int = MAX_OUTPUT_CHARS) -> str:
    if len(text) <= limit:
        return text
    return text[:limit] + f"\n... [truncated {len(text) - limit} chars]"


def _shell(cmd: list[str], cwd: Path, timeout: int) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd,
        cwd=cwd,
        timeout=timeout,
        capture_output=True,
        text=True,
        check=False,
    )


def _fmt_result(proc: subprocess.CompletedProcess, timeout: int) -> str:
    out = (proc.stdout or "") + (proc.stderr or "")
    return f"exit_code={proc.returncode}\n{_truncate(out)}"


@workflows.activity(start_to_close_timeout=timedelta(minutes=10))
async def setup_workspace(repo: str, base_branch: str) -> dict:
    """Clone the target repository into the worker workspace.

    Args:
        repo: GitHub repository in "owner/name" form.
        base_branch: Branch to check out.
    """
    if WORKDIR.parent.exists():
        shutil.rmtree(WORKDIR.parent, ignore_errors=True)
    WORKDIR.parent.mkdir(parents=True, exist_ok=True)
    proc = _shell(
        ["git", "clone", "--depth", "50", "--branch", base_branch, clone_url(repo), str(WORKDIR)],
        cwd=WORKDIR.parent,
        timeout=540,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"git clone failed:\n{_truncate((proc.stdout or '') + (proc.stderr or ''))}")
    _shell(["git", "config", "user.name", "mistral-vibe-bot"], cwd=WORKDIR, timeout=30)
    _shell(["git", "config", "user.email", "mistral-vibe-bot@users.noreply.github.com"], cwd=WORKDIR, timeout=30)
    log = _shell(["git", "log", "--oneline", "-15"], cwd=WORKDIR, timeout=30)
    head = _shell(["git", "rev-parse", "HEAD"], cwd=WORKDIR, timeout=30)
    return {
        "repo": repo,
        "base_branch": base_branch,
        "head": head.stdout.strip(),
        "log": _truncate(log.stdout, 4000),
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
    }


@workflows.activity(start_to_close_timeout=timedelta(minutes=2))
async def repo_overview() -> str:
    """Return a compact overview of the checked-out repository: file tree, README, manifest, recent history."""
    files = _shell(["git", "ls-files"], cwd=WORKDIR, timeout=60)
    paths = [p for p in files.stdout.splitlines() if p and not p.startswith(("target/", "automation/"))]
    tree = "\n".join(sorted(paths)[:400])
    if len(paths) > 400:
        tree += f"\n... [{len(paths) - 400} more files]"
    readme = ""
    readme_path = WORKDIR / "README.md"
    if readme_path.exists():
        readme = _truncate(readme_path.read_text(errors="replace"), 6000)
    manifest_path = WORKDIR / "Cargo.toml"
    manifest = _truncate(manifest_path.read_text(errors="replace"), 3000) if manifest_path.exists() else ""
    log = _shell(["git", "log", "--oneline", "-15"], cwd=WORKDIR, timeout=30)
    return _truncate(
        f"FILE TREE ({len(paths)} files):\n{tree}\n\nREADME.md:\n{readme}\n\nCargo.toml:\n{manifest}\n\nRECENT COMMITS:\n{log.stdout}",
        MAX_OUTPUT_CHARS,
    )


@workflows.activity(start_to_close_timeout=timedelta(minutes=2))
async def list_files(pattern: str = "*") -> str:
    """List files in the repository matching a glob pattern.

    Args:
        pattern: Glob pattern relative to the repository root, e.g. "src/**/*.rs" or "*.toml".
    """
    import fnmatch

    matches = [
        str(p.relative_to(WORKDIR))
        for p in sorted(WORKDIR.rglob("*"))
        if p.is_file()
        and ".git" not in p.parts
        and "target" not in p.parts
        and fnmatch.fnmatch(str(p.relative_to(WORKDIR)), pattern)
    ]
    return _truncate("\n".join(matches[:500]) + (f"\n... [{len(matches) - 500} more]" if len(matches) > 500 else ""), MAX_OUTPUT_CHARS)


@workflows.activity(start_to_close_timeout=timedelta(minutes=2))
async def read_file(path: str, max_lines: int = 500) -> str:
    """Read a file from the repository.

    Args:
        path: Path relative to the repository root.
        max_lines: Maximum number of lines to return (default 500).
    """
    target = _safe_path(path)
    if not target.exists() or not target.is_file():
        return f"error: no such file: {path}"
    content = target.read_text(errors="replace")
    lines = content.splitlines()
    truncated = len(lines) > max_lines
    return _truncate("\n".join(lines[:max_lines]) + (f"\n... [file has {len(lines)} lines, showing first {max_lines}]" if truncated else ""), MAX_OUTPUT_CHARS)


@workflows.activity(start_to_close_timeout=timedelta(minutes=2))
async def write_file(path: str, content: str) -> str:
    """Create or overwrite a file in the repository with the given full content.

    Args:
        path: Path relative to the repository root. Must not be under automation/ or .git/.
        content: The complete new file content.
    """
    if len(content) > MAX_FILE_CONTENT_CHARS:
        raise ValueError(f"content too large: {len(content)} chars (max {MAX_FILE_CONTENT_CHARS})")
    target = _safe_path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)
    return f"wrote {path} ({content.count(chr(10)) + 1} lines)"


@workflows.activity(start_to_close_timeout=timedelta(minutes=2))
async def delete_file(path: str) -> str:
    """Delete a file from the repository.

    Args:
        path: Path relative to the repository root. Must not be under automation/ or .git/.
    """
    target = _safe_path(path)
    if not target.exists():
        return f"error: no such file: {path}"
    target.unlink()
    return f"deleted {path}"


@workflows.activity(start_to_close_timeout=timedelta(minutes=30), retry_policy_max_attempts=1)
async def run_command(command: str, timeout_seconds: int = 600) -> str:
    """Run a shell command inside the repository checkout and return its output.

    Use this to build, test and lint the project, e.g. "cargo test", "./clippy.sh", "cargo build".

    Args:
        command: Shell command to run in the repository root.
        timeout_seconds: Maximum runtime in seconds (capped at 1800).
    """
    timeout = min(max(int(timeout_seconds), 30), 1800)
    proc = subprocess.run(
        ["bash", "-lc", command],
        cwd=WORKDIR,
        timeout=timeout,
        capture_output=True,
        text=True,
        check=False,
    )
    return _fmt_result(proc, timeout)


@workflows.activity(start_to_close_timeout=timedelta(minutes=2), retry_policy_max_attempts=1)
async def finish_development(summary: str) -> str:
    """Signal that the development is complete and all checks pass. Call this exactly once at the end.

    Args:
        summary: A markdown summary of the chosen development and what was changed. It becomes the PR body.
    """
    return "Recorded. The workflow will now verify the changes, commit them to a branch and open a pull request."


@workflows.activity(start_to_close_timeout=timedelta(minutes=45), retry_policy_max_attempts=1)
async def verify_changes() -> dict:
    """Run the project's test suite and linter on the current working tree. Returns pass/fail plus a log tail."""
    tests = subprocess.run(
        ["bash", "-lc", "cargo test --quiet"],
        cwd=WORKDIR,
        timeout=2400,
        capture_output=True,
        text=True,
        check=False,
    )
    lint = subprocess.run(
        ["bash", "-lc", "./clippy.sh"],
        cwd=WORKDIR,
        timeout=1200,
        capture_output=True,
        text=True,
        check=False,
    )
    ok = tests.returncode == 0 and lint.returncode == 0
    log_tail = _truncate(
        f"=== cargo test (exit {tests.returncode}) ===\n{(tests.stdout or '') + (tests.stderr or '')}\n"
        f"=== ./clippy.sh (exit {lint.returncode}) ===\n{(lint.stdout or '') + (lint.stderr or '')}",
        MAX_OUTPUT_CHARS,
    )
    return {"ok": ok, "log_tail": log_tail}


def commit_branch(branch: str, commit_message: str) -> str:
    """Stage all changes, create the branch and commit locally. Returns the commit SHA."""
    if not WORKDIR.exists():
        raise RuntimeError("workspace not set up")
    status = _shell(["git", "status", "--porcelain"], cwd=WORKDIR, timeout=60)
    if not status.stdout.strip():
        return ""
    _shell(["git", "checkout", "-b", branch], cwd=WORKDIR, timeout=60)
    _shell(["git", "add", "-A"], cwd=WORKDIR, timeout=120)
    _shell(["git", "commit", "-m", commit_message], cwd=WORKDIR, timeout=120)
    sha = _shell(["git", "rev-parse", "HEAD"], cwd=WORKDIR, timeout=30)
    return sha.stdout.strip()


def _github_api(repo: str, method: str, path: str, payload: dict | None = None) -> dict:
    url = f"https://api.github.com/repos/{repo}{path}"
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Bearer {_token()}")
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("X-GitHub-Api-Version", "2022-11-28")
    if data is not None:
        req.add_header("Content-Type", "application/json")
    with urllib.request.urlopen(req, timeout=60) as resp:
        return json.loads(resp.read().decode())


@workflows.activity(start_to_close_timeout=timedelta(minutes=10), retry_policy_max_attempts=2)
async def push_and_open_pr(repo: str, base_branch: str, branch: str, commit_message: str, pr_title: str, pr_body: str) -> dict:
    """Commit pending changes to a branch, push it, and create or update a pull request.

    Args:
        repo: GitHub repository in "owner/name" form.
        base_branch: Branch the pull request targets.
        branch: Branch to create and push.
        commit_message: Commit message for the automated change.
        pr_title: Pull request title.
        pr_body: Pull request body in markdown.
    """
    sha = commit_branch(branch, commit_message)
    if not sha:
        return {"pr_url": None, "pr_number": None, "branch": branch, "reason": "no_changes"}
    push = _shell(
        ["git", "push", "--force", clone_url(repo), f"HEAD:{branch}"],
        cwd=WORKDIR,
        timeout=300,
    )
    if push.returncode != 0:
        raise RuntimeError(f"git push failed:\n{_truncate((push.stdout or '') + (push.stderr or ''))}")
    owner = repo.split("/")[0]
    existing = _github_api(repo, "GET", f"/pulls?head={owner}:{branch}&state=open")
    if existing:
        pr = existing[0]
        _github_api(repo, "PATCH", f"/pulls/{pr['number']}", {"title": pr_title, "body": pr_body, "base": base_branch})
        return {"pr_url": pr["html_url"], "pr_number": pr["number"], "branch": branch, "reason": "updated"}
    pr = _github_api(
        repo,
        "POST",
        "/pulls",
        {"title": pr_title, "body": pr_body, "head": branch, "base": base_branch},
    )
    return {"pr_url": pr["html_url"], "pr_number": pr["number"], "branch": branch, "reason": "created"}
