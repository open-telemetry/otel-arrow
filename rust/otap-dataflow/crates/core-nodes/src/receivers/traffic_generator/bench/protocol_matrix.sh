#!/usr/bin/env bash
# Three-process protocol matrix benchmark.
#
# Runs static traffic through:
#   traffic_generator -> protocol receiver/exporter under test -> noop backend
#
# The sender and backend are Rust df_engine processes. The SUT can be either
# the Rust engine or a Go Collector binary. For logs, STEF is reported as an
# unsupported scenario because both current STEF implementations are metrics-only.
#
# Usage, from rust/otap-dataflow/:
#   cargo build --release --bin df_engine
#   SIGNAL_KIND=logs SUT_IMPLS="rust go" CORES=1 \
#     bash crates/core-nodes/src/receivers/traffic_generator/bench/protocol_matrix.sh
#
# Useful overrides:
#   SIGNAL_KIND=metrics|logs
#   SUT_IMPLS="rust go"
#   PROTOCOLS="stef otlp otap"
#   TARGET_ITEMS_PER_SECOND=200000
#   TARGET_ITEMS_PER_BATCH=10000
#   LOG_BODY_SIZE_BYTES=1024
#   WIRE_COMPRESSION=zstd         # gRPC compression for OTLP/OTAP: zstd, gzip, deflate, or none.
#   STEF_COMPRESSION=zstd         # native STEF frame compression: zstd or none.
#   OTAP_ARROW_COMPRESSION=zstd   # zstd or none.
#   SETTLE=25 SAMPLE=15 CORES=1

set -euo pipefail

BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCH_DIR/../../../../../.."

ENGINE_BIN="${ENGINE_BIN:-target/release/df_engine}"
GO_COLLECTOR_BIN="${GO_COLLECTOR_BIN:-/home/lquerel/oss/otel-arrow-inbox-rename/bin/otelarrowcol}"
SIGNAL_KIND="${SIGNAL_KIND:-metrics}"
SUT_IMPLS="${SUT_IMPLS:-rust}"
PROTOCOLS="${PROTOCOLS:-stef otlp otap}"
METRIC_OBJECTS_PER_BATCH="${METRIC_OBJECTS_PER_BATCH:-10}"
NUM_METRIC_ATTRIBUTES="${NUM_METRIC_ATTRIBUTES:-3}"
NUM_LOG_ATTRIBUTES="${NUM_LOG_ATTRIBUTES:-2}"
LOG_BODY_SIZE_BYTES="${LOG_BODY_SIZE_BYTES:-1024}"
WIRE_COMPRESSION="${WIRE_COMPRESSION:-zstd}"
STEF_COMPRESSION="${STEF_COMPRESSION:-zstd}"
OTAP_ARROW_COMPRESSION="${OTAP_ARROW_COMPRESSION:-zstd}"
SETTLE="${SETTLE:-25}"
SAMPLE="${SAMPLE:-15}"
CORES="${CORES:-1}"
OUT_DIR="${OUT_DIR:-/tmp/otap-protocol-matrix-${SIGNAL_KIND}-$(date +%Y%m%d-%H%M%S)}"
RUST_METRICS_PATH="/api/v1/telemetry/metrics?format=prometheus&keep_all_zeroes=false"
GO_METRICS_PATH="/metrics"

if [[ ! -x "$ENGINE_BIN" ]]; then
  echo "Release binary not found at $ENGINE_BIN. Run: cargo build --release --bin df_engine" >&2
  exit 1
fi

case "$SIGNAL_KIND" in
  metrics)
    TARGET_ITEMS_PER_SECOND="${TARGET_ITEMS_PER_SECOND:-${TARGET_DATAPOINTS_PER_SECOND:-200000}}"
    TARGET_ITEMS_PER_BATCH="${TARGET_ITEMS_PER_BATCH:-${TARGET_DATAPOINTS_PER_BATCH:-10000}}"
    ITEM_NAME="data points"
    ITEM_COUNTER="metrics_produced"
    PIPELINE_NAME="metrics"
    ;;
  logs)
    TARGET_ITEMS_PER_SECOND="${TARGET_ITEMS_PER_SECOND:-${TARGET_LOGS_PER_SECOND:-200000}}"
    TARGET_ITEMS_PER_BATCH="${TARGET_ITEMS_PER_BATCH:-${TARGET_LOGS_PER_BATCH:-10000}}"
    ITEM_NAME="logs"
    ITEM_COUNTER="logs_produced"
    PIPELINE_NAME="logs"
    ;;
  *)
    echo "SIGNAL_KIND must be one of: metrics, logs" >&2
    exit 1
    ;;
