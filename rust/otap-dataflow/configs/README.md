# Pipeline Configuration Examples

This directory contains example pipeline configurations for the OTAP dataflow engine.

## Available Configurations

### `otlp-basic.json`

A basic OTLP pipeline configuration that:

- Receives OTLP data on `127.0.0.1:4317`
- Exports to `http://127.0.0.1:1235`
- Uses round-robin dispatch strategy
- Default channel sizes of 100

## Usage

You can use these configurations with the mini_collector example:

```bash
# Use the default basic OTLP configuration
cargo run --example mini_collector

# Use a specific configuration
cargo run --example mini_collector -- -p configs/otlp-otlp.json

# Combine with custom core count
cargo run --example mini_collector -- -p configs/otlp-otlp.json --num-cores 4
```

## Creating Custom Configurations

Pipeline configurations follow this JSON structure:

```json5
{
  "type": "otlp",
  "settings": {
    "default_control_channel_size": 100,
    "default_pdata_channel_size": 100
  },
  "nodes": {
    "node_name": {
      "kind": "receiver|processor|exporter",
      "plugin_urn": "urn:plugin:identifier",
      "config": {
        // Node-specific configuration
      },
      "out_ports": {
        "port_name": {
          "destinations": ["target_node"],
          "dispatch_strategy": "round_robin|broadcast|..."
        }
      }
    }
  }
}
```

Add your custom configuration files to this directory and reference them using
the `-p` parameter.
