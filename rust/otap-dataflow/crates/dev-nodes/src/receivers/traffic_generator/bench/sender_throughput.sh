#!/bin/bash
# Traffic Generator - sender throughput and CPU benchmark.
#
# Measures the cost of the traffic generator itself: one sender core
# (traffic-gen -> OTLP gRPC export) sending straight to a noop backend, so the
# backend is never the bottleneck.
#
#   sender(:8080, 1 core) -> backend(:4319, noop, admin :8082)
#
# Each scenario reports signals/sec, sender process CPU (percent of one core,
# from /proc/<pid>/stat), and CPU-us per 1k signals. Saturation scenarios (rate=null) measure max throughput per core;
# fixed-rate scenarios measure CPU at equal load.
#
# Usage (from anywhere):
#   bench/sender_throughput.sh <df_engine binary> <label> [runs]
#
# Environment:
#   SCENARIOS  space separated scenario names to run (default: all)
#   SETTLE     seconds before the first sample (default: 25)
#   SAMPLE     sample window in seconds (default: 15)
#   BACKEND_CORES  cores for the noop backend (default: 4) so it never limits
#              the sender
#   MAX_IN_FLIGHT  exporter max in-flight RPCs (default: 32)
#
# Output: one CSV row per run on stdout:
#   label,scenario,run,signals_per_sec,sender_cpu_pct,cpu_us_per_1k,backend_cpu_pct

set -eo pipefail

BIN=${1:?usage: $0 <df_engine binary> <label> [runs]}
LABEL=${2:?usage: $0 <df_engine binary> <label> [runs]}
RUNS=${3:-3}
SETTLE=${SETTLE:-25}
SAMPLE=${SAMPLE:-15}
BACKEND_CORES=${BACKEND_CORES:-4}
MAX_IN_FLIGHT=${MAX_IN_FLIGHT:-32}
# "Uncapped" mode (signals_per_second: null) targets 1024 batches per second,
# which caps throughput below the CPU limit for small batches. Saturation
# scenarios instead request a rate no single core can reach.
SATURATION_RATE=${SATURATION_RATE:-20000000}
# pre_generated materializes one second of payloads up front, so its rate is
# bounded to keep memory reasonable (~1.7 GB for 1 KB logs).
PREGEN_RATE=${PREGEN_RATE:-1500000}

BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
WORK_DIR="$(mktemp -d)"

cleanup() {
  [ -n "$SENDER" ] && kill "$SENDER" 2>/dev/null || true
  [ -n "$BACKEND" ] && kill "$BACKEND" 2>/dev/null || true
  wait 2>/dev/null || true
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

# name|sink|source|signal|strategy|signals_per_second|max_batch_size|extra traffic_config lines (';' separated)
#
# sink=noop: the sender's exporter is an in-process noop and production is
#   unpaced (production_mode: open), so the scenario measures the traffic
#   generator alone (max signals/sec per core).
# sink=grpc: the sender exports OTLP gRPC to the noop backend, so the scenario
#   measures realistic sender CPU including encoding size effects on export.
ALL_SCENARIOS=(
  "sat-logs-1k-b50|noop|synthetic|logs|fresh|$SATURATION_RATE|50|production_mode: open;log_body_size_bytes: 1024"
  "sat-logs-1k-b512|noop|synthetic|logs|fresh|$SATURATION_RATE|512|production_mode: open;log_body_size_bytes: 1024"
  "sat-logs-attrs10-b512|noop|synthetic|logs|fresh|$SATURATION_RATE|512|production_mode: open;num_log_attributes: 10"
  "sat-traces-b512|noop|synthetic|traces|fresh|$SATURATION_RATE|512|production_mode: open"
  "sat-metrics-6x3-b512|noop|synthetic|metrics|fresh|$SATURATION_RATE|512|production_mode: open;num_metric_attributes: 6;num_data_points_per_metric: 3"
  "sat-logs-1k-b512-pregen|noop|synthetic|logs|pre_generated|$PREGEN_RATE|512|production_mode: open;log_body_size_bytes: 1024"
  "grpc-logs-1k-b512-100k|grpc|synthetic|logs|fresh|100000|512|log_body_size_bytes: 1024"
  "grpc-logs-1k-b50-50k|grpc|synthetic|logs|fresh|50000|50|log_body_size_bytes: 1024"
  "grpc-traces-b512-100k|grpc|synthetic|traces|fresh|100000|512|"
  "grpc-metrics-6x3-b512-50k|grpc|synthetic|metrics|fresh|50000|512|num_metric_attributes: 6;num_data_points_per_metric: 3"
  "sat-semconv-logs-b512|noop|semantic_conventions|logs|fresh|$SATURATION_RATE|512|production_mode: open"
  "sat-semconv-traces-b512|noop|semantic_conventions|traces|fresh|$SATURATION_RATE|512|production_mode: open"
  "sat-semconv-metrics-b512|noop|semantic_conventions|metrics|fresh|$SATURATION_RATE|512|production_mode: open"
  "grpc-semconv-logs-b512-100k|grpc|semantic_conventions|logs|fresh|100000|512|"
)

write_sender_config() {
  local file=$1 sink=$2 source=$3 signal=$4 strategy=$5 rate=$6 batch=$7 extra=$8
  local log_w=0 trace_w=0 metric_w=0
  case $signal in
    logs) log_w=1 ;;
    traces) trace_w=1 ;;
    metrics) metric_w=1 ;;
  esac

  {
    cat <<EOF
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8080"
groups:
  default:
    pipelines:
      main:
        policies:
          channel_capacity:
            control:
              node: 100
              pipeline: 100
            pdata: 128
        nodes:
          receiver:
            type: receiver:traffic_generator
            policies:
              telemetry:
                item_counts: true
            config:
              data_source: $source
              generation_strategy: $strategy
              traffic_config:
                signals_per_second: $rate
                max_batch_size: $batch
                metric_weight: $metric_w
                trace_weight: $trace_w
                log_weight: $log_w
EOF
    if [ -n "$extra" ]; then
      echo "$extra" | tr ';' '\n' | sed 's/^/                /'
    fi
    if [ "$sink" = "noop" ]; then
      cat <<EOF
          exporter:
            type: exporter:noop
EOF
    else
      cat <<EOF
          exporter:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://127.0.0.1:4319"
              max_in_flight: $MAX_IN_FLIGHT
              num_connections: $BACKEND_CORES
EOF
    fi
    cat <<EOF
        connections:
          - from: receiver
            to: exporter
EOF
  } >"$file"
}

