# OpenTelemetry Protocol with Apache Arrow

[![Slack](https://img.shields.io/badge/slack-@cncf/otel/arrow-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C07S4Q67LTF)
[![Build](https://github.com/open-telemetry/otel-arrow/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/open-telemetry/otel-arrow/actions/workflows/ci.yml)
[![OpenSSF Scorecard for otel-arrow](https://api.scorecard.dev/projects/github.com/open-telemetry/otel-arrow/badge)](https://scorecard.dev/viewer/?uri=github.com/open-telemetry/otel-arrow)

The [OpenTelemetry Protocol with Apache
Arrow](https://github.com/open-telemetry/otel-arrow) project is an
effort within [OpenTelemetry](https://opentelemetry.io/) to use
[Apache Arrow](https://arrow.apache.org/) libraries for bulk data
transport in OpenTelemetry collection pipelines.  This repository is
the home of the OpenTelemetry Protocol with Apache Arrow protocol and
reference implementation.

## Quick start

Instructions for building an OpenTelemetry Collector with the modules
in this repository are provided in [BUILDING.md](./collector/BUILDING.md).

Examples for running the OpenTelemetry Collector with the modules in
this repository are documented in
[collector/examples](./collector/examples/README.md).

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
than can be achieved using a row-based data model and a stateless protocol.

## Project goals

This is organized in phases. Our initial aim is to facilitate traffic reduction
between a pair of OpenTelemetry collectors as illustrated in the following
diagram.

![Traffic reduction use case](docs/img/traffic_reduction_use_case.png)

The collector provided in this repository implements a new Arrow Receiver and
Exporter able to fallback on standard OTLP when needed. The following diagram is
an overview of this integration. In this first phase, the internal
representation of the telemetry data is still fundamentally row-oriented.

![collector internal overview](docs/img/collector_internal_overview.png)

Ultimately, we believe that an end-to-end OpenTelemetry Protocol with Apache
Arrow pipeline will enable telemetry pipelines with substantially lower
overhead to be built. These are our future milestones for OpenTelemetry and
Apache Arrow integration:

1. Extend OpenTelemetry client SDKs to natively support the OpenTelemetry  
   Protocol with Apache Arrow Protocol
2. Extend the OpenTelemetry collector with direct support for OpenTelemetry
   Protocol with Apache Arrow pipelines
3. Extend OpenTelemetry data model with native support for multi-variate
   metrics.
4. Output OpenTelemetry data to the Parquet file format, part of the Apache
   Arrow ecosystem

### Improve network-level compression with OpenTelemetry Protocol with Apache Arrow

The first general-purpose application for the project is traffic
reduction.  At a high-level, this protocol performs the following steps
to compactly encode and transmit telemetry using Apache Arrow.

1. Separate the OpenTelemetry Resource and Scope elements from the
   hierarchy, then encode and transmit each distinct entity once per
   stream lifetime.
2. Calculate distinct attribute sets used by Resources, Scopes,
   Metrics, Logs, Spans, Span Events, and Span Links, then encode and
   transmit each distinct entity once per stream lifetime.
3. Use Apache Arrow's built-in support for encoding dictionaries and leverage
   other purpose-built low-level facilities, such as delta-dictionaries and
   sorting, to encode structures compactly.

Here is a diagram showing how the protocol transforms OTLP Log Records
into column-oriented data, which also makes the data more compressible.

![OpenTelemetry Protocol with Apache Arrow](https://github.com/open-telemetry/oteps/blob/main/text/img/0156_logs_schema.png?raw=true)

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
housed in [OpenTelemetry
Collector-Contrib](https://github.com/open-telemetry/opentelemetry-collector-contrib):

- [OpenTelemetry Protocol with Apache Arrow Receiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md)
- [OpenTelemetry Protocol with Apache Arrow Exporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md)

The OpenTelemetry Protocol with Apache Arrow exporter and receiver components are drop-in compatible
with the core collector's OTLP exporter and receiver components.
Users with an established OTLP collection pipeline between two
OpenTelemetry Collectors can re-build their collectors with
`otelarrow` components, then simply replace the component name `otlp`
with `otelarrow`.  The exporter and receiver both support falling back
to standard OTLP in case either side does not recognize the protocol,
so the upgrade should be painless.  The OpenTelemetry Protocol with Apache Arrow receiver serves
both OpenTelemetry Protocol with Apache Arrow and OTLP on the standard port for OTLP gRPC (4317).

See the [Exporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md) and
[Receiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md)
documentation for details and sample configurations.

### Project documentation

This package is a reference implementation of the OpenTelemetry Protocol with Apache Arrow protocol
specified in this
[OTEP](https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md),
which is currently the best source of information about OpenTelemetry Protocol with Apache Arrow.
The [Donation
request](https://github.com/open-telemetry/community/issues/1332)
describes how the project began.

Here are several more resources that are available to learn more about OpenTelemetry Protocol with Apache Arrow.

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
compression) and the OpenTelemetry Protocol with Apache Arrow protocol (ZSTD) in comparison with an
uncompressed OTLP protocol.

![compression_ratio](./docs/img/compression_ratio_summary_std_metrics.png)

The next chart follows the same logic but shows the results for multivariate
metrics (see left column).

![compression_ratio](./docs/img/compression_ratio_summary_multivariate_metrics.png)

The following heatmap represents, for different combinations of batch sizes and
connection durations (expressed as the number of batches per stream), the
additional percentage of compression gain between this new protocol and OTLP,
both compressed with ZSTD. The data used here comes from a traffic of spans
captured in a production environment. The gains are substantial in most cases.
It is even interesting to note that these gains compared to OTLP+ZSTD are more
significant for moderate-sized batches (e.g., 100 and 1000 spans per batch),
which makes this protocol also interesting for scenarios where the additional
latency introduced by batching must be minimized. There is hardly any scenario
where micro-batches (e.g., 10 spans per batch) make the overhead of the Arrow
schema prohibitive, and the advantage of a columnar representation becomes
negligible. In other cases, this initial overhead is very quickly offset after
just the first few batches. The columnar organization also lends itself better
to compression. For very large batch sizes, ZSTD does an excellent job as long
as the compression window is sufficiently large, but even in this case, the new
protocol remains superior. As previously mentioned, these compression gains can
be higher for traffic predominantly containing multivariate metrics.

![Avg % of compressed size improvement of OpenTelemetry Protocol with Apache Arrow over OTLP (zstd compression)](./docs/img/average_improvement_heatmap.png)

For more details, see the following [benchmark results](docs/benchmarks.md) page.

### Contributing

Pull requests are welcome. For major changes, please open an issue
first to discuss what you would like to change.  For more information, please
read [CONTRIBUTING.md][].

## License

OpenTelemetry Protocol with Apache Arrow Protocol Adapter is licensed under Apache 2.0.

[CONTRIBUTING.md]: ./CONTRIBUTING.md
