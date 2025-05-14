# Pipeline Performance Testing Tool

This tool runs performance tests on the OpenTelemetry pipeline, measuring the
throughput of logs between a load generator and a collector, and then to a
backend service.

## Features

- Supports deployment to both Docker and Kubernetes
- Basic metrics:
  - Logs attempted, sent, failed, and received
  - Identifies failures at loadgen and in transit
  - Calculates loss rate and percentages
- Customizable test duration
- Records detailed results to a file for later analysis

## Prerequisites

### For Docker mode

- Docker installed and running
- Python 3.7 or later
- Required Python packages: `pip install -r requirements.txt`

### For Kubernetes mode

- A Kubernetes cluster and `kubectl` configured
- For the backend image, you need to build and make it available to your K8s
  cluster

## Usage

The orchestrator tracks logs through the entire pipeline, identifying where logs
are lost - whether at the load generator or in transit through the collector to
the backend service.

### Running with Docker

```bash
python3 orchestrator/orchestrator.py --collector-config system_under_test/otel-collector/collector-config.yaml --duration 30
```

### Running with Kubernetes

```bash
python3 orchestrator/orchestrator.py --deployment-target kubernetes --k8s-collector-manifest system_under_test/otel-collector/collector-manifest.yaml --k8s-backend-manifest backend/backend-manifest.yaml --k8s-loadgen-manifest load_generator/loadgen-manifest.yaml --k8s-namespace perf-test-otel --duration 30
```

## Command Line Options

### General Options

- `--duration` - Duration of the test in seconds (default: 10)
- `--keep-resources` - Don't delete resources after the test (useful for
  debugging)
- `--results-dir` - Directory to store test results (default: ./results)
- `--deployment-target` - Whether to deploy to "docker" or "kubernetes"
  (default: docker)

### Docker-specific Options

- `--collector-config` - Path to the OTEL collector configuration file
- `--skip-backend-build` - Skip building the backend Docker image (use existing
  image)
- `--skip-loadgen-build` - Skip building the loadgen Docker image (use existing
image)

### Kubernetes-specific Options

- `--k8s-namespace` - Kubernetes namespace to deploy to (default: default)
- `--k8s-collector-manifest` - Path to the collector Kubernetes manifest YAML
- `--k8s-backend-manifest` - Path to the backend Kubernetes manifest YAML
- `--k8s-loadgen-manifest` - Path to the load generator Kubernetes manifest YAML

## Results

The test results are saved to a file in the specified results directory. The
file contains:

- Test timestamp
- Test duration
- Deployment target and configuration details
- Total logs attempted
- Logs successfully sent by loadgen
- Logs failed at loadgen
- Logs received by backend
- Logs lost in transit
- Actual test duration
- Logs attempt rate (logs/second)
- Total logs lost (both at loadgen and in transit)
- Percentage of logs lost

### Example Output

```txt
Total logs attempted: 10000
Logs successfully sent by loadgen: 9800
Logs failed at loadgen: 200
Logs received by backend: 9750
Logs lost in transit: 50
Duration: 30.00 seconds
Logs attempt rate: 333.33 logs/second
Total logs lost: 250 (2.50% of attempted logs)
```
