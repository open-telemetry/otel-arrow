# OTel Arrow

The [OTel Arrow](https://github.com/open-telemetry/otel-arrow) project
is an effort within [OpenTelemetry](https://opentelemetry.io/) to use
[Apache Arrow](https://arrow.apache.org/) libraries for bulk data
transport in OpenTelemetry collection pipelines.  This repository is
the home of the OTel Arrow protocol and reference implementation.

## Overview

OpenTelemetry and Apache Arrow have similar charters, so it was
natural to think about combining them.  Both projects offer
vendor-neutral interfaces with a cross-language interface
specification, so that their implementation will feel familiar to
users as they move between programming languages, and both specify a
data model that is used throughout the project.

The OpenTelemetry project defines
[OTLP](https://opentelemetry.io/docs/specs/otlp/), the "OpenTeLemetry
Prototcol" as the standard form of telemetry data in OpenTelemetry,
being as similar as possible to the data model underlying the project.
OTLP is defined in terms of Google protocol buffer definitions.

OTLP is a stateless protocol, where export requests map directly into
the data model, nothing is omitted, and little is shared.  OTLP export
requests to not contain external or internal references, making the
data relatively simple and easy to interpret.  Because of this design,
users of OTLP will typically configure network compression.  In
environments where telemetry data will be shipped to a service
provider across a wide-area network, users would like more compression
than can be achieved using a stateless protocol.

## Project goals

The OTel Arrow project is organized in phases.  Our initial aim is to
facilitate traffic reduction between a pair of OpenTelemetry
collectors, and ultimately, we believe that an end-to-end OTel Arrow
pipeline will enable telemetry pipelines with substantially lower
overhead to be built.  These are our future milestones for
OpenTelemetry and Apache Arrow integration:

1. Improve compression performance for OpenTelemetry data collection
2. Extend OpenTelemetry client SDKs to natively support the OTel Arrow Protocol
3. Extend the OpenTelemetry collector with direct support for OTel Arrow pipelines
4. Extend OpenTelemetry data model with support for multi-variate metrics.
5. Output OpenTelemetry data to the Parquet file format, part of the Apache Arrow ecosystem

### Improve network-level compression with OTel Arrow

The first general-purpose application for OTel Arrow is traffic
reduction.  At a high-level, OTel Arrow performs the following steps
to compactly encode and transmit telemetry using Apache Arrow.

1. Separate the OpenTelemetry Resource and Scope elements from the
   hierarchy, then encode and transmit each distinct entity once per
   stream lifetime.
2. Calculate distinct attribute sets used by Resources, Scopes,
   Metrics, Logs, Spans, Span Events, and Span Links, then encode and
   transmit each distinct entity once per stream lifetime.
3. Use Apache Arrow's built-in support for encoding dictionaries,
   delta-dictionaries, and other low-level facilities to compactly
   encode the structure.

Here is a diagram showing how OTel Arrow transforms OTLP Log Records
into column-oriented data, which also makes the data more compressible.

![OTel Arrow Log Record](https://github.com/open-telemetry/oteps/blob/main/text/img/0156_logs_schema.png?raw=true)

## Project status

The first phase of the project has entered the [Beta stability level,
as defined by the OpenTelemetry collector
guidelines](https://github.com/open-telemetry/opentelemetry-collector#beta).
We do not plan to make breaking changes in this protocol without first
engineering an approach that ensures forwards and
backwards-compatibility for existing and new users.  We believe it is
safe to begin using these components for production data, non-critical
workloads.

### Project deliverables 

We are pleased to release two new collector components, presently
housed in this this repository.

- [OTel Arrow Receiver](./collector/receiver/otelarrowreceiver/README.md)
- [OTel Arrow Exporter](./collector/exporter/otelarrowexporter/README.md)

We are working with the maintainers of the [OpenTelemetry
Collector-Contrib](https://github.com/open-telemetry/opentelemetry-collector-contrib)
to merge these components into that repository.  [See our tracking
issue](https://github.com/open-telemetry/opentelemetry-collector-contrib/issues/26491).

The OTel Arrow exporter and receiver components are drop-in compatible
with the core collector's OTLP exporter and receiver components.
Users with an established OTLP collection pipeline between two
OpenTelemetry Collectors can re-build their collectors with
`otelarrow` components, then simply replace the component name `otlp`
with `otelarrow`.  The exporter and receiver both support falling back
to standard OTLP in case either side does not recognize the protocol,
so the upgrade should be painless.  The OTel Arrow receiver serves
both OTel Arrow and OTLP on the standard port for OTLP gRPC (4317).

See the [Exporter](collector/exporter/otelarrowexporter/README.md) and
[Receiver](collector/receiver/otelarrowreceiver/README.md)
documentation for details and sample configurations.

### Project documentation

This package is a reference implementation of the OTel Arrow protocol
specified in this
[OTEP](https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md),
which is currently the best source of information about OTel Arrow.
The [Donation
request](https://github.com/open-telemetry/community/issues/1332)
describes how the project began.

Here are several more resources that are available to learn more about OTel Arrow.

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
first to discuss what you would like to change.  For more information, please
read [CONTRIBUTING](CONTRIBUTING.md).

#### Version history shared with OpenTelemetry Collector

The OTel Arrow exporter and receiver components were derived from the
core OpenTelemetry Collector's OTLP exporter and receiver components.
This approach was taken to ensure that the OTel Arrow receiver and
exporter components remain "drop-in" compatible with OTLP exporters
and receivers.

The shared version history describing how the OTel Arrow components
were derived from the core OTLP components is recorded in [this
repository](https://github.com/open-telemetry/otel-arrow-collector),
which is not for general use.  Maintainers of OTel Arrow may use the
repository to maintain the version history going forward, [as discussed
here](https://github.com/open-telemetry/otel-arrow/issues/43).

#### Example collector configurations

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

#### Miscellaneous components

Several components were developed to facilitate testing and debugging
the primary OTel Arrow components.  Most importantly, these tools can
be used to report problematic data to the OTel Arrow maintainers.
These components are:

- `exporter/fileexporter`: Derived from the upstream [fileexporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/fileexporter), this component supports writing files that can be read by the corresponding `filereceiver` in this package (unlike the upstream).
- `receiver/filereceiver`: Derived from the upstream [filereceiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/filereceiver), this component supports reading files written by the corresponding `fileexporter` in this package (unlike the upstream).
- `processor/obfuscationprocessor`: Supports obfuscation of OpenTelemetry data using a [Feistel cipher](https://en.wikipedia.org/wiki/Feistel_cipher).
- `processor/experimentprocessor`: A probabilistic routing component for conducting experiments between exporters.
- `connector/validationconnector`: A component for on-the-fly validation of a local pipeline.

## License

OTel Arrow Protocol Adapter is licensed under Apache 2.0.