esac

if (( METRIC_OBJECTS_PER_BATCH < 1 || METRIC_OBJECTS_PER_BATCH > 12 )); then
  echo "METRIC_OBJECTS_PER_BATCH must be between 1 and 12 for STEF-compatible Sum/Gauge traffic." >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

if [[ "$SIGNAL_KIND" == "metrics" ]]; then
  DATA_POINTS_PER_METRIC=$(
    awk -v target="$TARGET_ITEMS_PER_BATCH" -v metrics="$METRIC_OBJECTS_PER_BATCH" \
      'BEGIN { print int((target + metrics - 1) / metrics) }'
  )
  ITEMS_PER_BATCH=$(( METRIC_OBJECTS_PER_BATCH * DATA_POINTS_PER_METRIC ))
  BATCHES_PER_SECOND=$(
    awk -v target="$TARGET_ITEMS_PER_SECOND" -v batch="$ITEMS_PER_BATCH" \
      'BEGIN { print int((target + batch - 1) / batch) }'
  )
  GENERATOR_SIGNALS_PER_SECOND=$(( BATCHES_PER_SECOND * METRIC_OBJECTS_PER_BATCH ))
  EXPECTED_ITEMS_PER_SECOND=$(( BATCHES_PER_SECOND * ITEMS_PER_BATCH ))
else
  DATA_POINTS_PER_METRIC=0
  ITEMS_PER_BATCH="$TARGET_ITEMS_PER_BATCH"
  BATCHES_PER_SECOND=$(
    awk -v target="$TARGET_ITEMS_PER_SECOND" -v batch="$ITEMS_PER_BATCH" \
      'BEGIN { print int((target + batch - 1) / batch) }'
  )
  GENERATOR_SIGNALS_PER_SECOND=$(( BATCHES_PER_SECOND * ITEMS_PER_BATCH ))
  EXPECTED_ITEMS_PER_SECOND="$GENERATOR_SIGNALS_PER_SECOND"
fi

CLK_TCK="$(getconf CLK_TCK)"
PIDS=()
LAST_PID=""

cleanup() {
  local pid
  for pid in "${PIDS[@]:-}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
    fi
  done

  for _ in $(seq 1 40); do
    local alive=0
    for pid in "${PIDS[@]:-}"; do
      if kill -0 "$pid" 2>/dev/null; then
        alive=1
        break
      fi
    done
    if (( alive == 0 )); then
      break
    fi
    sleep 0.25
  done

  for pid in "${PIDS[@]:-}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill -9 "$pid" 2>/dev/null || true
    fi
  done
  wait 2>/dev/null || true
  PIDS=()
}
trap cleanup EXIT

client_compression_line() {
  if [[ "$WIRE_COMPRESSION" != "none" ]]; then
    printf '              compression: "%s"\n' "$WIRE_COMPRESSION"
  fi
}

receiver_compression_line() {
  printf '                  request_compression: "%s"\n' "$WIRE_COMPRESSION"
}

stef_receiver_compression_line() {
  printf '              request_compression: "%s"\n' "$WIRE_COMPRESSION"
}

otap_exporter_compression_line() {
  printf '              compression_method: "%s"\n' "$WIRE_COMPRESSION"
}

otap_receiver_compression_line() {
  if [[ "$WIRE_COMPRESSION" != "none" ]]; then
    printf '              compression_method: "%s"\n' "$WIRE_COMPRESSION"
  fi
}

go_exporter_compression_line() {
  if [[ "$WIRE_COMPRESSION" != "none" ]]; then
    printf '    compression: %s\n' "$WIRE_COMPRESSION"
  fi
}

stef_exporter_compression_line() {
  if [[ "$STEF_COMPRESSION" != "none" ]]; then
    printf '              stef_compression: "%s"\n' "$STEF_COMPRESSION"
  fi
}

