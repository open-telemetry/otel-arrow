# OTLP Arrow Encoder/Decoder package

This package is a reference implementation of the OTLP Arrow Encoder/Decoder specified in this [OTEP](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md.
All OTLP entities are covered (metrics, logs, and traces) as well as all sub-elements such as events, links, gauge, sum, 
summary, histograms, ... The overall goal is to optimize the compression ratio for telemetry data transmission as well 
as the end-to-end performance between telemetry data producers and receivers.

**This package is still experimental and subject to change.** It is currently used by an [experimental OTLP/Arrow gRPC 
exporter and receiver](https://github.com/open-telemetry/experimental-arrow-collector).

Other important links:
- [Project Roadmap](https://github.com/f5/otel-arrow-adapter/milestones?direction=asc&sort=due_date&state=open).
- [Project Board](https://github.com/orgs/f5/projects/1/views/2) describing the current state of the project.
- [Arrow schemas](docs/arrow_schema.md) used by this package.
- The underlying [OTEP](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md) describing the 
rationale, specifications and different phases of this project.


## Testing and validation

The testing of this package and the validation of the OTLP Arrow encoding/decoding are the object of particular 
attention because of the central position of this package in the future OTEL collector.

Concerning the test, the plan is to:
- reach at least 80% of the tested code (probably more),
- implement fuzz tests on the encoding and decoding of OTLP Arrow messages,
- implement integration tests with the experimental collector.

Concerning the encoding/decoding validation, the plan is to:
- compare the OTLP entities before and after their conversion to OTLP Arrow entities.
- test the conversion procedure of the production data via a CLI tool or directly via the integration in the 
experimental collector.

A validation of the compression ratio stability is also part of the objectives. This validation will be performed on production data.

## Security

A thread model is being defined [WIP] (untrusted input data, what can go wrong at the protocol level, during the 
encoding or decoding phases, ...). Below the main risks identified so far:
- invalid, or compromised inputs causing security or reliability issues.
- very large input data causing denial of service.
- high cardinality data causing dictionary overflow (over multiple messages).
- ... TBD 

Check this issue for complementary information: https://github.com/open-telemetry/opentelemetry-specification/issues/1891 

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


## Integration with the OpenTelemetry Collector

The integration of this package with the OpenTelemetry Collector is done in the following experimental repository:
* [experimental-arrow-collector](https://github.com/open-telemetry/experimental-arrow-collector)


## Project management and status

- [Project Board](https://github.com/orgs/f5/projects/1/views/2)
- Milestones:
  - [Beta 1](https://github.com/f5/otel-arrow-adapter/milestone/1)
  - [Beta 2](https://github.com/f5/otel-arrow-adapter/milestone/2)
  - [Beta 3](https://github.com/f5/otel-arrow-adapter/milestone/3)
  - [Beta 4](https://github.com/f5/otel-arrow-adapter/milestone/4)

## Contributing

Pull requests are welcome. For major changes, please open an issue
first to discuss what you would like to change. For more information, please
read [CONTRIBUTING](CONTRIBUTING.md).

## License

OTEL Arrow Adapter is licensed under Apache 2.0.
