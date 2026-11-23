import subprocess

import pytest

import daily_dev.activities as acts
from daily_dev.prompts import (
    build_commit_message,
    build_pr_body,
    build_pr_title,
    build_task_prompt,
)


def test_safe_path_accepts_repo_relative_path():
    assert acts._safe_path("src/lib.rs").name == "lib.rs"


def test_safe_path_rejects_traversal():
    with pytest.raises(ValueError):
        acts._safe_path("../outside.txt")


def test_safe_path_rejects_forbidden_dirs():
    with pytest.raises(ValueError):
        acts._safe_path("automation/daily-dev-pr/Dockerfile")
    with pytest.raises(ValueError):
        acts._safe_path(".git/config")


def test_truncate():
    assert acts._truncate("abc", 10) == "abc"
    out = acts._truncate("x" * 100, 10)
    assert out.startswith("x" * 10)
    assert "truncated" in out


def test_build_task_prompt():
    prompt = build_task_prompt("jelni/ai-bullshit", "main", "FILE TREE ...")
    assert "jelni/ai-bullshit" in prompt
    assert "main" in prompt
    assert "FILE TREE ..." in prompt


def test_commit_and_pr_text_strip_markdown():
    summary = "## Add portals\n\nDetails here."
    assert build_commit_message(summary).startswith("Automated daily development: Add portals")
    assert build_pr_title(summary, "auto/daily-dev-2026-01-01") == "Automated daily development: Add portals"
    body = build_pr_body(summary, "log tail")
    assert "## Summary" in body
    assert "Add portals" in body
    assert "log tail" in body
    assert "daily-dev-pr" in body


def _git(*args, cwd):
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True)


def test_commit_branch_local_repo(tmp_path, monkeypatch):
    repo = tmp_path / "repo"
    repo.mkdir()
    monkeypatch.setattr(acts, "WORKDIR", repo)
    _git("init", "-b", "main", cwd=repo)
    _git("config", "user.name", "test", cwd=repo)
    _git("config", "user.email", "test@example.com", cwd=repo)
    (repo / "a.txt").write_text("one\n")
    _git("add", "-A", cwd=repo)
    _git("commit", "-m", "init", cwd=repo)

    assert acts.commit_branch("auto/x", "msg") == ""

    (repo / "a.txt").write_text("two\n")
    sha = acts.commit_branch("auto/x", "change a.txt")
    assert sha
    log = _git("log", "--oneline", "-1", cwd=repo).stdout
    assert "change a.txt" in log
    branch = _git("branch", "--show-current", cwd=repo).stdout.strip()
    assert branch == "auto/x"