go_stef_exporter_compression_line() {
  if [[ "$STEF_COMPRESSION" != "none" ]]; then
    printf '    compression: %s\n' "$STEF_COMPRESSION"
  fi
}

write_traffic_config() {
  if [[ "$SIGNAL_KIND" == "metrics" ]]; then
    cat <<YAML
                production_mode: smooth
                signals_per_second: ${GENERATOR_SIGNALS_PER_SECOND}
                max_signal_count: null
                max_batch_size: ${METRIC_OBJECTS_PER_BATCH}
                metric_weight: 1
                trace_weight: 0
                log_weight: 0
                num_metric_attributes: ${NUM_METRIC_ATTRIBUTES}
                num_data_points_per_metric: ${DATA_POINTS_PER_METRIC}
YAML
  else
    cat <<YAML
                production_mode: smooth
                signals_per_second: ${GENERATOR_SIGNALS_PER_SECOND}
                max_signal_count: null
                max_batch_size: ${ITEMS_PER_BATCH}
                metric_weight: 0
                trace_weight: 0
                log_weight: 1
                log_body_size_bytes: ${LOG_BODY_SIZE_BYTES}
                num_log_attributes: ${NUM_LOG_ATTRIBUTES}
YAML
  fi
}

write_sender_config() {
  local protocol="$1"
  local path="$2"
  local exporter_type endpoint exporter_extra

  case "$protocol" in
    stef)
      exporter_type="exporter:stef"
      endpoint="http://127.0.0.1:4320"
      exporter_extra="$(stef_exporter_compression_line)"
      ;;
    otlp)
      exporter_type="exporter:otlp_grpc"
      endpoint="http://127.0.0.1:4317"
      exporter_extra="$(client_compression_line)"
      ;;
    otap)
      exporter_type="exporter:otap"
      endpoint="http://127.0.0.1:4326"
      exporter_extra="$(otap_exporter_compression_line)
              arrow:
                payload_compression: \"${OTAP_ARROW_COMPRESSION}\""
      ;;
    *)
      echo "unknown protocol: $protocol" >&2
      exit 1
      ;;
  esac

  cat >"$path" <<YAML
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
            config:
              data_source: static
              generation_strategy: pre_generated
              traffic_config:
$(write_traffic_config)

          exporter:
            type: ${exporter_type}
            config:
              grpc_endpoint: "${endpoint}"
${exporter_extra}

        connections:
          - from: receiver
            to: exporter
YAML
}

write_rust_sut_config() {
  local protocol="$1"
  local path="$2"

  case "$protocol" in
    stef)
      cat >"$path" <<YAML
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8081"
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
            type: receiver:stef
            config:
              listening_addr: "127.0.0.1:4320"
$(stef_receiver_compression_line)
          exporter:
            type: exporter:stef
            config:
              grpc_endpoint: "http://127.0.0.1:4321"
$(stef_exporter_compression_line)
        connections:
          - from: receiver
            to: exporter
YAML
      ;;
    otlp)
      cat >"$path" <<YAML
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8081"
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
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
$(receiver_compression_line)
          exporter:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://127.0.0.1:4319"
$(client_compression_line)
        connections:
          - from: receiver
            to: exporter
YAML
      ;;
    otap)
      cat >"$path" <<YAML
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8081"
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
            type: receiver:otap
            config:
              listening_addr: "127.0.0.1:4326"
              response_stream_channel_size: 256
$(otap_receiver_compression_line)
          exporter:
            type: exporter:otap
            config:
              grpc_endpoint: "http://127.0.0.1:4327"
$(otap_exporter_compression_line)
              arrow:
                payload_compression: "${OTAP_ARROW_COMPRESSION}"
        connections:
          - from: receiver
            to: exporter
YAML
      ;;
    *)
      echo "unknown protocol: $protocol" >&2
      exit 1
      ;;
  esac
}

write_go_sut_config() {
  local protocol="$1"
  local path="$2"

  case "$protocol" in
    stef)
      cat >"$path" <<YAML
receivers:
  stef:
    endpoint: 127.0.0.1:4320

exporters:
  stef:
    endpoint: 127.0.0.1:4321
$(go_stef_exporter_compression_line)
    tls:
      insecure: true

