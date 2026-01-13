# Standard Load Batch Size Tests

## Overview

Batch size testing is split into two test suites:
- **Continuous CI** (`continuous/100klrps-docker.yaml`): Fast validation with baseline batch size
- **Nightly** (`nightly/100klrps-batch-sizes-docker.yaml`): Comprehensive batch size analysis

## Test Configuration

### Continuous CI Tests
- **File**: `test_suites/integration/continuous/100klrps-docker.yaml`
- **Tests**: 4 (one per protocol combination)
- **Batch Size**: 1000 (baseline)
- **Runtime**: ~2-3 minutes
- **Purpose**: Quick smoke test for protocol correctness

### Nightly Batch Size Tests
- **File**: `test_suites/integration/nightly/100klrps-batch-sizes-docker.yaml`
- **Tests**: 12 total
- **Runtime**: ~10-12 minutes

## Test Matrix (Nightly)

The nightly suite runs **12 total tests** covering:

### Protocol Combinations (4)
1. **OTLP → OTLP** - Standard protocol end-to-end
2. **OTLP → OTAP** - Standard ingress, Arrow egress
3. **OTAP → OTAP** - Native Arrow protocol end-to-end
4. **OTAP → OTLP** - Arrow ingress, standard egress

### Batch Sizes (3) - Production-Realistic
- **1000** - OpenTelemetry SDK defaults (most SDKs batch to 512-2048)
- **5000** - Intermediate/aggregating collectors
- **10000** - High-throughput gateway collectors

## Running the Tests

### Continuous CI Tests (Fast)
```bash
cd /home/ubuntu/otel-arrow/tools/pipeline_perf_test
source .venv/bin/activate
python ./orchestrator/run_orchestrator.py --config ./test_suites/integration/continuous/100klrps-docker.yaml
```

### Nightly Batch Size Tests (Comprehensive)
```bash
cd /home/ubuntu/otel-arrow/tools/pipeline_perf_test
source .venv/bin/activate
python ./orchestrator/run_orchestrator.py --config ./test_suites/integration/nightly/100klrps-batch-sizes-docker.yaml
```

## Test Execution Details

Each test will:
1. Deploy backend service (1 core)
2. Deploy df-engine (1 core) 
3. Deploy load generator (1 core)
4. Generate 100,000 log records/second at the specified batch size
5. Monitor for 60 seconds (nightly) or 20 seconds (continuous)
6. Collect CPU, memory, and throughput metrics
7. Clean up containers

## Results Location

- **Continuous**: `results/integration/gh-actions-benchmark/`
- **Nightly**: `results/nightly_100klrps_batch_sizes/gh-actions-benchmark/`

## Test Names (Nightly)

```
OTLP-ATTR-OTLP-BATCH1000
OTLP-ATTR-OTLP-BATCH5000
OTLP-ATTR-OTLP-BATCH10000

OTLP-ATTR-OTAP-BATCH1000
OTLP-ATTR-OTAP-BATCH5000
OTLP-ATTR-OTAP-BATCH10000

OTAP-ATTR-OTAP-BATCH1000
OTAP-ATTR-OTAP-BATCH5000
OTAP-ATTR-OTAP-BATCH10000

OTAP-ATTR-OTLP-BATCH1000
OTAP-ATTR-OTLP-BATCH5000
OTAP-ATTR-OTLP-BATCH10000
```

## Customizing Observation Time

To increase/decrease the observation window for nightly tests, modify the `observation_interval` variable:

```yaml
variables:
  result_dir: nightly_100klrps_batch_sizes
  engine_config_template: test_suites/integration/templates/configs/engine/continuous/otlp-attr-otlp.yaml
  loadgen_exporter_type: otlp
  backend_receiver_type: otlp
  max_batch_size: 1000
  observation_interval: 120  # Observe for 120 seconds instead of 60
```

## Expected Runtime

### Continuous CI
- **4 tests** × ~40 seconds each = **~2-3 minutes**
- Fast feedback for protocol correctness

### Nightly
- **12 tests** × ~90 seconds each = **~10-12 minutes**
- Comprehensive batch size performance characterization

## Data Mapping to PR Document

These test suites provide data for the "Standard Load Performance" sections in the PR:

### From Nightly Tests:
- **Standard Load - OTAP → OTAP (Native Protocol)** - Use OTAP-ATTR-OTAP-BATCH* results
- **Standard Load - OTLP → OTLP (Standard Protocol)** - Use OTLP-ATTR-OTLP-BATCH* results

For each batch size (1000, 5000, 10000), you'll have:
- CPU Usage (%)
- Memory Usage (MB)
- Network metrics
- Throughput statistics

### From Continuous Tests:
- Quick validation that changes don't break protocol-level functionality
- Baseline metrics at 1000 batch size for trend tracking
