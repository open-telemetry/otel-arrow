# OTAP Dataflow Integration Test Suite

This directory contains integration tests for the otap-dataflow engine.
These tests validate various configurations and data paths through the
engine under different load and protocol scenarios.

## Directory Layout

```shell
integration/
|-- configs/                # Rendered configuration files used in test runs
|   |-- backend/
|   |-- engine/
|   |-- loadgen/
|   |-- integration_report_logs.yaml
|-- continuous/             # Continuous integration test configurations (e.g. 100k LRPS)
|-- nightly/                # Nightly, longer-running test configs (e.g. syslog scenarios)
|-- templates/              # Jinja2 templates for configs and test step workflows
    |-- configs/            # Templates in this directory are rendered into integration/configs/. and run by components
    |-- test_steps/
```

## Running The Suites

Running the test suites is currently a 2 step process:

1. Manually build the dataflow engine container.
2. Run the orchestrator python script with one of the suite configurations.

### Build Instructions

Docker images must be built manually due to current build-context
limitations with proto compilation:

```shell
# From the rust/otap-dataflow directory
docker build \
  --build-context otel-arrow=../../ \
  -f Dockerfile \
  -t df_engine .
```

Automated Docker builds are not yet supported. See comments in the suite
config for details.

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
# OR
python ./orchestrator/run_orchestrator.py --debug --config ./test_suites/integration/nightly/syslog-docker.yaml
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
