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
        otap_exporter/
        otlp_grpc_exporter/
        otlp_http_exporter/
        parquet_exporter/
        perf_exporter/
        topic_exporter/
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
        log_sampling_processor/
        retry_processor/
        temporal_reaggregation_processor/
        signal_type_router/
        transform_processor/
      receivers/
        mod.rs (category exports)
        fake_data_generator/
        internal_telemetry_receiver/
        otap_receiver/
        otlp_receiver/
        syslog_cef_receiver/
        topic_receiver/
      lib.rs

## Components

### Exporters

<!-- markdownlint-disable MD013 -->
| Node | URN | Module |
| ---- | --- | ------ |
| console_exporter | `urn:otel:exporter:console` | `src/exporters/console_exporter/` |
| error_exporter | `urn:otel:exporter:error` | `src/exporters/error_exporter/` |
| noop_exporter | `urn:otel:exporter:noop` | `src/exporters/noop_exporter/` |
| otap_exporter | `urn:otel:exporter:otap` | `src/exporters/otap_exporter/` |
| otlp_grpc_exporter | `urn:otel:exporter:otlp_grpc` | `src/exporters/otlp_grpc_exporter/` |
| otlp_http_exporter | `urn:otel:exporter:otlp_http` | `src/exporters/otlp_http_exporter/` |
| parquet_exporter | `urn:otel:exporter:parquet` | `src/exporters/parquet_exporter/` |
| perf_exporter | `urn:otel:exporter:perf` | `src/exporters/perf_exporter/` |
| topic_exporter | `urn:otel:exporter:topic` | `src/exporters/topic_exporter/` |
<!-- markdownlint-enable MD013 -->

#### console_exporter

- Hierarchical ANSI-colored console output for OTLP logs, metrics, and traces
- Useful for debugging and visual inspection of data

#### error_exporter

- Test exporter that NACKs all messages with a configurable error message
- Used primarily in testing and error path validation

#### noop_exporter

- Placeholder exporter that ACKs all messages without processing
- Lightweight for performance testing and pipeline validation

#### otap_exporter

- Streams OTAP Arrow payloads over gRPC to OTAP-compatible downstream receivers
- Reuses shared OTAP transport, compression, and Arrow encoding support from
  `otap-df-otap`

#### otlp_grpc_exporter

- Sends OTLP logs, metrics, and traces over unary gRPC export requests
- Supports concurrent in-flight exports with ack/nack propagation back into the
  pipeline
- Supports endpoint overrides, partial-success handling, and TLS via shared OTAP
  HTTP helpers

#### otlp_http_exporter

- Sends OTLP logs, metrics, and traces over OTLP/HTTP endpoints
- Supports concurrent in-flight exports with ack/nack propagation back into the
  pipeline
- Supports endpoint overrides, partial-success handling, and TLS via shared OTAP
  HTTP helpers

#### parquet_exporter

- Writes OTAP batches into partitioned Parquet files
- Supports schema normalization, ID generation, and configurable flush behavior

#### perf_exporter

- Measures item throughput by signal type for benchmarking scenarios
- Emits pdata-oriented telemetry metrics during pipeline execution

#### topic_exporter

- Publishes pdata into configured runtime topics
- Supports tracked end-to-end ack/nack propagation through topic boundaries

### Processors

<!-- markdownlint-disable MD013 -->
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
| log_sampling_processor | `urn:otel:processor:log_sampling` | `src/processors/log_sampling_processor/` |
| retry_processor | `urn:otel:processor:retry` | `src/processors/retry_processor/` |
| signal_type_router | `urn:otel:processor:type_router` | `src/processors/signal_type_router/` |
| temporal_reaggregation_processor | `urn:otel:processor:temporal_reaggregation` | `src/processors/temporal_reaggregation_processor/` |
| transform_processor | `urn:otel:processor:transform` | `src/processors/transform_processor/` |
<!-- markdownlint-enable MD013 -->

#### attributes_processor

- Adds, updates, renames, and deletes OpenTelemetry attributes
- Works directly on OTAP-native data structures for low-copy mutations

#### content_router

- Routes telemetry to named output ports based on resource attribute values
- Supports configurable selected-route admission for matched and default routes
  with `reject_immediately` (default) or `backpressure`
- Always emits a route-local retryable NACK when the selected route is closed,
  and uses the configured policy when the selected route is full
- Supports default routing and mixed-batch validation; see
  [Content Router Processor](src/processors/content_router/README.md)

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

#### log_sampling_processor

- Discards a portion of incoming logs according to a configurable sampling policy
- Useful for reducing data volume in a telemetry backend

#### retry_processor

- Retries downstream delivery with exponential backoff using Ack/Nack handling
- Preserves retry state in call data instead of external durable state

#### signal_type_router

- Routes signals by type (logs, metrics, traces) to named output ports
- Falls back to default routing only when a type-specific port is not connected
- Supports configurable selected-route admission with `reject_immediately`
  (default) or `backpressure`
- Always emits a route-local retryable NACK when the selected named or default
  route is closed, and uses the configured policy when the selected route is
  full
- Exposes per-signal routing and route-rejection telemetry; see
  [Signal Type Router Processor](src/processors/signal_type_router/README.md)

#### transform_processor

- Applies KQL or OPL transformations to OTAP batches via the query engine
- Supports routed outputs while preserving upstream Ack/Nack semantics

#### temporal_reaggregation_processor

- Reaggregates metrics at a lower frequency to reduce telemetry volume

### Receivers

<!-- markdownlint-disable MD013 -->
| Node | URN | Module |
| ---- | --- | ------ |
| fake_data_generator | `urn:otel:receiver:traffic_generator` | `src/receivers/fake_data_generator/` |
| internal_telemetry_receiver | `urn:otel:receiver:internal_telemetry` | `src/receivers/internal_telemetry_receiver/` |
| otap_receiver | `urn:otel:receiver:otap` | `src/receivers/otap_receiver/` |
| otlp_receiver | `urn:otel:receiver:otlp` | `src/receivers/otlp_receiver/` |
| syslog_cef_receiver | `urn:otel:receiver:syslog_cef` | `src/receivers/syslog_cef_receiver/` |
| topic_receiver | `urn:otel:receiver:topic` | `src/receivers/topic_receiver/` |
<!-- markdownlint-enable MD013 -->

#### fake_data_generator

- Generates synthetic OTAP/OTLP signals for testing and benchmarking
- Configurable signal generation strategies and volume constraints
- Includes support for pregenerated, dynamic, and rate-based signal generation

#### internal_telemetry_receiver

- Receives internal engine telemetry events from the internal log channel
- Emits them as OTLP log pdata into the configured pipeline

#### otap_receiver

- Accepts OTAP Arrow streams over gRPC and forwards them into the pipeline as
  `OtapPdata`
- Supports downstream wait-for-result ack/nack routing back to OTAP clients

#### otlp_receiver

- Accepts OTLP over gRPC, OTLP/HTTP, or both, and forwards serialized OTLP
  payloads into the pipeline
- Shares gRPC, HTTP, concurrency, TLS, and ack-routing support from
  `otap-df-otap`

#### syslog_cef_receiver

- Ingests RFC 3164/RFC 5424 syslog and CEF logs over TCP or UDP
- Converts incoming records into OTAP pdata with parser/format metadata

#### topic_receiver

- Subscribes to runtime topics and forwards messages into the pipeline
- Supports broadcast/balanced subscription modes and topic ack/nack bridging
