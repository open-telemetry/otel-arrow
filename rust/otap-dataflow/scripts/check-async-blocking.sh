#!/bin/bash
# Script to detect potentially blocking operations in async contexts
# Usage: ./scripts/check-async-blocking.sh

set -e

echo "🔍 Checking for potentially blocking operations in async contexts..."

# Function to check for blocking patterns in files that contain async functions
check_blocking_patterns() {
    local pattern="$1"
    local description="$2"
    local files
    
    # Find files that contain async functions
    files=$(find . -name "*.rs" -type f | grep -v target/ | grep -v .git/ | xargs grep -l "async fn\|async move\|async {" 2>/dev/null || true)
    
    if [ -z "$files" ]; then
        return 0
    fi
    
    # Check for the pattern in those files, but exclude test modules and certain safe patterns
    local matches
    matches=$(echo "$files" | xargs grep -n "$pattern" 2>/dev/null | grep -v "#\[cfg(test)\]" | grep -v "mod tests" | grep -v "SocketAddr" | grep -v "\.await" || true)
    
    if [ -n "$matches" ]; then
        echo "⚠️  WARNING: Found potential $description:"
        echo "$matches"
        echo ""
        return 1
    fi
    return 0
}

# Check for common blocking operations
checks_passed=0

# Check for std::io::Write usage in async contexts (but not AsyncWrite)
if ! check_blocking_patterns "use std::io::Write\b" "std::io::Write usage (use tokio::io::AsyncWrite instead)"; then
    checks_passed=1
fi

# Check for std::fs operations in async contexts (but not in tests)
if ! check_blocking_patterns "std::fs::\(read\|write\|create\|remove\|copy\|rename\)" "blocking std::fs operations (use tokio::fs instead)"; then
    checks_passed=1
fi

# Check for thread::sleep in async contexts
if ! check_blocking_patterns "thread::sleep\|std::thread::sleep" "thread::sleep usage (use tokio::time::sleep instead)"; then
    checks_passed=1
fi

# Check for blocking file operations
if ! check_blocking_patterns "File::open\|File::create" "blocking File operations (use tokio::fs::File instead)"; then
    checks_passed=1
fi

if [ $checks_passed -eq 0 ]; then
    echo "✅ No critical blocking operations detected in async contexts!"
    echo "ℹ️  Note: This script checks for common blocking patterns. Always review async code manually."
else
    echo "❌ Found potentially blocking operations. Please review and use async alternatives."
    echo ""
    echo "Common fixes:"
    echo "  • std::io::Write → tokio::io::AsyncWrite + AsyncWriteExt"
    echo "  • std::fs → tokio::fs"
    echo "  • File::open/create → tokio::fs::File"
    echo "  • thread::sleep → tokio::time::sleep"
    echo ""
    echo "💡 Consider using tokio::task::spawn_blocking() for unavoidable blocking operations"
    exit 1
fi
