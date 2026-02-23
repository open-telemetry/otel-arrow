#!/bin/bash
# Validate all OtelDataflow configuration files in the repository.
#
# Finds every .yaml/.yml file containing "version: otel_dataflow/v1" under the
# otap-dataflow tree and runs the engine binary with --validate-and-exit to
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
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Build or locate the binary
if [ $# -ge 1 ]; then
    BINARY="$1"
else
    echo "Building df_engine with all component features..."
    cargo build --release \
        --features experimental-tls,contrib-exporters,contrib-processors,recordset-kql-processor,azure-monitor-exporter,geneva-exporter,condense-attributes-processor,resource-validator-processor,azure,aws \
        --manifest-path "$REPO_ROOT/Cargo.toml"
    BINARY="$REPO_ROOT/target/release/df_engine"
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
FAILED_FILES=""

for config in $CONFIG_FILES; do
    TOTAL=$((TOTAL + 1))
    REL_PATH="${config#"$REPO_ROOT/"}"

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
echo "Config validation: $PASSED/$TOTAL passed, $FAILED failed."

if [ "$FAILED" -gt 0 ]; then
    echo -e "\nFailed configs:$FAILED_FILES"
    exit 1
fi