service:
  telemetry:
    metrics:
      readers:
        - pull:
            exporter:
              prometheus:
                host: 127.0.0.1
                port: 8081
  pipelines:
    ${PIPELINE_NAME}:
      receivers: [stef]
      exporters: [stef]
YAML
      ;;
    otlp)
      cat >"$path" <<YAML
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4317

exporters:
  otlp:
    endpoint: 127.0.0.1:4319
$(go_exporter_compression_line)
    tls:
      insecure: true

service:
  telemetry:
    metrics:
      readers:
        - pull:
            exporter:
              prometheus:
                host: 127.0.0.1
                port: 8081
  pipelines:
    ${PIPELINE_NAME}:
      receivers: [otlp]
      exporters: [otlp]
YAML
      ;;
    otap)
      cat >"$path" <<YAML
receivers:
  otelarrow:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4326

exporters:
  otelarrow:
    endpoint: 127.0.0.1:4327
$(go_exporter_compression_line)
    arrow:
      payload_compression: ${OTAP_ARROW_COMPRESSION}
    tls:
      insecure: true

service:
  telemetry:
    metrics:
      readers:
        - pull:
            exporter:
              prometheus:
                host: 127.0.0.1
                port: 8081
  pipelines:
    ${PIPELINE_NAME}:
      receivers: [otelarrow]
      exporters: [otelarrow]
YAML
      ;;
    *)
      echo "unknown protocol: $protocol" >&2
      exit 1
      ;;
  esac
}

write_backend_config() {
  local protocol="$1"
  local path="$2"

  case "$protocol" in
    stef)
      cat >"$path" <<YAML
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8082"
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
            type: receiver:stef
            config:
              listening_addr: "127.0.0.1:4321"
$(stef_receiver_compression_line)
          exporter:
            type: exporter:noop
        connections:
          - from: receiver
            to: exporter
YAML
      ;;
    otlp)
      cat >"$path" <<YAML
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8082"
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
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4319"
$(receiver_compression_line)
          exporter:
            type: exporter:noop
        connections:
          - from: receiver
            to: exporter
YAML
      ;;
    otap)
      cat >"$path" <<YAML
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:8082"
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
            type: receiver:otap
            config:
              listening_addr: "127.0.0.1:4327"
              response_stream_channel_size: 256
$(otap_receiver_compression_line)
          exporter:
            type: exporter:noop
        connections:
          - from: receiver
            to: exporter
YAML
      ;;
    *)
      echo "unknown protocol: $protocol" >&2
      exit 1
      ;;
  esac
}

start_rust_engine() {
  local config="$1"
  local log="$2"

  "$ENGINE_BIN" --config "$config" --num-cores "$CORES" >"$log" 2>&1 &
  local pid=$!
  PIDS+=("$pid")
  LAST_PID="$pid"
}

start_go_collector() {
  local config="$1"
  local log="$2"

  env GOMAXPROCS="$CORES" "$GO_COLLECTOR_BIN" --config "file:${config}" >"$log" 2>&1 &
  local pid=$!
  PIDS+=("$pid")
  LAST_PID="$pid"
}

wait_http() {
  local role="$1"
  local url="$2"
  local pid="$3"
  local log="$4"

  for _ in $(seq 1 80); do
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "$role exited before endpoint became ready. Log: $log" >&2
      tail -80 "$log" >&2 || true
      exit 1
    fi
    if curl -fsS "$url" >/dev/null 2>&1; then
      return
    fi
    sleep 0.25
  done

  echo "$role endpoint did not become ready at $url. Log: $log" >&2
  tail -80 "$log" >&2 || true
  exit 1
}

wait_rust_admin() {
  wait_http "$1" "http://127.0.0.1:${2}${RUST_METRICS_PATH}" "$3" "$4"
}

wait_go_admin() {
  wait_http "$1" "http://127.0.0.1:${2}${GO_METRICS_PATH}" "$3" "$4"
}

fetch_rust_metrics() {
  local port="$1"
  local path="$2"
  curl -fsS "http://127.0.0.1:${port}${RUST_METRICS_PATH}" -o "$path" 2>/dev/null || : >"$path"
}

fetch_go_metrics() {
  local port="$1"
  local path="$2"
  curl -fsS "http://127.0.0.1:${port}${GO_METRICS_PATH}" -o "$path" 2>/dev/null || : >"$path"
}

