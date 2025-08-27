# Pipeline Configuration Examples

This directory contains example pipeline configurations for the OTAP dataflow engine.

Note: These configurations are based on the native OTAP dataflow engine
configuration model, which is a superset of the Go Collector configuration
model. Support for the Go Collector YAML format is planned for the future.

## Available Configurations

### `otap-otap.yaml`

A basic OTAP pipeline configuration that:

- Receives OTAP traffic on `127.0.1:4317`
- Exports OTAP traffic to `http://127.0.1:1235`
- Default channel sizes of 100

### `otap-otlp.yaml`

A basic OTAP to OTLP pipeline configuration that:

- Receives OTAP traffic on `127.0.0.1:4317`
- Exports OTLP traffic to `http://127.0.0.1:1235`
- Default channel sizes of 100

### `otlp-otap.yaml`

A basic OTLP to OTAP pipeline configuration that:

- Receives OTLP traffic on `127.0.0.1:4317`
- Exports OTAP traffic to `http://127.0.0.1:1235`
- Default channel sizes of 100

### `otlp-otlp.yaml`

A basic OTLP pipeline configuration that:

- Receives OTLP traffic on `127.0.0.1:4317`
- Exports OTLP traffic to `http://127.0.0.1:1235`
- Default channel sizes of 100

### `otap-perf.yaml`

A pipeline configuration to measure performance metrics, which:

- Receives OTAP traffic on `127.0.0.1:4317`
- Measures and exports performance metrics
- Default channel sizes of 100

### `otlp-perf.yaml`

A pipeline configuration to measure performance metrics, which:

- Receives OTLP traffic on `127.0.0.1:4317`
- Measures and exports performance metrics
- Default channel sizes of 100

## Usage

You can use these configurations with the following CLI command:

```bash
# Use a specific configuration
cargo run -- -p configs/otlp-otlp.yaml

# Combine with custom core count
cargo run -- -p configs/otlp-otlp.yaml --num-cores 4
```
