#!/bin/bash

echo "🔍 VERIFYING SUBMISSION READINESS..."
echo ""

ERRORS=0

# Check for personal info
echo "Checking for personal information..."
if grep -r "/mnt/c/Users/prate" --include="*.md" --include="*.ts" --include="*.tsx" . 2>/dev/null | grep -v node_modules | grep -q .; then
    echo "❌ Found personal paths!"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No personal paths found"
fi

# Check for placeholders
echo "Checking for placeholders..."
if grep -r "\[Your Name\]\|\[Your Handle\]\|\[Competition Name\]" --include="*.md" . 2>/dev/null | grep -v node_modules | grep -q .; then
    echo "❌ Found placeholder text!"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No placeholder text found"
fi

# Check for sensitive data
echo "Checking for sensitive data..."
if grep -r "API_KEY\|SECRET\|PASSWORD.*=" --include="*.md" --include="*.ts" --include="*.env" . 2>/dev/null | grep -v node_modules | grep -v "# " | grep -q .; then
    echo "❌ Found potential sensitive data!"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No sensitive data found"
fi

# Check main files exist
echo "Checking required files..."
REQUIRED_FILES=(
    "README.md"
    "ARCHITECTURE.md"
    "DEPLOYMENT_INFO.md"
    "frontend/package.json"
    "contracts/factory/Cargo.toml"
)
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo "❌ Missing: $file"
        ERRORS=$((ERRORS + 1))
    fi
done
echo "✅ All required files present"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ERRORS -eq 0 ]; then
    echo "✅ SUBMISSION READY! No issues found! 🎉"
else
    echo "⚠️  Found $ERRORS issue(s). Please review above."
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
