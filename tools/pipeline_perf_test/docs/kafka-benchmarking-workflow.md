# Kafka Benchmarking Workflow

This document maps the benchmark systems in `otel-arrow` and explains where the
Dataflow Engine (DFE) Kafka receiver benchmarks fit. It describes the current
repository state; it does not define performance conclusions or replace the
instructions for reproducing a specific run.

## Terminology

- **DFE**: the Rust OTAP Dataflow Engine under `rust/otap-dataflow`.
- **OTLP**: the row-oriented OpenTelemetry Protocol protobuf encoding.
- **OTAP**: the OpenTelemetry Arrow Protocol encoding.
- **Suite**: a YAML test definition that selects components, configurations,
  workloads, monitoring, test steps, and reports.
- **Orchestrator**: the Python framework under `tools/pipeline_perf_test` that
  renders and executes suites.

## Benchmark Systems

| Benchmark system | Purpose | Definition and entry point | Automation | Publication |
| --- | --- | --- | --- | --- |
| Go protocol benchmarks | Compare OTLP and OTAP protocol behavior, including compression and profiling | `go/pkg/benchmark`, `go/tools/*_benchmark`, and `docs/benchmarks-phase1.md` | No dedicated benchmark workflow | Historical results are checked into `docs/benchmarks-phase1.md` |
| Rust microbenchmarks | Measure functions and components in isolation with Criterion | `rust/otap-dataflow/benchmarks`, crate-local `benches` directories, and `cargo bench` | `.github/workflows/rust-bench.yml` runs on labeled pull requests | Workflow logs only; no chart publication |
| Pipeline performance tests | Measure complete telemetry pipelines with controlled load, monitoring, and reports | `tools/pipeline_perf_test` and `orchestrator/run_orchestrator.py` | Pull request, continuous, nightly, label-triggered, and manual workflows | Selected continuous and nightly JSON results are published to the `benchmarks` branch |
| Comparison dashboard suites | Compare engines, protocols, and isolated components using a common suite model | `tools/comparison_dashboard/dashboard.py`, `manifest.yaml`, and `suites/` | Site rebuild only; suite execution is not automated | Suite data and the static site are stored on the `benchmarks` branch |

The top-level Phase 2 result index is
[`docs/benchmarks.md`](../../../docs/benchmarks.md). The comparison dashboard is
a separate presentation path built from suite data under
`docs/comparison_data` on the `benchmarks` branch.

## Pipeline Performance Framework

The pipeline framework uses declarative YAML and a plugin-based orchestrator.
The main pieces are:

- `test_suites/`: runnable suite configurations.
- `test_suites/*/templates/`: reusable component, engine, step, and report
  templates.
- `orchestrator/`: suite loading, lifecycle coordination, monitoring, hooks,
  reporting, and command-line execution.
- `load_generator/`: configurable telemetry workload generation.
- `results/`: output from the integration suites.

A suite can deploy components as processes, Docker containers, or Kubernetes
resources. Current CI performance workflows use Docker. Monitoring combines
container statistics with component Prometheus endpoints. Reports derive
throughput, delivery loss, CPU, memory, and network metrics from a controlled
observation window.

The primary direct entry point is:

```shell
cd tools/pipeline_perf_test
python orchestrator/run_orchestrator.py --config <suite-config>
```

## Automated Execution

| Workflow | Trigger | Environment | Scope | Result handling |
| --- | --- | --- | --- | --- |
| `rust-ci.yml` | Pull request validation | GitHub-hosted `ubuntu-latest` | Runs the standard `100klrps-docker.yaml` pipeline suite | Required validation only; does not update charts |
| `rust-bench.yml` | Pull request with the `cargobench` label | GitHub-hosted `ubuntu-latest` | Runs `cargo bench` for Rust workspaces and contrib nodes | Results remain in workflow logs |
| `pipeline-perf-on-label.yaml` | Pull request with the `pipelineperf` label | `oracle-bare-metal-64cpu-1024gb-x86-64-ubuntu-24` | Runs standard, scaling, saturation, and pass-through suites | Does not update charts |
| `pipeline-perf-test-manual-pr.yaml` | Manual dispatch with a pull request number and suite path | Oracle bare-metal runner | Runs the selected integration suite against the pull request merge ref | Sets the `perf-tests` commit status; does not update charts |
| `pipeline-perf-test-continuous.yml` | Push to `main` affecting Rust, performance tools, dashboard files, or the workflow; also manual dispatch | Oracle bare-metal runner | Runs `100klrps-docker.yaml` | Uploads JSON, consolidates it, and pushes chart data to `docs/benchmarks/continuous` on `benchmarks` |
| `pipeline-perf-test-nightly.yml` | Daily at 16:00 UTC and 00:00 UTC; also manual dispatch | Oracle bare-metal runner | Runs Syslog, batch-size, backpressure, filter, ClickHouse, idle, scaling, saturation, and pass-through suites | Uploads and consolidates JSON, then pushes scenario-specific chart data to `docs/benchmarks/*` on `benchmarks` |
| `comparison-dashboard.yml` | Push to `main` changing non-Markdown comparison-dashboard files; also manual dispatch | GitHub-hosted `ubuntu-latest` | Rebuilds the static comparison site from data already on `benchmarks` | Commits `docs/compare` and derived `docs/comparison_data` files directly to `benchmarks` |

The Oracle runner provides stable, dedicated capacity for meaningful pipeline
measurements. Pull request validation on `ubuntu-latest` is useful as a
functional smoke test but is not a controlled capacity baseline.

## Result Paths

The regular pipeline workflows and the comparison dashboard have different
result formats and destinations.

### Continuous and Nightly Charts

1. The orchestrator writes GitHub Actions benchmark JSON below
   `tools/pipeline_perf_test/results`.
