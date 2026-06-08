#!/bin/sh
# rs-guard pre-commit hook
#
# Install:
#   cp examples/local-review/pre-commit-hook.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
#
# This hook analyzes staged changes with rs-guard and aborts the commit
# if the review returns REQUEST_CHANGES.

# Set your preferred provider and API key here, or rely on env vars.
# export RS_GUARD_PROVIDER="deepseek"
# export DEEPSEEK_API_KEY="your-api-key"

if ! command -v rs-guard >/dev/null 2>&1; then
    echo "rs-guard: not found in PATH. Skipping AI review."
    echo "Install from: https://github.com/nebulaideas/rs-guard/releases"
    exit 0
fi

echo "Running rs-guard pre-commit review..."

rs-guard
EXIT_CODE=$?

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "rs-guard: Review passed."
    exit 0
elif [ "$EXIT_CODE" -eq 2 ]; then
    echo "rs-guard: Review returned REQUEST_CHANGES. Commit aborted."
    echo "Address the issues above or bypass with: git commit --no-verify"
    exit 1
else
    echo "rs-guard: Error occurred (exit code $EXIT_CODE)."
    exit 1
fi
