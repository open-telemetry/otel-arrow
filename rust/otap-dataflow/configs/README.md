# Engine Configuration Examples

This directory contains example engine configurations for the OTAP dataflow engine.
Each file uses `version: otel_dataflow/v1` at the root.

If you are learning how to write these files, start with
[Configuration](../docs/configuration.md).

Note: These configurations are based on the native OTAP dataflow engine
configuration model, which is intentionally distinct from the OpenTelemetry Collector
YAML model. Support for the OTel Collector YAML format will be explored in the
future.

## Available Configurations

### `internal-telemetry.yaml`

Routes the engine's own logs and metrics through the Internal Telemetry System:

- Uses the dedicated engine observability pipeline
- Receives internal OTLP logs and metrics with `receiver:internal_telemetry`
- Collects metric snapshots every second and emits them every two seconds
- Applies metric views for engine, pipeline, and flow metrics
- Routes logs to the console exporter and decoded metrics to the debug processor

Validate the configuration, then run the demo on one core and let it produce
at least three metric batches:

```bash
cargo run -- --config configs/internal-telemetry.yaml --validate-and-exit
cargo run -- --config configs/internal-telemetry.yaml --num-cores 1 \
  2>&1 | tee /tmp/its-metrics.log
```

The detailed metric output should contain the viewed stream names
`process_memory_usage`, `process_cpu_utilization`, `process_uptime`,
`processor_incoming_items`, `processor_outgoing_items`, and
`processing_duration`. Stop the process with `Ctrl-C` after inspection, then
list the viewed streams with:

```bash
rg 'Name: (process_|processor_|processing_)' /tmp/its-metrics.log
```

### `trafficgen-batch-debug-noop.yaml`

Demonstrates the batch processor:

- Generates synthetic traffic -> batch processor -> debug processor -> noop exporter

### `trafficgen-debug-noop-telemetry.yaml`

A basic pipeline with telemetry export enabled:

- Generates synthetic traffic -> debug processor -> noop exporter
- Routes internal metrics through the engine observability pipeline to an OTLP
  gRPC exporter. Logs retain the default asynchronous console behavior.

### `trafficgen-filter-debug-noop.yaml`

Demonstrates the filter processor:

- Generates synthetic traffic -> filter processor -> debug processor -> noop exporter

### `trafficgen-metric-filter-debug-noop.yaml`

Demonstrates metric-name filtering:

- Generates synthetic metrics -> filter processor by metric name -> debug processor
  -> noop exporter

### `trafficgen-transform-debug-noop.yaml`

Demonstrate using the transform processor to transform data

- Generates synthetic traffic -> debug -> transform -> debug -> noop exporter

The input data can be viewed at /tmp/debug1.log and the transformed output at
/tmp/debug2.log

### `trafficgen-otap.yaml`

Generates synthetic traffic and exports via OTAP:

- Generates synthetic traffic -> OTAP exporter to `http://127.0.0.1:4318`

### `trafficgen-otlp.yaml`

Generates synthetic traffic and exports via OTLP:

- Generates synthetic traffic -> OTLP exporter to `http://127.0.0.1:4317`

### `trafficgen-parquet.yaml`

Generates synthetic traffic and exports to Parquet files:

- Generates synthetic traffic -> Parquet exporter to `/tmp`

Parquet exporter configs can include an optional `retry` block for cloud-backed
object stores. Any omitted fields use the `object_store` defaults.

```yaml
retry:
  max_retries: 10
  init_backoff: "200ms"
  max_backoff: "30s"
  backoff_base: 2.0
  retry_timeout: "2min"
```

This configures the `object_store` layer request retry loop for transient
storage requests. Local file storage accepts valid retry settings but ignores
them; invalid retry values are still rejected during config validation. It does
not replay consumed Parquet writers after `AsyncArrowWriter::close` fails, and
it is separate from the retry processor's whole-batch redelivery policy.

### `trafficgen-perf.yaml`

Generates synthetic traffic with performance metrics:

- Generates synthetic traffic -> performance exporter
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

### `trafficgen-multi-tenant-perf.yaml`

Generates mixed-tenant traffic using weighted resource attribute rotation:

