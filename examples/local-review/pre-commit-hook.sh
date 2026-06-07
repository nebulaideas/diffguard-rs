#!/bin/sh
# diffguard-rs pre-commit hook
# Install: cp pre-commit-hook.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

# Run diffguard on staged changes
if command -v diffguard >/dev/null 2>&1; then
    diffguard --prompt-file .github/review-prompt.md
    EXIT_CODE=$?
    
    if [ $EXIT_CODE -eq 2 ]; then
        echo ""
        echo "❌ Commit blocked: diffguard review returned REQUEST_CHANGES."
        echo "   Address the issues above or use git commit --no-verify to bypass."
        exit 1
    elif [ $EXIT_CODE -eq 1 ]; then
        echo ""
        echo "⚠️  diffguard encountered an error. Check output above."
        exit 1
    fi
else
    echo "⚠️  diffguard not found in PATH. Skipping AI review."
    echo "   Install from: https://github.com/nebulaideas/diffguard-rs/releases"
fi

exit 0
