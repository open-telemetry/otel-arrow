# Core Nodes

This crate contains essential node implementations (exporters, receivers, and
processors) for the OTAP Dataflow Engine. Components are organized by category
with per-component subfolders.

## Architecture

Each component lives in its own subfolder within a category:

    src/
      exporters/
        mod.rs (category exports)
        console_exporter/
        error_exporter/
        noop_exporter/
      processors/
        mod.rs (category exports)
        debug_processor/
        delay_processor/
      receivers/
        mod.rs (category exports)
        fake_data_generator/
      lib.rs

## Components

### Exporters

| Node | URN | Module |
| ---- | --- | ------ |
| console_exporter | `urn:otel:exporter:console` | `src/exporters/console_exporter/` |
| error_exporter | `urn:otel:exporter:error` | `src/exporters/error_exporter/` |
| noop_exporter | `urn:otel:exporter:noop` | `src/exporters/noop_exporter/` |

#### console_exporter

- Hierarchical ANSI-colored console output for OTLP logs, metrics, and traces
- Useful for debugging and visual inspection of data

#### error_exporter

- Test exporter that NACKs all messages with a configurable error message
- Used primarily in testing and error path validation

#### noop_exporter

- Placeholder exporter that ACKs all messages without processing
- Lightweight for performance testing and pipeline validation

### Processors

| Node | URN | Module |
| ---- | --- | ------ |
| debug_processor | `urn:otel:processor:debug` | `src/processors/debug_processor/` |
| delay_processor | `urn:otel:processor:delay` | `src/processors/delay_processor/` |

#### debug_processor

- Outputs received signals with configurable filtering, sampling, and verbosity
- Supports multiple output modes (console, file, outports) and marshaling
  formats
- Useful for understanding data flow through pipelines

#### delay_processor

- Introduces artificial delays between signal processing
- Configured via humantime duration strings
- Used for testing timeout handling and rate control

### Receivers

| Node | URN | Module |
| ---- | --- | ------ |
| fake_data_generator | `urn:otel:receiver:traffic_generator` | `src/receivers/fake_data_generator/` |

#### fake_data_generator

- Generates synthetic OTAP/OTLP signals for testing and benchmarking
- Configurable signal generation strategies and volume constraints
- Includes support for pregenerated, dynamic, and rate-based signal generation