2. The workflow uploads those files as artifacts.
3. `.github/workflows/scripts/consolidate-benchmarks.sh` combines the files.
4. `benchmark-action/github-action-benchmark` writes chart data to the
   `benchmarks` branch.
5. GitHub Pages serves the data below:
   - <https://open-telemetry.github.io/otel-arrow/benchmarks/continuous/>
   - <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/>

### Comparison Dashboard

`dashboard.py` provides four commands:

- `validate`: validate the manifest, suites, and comparisons.
- `run`: render and execute one or more suites through the orchestrator.
- `build`: generate static comparison pages and per-suite `data.js` files.
- `serve`: serve the generated site locally.

By default, a local run stages artifacts in
`.data/<suite-slug>/<timestamp>/` and publishes them to
`.site/data/suite/<suite-slug>/`. A publication run can override `--data-dir`;
the deployed branch layout is:

```text
docs/
|-- comparison_data/
|   `-- suite/
|       `-- <suite-slug>/
|           |-- suite.yaml
|           |-- data.js
|           |-- run_env.json
|           `-- <test-name>/
`-- compare/
    |-- index.html
    `-- <comparison-slug>/
        `-- index.html
```

The comparison publisher does not execute suites. It checks out `main` for the
dashboard implementation and `benchmarks` for existing data, rebuilds the site,
and pushes the generated files to `benchmarks`.

## DFE Kafka Receiver Suites

The comparison-dashboard manifest registers two DFE Kafka receiver suites:

- `suites/dfe/dfe-logs-kafka-otlp-recv-baseline.yaml`
- `suites/dfe/dfe-logs-kafka-otap-recv-baseline.yaml`

Both use
`test_suites/comparison_dashboard/templates/orchestrator/dfe-kafka-recv-single-core-multi-rate.yaml`.
The receiver-isolation topology is:

```text
DFE traffic generator
  -> fixed DFE Kafka exporter
  -> single-broker Kafka cluster
  -> benchmarked DFE Kafka receiver
  -> fixed DFE OTLP or OTAP backend
```

Only the Kafka consumer process is the system under test. The producer and
backend are fixed references so receiver work is not mixed with exporter
performance.

The current suites:

- generate synthetic logs with 12 attributes;
- test 100k, 200k, 300k, 400k, 600k, 800k, and 1,000k records per second;
- allocate one core to the benchmarked Kafka consumer;
- use one Kafka broker and one-partition topics;
- test uncompressed OTLP protobuf and OTAP payloads;
- use a 20-second observation interval by default;
- report produced and received log rates, dropped-log percentage, normalized
  average and maximum CPU, average and maximum memory, and network rates.

The report logic is in
`test_suites/comparison_dashboard/reports/report_logs.yaml`. Engine and broker
configuration is rendered from the templates below
`test_suites/comparison_dashboard/templates`.

Published data currently exists on the `benchmarks` branch at:

```text
docs/comparison_data/suite/dfe_logs_kafka_otlp_recv/
docs/comparison_data/suite/dfe_logs_kafka_otap_recv/
```

## Running a Kafka Suite Locally

The detailed clean-machine reproduction procedure is maintained as separate
work because it must be verified on a real host. The current entry point is:

```shell
cd tools/comparison_dashboard
python dashboard.py validate
python dashboard.py run \
  suites/dfe/dfe-logs-kafka-otlp-recv-baseline.yaml \
  --tests 100k
```

At minimum, execution requires:

- Git submodules initialized;
- Docker with Buildx and Compose support;
- a locally built `df_engine:latest` image;
- Python and the comparison-dashboard and orchestrator dependencies;
- enough CPU, memory, disk, and available local ports for four containers.

Use `--generate-only` first to render and inspect the orchestrator configuration
without starting containers. A performance result should also record the commit
SHA, host topology, software versions, rendered suite variables, and generated
artifacts.

## Current Kafka Workflow Gaps

The following gaps are supported by the current workflow and manifest files:

1. No GitHub Actions workflow invokes a Kafka comparison-dashboard suite.
2. Existing Kafka suite data therefore has no recurring refresh path.
3. `comparison-dashboard.yml` rebuilds the site but does not produce benchmark
   data.
4. The manifest registers the Kafka suites but no comparison definition
   references them, so they do not have a dedicated comparison page.
5. Receiver coverage is limited to logs encoded as OTLP or OTAP. There is no
   Syslog-over-Kafka suite and no Kafka receiver suite for metrics or traces.
6. The manifest uses floating `df_engine:latest` and `apache/kafka:latest`
   image references, which are not sufficient by themselves to reproduce a
   historical result.
7. The comparison-dashboard README describes publication concepts that differ
   from the implemented workflow: the workflow is triggered by changes on
   `main` and pushes directly to `benchmarks`; it does not watch benchmark data
   changes on `benchmarks` or open a pull request.

These are observations, not implementation decisions. Recurring execution,
publication, regression thresholds, expanded coverage, and documentation fixes
should be prioritized and tracked as follow-up work.

## Source Map

- General benchmark results: `docs/benchmarks.md`
- Historical Phase 1 results: `docs/benchmarks-phase1.md`
- Rust Criterion benchmarks: `rust/otap-dataflow/benchmarks/README.md`
- Pipeline framework overview: `tools/pipeline_perf_test/readme.md`
- Orchestrator documentation: `tools/pipeline_perf_test/orchestrator/`
- Comparison dashboard documentation:
  `tools/comparison_dashboard/README.md`
- Comparison suite registry: `tools/comparison_dashboard/manifest.yaml`
- Kafka receiver documentation:
  `rust/otap-dataflow/crates/contrib-nodes/src/receivers/kafka_receiver/README.md`
- Workflow definitions: `.github/workflows/`
