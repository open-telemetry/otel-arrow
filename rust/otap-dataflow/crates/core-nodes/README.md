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
        batch_processor/
        debug_processor/
        delay_processor/
        fanout_processor/
        filter_processor/
        signal_type_router/
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
| batch_processor | `urn:otel:processor:batch` | `src/processors/batch_processor/` |
| debug_processor | `urn:otel:processor:debug` | `src/processors/debug_processor/` |
| delay_processor | `urn:otel:processor:delay` | `src/processors/delay_processor/` |
| fanout_processor | `urn:otel:processor:fanout` | `src/processors/fanout_processor/` |
| filter_processor | `urn:otel:processor:filter` | `src/processors/filter_processor/` |
| signal_type_router | `urn:otel:processor:type_router` | `src/processors/signal_type_router/` |

#### batch_processor

- Batches OTAP and OTLP payloads by size and/or time-based flush criteria
- Supports preserve/force format modes and ack-aware request tracking
- Useful for throughput optimization and controlled downstream pressure

#### debug_processor

- Outputs received signals with configurable filtering, sampling, and verbosity
- Supports multiple output modes (console, file, outports) and marshaling
  formats
- Useful for understanding data flow through pipelines

#### delay_processor

- Introduces artificial delays between signal processing
- Configured via humantime duration strings
- Used for testing timeout handling and rate control

#### fanout_processor

- Clones incoming data to multiple downstream outputs
- Supports parallel or sequential delivery modes with configurable ack policies
- Supports fallback chains and timeout handling per destination

#### filter_processor

- Filters OTAP signals according to configured trace/log filter rules
- Tracks consumed and filtered signal metrics for telemetry
- Useful for reducing data volume before downstream processing

#### signal_type_router

- Routes signals by type (logs, metrics, traces) to named output ports
- Falls back to default routing when a type-specific port is not connected
- Exposes per-signal routing and drop telemetry counters

### Receivers

| Node | URN | Module |
| ---- | --- | ------ |
| fake_data_generator | `urn:otel:receiver:traffic_generator` | `src/receivers/fake_data_generator/` |

#### fake_data_generator

- Generates synthetic OTAP/OTLP signals for testing and benchmarking
- Configurable signal generation strategies and volume constraints
- Includes support for pregenerated, dynamic, and rate-based signal generation
