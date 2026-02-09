#!/bin/bash
# Script to detect direct use of tracing/log macros instead of otel_* wrappers.
# All events MUST be emitted using the otel_* macros from otap_df_telemetry.
# See docs/telemetry/events-guide.md
#
# Usage: ./scripts/check-direct-telemetry-macros.sh

set -e

echo "🔍 Checking for direct use of tracing/log macros (should use otel_* macros instead)..."

# Function to check for a banned pattern in .rs files
check_banned_pattern() {
    local pattern="$1"
    local description="$2"
    local files

    # Find all .rs files, excluding:
    #   - target/         build artifacts
    #   - .git/           git internals
    #   - crates/telemetry/  implements the otel_* wrappers (legitimate tracing usage)
    #   - crates/quiver/src/logging.rs     implements the otel_* wrappers internal to quiver (legitimate tracing usage)
    #   - benchmarks/        benchmarks may legitimately use tracing directly
    files=$(find . -name "*.rs" -type f \
        | grep -v target/ \
        | grep -v .git/ \
        | grep -v crates/telemetry/ \
        | grep -v crates/quiver/src/logging.rs \
        | grep -v benchmarks/ \
        || true)

    if [ -z "$files" ]; then
        return 0
    fi

    local matches
    matches=$(echo "$files" | xargs grep -n "$pattern" 2>/dev/null || true)

    if [ -n "$matches" ]; then
        echo "⚠️  Found direct $description:"
        echo "$matches"
        echo ""
        return 1
    fi
    return 0
}

checks_passed=0

# Check for fully-qualified tracing macro calls
if ! check_banned_pattern "tracing::info!" "tracing::info! usage (use otel_info! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "tracing::warn!" "tracing::warn! usage (use otel_warn! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "tracing::error!" "tracing::error! usage (use otel_error! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "tracing::debug!" "tracing::debug! usage (use otel_debug! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "tracing::trace!" "tracing::trace! usage (use otel_debug! instead)"; then
    checks_passed=1
fi

# Check for fully-qualified log crate macro calls
if ! check_banned_pattern "log::info!" "log::info! usage (use otel_info! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "log::warn!" "log::warn! usage (use otel_warn! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "log::error!" "log::error! usage (use otel_error! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "log::debug!" "log::debug! usage (use otel_debug! instead)"; then
    checks_passed=1
fi

if ! check_banned_pattern "log::trace!" "log::trace! usage (use otel_debug! instead)"; then
    checks_passed=1
fi

if [ $checks_passed -eq 0 ]; then
    echo "✅ No direct tracing/log macro usage found!"
    echo "ℹ️  All events use otel_* macros as required by docs/telemetry/events-guide.md."
else
    echo "❌ Found direct tracing/log macro usage. Use otel_* macros from otap_df_telemetry instead."
    echo ""
    echo "How to fix:"
    echo "  • tracing::info!(...)  → otel_info!(\"event.name\", ...)"
    echo "  • tracing::warn!(...)  → otel_warn!(\"event.name\", ...)"
    echo "  • tracing::error!(...) → otel_error!(\"event.name\", ...)"
    echo "  • tracing::debug!(...) → otel_debug!(\"event.name\", ...)"
    echo ""
    echo "  Import from otap_df_telemetry:"
    echo "    use otap_df_telemetry::otel_info;"
    echo ""
    echo "  See docs/telemetry/events-guide.md for full details."
    exit 1
fi
