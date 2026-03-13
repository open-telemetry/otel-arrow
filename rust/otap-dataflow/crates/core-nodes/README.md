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
        attributes_processor/
        batch_processor/
        content_router/
        debug_processor/
        delay_processor/
        durable_buffer_processor/
        fanout_processor/
        filter_processor/
        retry_processor/
        signal_type_router/
        transform_processor/
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
| attributes_processor | `urn:otel:processor:attribute` | `src/processors/attributes_processor/` |
| batch_processor | `urn:otel:processor:batch` | `src/processors/batch_processor/` |
| content_router | `urn:otel:processor:content_router` | `src/processors/content_router/` |
| debug_processor | `urn:otel:processor:debug` | `src/processors/debug_processor/` |
| delay_processor | `urn:otel:processor:delay` | `src/processors/delay_processor/` |
| durable_buffer_processor | `urn:otel:processor:durable_buffer` | `src/processors/durable_buffer_processor/` |
| fanout_processor | `urn:otel:processor:fanout` | `src/processors/fanout_processor/` |
| filter_processor | `urn:otel:processor:filter` | `src/processors/filter_processor/` |
| retry_processor | `urn:otel:processor:retry` | `src/processors/retry_processor/` |
| signal_type_router | `urn:otel:processor:type_router` | `src/processors/signal_type_router/` |
| transform_processor | `urn:otel:processor:transform` | `src/processors/transform_processor/` |

#### attributes_processor

- Adds, updates, renames, and deletes OpenTelemetry attributes
- Works directly on OTAP-native data structures for low-copy mutations

#### content_router

- Routes telemetry to named output ports based on resource attribute values
- Supports default routing and mixed-batch validation

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

#### durable_buffer_processor

- Persists telemetry to WAL and segment storage before forwarding downstream
- Supports crash recovery, retry backoff, and bounded retention policies

#### fanout_processor

- Clones incoming data to multiple downstream outputs
- Supports parallel or sequential delivery modes with configurable ack policies
- Supports fallback chains and timeout handling per destination

#### filter_processor

- Filters OTAP signals according to configured trace/log filter rules
- Tracks consumed and filtered signal metrics for telemetry
- Useful for reducing data volume before downstream processing

#### retry_processor

- Retries downstream delivery with exponential backoff using Ack/Nack handling
- Preserves retry state in call data instead of external durable state

#### signal_type_router

- Routes signals by type (logs, metrics, traces) to named output ports
- Falls back to default routing when a type-specific port is not connected
- Exposes per-signal routing and drop telemetry counters

#### transform_processor

- Applies KQL or OPL transformations to OTAP batches via the query engine
- Supports routed outputs while preserving upstream Ack/Nack semantics

### Receivers

| Node | URN | Module |
| ---- | --- | ------ |
| fake_data_generator | `urn:otel:receiver:traffic_generator` | `src/receivers/fake_data_generator/` |

#### fake_data_generator

- Generates synthetic OTAP/OTLP signals for testing and benchmarking
- Configurable signal generation strategies and volume constraints
- Includes support for pregenerated, dynamic, and rate-based signal generation
