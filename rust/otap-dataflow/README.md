# OTAP Pipeline Library

[![build](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml)
[![build](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-audit.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-audit.yml)
[![codecov](https://codecov.io/gh/open-telemetry/otel-arrow/graph/badge.svg?token=tmWKFoMT2G&component=otap-dataflow)](https://codecov.io/gh/open-telemetry/otel-arrow)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Slack](https://img.shields.io/badge/Slack-OTEL_Arrow-purple)](https://cloud-native.slack.com/archives/C07S4Q67LTF)

----

[Quickstart Guide]((#-quickstart-guide))
| [Design Principles](docs/design-principles.md)
| [Architecture](docs/architecture.md) | [Change log](CHANGELOG.md)
| [Contributing](CONTRIBUTING.md) |

> [!NOTE]
> This Rust library will be the main deliverable of phase 2 of the
> otel-arrow project, as defined in this
> [PR](https://github.com/open-telemetry/community/pull/2634).
>
> The other Rust projects located in the same root directory as this project
> will gradually be integrated into it.

## Overview

TBD

## Features

TBD

## Use Cases

Some examples of use cases include:

- End-to-end OTAP pipeline
- ...

## Why

TBD (Explain why Rust, why Arrow, ...)

## :construction: Workspace Structure

```text
.
|-- Cargo.toml
|-- crates
|   |-- channel        # Async Channel Implementations
|   |-- config         # Pipeline Configuration Model
|   |-- engine         # Async Pipeline Engine
|   |-- otap           # OTAP Nodes
|   |-- otlp           # OTLP Nodes
|-- docs               # Documentation
|-- examples           # Rust Examples
|-- src                # Main library source code
|-- xtask              # Xtask for project management
|-- examples           # Examples or demo applications
```

## Quickstart Guide

### Installation

TBD

### :dart: Usage Example

The OTAP dataflow library requires explicit configuration for all processors, including the SignalTypeRouter. Here's a basic example:

```yaml
type: otap
description: "Example pipeline with explicit signal type routing"
settings:
  default_control_channel_size: 100
  default_pdata_channel_size: 100
nodes:
  # OTLP Receiver
  otlp_receiver:
    kind: receiver
    plugin_urn: "urn:otap:receiver:otlp"
    config:
      endpoint: "0.0.0.0:4317"
      protocols:
        grpc:
          enabled: true

  # SignalTypeRouter with explicit configuration
  signal_router:
    kind: processor
    plugin_urn: "urn:otap:processor:signal_type_router"
    out_ports:
      traces_out:
        dispatch_strategy: round_robin
        destinations: [traces_processor]
      metrics_out:
        dispatch_strategy: broadcast
        destinations: [metrics_processor]
      logs_out:
        dispatch_strategy: random
        destinations: [logs_processor]
    config:
      drop_unknown_signals: false

  # Downstream processors
  traces_processor:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    config:
      timeout: "1s"
      send_batch_size: 512

  metrics_processor:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    config:
      timeout: "5s"
      send_batch_size: 1024

  logs_processor:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    config:
      timeout: "2s"
      send_batch_size: 256
```

**Key Features:**
- **Explicit Configuration**: All nodes require complete configuration specification
- **Zero-Copy Routing**: SignalTypeRouter forwards signal references without data duplication
- **Multi-Port Dispatch**: Different signal types route to dedicated output ports
- **Flexible Strategies**: Support for broadcast, round-robin, random, and least-loaded routing

## :books: Documentation

- [Developer/Contributing Guide](CONTRIBUTING.md)
- [Design Principles](docs/design-principles.md)
- [Architecture](docs/architecture.md)
- [Tracing Proposal](docs/tracing-proposal.md)
- [Glossary](docs/glossary.md)

## Development Setup

**Requirements**:

- Rust >= 1.86.0
- Cargo

**Clone & Build**:

```bash
git clone https://github.com/open-telemetry/otel-arrow
cd otel-arrow/rust/otap-dataflow
cargo build --workspace
```

**Run Tests**:

```bash
cargo test --workspace
```

**Run Examples**:

```bash
cargo run --example <example_name>
```

## Contributing

- [Contribution Guidelines](CONTRIBUTING.md)
- Code of Conduct (TBD)

Before submitting a PR, please run the following commands:

```bash
# Prepare and check the entire project before submitting a PR or a commit
cargo xtask check
```

## :memo: License

Licensed under the [Apache License, Version 2.0](LICENSE).

## :telephone_receiver: Support & Community

CNCF Slack channel: [#otel-arrow](https://slack.cncf.io/)

## :star2: Roadmap

See our detailed [Roadmap](ROADMAP.md) for upcoming features and improvements.

## :white_check_mark: Changelog

- [CHANGELOG.md](CHANGELOG.md)
