# otel-arrow-adapter

Adapter used to convert OTEL batches to/from OTEL Arrow batches in both directions.

See [OTEP 0156](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md) for more details on the OTEL Arrow protocol.

## Status [WIP]

### Framework to convert row-oriented structured data to Arrow columnar data
- [X] Record Repository
- [X] Record Builder
- [X] Record, fields, and values
- [X] Generate Arrow records
  - [X] Scalar values
  - [X] Struct values
  - [ ] List values
- [X] Optimizations
  - [X] Dictionary encoding for string fields
  - [ ] Dictionary encoding for binary fields
  - [X] Multi-field sorting (string field)
  - [ ] Multi-field sorting (binary field)

### OTLP --> OTLP Arrow
  - **General**
    - [X] Complex attributes
    - [X] Complex body
  - **OTLP metrics --> OTLP_ARROW events**
    - [X] Gauge
    - [X] Sum
    - [ ] Summary
    - [ ] Histogram and exponential histogram
    - [X] Univariate metrics to multivariate metrics
    - [ ] Exemplar
  - **OTLP logs --> OTLP_ARROW events**
    - [X] Logs
  - **OTLP trace --> OTLP_ARROW events**
    - [X] Trace
    - [ ] Links
    - [ ] Events
  - **Arrow IPC format**
    - [ ] Producer

### OTLP Arrow --> OTLP
  - **General**
    - [ ] Complex attributes
    - [ ] Complex body
  - **OTLP_ARROW events --> OTLP metrics**
    - [ ] Gauge
    - [ ] Sum
    - [ ] Summary
    - [ ] Histogram and exponential histogram
    - [ ] Univariate metrics to multivariate metrics
    - [ ] Exemplar
  - **OTLP_ARROW events --> OTLP logs**
    - [ ] Logs
  - **OTLP_ARROW events --> OTLP trace**
    - [ ] Trace
    - [ ] Links
    - [ ] Events
  - **Arrow IPC format**
    - [ ] Consumer

### Protocol
  - [X] OTLP proto
  - [X] Event service
  - [ ] gRPC service implementation

### Benchmarks 
  - [X] Fake data generator
    - [X] ExportMetricsServiceRequest (except for histograms and summary)
    - [X] ExportLogsServiceRequest
    - [X] ExportTraceServiceRequest (except for links and events)
  - [ ] OTLP batch creation + serialization + compression + decompression + deserialization
  - [ ] OTLP_ARROW batch creation + serialization + compression + decompression + deserialization
  - [ ] Framework to compare OTLP and OTLP_ARROW performances (i.e. size and time)
  - [ ] Performance and memory optimizations
  - [ ] Check memory leaks (e.g. Arrow related memory leaks)