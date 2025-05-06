# Performance Test Framework for Telemetry Pipeline

## Goals

1. Run performance benchmarks across a full telemetry pipeline — including
   OpenTelemetry SDK clients, Collector(s), and an ingestion backend, supporting
   the needs of [OTel Arrow
   Phase2](../../docs/phase2-design.md).
2. Allow easy customization of pipeline components (e.g., swapping processors,
   modifying Collector config, etc.).
3. Be designed in a way that the framework itself can be forked and reused by
   vendors and other users — similar to how OpenTelemetry Demo is adopted by the
   community for vendor-specific customization.

## High-Level Components

### System Under Test

- Starts with the latest OTel Collector image and evolves to support
  - Custom Collector builds and config variations
  - Multi-instance topologies via Docker Compose or Kubernetes

### Load Generator

- Initially, a basic script for load generation, later extended to:
  - Use language-specific OpenTelemetry SDKs
  - Integrate load gen tools like Locust or custom telemetry generators

### Ingestion Service / Backend

- Options include:
  - Null Sink (i.e., drop everything)
  - A fake service that drops telemetry but tracks counts.
  - A real backend for validating full pipeline integrity (vendor forks may
    leverage this)

### Orchestrator

- Responsible for:
  - Spinning up and tearing down the pipeline
  - Measuring throughput, loss rate, and system metrics (CPU, memory, disk etc.)
- Can be written using Python for portability and ease of scripting

### Reporting

- Tracks historical results across test runs, enabling:
  - Trend analysis
  - Nightly benchmark runs on few well-defined scenarios

## Example scenarios

Listing a few examples, not in any particular order:

- OTel Arrow developers can use the framework to measure the improvements OTAP
  brings over OTLP under various scenarios.
- A user has a K8S workload to which they want to add OTel Collector based
  pipeline, and they can use the framework to predict how much capacity they
  should plan for for a given expected volume/load.
- An observability vendor can evaluate how their backend performs when receiving
  data through different collector configurations, helping them optimize their
  ingest pipelines and provide guidance to customers.
- Platform engineers can benchmark different collector topologies (e.g.,
  agent-per-node vs. sidecar vs. centralized deployment) to identify the most
  efficient architecture for their environment's scale and requirements.

## Deployment Environments

The framework is planned to be easily accessible, allowing anyone to clone the
repository and run benchmarks locally, whether using the existing configurations
or applying custom modifications. Additionally, nightly or on-demand performance
tests can be executed on dedicated performance testing machines managed by the
OpenTelemetry organization using GitHub Actions. Over time, the framework can
support large-scale testing by enabling deployments in cloud environments.

## Plan for Iterative Development

Given perf benchmarking can be a very extensive project with large scope, I'd
like to start with small steps and iterate quickly based on feedback.

I'd get started with basic load test that uses OTel Collector's latest release,
later adding ability to 'bring-you-own', where users can use a custom built
Collector or leverage OTAP pipelines or customize the config etc.

Then expand to:

- Adding pluggable load generators, such as language-specific OpenTelemetry SDKs
or tools like Locust.
- Supporting various ingestion services, including fake services for telemetry
tracking or real backends for full pipeline validation.
