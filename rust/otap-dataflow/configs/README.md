# Pipeline Configuration Examples

This directory contains example pipeline configurations for the OTAP dataflow engine.

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
- Includes `service.telemetry` configuration with console metrics export

### `fake-debug-out-port.yaml`

Demonstrates multiple output ports:

- Generates fake data -> debug processor with multiple out ports -> noop exporter

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

## Usage

You can use these configurations with the following CLI command:

```bash
# Use a specific configuration
cargo run -- -p configs/otlp-otlp.yaml

# Combine with custom core count
cargo run -- -p configs/otlp-otlp.yaml --num-cores 4
```
