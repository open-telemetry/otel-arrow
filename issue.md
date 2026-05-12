# Idle CPU Overhead: ~7.5 millicores per pipeline core

## Summary

At idle (no data flowing), each pipeline core consumes approximately **7.5 millicores** of CPU due to hardcoded periodic wakeups. At scale this is significant:

| Cores | Idle CPU (millicores) | Equivalent |
|-------|----------------------|------------|
| 1 | ~7.5 | Barely noticeable |
| 16 | ~120 | Measurable (0.75% of host) |
| 128 | ~960 | Nearly 1 full core wasted |

## Root Cause: 4 Wakeups/sec Per Core

Each pipeline core runs a tokio `current_thread` runtime that wakes up **4 times per second** even with zero data flowing:

| Wakeup Source | Frequency | Location | Configurable? |
|--------------|-----------|----------|---------------|
| Control-plane metrics flush | 1/sec | `pipeline_ctrl.rs` L383 | No (hardcoded) |
| Node telemetry timer × N nodes | N/sec (1 per node) | Every node's `start()` calls `start_periodic_telemetry(1s)` | No (hardcoded) |

With a typical 3-node pipeline (receiver → processor → exporter): **4 wakeups/sec/core**.

Each wakeup involves: tokio timer fire → task poll → metric collection → channel send → aggregation → park. This explains ~7.5 millicores per core.

### Additionally (always-on, not per-core)

| Task | Frequency | Location |
|------|-----------|----------|
| `engine-metrics` (RSS, CPU sampling) | Every 5s | `controller/src/lib.rs` L1189 |
| `metrics-aggregator` | Event-driven (channel recv) | `controller/src/lib.rs` L1151 |
| `observed-state-store` | Event-driven (channel recv) | `controller/src/lib.rs` L1169 |

## Measured Data

### 1-Core Idle (3 runs each, 15s observation window)

Varying engine config had no meaningful CPU impact at 1 core (all within noise ~0.1-0.2%):

| Config | Avg CPU % | RAM MiB |
|--------|-----------|---------|
| Baseline (reporting_interval=1s) | 0.11 | 32.8 |
| Slow reporting (reporting_interval=30s) | 0.14 | 11.9 |
| Minimal (30s + logs=error) | 0.12 | 7.5 |
| Noop pipeline (30s + error + noop exporter) | 0.11 | 7.4 |

### 16-Core Idle (3 runs each)

Existing telemetry policy knobs show measurable impact at scale:

| Config | Avg CPU % | RAM MiB |
|--------|-----------|---------|
| ALL ON (default) | 0.75 | 16.0 |
| `tokio_metrics: false` | 0.69 | 15.9 |
| `runtime_metrics: none` | 0.79 | 14.8 |
| ALL OFF | 0.46 | 14.4 |

ALL OFF reduces CPU by ~40% vs ALL ON, but the irreducible baseline (~0.46%) comes from the hardcoded timer wakeups.

## Existing Config Knobs

The engine already has telemetry policy controls (in `policies.telemetry`):

```yaml
policies:
  telemetry:
    pipeline_metrics: false    # Disables pipeline.metrics set (12 series/core)
    tokio_metrics: false       # Disables tokio.runtime set (6 series/core)
    runtime_metrics: none      # Disables channel.* + pipeline.runtime_control sets
```

These reduce the number of exposed **metric series** (69 → fewer) and provide modest CPU savings at high core counts, but do NOT reduce the wakeup frequency since the timers still fire.

## Prometheus Metrics at Idle (1 core, 3 nodes)

- **46 unique metric names**, **69 time series**
- These scale as: `series ≈ 69 × cores × (nodes/3)`

| Category (`set` label) | Series | Scales with |
|------------------------|--------|-------------|
| `channel.receiver` | 20 | cores × channels |
| `pipeline.runtime_control` | 20 | cores |
| `channel.sender` | 9 | cores × channels |
| `pipeline.metrics` | 12 | cores |
| `tokio.runtime` | 6 | cores |
| `engine.metrics` | 2 | fixed |

## Proposed Optimizations

### High Impact (order of magnitude idle CPU reduction)

1. **Make per-node telemetry collection interval configurable** — Currently every node calls `start_periodic_telemetry(Duration::from_secs(1))` at startup. Extending to 5-10s (or making it respect `reporting_interval`) would eliminate 3 of 4 wakeups/sec/core.

2. **Make control-plane metrics flush interval configurable** — Currently hardcoded to 1s in `pipeline_ctrl.rs`. Should respect the existing `reporting_interval` config (already user-settable). When set to 10s, this eliminates 1 wakeup/sec/core.

3. **Combined**: Moving both from 1s → 10s would reduce wakeups from 4/sec/core to 0.4/sec/core — a **10× reduction** in idle CPU, bringing it under 1 millicore/core.

### Medium Impact

4. **Make `engine-metrics` interval configurable** — Currently hardcoded to 5s (`controller/src/lib.rs` L1189). There's even a `TODO` comment in the code for this.

### Lower Priority

5. **Consider disabling tokio's I/O driver** on cores that don't need it (`.enable_time()` instead of `.enable_all()`).

6. **Consider lazy telemetry timer registration** — Don't start per-node telemetry timers until the first message arrives (like durable_buffer and other processors already do).

## Files Referenced

- Per-node telemetry timer: `crates/core-nodes/src/receivers/otlp_receiver/mod.rs` (and every other node)
- Control-plane metrics flush: `crates/engine/src/pipeline_ctrl.rs` L383
- Engine metrics interval: `crates/controller/src/lib.rs` L1189
- Telemetry policy config: `crates/config/src/policy.rs` (TelemetryPolicy struct)
- Reporting interval config: `crates/config/src/pipeline/telemetry.rs` (default 1s)

## Bug Fix Found During Investigation

The idle state performance test template had stale Prometheus endpoint URLs (`/telemetry/metrics` instead of `/api/v1/telemetry/metrics`), causing 404 errors during monitoring. This was fixed in `idle-state-template.yaml.j2`.
