# OpenTelemetry Protocol with Apache Arrow

[![Slack](https://img.shields.io/badge/slack-@cncf/otel/arrow-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C07S4Q67LTF)
[![Go-CI](https://github.com/open-telemetry/otel-arrow/actions/workflows/go-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/go-ci.yml)
[![Rust-CI](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml)
[![OpenSSF Scorecard for otel-arrow](https://api.scorecard.dev/projects/github.com/open-telemetry/otel-arrow/badge)](https://scorecard.dev/viewer/?uri=github.com/open-telemetry/otel-arrow)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/10684/badge)](https://www.bestpractices.dev/projects/10684)
[![codecov](https://codecov.io/gh/open-telemetry/otel-arrow/graph/badge.svg?token=7u3gFLH54G)](https://codecov.io/gh/open-telemetry/otel-arrow)

**The OTel-Arrow project is building a high-performance, end-to-end
column-oriented telemetry pipeline for
[OpenTelemetry](https://opentelemetry.io/) data based on [Apache
Arrow](https://arrow.apache.org/).**

The OpenTelemetry Protocol with Apache Arrow (OTAP) is designed as the
column-oriented equivalent of OpenTelemetry Protocol (OTLP) that
dramatically reduces network usage, and our [protocol
specification](./docs/otap-spec.md) ensures that OTAP and OTLP are
always convertible, without loss, in both directions.

The [OTAP Dataflow Engine](./rust/otap-dataflow/README.md) is our new
Rust OpenTelemetry code base, a pipeline engine that dramatically
reduces network, memory, and CPU usage, gaining efficiency through a
number of optimizations, including the **shared-nothing** and
**thread-per-core** design patterns and extensive use of **zero-copy**
data types.

The OTAP Dataflow Engine is **embeddable software**. Our repo builds a
demonstration `df_engine` artifact with core nodes included and
features YAML configuration, but really it's designed for safely
adding OpenTelemetry features and capabilities in other programs,
anywhere that Rust can be compiled, with fine-grained control over
memory and CPU resources.

The OTAP Dataflow Engine has built-in support for OTAP and OTLP,
receivers and exporters, and built-in procsesors featuring batching,
fanout, failover, retry, and routing by signal type. It has processors
for common forms of filtering, transforming, sampling, and temporal
aggregation. We have a **durable buffer** processor, based on the
[Arrow IPC format](https://arrow.apache.org/docs/format/IPC.html),
introducing disk-based storage into pipelines for reliable delivery,
and there are others such as a Syslog receiver and a Console exporter.

Our transform procsesor is built using [Apache
Datafusion](https://datafusion.apache.org/), the industry-leading
embedded query engine, itself based on [Apache
Arrow](https://arrow.apache.org/), and our Parquet exporter for OTAP
makes OpenTelemetry data directly accessible to wide range of tools,
thanks to the [Apache Parquet](https://parquet.apache.org/) ecosystem.

We're [self-instrumenting
ourselves](./rust/otap-dataflow/crates/telemetry/README.md) with an
experimental OpenTelemetry SDK that emits OTAP directly, meaning we
have an **end-to-end column-oriented telemetry pipeline** in Rust.

Our Golang Collector components
[`otelarrowreceiver`](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md)
and
[`otelarrowexporter`](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md)
have been included in the OpenTelemetry Collector-Contrib distribution
since [the July 2024 release of
v0.104.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.104.0).

Our Rust and Go codebases are both used in production. Our project has
55+ contributors and 1500+ pull requests merged.  **[Join us in
`#otel-arrow` on the CNCF Slack!](https://cloud-native.slack.com/archives/C07S4Q67LTF)**

## What is Apache Arrow?

The [Apache Arrow](https://arrow.apache.org/) is a major open-source
project for in-memory and on-wire data exchange using a
columnn-oriented representation. For OpenTelemetry readers, Apache
Arrow is a lot like us, as the project encompasses a data format, a
set of libraries, and an ecosystem. 

The Apache Arrow format is a specification for the in-memory layout of
a **record batch**, including details about the schema, the column
names and types, and a length, and then a set of Arrays, one per
column of the correct type and matching length. Arrow record batches
support a number of types, including scalars of various width, strings
and binary data, arrays, lists, structs, maps, as well as
dictionary-encodings over the other types.

Apache Arrow libraries follow standard patterns and naming
conventions, organized around specification documents, and as with
OpenTelemetry, the libraries have each their own ways to address
language-specific challenges. Using an Apache Arrow library to
construct a record batch makes it easy and efficient to exchange data
across language, process, and network boundaries. You can build a
record batch in one language, and pass it to another language without
copying the data. Record batches support zero-copy operations, and for
network and file-based communications, each library includes a reader
and writer for [Arrow IPC](https://arrow.apache.org/docs/format/IPC.html).

## What is OTAP?

OTAP is formally the **OpenTelemetry Protocol with Apache Arrow**,
abbreviated **OTAP** for OTel-Arrow Protocol, a column-oriented
representation for OpenTelemetry data supporting efficient in-memory
and on-wire telemetry exchange. Where OpenTelemetry's OTLP is a
row-oriented protocol, OTel-Arrow's OTAP protocol uses Apache Arrow to
encode telemetry in a columnar format that is more efficient for CPUs
to work with, because of vectorization, and compresses dramatically
better, especially over long-lived streams with the use of [Arrow
IPC](https://arrow.apache.org/docs/format/IPC.html) stream encoding.

OTAP maintains 100% compatibility with the OpenTelemetry data model,
with a straight-forward and non-lossy round trip from OTLP to OTAP and
back.  In both our Rust and Golang implementations, we support
combined OTLP and OTAP services on a single port, making OTel-Arrow
OTAP/OTLP receivers and exporters effective as drop-in replacements
for OTLP receivers and exporters, and making it easy migrate between
the two protocols.

In the OTAP Dataflow Engine, batches of OTAP data are represented
using **multiple record batches** in an arrangement referred to as a
"star schema". The number of record batches varies by OpenTelemetry
signal type, see our [data model documentation](./docs/data_model.md)
for details. Adapter libraries for conversion between OTAP and OTLP
representations are provided in
[Rust](./rust/otap-dataflow/crates/pdata/README.md) and
[Golang](./go/README.md) in this repository.

## Quick Start

### OpenTelemetry Collector example

OTel-Arrow components ship in the [OpenTelemetry
Collector-Contrib][COLLECTOR-CONTRIB] distribution. The two components
extend the configuration model and settings of the core OTLP receiver
and exporter. By design, you can swap `otlp` for `otelarrow` in your
Collector configuration.  To locate one, see [OpenTelemetry Collector
releases](https://opentelemetry.io/docs/collector/#releases).

See the [Exporter][EXPORTER] and [Receiver][RECEIVER] docs for
complete and up-to-date configuration details.

### OTAP Dataflow Engine example

**We are not at this time providing pre-built OTAP Dataflow Engine
releases.** Developers can build and test our code with the following:

```bash
git clone https://github.com/open-telemetry/otel-arrow.git
cd otel-arrow/rust/otap-dataflow
cargo test --workspace
cargo build --release --workspace
```

A [directory of example
configurations](./rust/otap-dataflow/configs/README.md) is
provided. For example, to print syslog records to the console:

```
./target/release/df_engine -c ./configs/syslog-console.yaml  --num-cores=1
```

See the [OTAP Dataflow Engine
documentation](rust/otap-dataflow/README.md) for more details.

[COLLECTOR-CONTRIB]: https://github.com/open-telemetry/opentelemetry-collector-contrib
[RECEIVER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md
[EXPORTER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md

---

## Project Phases

### Phase 1 — Wire Protocol ✅

*Completed 2023–2024*

Phase 1 delivered OTAP as a wire protocol between OpenTelemetry
Collectors, achieving 30–50%+ better compression than OTLP+ZSTD. The
`otelarrow` exporter and receiver are production-ready in
Collector-Contrib at Beta stability.

→ [Phase 1 overview and benchmark results](docs/phase1-overview.md)

### Phase 2 — OTAP Dataflow Engine 🚀

*Substantially complete — 2025*

Phase 2 took the columnar approach end-to-end: a full dataflow engine
in Rust where Apache Arrow record batches are the native in-memory
representation throughout the pipeline.

**Why it matters:**

| Capability | Detail |
|---|---|
| **Zero-copy conversion** | Custom protobuf implementation converts between OTLP bytes and OTAP records without intermediate objects |
| **Thread-per-core architecture** | Share-nothing design with single-threaded async per CPU core — no lock contention on the data path |
| **20+ specialized crates** | Receivers, processors, exporters for OTAP, OTLP, Syslog/CEF, Parquet, and more |
| **Column-oriented processing** | Operations like attribute renaming are O(1) schema mutations, not per-record work |
| **Durable buffering** | Arrow IPC-based persistent queue with write-ahead log |
| **Built-in observability** | Admin HTTP portal, Prometheus metrics, self-diagnostic telemetry |

**Performance highlights** — on a single CPU core:

- **100k+ logs/sec** throughput (standard load)
- **Linear scaling** across cores (1, 2, 4, 8, 16 core benchmarks)
- **Predictable memory** — `Memory (MiB) = C + N × R` per-core model
- **Minimal idle overhead** validated across core counts

→ [Live continuous benchmarks](https://open-telemetry.github.io/otel-arrow/benchmarks/continuous/) ·
[Nightly benchmark suite](https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/) ·
[Full benchmark details](docs/benchmarks.md) ·
[OTAP Dataflow Engine](rust/otap-dataflow/README.md) ·
[Phase 2 design document](docs/phase2-design.md)

### Phase 3 — What's Next 🗺️

Phase 2 has demonstrated that a column-oriented, Arrow-native pipeline
delivers transformative performance for OpenTelemetry. Now we're ready
to work with the community on what comes next. Ideas under discussion
include:

- Extending OpenTelemetry client SDKs with native OTAP support
- Integrating OTAP pipelines with the OpenTelemetry Collector (via FFI, WASM, or direct support)
- DataFusion-powered query and transformation of telemetry data
- Native multi-variate metrics support
- Parquet output for the Arrow ecosystem

Phase 3 planning is happening now — [join the
conversation](https://cloud-native.slack.com/archives/C07S4Q67LTF).

→ [Project phases overview](docs/project-phases.md)

---

## Repository Layout

```
go/                  Golang reference implementation (Phase 1 — OTLP↔OTAP conversion)
rust/otap-dataflow/  OTAP Dataflow Engine (Phase 2 — 20+ Rust crates)
proto/               OTAP protocol buffer definitions
collector/           Test collector (otelarrowcol) and examples
docs/                Specifications, benchmarks, and design documents
```

## Documentation

| Document | Description |
|---|---|
| [OTAP Spec](docs/otap-spec.md) | Formal protocol specification |
| [Data Model](docs/data_model.md) | Arrow schema mappings for OTLP entities |
| [OTAP Basics](docs/otap_basics.md) | Introduction to the OTAP protocol |
| [Phase 2 Design](docs/phase2-design.md) | End-to-end pipeline architecture |
| [Benchmarks](docs/benchmarks.md) | Current performance results |
| [Phase 1 Overview](docs/phase1-overview.md) | Wire protocol details and historical benchmarks |
| [Dataflow Engine](rust/otap-dataflow/README.md) | Rust crate architecture and component reference |

## Contributing

We meet weekly — alternating between Tuesday at 4:00 PM PT and
Thursday at 8:00 AM PT. Check the [OpenTelemetry community
calendar][OTELCAL] for dates and Zoom links.

The meeting is open to everyone, regardless of experience level. We'd
love to have you!

- [Contribution guidelines](CONTRIBUTING.md)
- [Meeting notes](https://docs.google.com/document/d/1z8_Ra-ALDaYNa88mMj1gOZtOpLZLRk0-dZEmDjPmcUs)
- [CNCF Slack `#otel-arrow`](https://cloud-native.slack.com/archives/C07S4Q67LTF)

### Thanks to All Contributors

[![OpenTelemetry-Arrow
contributors](https://contributors-img.web.app/image?repo=open-telemetry/otel-arrow)](https://github.com/open-telemetry/otel-arrow/graphs/contributors)

## References

- [OpenTelemetry Project](https://opentelemetry.io/)
- [OpenTelemetry Specification](https://github.com/open-telemetry/opentelemetry-specification)
- [Apache Arrow](https://arrow.apache.org/)
- [Project definition](https://github.com/open-telemetry/community/blob/main/projects/otelarrow.md)
- [OTEP-0156: Columnar Encoding](https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md)
- [Donation request](https://github.com/open-telemetry/community/issues/1332)
- [OTel-Arrow blog: Introduction](https://opentelemetry.io/blog/2023/otel-arrow/)
- [OTel-Arrow blog: Production Readiness](https://opentelemetry.io/blog/2024/otel-arrow-production)

[OTELCAL]: https://github.com/open-telemetry/community/blob/main/README.md#calendar
