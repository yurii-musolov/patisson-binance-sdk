# Git hooks

Repository-tracked git hooks that run code-quality checks before commits
and pushes. Lives here (not in `.git/hooks/`) so the team shares the same
checks; `.git/hooks/` isn't tracked by git.

## Install

Once per clone:

```sh
./.githooks/install.sh
```

This runs `git config core.hooksPath .githooks` for the current repo.

## What runs

### `pre-commit` (fast — runs on every commit)

| Command | Catches |
|---|---|
| `cargo fmt --all -- --check` | unformatted code (does not auto-format) |
| `cargo check --all-targets` | compile errors across lib, bins, examples, tests |

### `pre-push` (heavier — runs before sharing commits)

| Command | Catches |
|---|---|
| `cargo fmt --all -- --check` | belt + suspenders against the pre-commit hook |
| `cargo clippy --all-targets -- -D warnings` | lint errors **and** warnings (strict — any new warning fails the push) |
| `cargo test --all-targets` | failing tests |

## Bypassing

Either hook can be skipped on an individual operation:

```sh
git commit --no-verify -m "WIP"
git push --no-verify
```

Use sparingly — they're there to keep `main` green.

## Strictness

The pre-push hook passes `-D warnings` to clippy — any new clippy warning
fails the push. Keep `cargo clippy --all-targets -- -D warnings` clean as
you work; the pre-push hook will catch regressions before they hit the
remote.
