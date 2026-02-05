#!/bin/bash
# Run idle state tests and analyze memory scaling
#
# This script runs idle state tests with different core counts and produces
# a memory scaling analysis to verify the linear memory model (Memory = C + N * R).
#
# Usage:
#   ./scripts/run-idle-state-tests.sh [--output-json <path>]
#
# Options:
#   --output-json <path>  Write memory scaling metrics to JSON file
#                         (for github-action-benchmark integration)
#
# Examples:
#   # Run locally and just see the report
#   ./scripts/run-idle-state-tests.sh
#
#   # Run with JSON output for CI
#   ./scripts/run-idle-state-tests.sh --output-json results/idle-memory-scaling.json

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
echo "Running Idle State Memory Scaling Tests"
echo "=============================================="
echo ""

# Run idle state tests for specific core counts (1, 2, 4, 8, 16, 32)
# These are used to fit the linear memory model: Memory = C + N * R
CORE_COUNTS="1 2 4 8 16 32"

for cores in $CORE_COUNTS; do
    if [[ "$cores" == "1" ]]; then
        config="test_suites/integration/continuous/idle-state-docker.yaml"
    else
        config="test_suites/integration/continuous/idle-state-${cores}cores-docker.yaml"
    fi
    
    if [[ -f "$config" ]]; then
        echo "Running: $config ($cores core(s))"
        python orchestrator/run_orchestrator.py --config "$config"
        echo ""
    else
        echo "Warning: Config not found: $config"
    fi
done

echo ""
echo "=============================================="
echo "Analyzing Memory Scaling"
echo "=============================================="
echo ""

# Build the analysis command
ANALYZE_CMD="python $REPO_ROOT/.github/workflows/scripts/analyze-idle-state-scaling.py $PERF_TEST_DIR/results"

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
    echo "Memory scaling JSON written to: $OUTPUT_JSON"
fi
