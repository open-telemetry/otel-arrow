# Pipeline Performance Test - Dataflow Engine with OTLP Logs

This repository contains a performance testing suite to evaluate the dataflow
engine's handling of OTLP logs.

## Prerequisites

Before running the tests, you need to set up a virtual environment and install
the required dependencies.

### 1. Set Up a Virtual Environment

If you don't already have a virtual environment for Python, you can create one
using the following commands:

```bash
python3 -m venv .venv
source .venv/bin/activate  # On Windows use: .venv\Scripts\activate
```

### 2. Install Dependencies

After activating your virtual environment, install the required dependencies for
the project:

```bash
pip install -r ./otel-arrow/tools/pipeline_perf_test/orchestrator/requirements.txt
pip install -r ./otel-arrow/tools/pipeline_perf_test/backend/requirements.txt
```

This will install all the necessary Python packages, including dependencies for
orchestrating and running performance tests.

### 3. Download Otel Collector Binary

This suite uses subprocess-based deployment, so an otel-collector is required at
the expected path.

See [Collector Installation](https://opentelemetry.io/docs/collector/installation/).

The defaults are set as follows (you can modify line 31 of the test-suite-comparison
with your path).

```yaml
    deployment:
      process:
        # Path to your otel collector binary
        command: /tmp/otelcol --config ./test_suites/dataflow_engine_e2e_logs/collector-config-without-batch-processor.yaml
```

## Running the Test Suite

The test suite is executed using the run_orchestrator.py script. This script
will orchestrate the deployment, execution, monitoring, and reporting phases
of the test based on the configuration file.

To run the test suite, execute the following command
(from the otel-arrow/tools/pipeline_perf_test directory):

```bash
python ./orchestrator/run_orchestrator.py --config ./test_suites/dataflow_engine_e2e_logs/test-suite-comparison.yaml --debug
```
