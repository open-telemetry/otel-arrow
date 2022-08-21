# otel-arrow-adapter

Adapter used to convert OTEL batches to/from OTEL Arrow batches in both directions.

See [OTEP 0156](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md) for more details on the OTEL Arrow protocol.

## Packages

| Package       | Description                                                                                                                  |
|---------------|------------------------------------------------------------------------------------------------------------------------------|
| pkg/air       | Arrow Intermediate Representation used to translate batch of row-oriented entities into columnar-oriented batch of entities. |
| pkg/benchmark | Benchmark infrastructure to compare OTLP and OTLP Arrow representations.                                                     |
| pkg/datagen   | Metrics, logs, and traces generator (fake data).                                                                             |
| pkg/otel      | Conversion functions to translate OTLP entities into OTLP Arrow Event and vice versa.                                        |
| tools         | Set of command line tools to generate synthetic OTLP entities and to compare OTLP vs OTLP Arrow protocols.                   |

## Synthetic OTLP entities generator

### Metrics

This tool generates synthetic metrics and store them as protobuf binary files.

### Logs

This tool generates synthetic metrics and store them as protobuf binary files.

### Traces

This tool generates synthetic metrics and store them as protobuf binary files.


## Benchmarking tools

These command line tools are used to compare the performance of OTLP and OTLP Arrow entity representations. Each tool
takes as input a set of OTLP entities encoded as protobuf binary files and produces an evaluation report and a set of 
data files (CSV format).

### Metrics [WIP]

This tool compares the performance of OTLP and OTLP Arrow representations for metrics.

### Logs [WIP]

This tool compares the performance of OTLP and OTLP Arrow representations for metrics.

### Traces

This tool compares the performance of OTLP and OTLP Arrow representations for metrics.

```bash
$ go run tools/trace_benchmark/main.go <INPUT FILES>
```

The input files must be one or more OTLP traces (ExportTraceServiceRequest) encoded as protobuf binary files.


## Integration with the OpenTelemetry Collector

TBD


## Status [WIP]

**Arrow Intermediate Representation (framework to convert row-oriented structured data to Arrow columnar data)**
- [X] Values (supported types: bool, i[8|16|32|64], u[8|16|32|64], f[32|64], string, binary, list, struct)
- [X] Fields
- [X] Record
- [X] Record Builder
- [X] Record Repository
- [X] Generate Arrow records
  - [X] Scalar values
  - [X] Struct values
  - [X] List values (except list of list)
- [X] Optimizations
  - [X] Dictionary encoding for string fields (uint8 and uint16 indices)
  - [X] Dictionary encoding for binary fields (uint8 and uint16 indices)
  - [X] Multi-field sorting (string field)
  - [X] Multi-field sorting (binary field)

**Transform OTLP entities into OTLP Arrow entities**
This capability will be used to implement a receiver into the existing collector (phase 1)
  - **General**.
    - [X] Complex attributes
    - [X] Complex body
    - [ ] Schema URL
    - [X] Description
    - [X] Unit
  - **OTLP metrics --> OTLP_ARROW events**
    - [X] Gauge
    - [X] Sum
    - [X] Summary
    - [X] Histogram
    - [X] Exponential histogram
    - [X] Univariate metrics to multivariate metrics
    - [ ] Aggregation temporality
    - [ ] Exemplar
  - **OTLP logs --> OTLP_ARROW events**
    - [X] Logs
  - **OTLP trace --> OTLP_ARROW events**
    - [X] Trace
    - [X] Links
    - [X] Events

**Transform OTLP Arrow entities into OTLP entities**
This capability will be used to implement an exporter into the existing collector (phase 1).
  - **General**
    - [X] Complex attributes 
    - [X] Complex body
    - [ ] Schema URL 
    - [X] Description
    - [X] Unit
  - **OTLP_ARROW events --> OTLP metrics**
    - [ ] Gauge
    - [X] Sum
    - [ ] Summary
    - [ ] Histogram
    - [ ] Exponential histogram
    - [ ] Univariate metrics to multivariate metrics
    - [ ] Aggregation temporality
    - [ ] Exemplar
  - **OTLP_ARROW events --> OTLP logs**
    - [X] Logs
  - **OTLP_ARROW events --> OTLP trace**
    - [X] Trace
    - [X] Links 
    - [X] Events

**Protocol**
  - [X] OTLP proto [WIP change a little bit the BatchEvent specification]
  - [X] Event service
  - [x] BatchEvent producer
  - [X] BatchEvent consumer
  - [ ] gRPC service implementation (most likely to be implemented in the OpenTelemetry collector itself)

**Benchmarking tools** 
  - Synthetic data generator
    - [X] ExportMetricsServiceRequest (except for histograms and summary)
    - [X] ExportLogsServiceRequest
    - [X] ExportTraceServiceRequest 
  - Framework to compare OTLP and OTLP_ARROW performances (i.e. size and time)
    - [X] General framework
    - [X] Compression algorithms (lz4 and zstd)
    - [X] Console output
    - [X] Export CSV
  - [X] OTLP batch creation + serialization + compression + decompression + deserialization
  - [X] OTLP_ARROW batch creation + serialization + compression + decompression + deserialization
  - Synthetic data generators
    - [X] logs_gen
    - [X] metrics_gen
    - [X] traces_gen
  - Benchmarking tools
    - [X] logs_benchmark 
    - [X] metrics_benchmark [WIP define a way to specify the multivariate metrics configuration]  
    - [X] traces_benchmark 
  - [ ] Assertions to check the correctness of the OTLP_ARROW representation
    - [ ] [WIP] Metrics
    - [ ] Logs
    - [ ] Trace

**Performance**
  - [ ] Performance and memory optimizations
  - [ ] Check memory leaks (e.g. Arrow related memory leaks)

**CI**
  - [ ] GitHub Actions to build, test, check at every commit.

**Integration**
  - [ ] Integration with Open Telemetry Collector.

**Documentation**
  - [ ] Update OTEP 0156.

**Feedback to implement**
  - [ ] Feedback provided by @atoulme 