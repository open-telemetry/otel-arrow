# OTLP Arrow Encoder/Decoder package

Adapter used to convert OTEL batches to/from OTEL Arrow batches in both directions.

This package is still experimental and subject to change. It is currently used by an [experimental OTLP/Arrow gRPC 
exporter and receiver](https://github.com/open-telemetry/experimental-arrow-collector).

Other important links:
- [Project Roadmap](https://github.com/f5/otel-arrow-adapter/milestones?direction=asc&sort=due_date&state=open).
- [Project Board](https://github.com/orgs/f5/projects/1/views/2) describing the current state of the project.
- [Arrow schemas](docs/arrow_schema.md) used by this package.
- The underlying [OTEP](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md) describing the 
rationale, specifications and different phases of this project.


## Developers 

### How to change the protobuf specification

To (re)generate the ArrowStreamService gRPC service, you need to install the `protoc` compiler and the `protoc-gen-grpc` plugin.
```shell
go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.28
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.2
export PATH="$PATH:$(go env GOPATH)/bin"
cd ./proto
./generate.sh
```
Once the `*.pb.go` files are generated, you need to replace the content of the `api/collector/arrow/v1` directory by the
generated files present in the `./proto/api/collector/arrow/v1` directory.

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

### Traces

This tool compares the performance of OTLP and OTLP Arrow representations for metrics.

```bash
$ go run tools/trace_benchmark/main.go <INPUT FILES>
```

The input files must be one or more OTLP traces (ExportTraceServiceRequest) encoded as protobuf binary files.

### Metrics [WIP]

This tool compares the performance of OTLP and OTLP Arrow representations for metrics.

### Logs [WIP]

This tool compares the performance of OTLP and OTLP Arrow representations for metrics.

## Integration with the OpenTelemetry Collector

The integration of this package with the OpenTelemetry Collector is done in the following experimental repository:
* [experimental-arrow-collector](https://github.com/open-telemetry/experimental-arrow-collector)

### Traces 

Below some utility functions in package `pkg/otel/traces` than can be used to convert OTLP traces to/from OTLP Arrow traces:
* `OtlpArrowProducer` takes OTLP Traces and produces OTLP Arrow Traces.
* `OtlpProducer` takes OTLP Arrow Traces and produces OTLP Traces.

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
    - [X] Schema URL
    - [X] Description
    - [X] Unit
  - **OTLP metrics --> OTLP_ARROW events**
    - [X] Gauge
    - [X] Sum
    - [X] Summary
    - [X] Histogram
    - [X] Exponential histogram
    - [X] Univariate metrics to multivariate metrics
    - [X] Aggregation temporality
    - [X] Exemplar
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
    - [X] Schema URL 
    - [X] Description
    - [X] Unit
  - **OTLP_ARROW events --> OTLP metrics**
    - [X] Gauge
    - [X] Sum
    - [X] Summary
    - [X] Histogram
    - [X] Exponential histogram
    - [X] Univariate metrics to multivariate metrics
    - [X] Aggregation temporality
    - [X] Exemplar
  - **OTLP_ARROW events --> OTLP logs**
    - [X] Logs
  - **OTLP_ARROW events --> OTLP trace**
    - [X] Trace
    - [X] Links 
    - [X] Events

**Protocol**
  - [X] BatchArrowRecords proto 
  - [X] Arrow Stream service
  - [x] BatchArrowRecords producer
  - [X] BatchArrowRecords consumer

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
  - [X] Assertions to check the correctness of the OTLP_ARROW representation
    - [X] Metrics
    - [X] Logs
    - [X] Trace

**Performance**
  - [ ] Performance and memory optimizations
  - [X] Check memory leaks (e.g. Arrow related memory leaks)

**CI**
  - [X] GitHub Actions to build, test, check at every commit.

**Integration**
  - [ ] Integration with Open Telemetry Collector.

**Documentation**
  - [ ] Update OTEP 0156.

**Feedback to implement**
  - [ ] @jmacd's feedback
  - [ ] @atoulme's feedback 

## Contributing

Pull requests are welcome. For major changes, please open an issue
first to discuss what you would like to change. For more information, please
read [CONTRIBUTING](CONTRIBUTING.md).

## License

OTEL Arrow Adapter is licensed under Apache 2.0.
