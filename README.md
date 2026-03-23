# OpenTelemetry Protocol with Apache Arrow

[![Slack](https://img.shields.io/badge/slack-@cncf/otel/arrow-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C07S4Q67LTF)
[![Go-CI](https://github.com/open-telemetry/otel-arrow/actions/workflows/go-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/go-ci.yml)
[![Rust-CI](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml)
[![OpenSSF Scorecard for otel-arrow](https://api.scorecard.dev/projects/github.com/open-telemetry/otel-arrow/badge)](https://scorecard.dev/viewer/?uri=github.com/open-telemetry/otel-arrow)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/10684/badge)](https://www.bestpractices.dev/projects/10684)
[![codecov](https://codecov.io/gh/open-telemetry/otel-arrow/graph/badge.svg?token=7u3gFLH54G)](https://codecov.io/gh/open-telemetry/otel-arrow)

**High-performance, column-oriented telemetry pipelines for
[OpenTelemetry](https://opentelemetry.io/), powered by [Apache
Arrow](https://arrow.apache.org/).**

OTel-Arrow brings the efficiency of columnar data processing to
OpenTelemetry. We've built a production-ready wire protocol that
dramatically reduces network overhead (Phase 1), and an end-to-end
dataflow engine in Rust that processes telemetry at 100k+ records/sec
on a single CPU core with minimal memory overhead (Phase 2). With 55+
contributors, 1800+ commits, and 2300+ pull requests, the project is
entering an exciting new chapter — and we'd love your help shaping
what comes next.

**[Join us in `#otel-arrow` on CNCF Slack →](https://cloud-native.slack.com/archives/C07S4Q67LTF)**

---

## What Is OTAP?

The **OpenTelemetry Protocol with Apache Arrow** (OTAP) is a
column-oriented transport for OpenTelemetry data. Where the standard
OTLP protocol is row-oriented and stateless, OTAP uses Apache Arrow to
encode telemetry in a columnar form that compresses dramatically
better, especially over long-lived streams. OTAP maintains 100%
compatibility with the OpenTelemetry data model — it's a better
encoding, not a different schema.

Key properties:

- **Drop-in compatible** with existing OTLP infrastructure — seamless
  fallback on the same gRPC port
- **30–50%+ compression improvement** over OTLP+ZSTD for all signal
  types (logs, traces, metrics)
- **Streaming efficiency** through per-stream dictionary encoding,
  delta dictionaries, and schema deduplication
- **Formal specification** — see the [OTAP spec](docs/otap-spec.md)
  and [data model](docs/data_model.md)

## Quick Start

### Using OTAP Today (Golang Collector Components)

OTel-Arrow components ship in the [OpenTelemetry Collector-Contrib][COLLECTOR-CONTRIB]
distribution. Swap `otlp` for `otelarrow` in your collector config:

```yaml
receivers:
  otelarrow:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

exporters:
  otelarrow:
    endpoint: backend:4317
```

See the [Exporter][EXPORTER] and [Receiver][RECEIVER] docs for full
configuration options.

### Building the OTAP Dataflow Engine (Rust)

```bash
git clone https://github.com/open-telemetry/otel-arrow
cd otel-arrow/rust/otap-dataflow
cargo build --workspace
cargo test --workspace
```

See the [OTAP Dataflow Engine documentation](rust/otap-dataflow/README.md) for
architecture details, example configurations, and component reference.

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
