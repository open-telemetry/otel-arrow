# Engine Configuration Examples

This directory contains example engine configurations for the OTAP dataflow engine.
Each file uses `version: otel_dataflow/v1` at the root.

Note: These configurations are based on the native OTAP dataflow engine
configuration model, which is a superset of the Go Collector configuration
model. Support for the Go Collector YAML format is planned for the future.

## Available Configurations

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

To send test syslog messages:

```bash
# Send a single syslog message
echo "<134>$(date '+%b %d %H:%M:%S') testhost testtag: Test message" | nc -u -w1 127.0.0.1 5140

# Send multiple messages
for i in {1..100}; do
  echo "<134>$(date '+%b %d %H:%M:%S') testhost testtag: Test message #$i" | nc -u -w1 127.0.0.1 5140
done

# Send CEF format message
echo "<134>$(date '+%b %d %H:%M:%S') testhost CEF:0|Security|IDS|1.0|100|Test Event|5|src=192.168.1.100 dst=10.0.0.50" | nc -u -w1 127.0.0.1 5140
```

## Usage

You can use these configurations with the following CLI command:

```bash
# Use a specific configuration
cargo run -- --config configs/otlp-otlp.yaml

# Validate a configuration without starting the engine
cargo run -- --config configs/otlp-otlp.yaml --validate-and-exit
```
