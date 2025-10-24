# Performance Test Framework for Telemetry Pipeline

## Overview

This framework provides a modular, extensible way to
**benchmark end-to-end telemetry pipelines**.
It allows developers, platform engineers, and observability vendors to:

- Spin up configurable pipeline topologies.
- Run structured performance tests with repeatable workflows.
- Observe and report on system behavior under various load and deployment scenarios.
- Automate benchmarking at scale using CI/CD systems or local environments.

The framework has evolved from simple load tests to a
**strategy-driven orchestration engine** with first-class support for
**component lifecycle management**, **test hooks**, **dynamic configuration**,
and **automated reporting**.

---

## Goals

1. Run performance benchmarks across a full telemetry pipeline - from
  OpenTelemetry SDK clients, through Collectors, to ingestion backends.
2. Enable easy customization and reconfiguration of pipeline components without
  changing the test harness itself.
3. Provide a flexible orchestration model with lifecycle hooks, error handling,
  and reporting.
4. Allow community and vendors to fork and adapt the framework for their own use
  cases.
5. Support both lightweight local runs and scalable, automated performance test environments.

---

## High-Level Architecture

A **test suite** defines all elements of a performance test in YAML: components,
deployment and execution strategies, monitoring, test scenarios, and reporting
hooks.
The orchestrator runs these suites, coordinating setup, execution, teardown, and
data collection.

### Core Concepts

| Concept              | Purpose                                                              |
|-----------------------|-----------------------------------------------------------------------|
| **Components**        | Services or tools under test (e.g., load generators, collectors, backends). |
| **Strategies**        | Plugin-based behaviors for deployment, execution, monitoring, and configuration. |
| **Tests**             | Ordered scenarios made up of steps that deploy, start, monitor, load, and tear down components. |
| **Hooks**             | Extensible automation points at suite, scenario, step, or component level. |
| **Reports**           | Post-test analysis of throughput, latency, resource usage, and trends. |
| **Error Handling**    | Flexible retry and continuation behavior for resilient test execution. |

---

## System Components

### Load Generator

- Pluggable execution strategies.
- Runs as a Docker container or process.
- Supports configurable load profiles (threads, batch sizes, endpoints).

### Telemetry Collector

- Runs latest OpenTelemetry Collector builds or custom versions.
- Configurable via mounted configs or dynamic reconfiguration mid-test.
- Supports multi-instance topologies (agent, sidecar, centralized).

### Backend / Ingestion Service

- Multiple options:
  - Null sink (drop everything)
  - Fake service with telemetry counters
  - Real vendor backends for end-to-end validation

---

## Orchestrator

The **orchestrator** is responsible for coordinating the entire test lifecycle:

- Deploying and tearing down components
- Starting and stopping load and monitoring
- Applying hooks and runtime strategy reconfigurations
- Handling errors and retries gracefully
- Emitting structured events for reporting

This enables reproducible and automated benchmark runs across varied scenarios.

---

## Strategies

All component behaviors are defined via pluggable **strategies**:

- **Configuration** - optional pre-deployment configuration logic (templating,
  remote fetch, etc.)
- **Deployment** - how a component is launched (Docker, process, Kubernetes)
- **Execution** - what it does during the test (e.g., send telemetry)
- **Monitoring** - how its metrics are observed (Prometheus, Docker stats, etc.)

Strategies can be swapped or updated dynamically during a test, enabling advanced
scenarios such as A/B testing collector configurations.

---

## Monitoring

The framework supports multiple monitoring strategies per component.
Examples include:

- **`docker_component`**: resource usage from container runtime (CPU, memory, etc.)
- **`prometheus`**: scraping custom metrics endpoints

Monitoring can be started and stopped at controlled phases, and
**observation windows** can be defined using recorded events.

---

## Test Scenarios

Tests are defined as ordered scenarios that can:

- Deploy and configure components
- Start monitoring and load generation
- Wait for steady state
- Reconfigure components at runtime
- Tear down the environment

Each step can define its own **hooks**, error handling, and timing.
This allows fine-grained control over complex test flows.

---

## Hooks and Automation

Hooks can be attached to:

- **Suite** - before or after all tests (e.g., global reporting)
- **Scenario** - before or after a single test scenario
- **Step** - pre/post logic for individual actions
- **Component** - during deployment, start, stop, etc.

Hooks can:

- Run setup or cleanup logic
- Emit events
- Trigger external systems
- Record metrics or logs
- Handle errors gracefully (`on_error` configuration)

---

## Reporting

The framework generates structured reports from collected metrics and events:

- **`pipeline_perf_report`** - throughput and telemetry pipeline performance
- **`process_report`** - resource utilization (CPU, memory, etc.) for specific components

Reports can be output to the console or other destinations and are typically
defined in suite-level post hooks.

---

## Error Handling

Tests can be resilient by configuring error handling at any level:

- Automatic retries with configurable delays
- Optional continuation on failure
- Consistent behavior across hooks, steps, and component phases

This is particularly useful in distributed or flaky test environments.

---

## Deployment Environments

- **Local**: Clone the repo and run a test suite on a workstation using Docker.
- **CI/CD**: Run nightly or scheduled benchmarks on GitHub Actions or other runners.
- **Cloud**: Scale tests out to larger environments for stress and capacity planning.

---

## Example Use Cases

- **Protocol evaluation**: Compare OTLP vs. OTAP under varying loads.
- **Capacity planning**: Estimate required Collector resources for a given
  telemetry volume.
- **Vendor benchmarking**: Evaluate backend ingestion performance under
  different pipeline topologies.
- **Architecture testing**: Compare agent vs. sidecar vs. centralized deployment
  models.

---

## Roadmap

The framework is actively evolving:

- Expanded configuration templating and remote fetch support
- Additional monitoring and reporting plugins
- Support for mixed telemetry signals and distributed topologies
- Automated baseline comparison and regression tracking

---

## Getting Started

1. Clone the repository.
2. Write a test suite YAML defining components, tests, and hooks - or use one of
    the existing suites.
3. Run the orchestrator to execute the suite locally or in CI.
4. View performance reports and metrics.

Suites in the test_suites directory include detailed setup and execution
instructions. Generally applicable instructions for the continuous integration suite
are provided below for reference.

### Run Test Suites

#### Pre-Reqs (first time)

Ensure you are in the otel-arrow/tools/pipeline_perf_test directory, then:

```shell
# Create and activate a virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -r orchestrator/requirements.txt
```

#### Run The Orchestrator

Ensure you are in the otel-arrow/tools/pipeline_perf_test directory, then:

```shell
python ./orchestrator/run_orchestrator.py --debug --config ./test_suites/integration/continuous/100klrps-docker.yaml
```

```shell
$ python ./orchestrator/run_orchestrator.py --help

usage: run_orchestrator.py [-h] --config CONFIG [--debug] [--otlp-endpoint OTLP_ENDPOINT] [--export-traces] [--export-metrics] [--docker.no-build]

Test Orchestration Framework CLI

options:
  -h, --help            show this help message and exit
  --config, -c CONFIG   Path to test suite YAML config.
  --debug               Enable debug mode (verbose output, etc.).

OTLP Export:
  --otlp-endpoint OTLP_ENDPOINT
                        OTLP exporter endpoint (e.g., http://localhost:4317)
  --export-traces       Enable OpenTelemetry tracing to external otlp endpoint
  --export-metrics      Enable OpenTelemetry metrics export to external otlp endpoint

Docker Options:
  --docker.no-build     Skip build of Docker containers.
```
