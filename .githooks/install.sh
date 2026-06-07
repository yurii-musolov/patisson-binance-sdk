#!/usr/bin/env bash
#
# Point this repository's git hooks at .githooks/ — a one-shot setup that
# enables pre-commit and pre-push for every git operation in this clone.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

git config core.hooksPath .githooks

echo "Git hooks installed."
echo "  pre-commit: cargo fmt --check + cargo check"
echo "  pre-push:   cargo fmt --check + cargo clippy + cargo test"
echo
echo "Skip individual runs with --no-verify when needed."
