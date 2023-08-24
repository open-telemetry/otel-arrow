# OTel Arrow Protocol 

**Repository under maintenance.**

This repository recently migrated from the
[github.com/f5/otel-arrow-adapter](https://github.com/f5/otel-arrow-adapter),
and we are preparing it for a beta release.  While the repository is
still in transition, the primary OpenTelemetry Collector components
are housed here:

- [OTel Arrow Receiver](./collector/receiver/otelarrowreceiver/README.md)
- OTel Arrow Exporter (TODO)

## Reference implementation.

This package is a reference implementation of the OTel Arrow protocol specified in this [OTEP](https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md).
All OTLP entities are covered (metrics, logs, and traces) as well as all sub-elements such as events, links, gauge, sum, 
summary, histograms, ... The overall goal is to optimize the compression ratio for telemetry data transmission as well 
as the end-to-end performance between telemetry data producers and receivers.

**This package is still experimental and subject to change.** It is currently used by an [experimental OTLP/Arrow gRPC 
exporter and receiver](https://github.com/open-telemetry/experimental-arrow-collector).

Important links:
- [OTEP](https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md) - protocol specification.
- [Donation](https://github.com/open-telemetry/community/issues/1332) - approved by the Technical Committee (repo not yet transferred in OTel org).
- [Arrow Data Model](docs/data_model.md) - Mapping OTLP entities to Arrow Schemas.
- [Benchmark results](docs/benchmarks.md) - Based on synthetic and production data.
- [Validation process](docs/validation_process.md) - Encoding/Decoding validation process. 
- Articles describing some of the Arrow techniques used behind the scenes to optimize compression ratio and memory usage:
  - [Data types, encoding, hierarchical data, denormalization](https://arrow.apache.org/blog/2023/04/11/our-journey-at-f5-with-apache-arrow-part-1/)
  - [Adaptive Schemas and Sorting to Optimize Arrow Usage](https://arrow.apache.org/blog/2023/06/26/our-journey-at-f5-with-apache-arrow-part-2/)

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

This above repository houses a fork of the entire core OpenTelemetry
Collector, where the complete branch history is kept, including
"mainline" Collector commits as well as Arrow-component development
commits.

Because that repository contains portions that are not part of the
OTel-Arrow project, [the components are being maintained in this
repository](https://github.com/open-telemetry/experimental-arrow-collector/issues/48)
until they can be merged into the
[OpenTemetry-Collector-Contrib](github.com/open-telemetry/opentelemetry-collector-contrib)
repository.

Collector components copied from that repository are currently
available in the
[`./collector`](https://github.com/f5/otel-arrow-adapter/blob/main/collector/README.md)
sub-package of this repository.

Examples demonstrating how to configure and test an OpenTelemetry
Collector with OTel-Arrow exporter and receiver components are located
in `./collector/examples`, including:

- [`examples/bridge`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/bridge):
  A compression bridge between "edge" and "saas" collectors.
- [`examples/metadata-bridge`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/metadata-bridge):
  A compression bridge between "edge" and "saas" collectors with metadata support, allowing request headers to transit via OTel-Arrow.
- [`examples/loopback`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/loopback):
  A collector that writes Arrow to and from itself.
- [`examples/recorder`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/recorder):
  A collector with support for recording data files for diagnostic and benchmark purposes.
- [`examples/synthesize`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/synthesize):
  A collector with support for synthesizing telemetry data using a [telemetry-generator](https://github.com/lightstep/telemetry-generator) component.

## License

OTel Arrow Protocol Adapter is licensed under Apache 2.0.
