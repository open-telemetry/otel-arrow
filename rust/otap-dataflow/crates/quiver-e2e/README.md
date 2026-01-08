# Quiver E2E Test Harness

End-to-end stress test harness for the Quiver persistence layer.

## Overview

This crate provides a comprehensive stress testing tool for validating Quiver's
persistence guarantees under sustained load. It exercises the complete data flow:
ingestion → WAL → segment finalization → subscriber consumption → cleanup.

## Features

- **Steady-State Testing**: Continuous ingest/consume loop with configurable throughput
- **Memory Leak Detection**: Tracks heap allocations via jemalloc to detect leaks
- **Disk Usage Monitoring**: Monitors segment and WAL growth over time
- **TUI Dashboard**: Real-time visualization with sparklines and metrics
- **Configurable Durability**: Test with full WAL protection or high-throughput segment-only mode

## Usage

```bash
# Basic steady-state test (10 seconds, TUI disabled)
cargo run -p otap-df-quiver-e2e --release -- --steady-state --duration 10s --no-tui

# With TUI dashboard
cargo run -p otap-df-quiver-e2e --release -- --steady-state --duration 60s

# High-throughput mode (no WAL)
cargo run -p otap-df-quiver-e2e --release -- --steady-state --duration 30s --no-wal --no-tui

# Custom configuration
cargo run -p otap-df-quiver-e2e --release -- \
    --steady-state \
    --duration 5m \
    --bundles 100 \
    --rows-per-bundle 1000 \
    --string-size 256 \
    --segment-size-mb 32
```

## Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--steady-state` | Run steady-state mode | false |
| `--duration` | Test duration (e.g., 10s, 5m, 1h) | 60s |
| `--bundles` | Bundles per iteration | 50 |
| `--rows-per-bundle` | Rows per bundle | 100 |
| `--string-size` | Size of string fields in bytes | 64 |
| `--segment-size-mb` | Target segment size | 16 |
| `--no-wal` | Disable WAL for higher throughput | false |
| `--no-tui` | Disable TUI dashboard | false |
| `--keep-temp` | Keep temp directory after test | false |

## Architecture

The harness spawns multiple threads:

- **Ingest Thread**: Generates and ingests synthetic bundles
- **Consumer Threads**: Subscribe and consume bundles from segments
- **Cleanup Thread**: Periodically removes fully-consumed segments
- **Stats Thread**: Collects memory and disk metrics
