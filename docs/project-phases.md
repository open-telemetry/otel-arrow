# OTel-Arrow Project Phases

## Overview

This document outlines a phased implementation approach for the OTel-Arrow project. For a complete project overview, please refer to the [top-level README.md](../README.md).

OTel-Arrow aims to integrate OpenTelemetry with Apache Arrow to enable high-performance telemetry data processing. The project will evolve through multiple phases, each delivering specific functionality while incrementally expanding the project's capabilities and scope.

## Phase 0: Foundation documents

**Timeline:** 2021-2023

**Key Deliverables:**

- Initial demo created by @lquerel showcasing the concept ([YouTube presentation](https://www.youtube.com/watch?v=9dGGjREaggY), December 2021)
- OpenTelemetry Technical Committee sponsorship by @jmacd
- OpenTelemetry Enhancement Proposal (OTEP) for columnar encoding ([OTEP-0156](https://github.com/open-telemetry/opentelemetry-specification/blob/main/oteps/0156-columnar-encoding.md)).

**Success Criteria:**

- Demonstration of feasibility and potential performance benefits
- OpenTelemetry spec-approvers accepted the OTEP, June 2023.

## Phase 1: Streaming compression capability

**Objective:** Establish the mapping between OpenTelemetry data types and Apache Arrow columnar format, with emphasis on streaming compression results.

**Timeline:** 2023-2024

**Key Deliverables:**

- Arrow schema definitions for OpenTelemetry spans, metrics, and logs
- Core library for serializing/deserializing between OTel and Arrow formats in Golang
- Define multi-variate OTel-Arrow metrics representation compatible with OpenTelemetry metrics data model
- Benchmark suite comparing CPU/memory/compression performance against OTLP
- Unit tests and validation tools
- OpenTelemetry Collector-contrib [exporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md) and [receiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md) components
- [Project kick-off blog post](https://opentelemetry.io/blog/2023/otel-arrow/)
- [Phase 1 completed blog post](https://opentelemetry.io/blog/2024/otel-arrow-production).

**Success Criteria:**

- 100% compatibility with OTLP; non-lossy bi-directional translation (including multi-variate metrics)
- Seamless transition supporting combined OTAP/OTLP transport modes on the same port
- Uses Apache Arrow IPC over gRPC streams for compatibility with OpenTelemetry ecosystem
- Compression improvements of at least 30% for all signals, typical 50% improvement compared with gRPC-OTLP/zstd.

**Restrictions and governance:**

- Although the prototype and original demo was given in Rust, the project commits to working in the Golang ecosystem
- Compatibility commitment: the project aims at making OTLP and OTAP as compatible as possible and will support all signals through Golang components in Collector-Contrib.

## Phase 2: End-to-end pipeline with Arrow in Rust

**Objective:** Establish a foundation for working with OTel-Arrow data in Rust, for access to the Arrow-Rust ecosystem.

**Timeline:** 2025

**Key Deliverables:**

- In-process OTAP pipeline implemented as Rust libraries
- Explore API design for column-oriented pipeline data using OTAP data frames (i.e., `pdata`)
- Prototype for DataFusion integration with OpenTelemetry data, OTTL-transform feasibility study
- Benchmarks measuring OTAP and OTLP pipelines in Rust and Golang.

**Success Criteria:**

- Interoperability testing between Golang components from Phase 1
- OTAP/Rust gains 2x to 10x in data processing speed compared with OTLP/Golang, depending on pipeline configuration and complexity, at lower memory cost, and with better compression
- Summarize what it would look like to implement OTAP pipelines directly in Golang
- Feasibility study: how to integrate Rust OTAP pipelines as foreign function calls from Golang
- Demonstration of the value of integrating OpenTelemetry data with Apache Arrow.

**Restrictions and governance:**

- We are not building a Rust Collector; we are building OTAP pipelines as embeddable software libraries with access to the Apache Arrow ecosystem in Rust
- We are not building a Rust Collector; we are evaluating an end-to-end OTAP pipeline, including an experimental "OTAP-direct" SDK in Rust
- We will not publish software in source or binary form that acts like a stand-alone Collector
- We will (intentionally) not support parsing YAML configuration files to configure pipeline graphs
- We will not interfere with OpenTelemetry Collector or OpenTelemetry Rust during this phase by asking those teams to review or accept our work.

## Future Phases

Additional project phases will be defined as the project evolves, based on the outcome of earlier phases.

Phase N+1 planning will be discussed when Phase N comes to a close.
