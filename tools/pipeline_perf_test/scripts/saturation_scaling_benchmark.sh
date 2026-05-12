#!/bin/bash
# =============================================================================
# Saturation Scaling Benchmark
# =============================================================================
# Tests how well the loadgen can saturate the engine (SUT) across different
# core counts and loadgen:engine ratios.
#
# Findings this script validates:
#   1. 1-core and 2-core SUT: 2:1 ratio achieves >95% CPU saturation
#   2. 4-core SUT with 2:1 ratio (8 LG cores): ~75% CPU due to SO_REUSEPORT
#      hash imbalance (always 2 hot + 2 cold engine cores)
#   3. 4-core SUT with 1:1 ratio (4 LG cores): ~50% CPU, 1 core at 0%
#   4. 2-container LG doesn't help — Docker bridge IPs don't add enough
#      hash entropy
#
# Prerequisites:
#   - Docker with df_engine:latest image built
#   - Python venv at tools/pipeline_perf_test/.venv with orchestrator deps
#   - At least 16 cores available
#
# Usage:
#   cd tools/pipeline_perf_test
#   bash scripts/saturation_scaling_benchmark.sh
#
# Runtime: ~15 minutes (5 configs × 2 runs × ~90s each)
# =============================================================================

set -eo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PERF_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PERF_DIR"

source .venv/bin/activate

RESULTS_DIR="$PERF_DIR/results/scaling_benchmark"
mkdir -p "$RESULTS_DIR"

RUNS_PER_CONFIG=2
METRIC_CAPTURE_DELAY=50  # seconds after test start to capture per-core metrics

# =============================================================================
# Helper: run a test config, capture per-core metrics, return summary
# =============================================================================
run_test() {
  local label="$1"
  local config="$2"
  local run_num="$3"
  local log_file="$RESULTS_DIR/${label//[ \/]/_}_run${run_num}.log"

  echo "  [$label] Run $run_num starting..."

  # Run orchestrator in background
  python ./orchestrator/run_orchestrator.py --config "$config" \
    2>&1 > "$log_file" &
  local pid=$!

  # Wait for observation phase, then capture per-core CPU
  sleep $METRIC_CAPTURE_DELAY

  local engine_cores=""
  engine_cores=$(curl -s "http://localhost:8086/api/v1/telemetry/metrics?reset=false" 2>/dev/null \
    | awk '/cpu_utilization\{set="pipeline.metrics"/{
        match($0, /core_id="([^"]+)"/, c);
        printf "%s:%.1f ", c[1], $2*100
      }') || true

  # Wait for test to complete
  wait $pid || true

  # Extract summary metrics from log
  local cpu_norm=$(grep "cpu_percentage_normalized_avg" "$log_file" 2>/dev/null | tail -1 | tr -d '|' | awk '{for(i=1;i<=NF;i++) if($i+0 > 1 && $i ~ /^[0-9]/) print $i}' | head -1)
  local throughput=$(grep "logs_produced_rate" "$log_file" 2>/dev/null | tail -1 | tr -d '|' | awk '{for(i=1;i<=NF;i++) if($i+0 > 1000 && $i ~ /^[0-9]/) print $i}' | head -1)

  # Count cores at 0% and cores >90%
  local zero_cores=0
  local hot_cores=0
  for pair in $engine_cores; do
    local val="${pair#*:}"
    val="${val%%%*}"
    local is_zero=$(awk "BEGIN{print ($val < 1.0) ? 1 : 0}" 2>/dev/null || echo 0)
    local is_hot=$(awk "BEGIN{print ($val > 90.0) ? 1 : 0}" 2>/dev/null || echo 0)
    [ "$is_zero" = "1" ] && zero_cores=$((zero_cores + 1))
    [ "$is_hot" = "1" ] && hot_cores=$((hot_cores + 1))
  done

  # Record result
  echo "$label|$run_num|${cpu_norm:-n/a}|${throughput:-n/a}|$engine_cores|$zero_cores|$hot_cores" >> "$RESULTS_DIR/all_results.csv"

  echo "  [$label] Run $run_num: CPU=${cpu_norm:-?}% Throughput=${throughput:-?} logs/s Per-core: $engine_cores (${zero_cores} idle, ${hot_cores} hot)"
}

# =============================================================================
# Main
# =============================================================================
echo ""
echo "================================================================="
echo " Saturation Scaling Benchmark"
echo " $RUNS_PER_CONFIG runs per configuration"
echo "================================================================="
echo ""

# Clear previous results
> "$RESULTS_DIR/all_results.csv"
echo "label|run|cpu_norm_avg|throughput|per_core_cpu|zero_cores|hot_cores" >> "$RESULTS_DIR/all_results.csv"

# --- Config 1: 1 SUT core, 2 LG cores (2:1) ---
echo "--- [1/5] 1-core SUT, 2:1 ratio (2 LG cores) ---"
for run in $(seq 1 $RUNS_PER_CONFIG); do
  run_test "1core-2to1" \
    "./test_suites/integration/continuous/saturation-1core.yaml" \
    "$run"