counter_value() {
  local path="$1"
  local metric="$2"
  awk -v metric="$metric" '
    $1 == metric || index($1, metric "{") == 1 ||
    $1 == metric "_total" || index($1, metric "_total{") == 1 { sum += $2 }
    END { printf "%.0f", sum + 0 }
  ' "$path"
}

wire_bytes_value() {
  local path="$1"
  local metric="$2"
  awk -v metric="$metric" '
    $1 == metric || index($1, metric "{") == 1 { sum += $2 }
    END { printf "%.0f", sum + 0 }
  ' "$path"
}

proc_jiffies() {
  local pid="$1"
  awk '{ print $14 + $15 + $16 + $17 }' "/proc/${pid}/stat"
}

proc_hwm_kib() {
  local pid="$1"
  awk '/^VmHWM:/ { print $2; found=1 } END { if (!found) print 0 }' "/proc/${pid}/status"
}

lo_bytes() {
  awk '$1 ~ /^lo:/ { print $2, $10 }' /proc/net/dev
}

seconds_between_ns() {
  awk -v start="$1" -v end="$2" 'BEGIN { printf "%.6f", (end - start) / 1000000000 }'
}

rate_per_sec() {
  awk -v delta="$1" -v seconds="$2" 'BEGIN { if (seconds > 0) printf "%.0f", delta / seconds; else print 0 }'
}

cpu_pct() {
  awk -v delta="$1" -v hz="$CLK_TCK" -v seconds="$2" \
    'BEGIN { if (seconds > 0) printf "%.1f", (delta / hz) / seconds * 100; else print "0.0" }'
}

mib() {
  awk -v kib="$1" 'BEGIN { printf "%.1f", kib / 1024 }'
}

mib_per_sec() {
  awk -v bytes="$1" -v seconds="$2" \
    'BEGIN { if (seconds > 0) printf "%.2f", bytes / 1048576 / seconds; else print "0.00" }'
}

go_collector_has_stef() {
  [[ -x "$GO_COLLECTOR_BIN" ]] || return 1
  "$GO_COLLECTOR_BIN" components 2>/dev/null | awk '
    $0 == "    - name: stef" { found=1 }
    END { exit found ? 0 : 1 }
  '
}

unsupported_reason() {
  local sut_impl="$1"
  local protocol="$2"

  if [[ "$protocol" == "stef" && "$SIGNAL_KIND" != "metrics" ]]; then
    echo "STEF receiver/exporter support is metrics-only in the current Rust and Go implementations."
    return 0
  fi

  if [[ "$sut_impl" == "go" && "$protocol" == "stef" ]] && ! go_collector_has_stef; then
    echo "GO_COLLECTOR_BIN does not include stef receiver/exporter components: ${GO_COLLECTOR_BIN}."
    return 0
  fi

  if [[ "$sut_impl" == "go" && ! -x "$GO_COLLECTOR_BIN" ]]; then
    echo "GO_COLLECTOR_BIN is not executable: ${GO_COLLECTOR_BIN}."
    return 0
  fi

  return 1
}

