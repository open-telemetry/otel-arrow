# Quiver (Experimental) - Arrow-Based Persistence for OTAP Dataflow

Quiver is a standalone, embeddable Arrow-based segment store packaged as a
reusable Rust crate. It provides durable buffering with crash recovery for
telemetry pipelines. See `ARCHITECTURE.md` for design details.

## Features

- **Write-Ahead Log (WAL)**: Crash recovery with configurable flush policies
- **Segment Storage**: Immutable Arrow IPC files with optional memory-mapped reads
- **Multi-Subscriber**: Independent consumers with at-least-once delivery
- **Progress Tracking**: Persistent progress files for subscriber state
- **Automatic Cleanup**: Delete segments after all subscribers complete

## Status

**Experimental** - This crate is under active development and the API may change.
Not yet suitable for production use.

## Quick Start

```bash
cd rust/otap-dataflow
cargo test -p otap-df-quiver      # unit tests + doc tests
cargo bench -p otap-df-quiver     # Criterion benchmarks
```

## Usage

```rust,ignore
use quiver::{QuiverEngine, QuiverConfig, DiskBudget, RetentionPolicy, SubscriberId};
use std::path::PathBuf;
use std::sync::Arc;

// Use a durable filesystem path, not /tmp (which may be tmpfs)
let data_dir = PathBuf::from("/var/lib/quiver/data");
let config = QuiverConfig::default().with_data_dir(&data_dir);

// Configure disk budget (10 GB cap with backpressure)
let budget = Arc::new(DiskBudget::new(10 * 1024 * 1024 * 1024, RetentionPolicy::Backpressure));
let engine = QuiverEngine::new(config, budget)?;

// Register a subscriber
let sub_id = SubscriberId::new("my-exporter")?;
engine.register_subscriber(sub_id.clone())?;
engine.activate_subscriber(&sub_id)?;

// Ingest data (bundles from upstream)
// engine.ingest(&bundle)?;

// Consume bundles
while let Some(handle) = engine.next_bundle(&sub_id)? {
    // Process the bundle...
    handle.ack();  // Acknowledge successful processing
}

// Periodic maintenance
engine.maintain()?;
```

## Cargo Features

- `mmap` (default): Memory-mapped segment reads for zero-copy access
- `serde`: Serialization support for configuration types
- `otap-dataflow-integrations`: Integration with otap-dataflow types
