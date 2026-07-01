#!/bin/bash
# Validate all OtelDataflow configuration files in the repository.
#
# Finds every .yaml/.yml file containing "version: otel_dataflow/v1" across the
# entire repository and runs the engine binary with --validate-and-exit to
# verify that each config is structurally valid, references only known components,
# and has correct component-specific configuration.
#
# Usage:
#   ./scripts/validate-configs.sh [path-to-df_engine-binary]
#
# When no binary path is given, the script builds a release binary with all
# optional component features enabled and uses that.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Build or locate the binary
if [ $# -ge 1 ]; then
    BINARY="$1"
else
    echo "Building df_engine with all component features..."
    # Note: --all-features cannot be used because jemalloc and mimalloc are
    # mutually exclusive (compile_error! in non-test builds).
    cargo build \
        --locked \
        --features azure,aws,contrib-exporters,contrib-processors,contrib-receivers,recordset-kql-processor,azure-monitor-exporter,geneva-exporter,condense-attributes-processor,resource-validator-processor,user_events-eventheader \
        --manifest-path "$PROJECT_DIR/Cargo.toml"
    BINARY="$PROJECT_DIR/target/debug/df_engine"
fi

if [ ! -x "$BINARY" ]; then
    echo "ERROR: Binary not found or not executable: $BINARY"
    exit 1
fi

# Discover all otel_dataflow config files
CONFIG_FILES=$(grep -rl "version: otel_dataflow/v1" "$REPO_ROOT" \
    --include="*.yaml" --include="*.yml" \
    | grep -v target/ \
    | sort)

if [ -z "$CONFIG_FILES" ]; then
    echo "ERROR: No otel_dataflow config files found."
    exit 1
fi

TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0
FAILED_FILES=""

host_os="$(uname -s)"

for config in $CONFIG_FILES; do
    TOTAL=$((TOTAL + 1))
    REL_PATH="${config#"$REPO_ROOT/"}"

    # Fail (don't silently skip) if a discovered .yaml/.yml dataflow config
    # contains Jinja2 "{{ ... }}" placeholders. Such files are templates that
    # are rendered (e.g. by the pipeline_perf_test harness) before use and are
    # not valid as-is. Templates must use the ".yaml.j2" extension so they are
    # excluded from discovery instead of being validated as literal configs.
    if grep -Eq '\{\{[^}]*\}\}' "$config"; then
        echo "  ❌ $REL_PATH (Jinja2 placeholder found; rename template to *.yaml.j2)"
        FAILED=$((FAILED + 1))
        FAILED_FILES="$FAILED_FILES\n  - $REL_PATH (Jinja2 template must use *.yaml.j2 extension)"
        continue
    fi

    # Check for a "# platform:" marker at the top of the file.
    required_platform=$(sed -n 's/^# platform: //p' "$config" | head -1)

    case "$required_platform" in
        "")
            ;;  # platform-agnostic — always validate
        windows-only)
            case "$host_os" in
                MINGW*|MSYS*|CYGWIN*) ;;
                *) echo "  ⏭  $REL_PATH (skipped: requires Windows)"; SKIPPED=$((SKIPPED+1)); continue ;;
            esac ;;
        linux-only)
            if [ "$host_os" != "Linux" ]; then
                echo "  ⏭  $REL_PATH (skipped: requires Linux)"; SKIPPED=$((SKIPPED+1)); continue
            fi ;;
        macos-only)
            if [ "$host_os" != "Darwin" ]; then
                echo "  ⏭  $REL_PATH (skipped: requires macOS)"; SKIPPED=$((SKIPPED+1)); continue
            fi ;;
        *)
            echo "  ❌ $REL_PATH (invalid platform marker: $required_platform)"; FAILED=$((FAILED+1)); FAILED_FILES="$FAILED_FILES\n  - $REL_PATH (invalid platform marker)"; continue ;;
    esac

    if "$BINARY" --config "$config" --validate-and-exit > /dev/null 2>&1; then
        echo "  ✅ $REL_PATH"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $REL_PATH"
        # Re-run to capture the error message
        "$BINARY" --config "$config" --validate-and-exit 2>&1 | tail -5 | sed 's/^/     /'
        FAILED=$((FAILED + 1))
        FAILED_FILES="$FAILED_FILES\n  - $REL_PATH"
    fi
done

echo ""
echo "Config validation: $PASSED/$TOTAL passed, $FAILED failed, $SKIPPED skipped."

if [ "$FAILED" -gt 0 ]; then
    echo -e "\nFailed configs:$FAILED_FILES"
    exit 1
fi