write_report_header() {
  cat >"$OUT_DIR/summary.md" <<MD
# Protocol Matrix Benchmark

- Signal kind: ${SIGNAL_KIND}
- Target ${ITEM_NAME}/s: ${TARGET_ITEMS_PER_SECOND}
- Expected generated ${ITEM_NAME}/s: ${EXPECTED_ITEMS_PER_SECOND}
- Target ${ITEM_NAME}/batch: ${TARGET_ITEMS_PER_BATCH}
- Actual ${ITEM_NAME}/batch: ${ITEMS_PER_BATCH}
- Wire compression: ${WIRE_COMPRESSION}
- STEF frame compression: ${STEF_COMPRESSION}
- OTAP Arrow payload compression: ${OTAP_ARROW_COMPRESSION}
- Cores per process: ${CORES}
- Settle/sample: ${SETTLE}s/${SAMPLE}s
MD

  if [[ "$SIGNAL_KIND" == "metrics" ]]; then
    cat >>"$OUT_DIR/summary.md" <<MD
- Metric objects/batch: ${METRIC_OBJECTS_PER_BATCH}
- Data points/metric: ${DATA_POINTS_PER_METRIC}
- Metric attributes/data point: ${NUM_METRIC_ATTRIBUTES}
MD
  else
    cat >>"$OUT_DIR/summary.md" <<MD
- Log body bytes: ${LOG_BODY_SIZE_BYTES}
- Log attributes/record: ${NUM_LOG_ATTRIBUTES}
MD
  fi

  cat >>"$OUT_DIR/summary.md" <<MD

Notes:

- Sender and backend are Rust df_engine processes. The SUT is selected by the
  SUT column.
- SUT/backend payload MiB/s is derived from Rust receiver 'request_bytes_total'
  or Go otelarrow 'otelcol_receiver_recv_wire_bytes_total' when available.
  Protocols without a comparable counter in this harness report '0.00'.
- loopback TX/RX MiB/s comes from '/proc/net/dev' for 'lo' over the sample
  window. It includes both local protocol hops and admin scrape overhead.

| SUT | Scenario | Status | Observed ${ITEM_NAME}/s | Sender CPU | SUT CPU | Backend CPU | Sender RSS MiB | SUT RSS MiB | Backend RSS MiB | SUT payload MiB/s | Backend payload MiB/s | loopback TX MiB/s | loopback RX MiB/s | Reason |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
MD

  cat >"$OUT_DIR/results.csv" <<CSV
sut,scenario,status,observed_items_per_sec,sender_cpu_pct,sut_cpu_pct,backend_cpu_pct,sender_rss_mib,sut_rss_mib,backend_rss_mib,sut_payload_mib_s,backend_payload_mib_s,loopback_tx_mib_s,loopback_rx_mib_s,reason
CSV
}

append_result() {
  local sut_impl="$1"
  local scenario="$2"
  local status="$3"
  local observed_rate="$4"
  local sender_cpu="$5"
  local sut_cpu="$6"
  local backend_cpu="$7"
  local sender_rss="$8"
  local sut_rss="$9"
  local backend_rss="${10}"
  local sut_payload="${11}"
  local backend_payload="${12}"
  local lo_tx="${13}"
  local lo_rx="${14}"
  local reason="${15}"

  printf '| %s | %s | %s | %s | %s%% | %s%% | %s%% | %s | %s | %s | %s | %s | %s | %s | %s |\n' \
    "$sut_impl" "$scenario" "$status" "$observed_rate" "$sender_cpu" "$sut_cpu" \
    "$backend_cpu" "$sender_rss" "$sut_rss" "$backend_rss" "$sut_payload" \
    "$backend_payload" "$lo_tx" "$lo_rx" "$reason" >>"$OUT_DIR/summary.md"

  local csv_reason="${reason//\"/\"\"}"
  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,"%s"\n' \
    "$sut_impl" "$scenario" "$status" "$observed_rate" "$sender_cpu" "$sut_cpu" \
    "$backend_cpu" "$sender_rss" "$sut_rss" "$backend_rss" "$sut_payload" \
    "$backend_payload" "$lo_tx" "$lo_rx" "$csv_reason" >>"$OUT_DIR/results.csv"
}

append_skip() {
  local sut_impl="$1"
  local scenario="$2"
  local reason="$3"

  append_result "$sut_impl" "$scenario" "skipped" "0" "0.0" "0.0" "0.0" \
    "0.0" "0.0" "0.0" "0.00" "0.00" "0.00" "0.00" "$reason"
  printf "%-8s %-16s skipped: %s\n" "$sut_impl" "$scenario" "$reason"
}