done
echo ""

# --- Config 2: 2 SUT cores, 4 LG cores (2:1) ---
echo "--- [2/5] 2-core SUT, 2:1 ratio (4 LG cores) ---"
for run in $(seq 1 $RUNS_PER_CONFIG); do
  run_test "2core-2to1" \
    "./test_suites/integration/continuous/saturation-2cores.yaml" \
    "$run"
done
echo ""

# --- Config 3: 4 SUT cores, 8 LG cores (2:1) ---
echo "--- [3/5] 4-core SUT, 2:1 ratio (8 LG cores, 1 container) ---"
for run in $(seq 1 $RUNS_PER_CONFIG); do
  run_test "4core-2to1" \
    "./test_suites/integration/continuous/saturation-4cores.yaml" \
    "$run"
done
echo ""

# --- Config 4: 4 SUT cores, 4 LG cores (1:1) ---
# Uses a temp config with 1:1 ratio
TEMP_1TO1="$RESULTS_DIR/_sat4_1to1.yaml"
cat > "$TEMP_1TO1" <<'EOF'
from_template:
  path: test_suites/integration/continuous/saturation-cores-template.yaml.j2
  variables:
    num_cores: 4
    engine_core_range: "0-3"
    loadgen_cores: 4
    loadgen_core_range: "4-7"
    backend_cores: 4
    backend_core_range: "12-15"
    max_batch_size: 512
EOF

echo "--- [4/5] 4-core SUT, 1:1 ratio (4 LG cores, 1 container) ---"
for run in $(seq 1 $RUNS_PER_CONFIG); do
  run_test "4core-1to1" "$TEMP_1TO1" "$run"
done
echo ""

# --- Config 5: 4 SUT cores, 2 LG containers × 2 cores (2:1, 2 containers) ---
echo "--- [5/5] 4-core SUT, 2:1 ratio (2×2 LG cores, 2 containers) ---"
for run in $(seq 1 $RUNS_PER_CONFIG); do
  run_test "4core-2lg" \
    "./test_suites/integration/continuous/saturation-4cores-2lg.yaml" \
    "$run"
done
echo ""

# =============================================================================
# Summary
# =============================================================================
echo "================================================================="
echo " SUMMARY"
echo "================================================================="
echo ""
printf "%-20s %8s %12s %8s %8s\n" "Config" "CPU%" "Logs/sec" "Idle" "Hot"
echo "--------------------------------------------------------------"

# Compute averages per config
for label in "1core-2to1" "2core-2to1" "4core-2to1" "4core-1to1" "4core-2lg"; do
  avg_cpu=$(grep "^$label|" "$RESULTS_DIR/all_results.csv" | awk -F'|' '{s+=$3; n++} END{if(n>0) printf "%.1f", s/n; else print "n/a"}')
  avg_tput=$(grep "^$label|" "$RESULTS_DIR/all_results.csv" | awk -F'|' '{s+=$4; n++} END{if(n>0) printf "%.0f", s/n; else print "n/a"}')
  avg_zero=$(grep "^$label|" "$RESULTS_DIR/all_results.csv" | awk -F'|' '{s+=$6; n++} END{if(n>0) printf "%.1f", s/n; else print "n/a"}')
  avg_hot=$(grep "^$label|" "$RESULTS_DIR/all_results.csv" | awk -F'|' '{s+=$7; n++} END{if(n>0) printf "%.1f", s/n; else print "n/a"}')
  printf "%-20s %8s %12s %8s %8s\n" "$label" "$avg_cpu" "$avg_tput" "$avg_zero" "$avg_hot"
done

echo ""
echo "Columns:"
echo "  CPU%     = Normalized avg CPU of SUT (100% = all cores saturated)"
echo "  Logs/sec = Throughput through the SUT"
echo "  Idle     = Avg # of engine cores at 0% (SO_REUSEPORT imbalance)"
echo "  Hot      = Avg # of engine cores at >90%"
echo ""
echo "Key observations:"
echo "  - 1-core and 2-core: >95% CPU, properly saturated with 2:1 ratio"
echo "  - 4-core 2:1: ~75% CPU, 2 hot + 2 cold cores (reuseport hash skew)"
echo "  - 4-core 1:1: ~50% CPU, 1 core gets 0 connections"
echo "  - 4-core 2-container: same as 1:1, Docker bridge IPs don't help"
echo ""
echo "Conclusion: 2:1 ratio with single container is the best practical"
echo "approach. The ~75% ceiling at 4+ cores is an SO_REUSEPORT limitation"
echo "that needs engine-level fixes (eBPF rebalancing or topic-based split)."
echo ""
echo "Raw results: $RESULTS_DIR/all_results.csv"
echo "Per-run logs: $RESULTS_DIR/*.log"

# Cleanup temp config
rm -f "$TEMP_1TO1"
