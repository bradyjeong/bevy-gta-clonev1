#!/bin/bash
# Quick format and basic check script
# Run this frequently during development

set -e

echo "🚀 Running quick checks and fixes..."

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "ok" ]; then
        echo -e "${GREEN}✅ $message${NC}"
    elif [ "$status" = "error" ]; then
        echo -e "${RED}❌ $message${NC}"
    else
        echo -e "${YELLOW}⚠️  $message${NC}"
    fi
}

# Auto-format code
print_status "info" "Auto-formatting code..."
cargo fmt --all
print_status "ok" "Code formatted"

# Quick compile check
print_status "info" "Running quick compile check..."
if RUSTFLAGS="-Dwarnings" cargo check --workspace --all-targets --all-features; then
    print_status "ok" "Compile check passed"
else
    print_status "error" "Compile check failed"
    exit 1
fi

# Run tests
print_status "info" "Running tests..."
if cargo test --workspace --all-features; then
    print_status "ok" "Tests passed"
else
    print_status "error" "Tests failed"
    exit 1
fi

print_status "ok" "Quick checks completed! 🎉"
echo "Ready for development. Run './scripts/pre-commit-check.sh' before committing."