- Uses `data_source: synthetic` with two resource attribute sets (`tenant.id:
prod` and `tenant.id: ppe`) weighted 3:1, producing a 75% / 25% batch split
  per pipeline.
- Generates synthetic traffic -> performance exporter
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

The `resource_attributes` field accepts three forms:

- Single map: all batches carry the same attributes (weight 1).
- List of maps: equal round-robin rotation across entries (weight 1 each).
- List of weighted entries (`attrs` + `weight`): each entry receives batches
  proportional to its weight.

> **Note:** `resource_attributes` only applies to `data_source: synthetic`.
> With `generation_strategy: pre_generated`, only the first attribute set is used.

### `otap-otap.yaml`

A basic OTAP pipeline configuration:

- Receives OTAP traffic on `127.0.0.1:4317`
- Exports OTAP traffic to `http://127.0.0.1:1235`

### `otap-otlp.yaml`

OTAP to OTLP protocol conversion:

- Receives OTAP traffic on `127.0.0.1:4317`
- Exports OTLP traffic to `http://127.0.0.1:1235`

### `otap-perf.yaml`

OTAP receiver with performance metrics:

- Receives OTAP traffic on `127.0.0.1:4317`
- Measures and exports performance metrics
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

### `otlp-otap.yaml`

OTLP to OTAP protocol conversion:

- Receives OTLP traffic on `127.0.0.1:4317`
- Exports OTAP traffic to `http://127.0.0.1:1235`

### `otlp-otlp.yaml`

A basic OTLP pipeline configuration:

- Receives OTLP traffic on `127.0.0.1:4317`
- Exports OTLP traffic to `http://127.0.0.1:1235`

### `otlp-http-otlp.yaml`

OTLP receiver over both protocols:

- Receives OTLP/gRPC on `127.0.0.1:4317`
- Receives OTLP/HTTP on `127.0.0.1:4318`
- Exports OTLP/gRPC traffic to `http://127.0.0.1:4319`

### `otlp-grpc-http-forward.yaml`

OTLP forwarding proxy with separate gRPC and HTTP pipelines:

- Receives OTLP/gRPC on `127.0.0.1:4315` and forwards to `http://127.0.0.1:4317`
- Receives OTLP/HTTP on `127.0.0.1:4316` and forwards to `http://127.0.0.1:4318`

Note: In this configuration, the pipeline does not decode or
encode OTLP messages; they are simply forwarded from one port
to another.

### `otlp-perf.yaml`

OTLP receiver with performance metrics:

- Receives OTLP traffic on `127.0.0.1:4317`
- Measures and exports performance metrics
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

### `syslog-perf.yaml`

Syslog/CEF receiver with performance metrics:

- Receives syslog messages on UDP `0.0.0.0:5140`
- Measures and exports performance metrics
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

To send a quick test message (UDP):

```bash
echo "<134>$(date '+%b %d %H:%M:%S') testhost testtag: Test message" \
  | nc -u -w1 127.0.0.1 5140
```

For sustained load testing, see the
[load generator](../../tools/pipeline_perf_test/load_generator/readme.md):

```bash
cd tools/pipeline_perf_test/load_generator
python loadgen.py \
  --load-type syslog \
  --syslog-server 127.0.0.1 \
  --syslog-port 5140 \
  --syslog-transport udp \
  --duration 15
```

> **Note:** The default `syslog-perf.yaml` config only enables UDP.
> To also accept TCP, add a `tcp` section under `protocol` in the config.

### `opamp-controller-extension.yaml`

Example demonstrating how to configure the Dataflow Engine to receive its
configuration from a remote OpAMP server. See the
[documentation](../crates/controller/src/extension/opamp/README.md)
for more details about how to run this example.

## Usage

You can use these configurations with the following CLI command:

```bash
# Use a specific configuration (bare path)
cargo run -- --config configs/otlp-otlp.yaml

# Explicit file: URI
cargo run -- --config file:configs/otlp-otlp.yaml

# Load config from an environment variable
export MY_CONFIG=$(cat configs/otlp-otlp.yaml)
cargo run -- --config env:MY_CONFIG

# Validate a configuration without starting the engine
cargo run -- --config configs/otlp-otlp.yaml --validate-and-exit
```

The `--config` argument supports `file:`, `env:`, and bare path forms.
See `src/README.md` for the full URI reference.
