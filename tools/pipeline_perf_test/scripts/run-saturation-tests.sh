#!/bin/bash
# Run saturation/scaling tests and analyze results
#
# This script runs saturation tests with different core counts and produces
# a scaling efficiency analysis to verify the shared-nothing architecture.
#
# Usage:
#   ./scripts/run-saturation-tests.sh [--output-json <path>]
#
# Options:
#   --output-json <path>  Write scaling efficiency metrics to JSON file
#                         (for github-action-benchmark integration)
#
# Examples:
#   # Run locally and just see the report
#   ./scripts/run-saturation-tests.sh
#
#   # Run with JSON output for CI
#   ./scripts/run-saturation-tests.sh --output-json results/scaling-efficiency.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PERF_TEST_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$PERF_TEST_DIR/../.." && pwd)"

# Parse arguments
OUTPUT_JSON=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --output-json)
            OUTPUT_JSON="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$PERF_TEST_DIR"

# Detect available CPU cores on this machine
AVAILABLE_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 0)
echo "=============================================="
echo "Running Saturation/Scaling Tests"
echo "Detected $AVAILABLE_CORES CPU cores on this machine"
echo "=============================================="
echo ""

# Compute total cores needed for a saturation test config.
# Each config defines: num_cores (engine) + loadgen_cores + backend_cores
# The pattern is: engine=N, loadgen=3*N, backend=N => total=5*N
# We parse the actual values from each YAML to be safe.
compute_required_cores() {
    local config="$1"
    local engine loadgen backend

    engine=$(grep -oP 'num_cores:\s*\K[0-9]+' "$config" 2>/dev/null || echo 0)
    loadgen=$(grep -oP 'loadgen_cores:\s*\K[0-9]+' "$config" 2>/dev/null || echo 0)
    backend=$(grep -oP 'backend_cores:\s*\K[0-9]+' "$config" 2>/dev/null || echo 0)

    echo $((engine + loadgen + backend))
}

# Find and run saturation test configs that fit on this machine
SKIPPED=0
RAN=0
for config in test_suites/integration/continuous/saturation-*.yaml; do
    if [[ -f "$config" ]]; then
        required=$(compute_required_cores "$config")
        if [[ "$required" -le "$AVAILABLE_CORES" ]]; then
            echo "Running: $config (requires $required cores, $AVAILABLE_CORES available)"
            python orchestrator/run_orchestrator.py --config "$config"
            RAN=$((RAN + 1))
            echo ""
        else
            echo "Skipping: $config (requires $required cores, only $AVAILABLE_CORES available)"
            SKIPPED=$((SKIPPED + 1))
        fi
    fi
done

echo ""
echo "Ran $RAN test(s), skipped $SKIPPED test(s) due to insufficient cores."

echo ""
echo "=============================================="
echo "Analyzing Scaling Efficiency"
echo "=============================================="
echo ""

# Build the analysis command
ANALYZE_CMD="python $REPO_ROOT/.github/workflows/scripts/analyze-saturation-scaling.py $PERF_TEST_DIR/results"

if [[ -n "$OUTPUT_JSON" ]]; then
    # Make path absolute if relative
    if [[ "$OUTPUT_JSON" != /* ]]; then
        OUTPUT_JSON="$PERF_TEST_DIR/$OUTPUT_JSON"
    fi
    ANALYZE_CMD="$ANALYZE_CMD $OUTPUT_JSON"
fi

# Run analysis
$ANALYZE_CMD

if [[ -n "$OUTPUT_JSON" ]]; then
    echo ""
    echo "Scaling efficiency JSON written to: $OUTPUT_JSON"
fi
