#!/bin/bash
# Fake Data Generator — SUT saturation benchmark.
# Measures whether 1 sender core (fake-gen → OTLP export) can saturate
# 1 SUT core (OTLP recv → OTLP export forwarding) over gRPC.
#
# Setup: 3 processes, 1 core each:
#   sender(:8080) → SUT(:8081) → backend(:8082)
#
# Prerequisites:
#   - Release build: cargo build --release
#   - No other engine instances running on ports 8080–8082, 4317, 4319
#
# Usage (from rust/otap-dataflow/):
#   bash crates/core-nodes/src/receivers/fake_data_generator/bench/bench.sh
#
# Each test runs for ~40s. Full suite takes ~3 minutes.

set -eo pipefail
BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BENCH_DIR/../../../../.."  # rust/otap-dataflow/

SETTLE=25
SAMPLE=15

cleanup() { pkill -f df_engine 2>/dev/null || true; sleep 2; }
trap cleanup EXIT

run_saturation() {
  local label=$1 sender_config=$2

  cargo run --release -- --config "$BENCH_DIR/backend-noop.yaml" --num-cores 1 >/dev/null 2>&1 &
  local BACKEND=$!; sleep 2
  cargo run --release -- --config "$BENCH_DIR/sut-otlp-forward.yaml" --num-cores 1 >/dev/null 2>&1 &
  local SUT=$!; sleep 3
  cargo run --release -- --config "$sender_config" --num-cores 1 >/dev/null 2>&1 &
  local SENDER=$!; sleep $SETTLE

  local sm=$(curl -s http://127.0.0.1:8080/metrics)
  local em=$(curl -s http://127.0.0.1:8081/metrics)
  local sender_cpu=$(echo "$sm" | awk '/^cpu_utilization\{set="pipeline/{print $2}')
  local sut_cpu=$(echo "$em" | awk '/^cpu_utilization\{set="pipeline/{print $2}')
  local logs1=$(echo "$sm" | awk '/^logs_produced\{/{print $2}')
  sleep $SAMPLE
  sm=$(curl -s http://127.0.0.1:8080/metrics)
  local logs2=$(echo "$sm" | awk '/^logs_produced\{/{print $2}')
  local rps=$(( (${logs2:-0} - ${logs1:-0}) / SAMPLE ))
  local s_pct=$(echo "scale=1; ${sender_cpu:-0} * 100" | bc)
  local e_pct=$(echo "scale=1; ${sut_cpu:-0} * 100" | bc)

  printf "%-35s %10s %10s %10s\n" "$label" "$rps" "${s_pct}%" "${e_pct}%"
  kill $SENDER $SUT $BACKEND 2>/dev/null || true; wait 2>/dev/null || true; sleep 3
}

echo ""
echo "================================================================="
echo " SUT saturation: sender → SUT(otlp-recv → otlp-export) → backend"
echo " 1 core each, ${SETTLE}s settle, ${SAMPLE}s sample"
echo "================================================================="
printf "%-35s %10s %10s %10s\n" "Config" "Logs/sec" "Sender" "SUT"
echo "-----------------------------------------------------------------"
run_saturation "static/fresh 1KB"       "$BENCH_DIR/sender-static-fresh.yaml"
run_saturation "static/pregen 1KB"      "$BENCH_DIR/sender-static-pregen.yaml"

echo ""
echo "Done."
