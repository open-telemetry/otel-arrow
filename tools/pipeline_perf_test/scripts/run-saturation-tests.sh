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

echo "=============================================="
echo "Running Saturation/Scaling Tests"
echo "=============================================="
echo ""

# Find and run all saturation test configs
for config in test_suites/integration/continuous/saturation-*.yaml; do
    if [[ -f "$config" ]]; then
        echo "Running: $config"
        python orchestrator/run_orchestrator.py --config "$config"
        echo ""
    fi
done

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
