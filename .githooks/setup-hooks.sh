#!/usr/bin/env bash
# Configures Git to use the repository .githooks directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$REPO_ROOT"

echo "⚙️  Configuring Git hooks for Laradoc Studio..."
git config core.hooksPath .githooks

chmod +x .githooks/pre-commit
chmod +x .githooks/commit-msg

echo "✅ Git hooks activated successfully! core.hooksPath is set to .githooks."