run_scenario() {
  local sut_impl="$1"
  local protocol="$2"
  local label="${protocol^^} -> ${protocol^^}"
  local reason

  if reason="$(unsupported_reason "$sut_impl" "$protocol")"; then
    append_skip "$sut_impl" "$label" "$reason"
    return
  fi

  local scenario_dir="$OUT_DIR/${sut_impl}-${protocol}"
  mkdir -p "$scenario_dir"

  local sender_config="$scenario_dir/sender.yaml"
  local sut_config="$scenario_dir/sut.yaml"
  local backend_config="$scenario_dir/backend.yaml"
  write_sender_config "$protocol" "$sender_config"
  write_backend_config "$protocol" "$backend_config"

  if [[ "$sut_impl" == "rust" ]]; then
    write_rust_sut_config "$protocol" "$sut_config"
  elif [[ "$sut_impl" == "go" ]]; then
    write_go_sut_config "$protocol" "$sut_config"
    if ! "$GO_COLLECTOR_BIN" validate --config "file:${sut_config}" >"$scenario_dir/sut-validate.log" 2>&1; then
      echo "Go Collector config validation failed for $label. Log: $scenario_dir/sut-validate.log" >&2
      tail -80 "$scenario_dir/sut-validate.log" >&2 || true
      exit 1
    fi
  else
    echo "unknown SUT implementation: $sut_impl" >&2
    exit 1
  fi

  echo "Running $sut_impl $label ..."

  local backend_pid sut_pid sender_pid
  start_rust_engine "$backend_config" "$scenario_dir/backend.log"
  backend_pid="$LAST_PID"
  wait_rust_admin backend 8082 "$backend_pid" "$scenario_dir/backend.log"

  if [[ "$sut_impl" == "rust" ]]; then
    start_rust_engine "$sut_config" "$scenario_dir/sut.log"
    sut_pid="$LAST_PID"
    wait_rust_admin sut 8081 "$sut_pid" "$scenario_dir/sut.log"
  else
    start_go_collector "$sut_config" "$scenario_dir/sut.log"
    sut_pid="$LAST_PID"
    wait_go_admin sut 8081 "$sut_pid" "$scenario_dir/sut.log"
  fi

  start_rust_engine "$sender_config" "$scenario_dir/sender.log"
  sender_pid="$LAST_PID"
  wait_rust_admin sender 8080 "$sender_pid" "$scenario_dir/sender.log"

  sleep "$SETTLE"

  fetch_rust_metrics 8080 "$scenario_dir/sender-start.prom"
  if [[ "$sut_impl" == "rust" ]]; then
    fetch_rust_metrics 8081 "$scenario_dir/sut-start.prom"
  else
    fetch_go_metrics 8081 "$scenario_dir/sut-start.prom"
  fi
  fetch_rust_metrics 8082 "$scenario_dir/backend-start.prom"

  local sender_cpu_start sut_cpu_start backend_cpu_start
  local sender_rss_start sut_rss_start backend_rss_start
  sender_cpu_start="$(proc_jiffies "$sender_pid")"
  sut_cpu_start="$(proc_jiffies "$sut_pid")"
  backend_cpu_start="$(proc_jiffies "$backend_pid")"
  sender_rss_start="$(proc_hwm_kib "$sender_pid")"
  sut_rss_start="$(proc_hwm_kib "$sut_pid")"
  backend_rss_start="$(proc_hwm_kib "$backend_pid")"

  local lo_start_rx lo_start_tx
  read -r lo_start_rx lo_start_tx < <(lo_bytes)
  local start_ns
  start_ns="$(date +%s%N)"

  sleep "$SAMPLE"

  local end_ns
  end_ns="$(date +%s%N)"
  local lo_end_rx lo_end_tx
  read -r lo_end_rx lo_end_tx < <(lo_bytes)

  local sender_cpu_end sut_cpu_end backend_cpu_end
  sender_cpu_end="$(proc_jiffies "$sender_pid")"
  sut_cpu_end="$(proc_jiffies "$sut_pid")"
  backend_cpu_end="$(proc_jiffies "$backend_pid")"

  fetch_rust_metrics 8080 "$scenario_dir/sender-end.prom"
  if [[ "$sut_impl" == "rust" ]]; then
    fetch_rust_metrics 8081 "$scenario_dir/sut-end.prom"
  else
    fetch_go_metrics 8081 "$scenario_dir/sut-end.prom"
  fi
  fetch_rust_metrics 8082 "$scenario_dir/backend-end.prom"

  local seconds
  seconds="$(seconds_between_ns "$start_ns" "$end_ns")"

  local produced_start produced_end observed_rate
  produced_start="$(counter_value "$scenario_dir/sender-start.prom" "$ITEM_COUNTER")"
  produced_end="$(counter_value "$scenario_dir/sender-end.prom" "$ITEM_COUNTER")"
  observed_rate="$(rate_per_sec "$(( produced_end - produced_start ))" "$seconds")"

  local sut_payload_start sut_payload_end backend_payload_start backend_payload_end
  if [[ "$sut_impl" == "go" ]]; then
    sut_payload_start="$(wire_bytes_value "$scenario_dir/sut-start.prom" otelcol_receiver_recv_wire_bytes_total)"
    sut_payload_end="$(wire_bytes_value "$scenario_dir/sut-end.prom" otelcol_receiver_recv_wire_bytes_total)"
  else
    sut_payload_start="$(counter_value "$scenario_dir/sut-start.prom" request_bytes)"
    sut_payload_end="$(counter_value "$scenario_dir/sut-end.prom" request_bytes)"
  fi
  backend_payload_start="$(counter_value "$scenario_dir/backend-start.prom" request_bytes)"
  backend_payload_end="$(counter_value "$scenario_dir/backend-end.prom" request_bytes)"

  local sender_cpu sut_cpu backend_cpu
  sender_cpu="$(cpu_pct "$(( sender_cpu_end - sender_cpu_start ))" "$seconds")"
  sut_cpu="$(cpu_pct "$(( sut_cpu_end - sut_cpu_start ))" "$seconds")"
  backend_cpu="$(cpu_pct "$(( backend_cpu_end - backend_cpu_start ))" "$seconds")"

  local sender_rss sut_rss backend_rss
  sender_rss="$(mib "$(printf '%s\n%s\n' "$sender_rss_start" "$(proc_hwm_kib "$sender_pid")" | sort -n | tail -1)")"
  sut_rss="$(mib "$(printf '%s\n%s\n' "$sut_rss_start" "$(proc_hwm_kib "$sut_pid")" | sort -n | tail -1)")"
  backend_rss="$(mib "$(printf '%s\n%s\n' "$backend_rss_start" "$(proc_hwm_kib "$backend_pid")" | sort -n | tail -1)")"

  local sut_payload_mib_s backend_payload_mib_s lo_tx_mib_s lo_rx_mib_s
  sut_payload_mib_s="$(mib_per_sec "$(( sut_payload_end - sut_payload_start ))" "$seconds")"
  backend_payload_mib_s="$(mib_per_sec "$(( backend_payload_end - backend_payload_start ))" "$seconds")"
  lo_tx_mib_s="$(mib_per_sec "$(( lo_end_tx - lo_start_tx ))" "$seconds")"
  lo_rx_mib_s="$(mib_per_sec "$(( lo_end_rx - lo_start_rx ))" "$seconds")"

  append_result "$sut_impl" "$label" "measured" "$observed_rate" "$sender_cpu" "$sut_cpu" \
    "$backend_cpu" "$sender_rss" "$sut_rss" "$backend_rss" "$sut_payload_mib_s" \
    "$backend_payload_mib_s" "$lo_tx_mib_s" "$lo_rx_mib_s" ""

  printf "%-8s %-16s %12s %s/s  CPU sender/sut/backend %6s%% %6s%% %6s%%  loopback TX %8s MiB/s\n" \
    "$sut_impl" "$label" "$observed_rate" "$ITEM_NAME" "$sender_cpu" "$sut_cpu" \
    "$backend_cpu" "$lo_tx_mib_s"

  cleanup
  sleep 2
}

write_report_header

echo "Output directory: $OUT_DIR"
echo "Expected generated rate: ${EXPECTED_ITEMS_PER_SECOND} ${ITEM_NAME}/s"
if [[ "$SIGNAL_KIND" == "metrics" ]]; then
  echo "Batch shape: ${METRIC_OBJECTS_PER_BATCH} Sum/Gauge metrics x ${DATA_POINTS_PER_METRIC} data points = ${ITEMS_PER_BATCH} data points/batch"
else
  echo "Batch shape: ${ITEMS_PER_BATCH} logs/batch, body=${LOG_BODY_SIZE_BYTES}B, attrs=${NUM_LOG_ATTRIBUTES}"
fi
echo ""

for sut_impl in $SUT_IMPLS; do
  for protocol in $PROTOCOLS; do
    run_scenario "$sut_impl" "$protocol"
  done
done

echo ""
echo "Summary: $OUT_DIR/summary.md"
echo "CSV:     $OUT_DIR/results.csv"
