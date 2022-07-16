# otel-arrow-adapter

> Status: WIP

Adapter used to convert OTEL batches to/from OTEL Arrow batches in both directions.

See [OTEP 0156](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md) for more details on the OTEL Arrow protocol.

## To be done

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
  - **OTLP metrics --> OTLP_ARROW events**
    - [X] Gauge
    - [X] Sum
    - [ ] Summary
    - [ ] Histogram and exponential histogram
    - [X] Univariate metrics to multivariate metrics
  - **OTLP logs --> OTLP_ARROW events**
      - [X] Basic fields
      - [X] Complex attributes
      - [X] Complex body
  - **OTLP trace --> OTLP_ARROW events**
    - [X] Basic fields
    - [X] Complex attributes
    - [ ] Links
    - [ ] Events
  - **Arrow IPC format**
    - [ ] Producer

### OTLP Arrow --> OTLP
  - **OTLP_ARROW events --> OTLP metrics**
    - [ ] Gauge
    - [ ] Sum
    - [ ] Summary
    - [ ] Histogram and exponential histogram
    - [ ] Univariate metrics to multivariate metrics
  - **OTLP_ARROW events --> OTLP logs**
    - [ ] Basic fields
    - [ ] Complex attributes
    - [ ] Complex body
  - **OTLP_ARROW events --> OTLP trace**
    - [ ] Basic fields
    - [ ] Complex attributes
    - [ ] Links
    - [ ] Events
  - **Arrow IPC format**
    - [ ] Consumer

### Protocol
  - [X] OTLP proto
  - [X] Event service
  - [ ] gRPC service implementation

### Benchmarks 
  - [ ] OTLP batch creation + serialization + compression + decompression + deserialization
  - [ ] OTLP_ARROW batch creation + serialization + compression + decompression + deserialization
  - [ ] Framework to compare OTLP and OTLP_ARROW performances (i.e. size and time)  