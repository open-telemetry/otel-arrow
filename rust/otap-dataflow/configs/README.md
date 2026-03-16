# Engine Configuration Examples

This directory contains example engine configurations for the OTAP dataflow engine.
Each file uses `version: otel_dataflow/v1` at the root.

Note: These configurations are based on the native OTAP dataflow engine
configuration model, which is a superset of the Go Collector configuration
model. Support for the Go Collector YAML format is planned for the future.

## Configuration Formats

The engine supports two configuration formats, selected by CLI flag:

### Pipeline Configuration (`-p` / `--pipeline`)

A **pipeline configuration** defines a single pipeline. It is a flat
YAML file with top-level keys such as `settings`, `quota`, `nodes`,
and `service`. The engine wraps it automatically into a default
pipeline group for execution.

```yaml
settings:
  default_pdata_channel_size: 100
quota:
  core_allocation:
    type: core_count
    count: 4
nodes:
  my_receiver:
    kind: receiver
    plugin_urn: "urn:otel:otap:receiver"
    out_ports:
      out_port:
        destinations:
          - my_exporter
        dispatch_strategy: round_robin
    config:
      listening_addr: "127.0.0.1:4327"
  my_exporter:
    kind: exporter
    plugin_urn: "urn:otel:noop:exporter"
    config:
```

Usage:

```bash
cargo run --release -- -p configs/otap-noop.yaml
```

All YAML files in the root of this directory use this format.

### Engine Configuration (`-c` / `--config`)

An **engine configuration** defines the full engine hierarchy: global
engine settings and one or more pipeline groups, each containing one
or more pipelines. Each pipeline has the same structure as a pipeline
configuration.

```yaml
settings:
  http_admin:
    bind_address: 127.0.0.1:8085
pipeline_groups:
  my_group:
    pipelines:
      pipeline_a:
        nodes:
          # ... same node structure as above
      pipeline_b:
        nodes:
          # ...
```

Usage:

```bash
cargo run --release -- -c configs/engine-conf/continuous_benchmark.yaml
```

Engine configuration files are in the [`engine-conf/`](engine-conf/)
subdirectory.

For the full configuration schema, see the
[config crate README](../crates/config/README.md).

## Available Pipeline Configurations

### `fake-batch-debug-noop.yaml`

Demonstrates the batch processor:

- Generates fake data -> batch processor -> debug processor -> noop exporter

### `fake-debug-noop-telemetry.yaml`

A basic pipeline with telemetry export enabled:

- Generates fake data -> debug processor -> noop exporter
- Includes `engine.telemetry` configuration with console metrics export

### `fake-debug-output-ports.yaml`

Demonstrates multiple output ports:

- Generates fake data -> debug processor with multiple output ports -> noop exporter

### `fake-filter-debug-noop.yaml`

Demonstrates the filter processor:

- Generates fake data -> filter processor -> debug processor -> noop exporter

### `fake-transform-debug-noop.yaml`

Demonstrate using the transform processor to transform data

- Generates fake data -> debug -> transform -> debug -> noop exporter

The input data can be viewed at /tmp/debug1.log and the transformed output at
/tmp/debug2.log

### `fake-otap.yaml`

Generates fake data and exports via OTAP:

- Generates fake data -> OTAP exporter to `http://127.0.0.1:4318`

### `fake-otlp.yaml`

Generates fake data and exports via OTLP:

- Generates fake data -> OTLP exporter to `http://127.0.0.1:4317`

### `fake-parquet.yaml`

Generates fake data and exports to Parquet files:

- Generates fake data -> Parquet exporter to `/tmp`

### `fake-perf.yaml`

Generates fake data with performance metrics:

- Generates fake data -> performance exporter
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

### `fake-multi-tenant-perf.yaml`

Generates mixed-tenant traffic using weighted resource attribute rotation:

- Uses `data_source: static` with two resource attribute sets (`tenant.id:
  prod` and `tenant.id: ppe`) weighted 3:1, producing a 75% / 25% batch split
  per  pipeline.
- Generates fake data -> performance exporter
- View metrics at: `http://127.0.0.1:8080/telemetry/metrics?format=prometheus&reset=false`

The `resource_attributes` field accepts three forms:

| Form | Description |
| ---- | ----------- |
| Single map | All batches carry the same attributes (weight 1) |
| List of maps | Equal round-robin rotation across entries (weight 1 each) |
| List of weighted entries (`attrs` + `weight`) | Each entry receives batches proportional to its weight |

> **Note:** `resource_attributes` only applies to `data_source: static`.
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
echo "<134>$(date '+%b %d %H:%M:%S') testhost testtag: Test message" | nc -u -w1 127.0.0.1 5140
```

For sustained load testing, see the [load generator](../../tools/pipeline_perf_test/load_generator/readme.md):

```bash
cd tools/pipeline_perf_test/load_generator
python loadgen.py --load-type syslog --syslog-server 127.0.0.1 --syslog-port 5140 --syslog-transport udp --duration 15
```

> **Note:** The default `syslog-perf.yaml` config only enables UDP.
> To also accept TCP, add a `tcp` section under `protocol` in the config.

## Usage

You can use these configurations with the following CLI commands:

```bash
# Run a single pipeline configuration
cargo run -- -p configs/otlp-otlp.yaml

# Combine with custom core count
cargo run -- -p configs/otlp-otlp.yaml --num-cores 4

# Run an engine configuration (multiple pipeline groups)
cargo run -- -c configs/engine-conf/continuous_benchmark.yaml

# Validate a configuration without starting the engine
cargo run -- -p configs/otlp-otlp.yaml --validate-and-exit
```
