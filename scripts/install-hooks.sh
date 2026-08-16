#!/bin/sh
#
# Install git hooks from scripts/ directory

set -e

HOOKS_DIR="$(git rev-parse --git-path hooks)"
SCRIPTS_DIR="$(dirname "$0")"

echo "Installing git hooks..."

# Install pre-commit hook
if [ -f "$SCRIPTS_DIR/pre-commit" ]; then
	cp "$SCRIPTS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
	chmod +x "$HOOKS_DIR/pre-commit"
	echo "✅ Installed pre-commit hook"
else
	echo "❌ pre-commit script not found"
	exit 1
fi

echo ""
echo "Git hooks installed successfully!"
echo "The following checks will run before each commit:"
echo "  - cargo fmt --check"
echo "  - cargo clippy --workspace --all-targets -- -D warnings"
