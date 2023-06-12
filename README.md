# OTel Arrow Protocol implementation

This package is a reference implementation of the OTel Arrow protocol specified in this [OTEP](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md).
All OTLP entities are covered (metrics, logs, and traces) as well as all sub-elements such as events, links, gauge, sum, 
summary, histograms, ... The overall goal is to optimize the compression ratio for telemetry data transmission as well 
as the end-to-end performance between telemetry data producers and receivers.

**This package is still experimental and subject to change.** It is currently used by an [experimental OTLP/Arrow gRPC 
exporter and receiver](https://github.com/open-telemetry/experimental-arrow-collector).

Important links:
- [OTEP](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md) - protocol specification 
  (status: [pending, unmerged](https://github.com/open-telemetry/oteps/pull/171)).
- [Donation](https://github.com/open-telemetry/community/issues/1332) - approved, but repo not yet transferred in OTel org.
- [Arrow Data Model](docs/data_model.md) - Mapping OTLP entities to Arrow Schemas.
- [Benchmark results](docs/benchmarks.md) - Based on synthetic and production data.
- [Validation process](docs/validation_process.md) - Encoding/Decoding validation process. 
- [Slides](https://docs.google.com/presentation/d/12uLXmMWNelAyAiKFYMR0i7E7N4dPhzBi2_HLshFOLak/edit?usp=sharing) (01/30/2023 Maintainers meeting).

## Benchmark summary

The following chart shows the compressed message size (in bytes) as a function
of the batch size for metrics (univariate), logs, and traces. The bottom of the
chart shows the reduction factor for both the standard OTLP protocol (with ZSTD
compression) and the OTel Arrow protocol (ZSTD) in comparison with an
uncompressed OTLP protocol.

![compression_ratio](./docs/img/compression_ratio_summary_std_metrics.png)

The next chart follows the same logic but shows the results for multivariate
metrics (see left column).

![compression_ratio](./docs/img/compression_ratio_summary_multivariate_metrics.png)

For more details, see the following [benchmark results](docs/benchmarks.md) page.
 
## Phase 1 (current implementation)

This first step is intended to address the specific use cases of traffic reduction. Based on community feedback, many
companies want to reduce the cost of transferring telemetry data over the Internet. By adding a collector that acts as
a point of integration and traffic conversion at the edge of a client environment, we can take advantage of the columnar
format to eliminate redundant data and optimize the compression rate. This is illustrated in the following diagram.

![Traffic reduction use case](docs/img/traffic_reduction_use_case.png)

> Note 1: A fallback mechanism can be used to handle the case where the new protocol is not supported by the target. 
> More on this mechanism in this [section](https://github.com/lquerel/oteps/blob/main/text/0156-columnar-encoding.md#protocol-extension-and-fallback-mechanism) of the OTEP. 

The experimental collector implements on top of this library a new Arrow Receiver and Exporter able to fallback on
standard OTLP when needed. The following diagram is an overview of this integration. The internal representation of the
data has not been updated and this collector is still fundamentally row-oriented internally.

![collector internal overview](docs/img/collector_internal_overview.png)

> Note 2: A future phase 2 of this project will focus on implementing end-to-end OTel Arrow to improve the overall
> performance.

### Developers

Pull requests are welcome. For major changes, please open an issue
first to discuss what you would like to change. For more information, please
read [CONTRIBUTING](CONTRIBUTING.md).

#### How to change the protobuf specification

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

## License

OTel Arrow Protocol Adapter is licensed under Apache 2.0.
