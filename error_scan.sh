#!/bin/bash

echo "# Compilation Error Summary" > ERROR_SUMMARY.md
echo "" >> ERROR_SUMMARY.md

mkdir -p errors/by_crate

# Count total errors
TOTAL_ERRORS=$(grep -c '"level":"error"' compile.log 2>/dev/null || echo "0")
echo "Total errors: $TOTAL_ERRORS" >> ERROR_SUMMARY.md
echo "" >> ERROR_SUMMARY.md

# Extract errors by package
grep '"level":"error"' compile.log | while IFS= read -r line; do
    # Extract package name (simplified)
    PACKAGE=$(echo "$line" | grep -o '"package_id":"[^"]*"' | cut -d'"' -f4 | cut -d'#' -f1)
    # Extract error message
    MESSAGE=$(echo "$line" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$PACKAGE" ] && [ -n "$MESSAGE" ]; then
        echo "$MESSAGE" >> "errors/by_crate/${PACKAGE}.md"
    fi
done

echo "Generated ERROR_SUMMARY.md and errors/by_crate/ files"
echo "Total compilation errors found: $TOTAL_ERRORS"