METRICS_URL="http://127.0.0.1:8080/api/v1/metrics?format=prometheus"
CLK_TCK=$(getconf CLK_TCK)

# The traffic generator receiver's successful output item counter for one signal.
items_for() {
  local metrics=$1 signal=$2
  echo "$metrics" | awk -v sig="signal=\"$signal\"" '
    /^items_total\{/ && index($0, sig) && index($0, "otel_scope_node_id=\"receiver\"") \
      && index($0, "outcome=\"success\"") { print $2; exit }'
}

# Total CPU time (user + system, all threads) of a process in clock ticks.
cpu_ticks() {
  awk '{ print $14 + $15 }' "/proc/$1/stat"
}

# A sender or backend exited before the sample finished. Report it and keep
# going rather than letting `set -e` silently abort the remaining scenarios.
report_failure() {
  echo "scenario $1 run $2 failed: a process exited early. Sender log tail:" >&2
  tail -5 "$WORK_DIR/sender.log" >&2 || true
  kill "$SENDER" "$BACKEND" 2>/dev/null || true
  wait "$SENDER" "$BACKEND" 2>/dev/null || true
  SENDER=""
  BACKEND=""
  sleep 2
}

run_one() {
  local scenario=$1 run=$2
  IFS='|' read -r name sink source signal strategy rate batch extra <<<"$scenario"
  local cfg="$WORK_DIR/$name.yaml"
  write_sender_config "$cfg" "$sink" "$source" "$signal" "$strategy" "$rate" "$batch" "$extra"

  "$BIN" --config "$BENCH_DIR/backend-noop.yaml" --num-cores "$BACKEND_CORES" >/dev/null 2>&1 &
  BACKEND=$!
  sleep 5
  "$BIN" --config "$cfg" --num-cores 1 >"$WORK_DIR/sender.log" 2>&1 &
  SENDER=$!
  sleep "$SETTLE"

  local m1 m2 i1 i2 t1 t2 b1 b2
  if ! t1=$(cpu_ticks "$SENDER") || ! b1=$(cpu_ticks "$BACKEND"); then
    report_failure "$name" "$run"
    return
  fi
  m1=$(curl -s "$METRICS_URL")
  i1=$(items_for "$m1" "$signal")
  sleep "$SAMPLE"
  if ! t2=$(cpu_ticks "$SENDER") || ! b2=$(cpu_ticks "$BACKEND"); then
    report_failure "$name" "$run"
    return
  fi
  m2=$(curl -s "$METRICS_URL")
  i2=$(items_for "$m2" "$signal")

  kill "$SENDER" "$BACKEND" 2>/dev/null || true
  wait "$SENDER" "$BACKEND" 2>/dev/null || true
  SENDER=""
  BACKEND=""
  sleep 2

  awk -v l="$LABEL" -v n="$name" -v r="$run" -v i1="${i1:-0}" -v i2="${i2:-0}" \
    -v t1="$t1" -v t2="$t2" -v b1="$b1" -v b2="$b2" -v hz="$CLK_TCK" -v s="$SAMPLE" 'BEGIN {
      rps = (i2 - i1) / s
      # Percent of one core used by the whole sender process.
      cpu = (t2 - t1) / hz / s * 100
      bcpu = (b2 - b1) / hz / s * 100
      per1k = rps > 0 ? (cpu / 100) * 1e6 / rps * 1000 : 0
      printf "%s,%s,%s,%.0f,%.1f,%.2f,%.1f\n", l, n, r, rps, cpu, per1k, bcpu
    }'
}

echo "label,scenario,run,signals_per_sec,sender_cpu_pct,cpu_us_per_1k,backend_cpu_pct"
for scenario in "${ALL_SCENARIOS[@]}"; do
  name=${scenario%%|*}
  if [ -n "$SCENARIOS" ] && ! [[ " $SCENARIOS " == *" $name "* ]]; then
    continue
  fi
  for run in $(seq 1 "$RUNS"); do
    run_one "$scenario" "$run"
  done
done
