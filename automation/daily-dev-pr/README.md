# daily-dev-pr

A [Mistral Workflows](https://docs.mistral.ai/studio/workflows/overview) worker that, on every run:

1. clones the target repository (`jelni/ai-bullshit` by default),
2. has a durable LLM agent (`mistral-large-4` by default) analyze the codebase and pick **exactly one** best next major development,
3. implements it end-to-end and fixes `cargo test` / `./clippy.sh` failures autonomously,
4. pushes a branch `auto/daily-dev-<UTC date>` and opens (or updates) a pull request.

It is driven by a **workflow schedule** (daily at 09:00 UTC), which is the Mistral/Vibe
"trigger" that creates the clean-up PR every day.

## Layout

- `src/daily_dev/activities.py` — all side effects (git, filesystem, shell, GitHub REST). The GitHub
  token is read from the `GITHUB_CLEANUP_PAT` env var inside activities only, never passed through
  workflow code, so it stays out of the execution history.
- `src/daily_dev/prompts.py` — agent system prompt and PR/commit text builders.
- `src/daily_dev/workflows/daily_dev_pr.py` — the `daily-dev-pr` workflow (durable agent loop + verify + PR).
- `src/daily_dev/entrypoint.py` — worker entrypoint.
- `tests/` — pytest suite (`create_test_worker` with external effects mocked).

## Prerequisites

- A GitHub fine-grained PAT with `Contents: read/write` and `Pull requests: write` on the target repo,
  stored as a **workspace secret named `github_cleanup_pat`** in Mistral Studio (bound to the deployment
  as env var `GITHUB_CLEANUP_PAT`). Never commit the token.
- The Mistral GitHub App installed on the target repo so managed deployments can clone it.

## Deploy

```bash
# from the repository root, after pushing this directory on a branch:
# deployment name: daily-dev-pr-worker
# build_directory: automation/daily-dev-pr
# dockerfile_path: Dockerfile
# revision: <pushed commit SHA>
# secrets: github_cleanup_pat -> GITHUB_CLEANUP_PAT
```

via `mistral_mcp_deploy_workflow_deployment`, or in the Studio console.

## Schedule (the daily trigger)

`mistral_mcp_create_workflow_schedule` with:

- `workflow_identifier`: `daily-dev-pr`
- `deployment_name`: `daily-dev-pr-worker`
- `schedule.input`: `{"repo": "jelni/ai-bullshit"}`
- `schedule.cron_expressions`: `["0 9 * * *"]` (09:00 UTC daily)
- `schedule.policy`: `{"overlap": 1, "pause_on_failure": true}`

## Run locally / test

```bash
python -m venv .venv && . .venv/bin/activate
pip install -e ".[dev]" 2>/dev/null || pip install mistralai-workflows[mistralai]==3.20.0 pytest pytest-asyncio
PYTHONPATH=src python -m pytest tests
```
