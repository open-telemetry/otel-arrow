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

```rust,no_run
use quiver::{QuiverEngine, QuiverConfig, DiskBudget, RetentionPolicy, SubscriberId, CancellationToken};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a durable filesystem path, not /tmp (which may be tmpfs)
    let data_dir = PathBuf::from("/var/lib/quiver/data");
    let config = QuiverConfig::default().with_data_dir(&data_dir);

    // Configure disk budget (10 GB cap with backpressure).
    // for_config() reads segment/WAL sizes from the config and validates
    // that hard_cap >= wal_max + 2 * segment_target.
    let budget = Arc::new(DiskBudget::for_config(
        10 * 1024 * 1024 * 1024,  // 10 GB hard cap
        &config,
        RetentionPolicy::Backpressure,
    )?);
    let engine = QuiverEngine::open(config, budget).await?;

    // Register a subscriber
    let sub_id = SubscriberId::new("my-exporter")?;
    engine.register_subscriber(sub_id.clone())?;
    engine.activate_subscriber(&sub_id)?;

    // Create a cancellation token for graceful shutdown
    let shutdown = CancellationToken::new();

    // Ingest data (bundles from upstream)
    // engine.ingest(&bundle).await?;

    // Consume bundles with timeout and cancellation support
    loop {
        match engine.next_bundle(&sub_id, Some(Duration::from_secs(5)), Some(&shutdown)).await {
            Ok(Some(handle)) => {
                // Process the bundle...
                handle.ack();  // Acknowledge successful processing
            }
            Ok(None) => continue,  // Timeout, check shutdown condition
            Err(e) if e.is_cancelled() => break,  // Graceful shutdown
            Err(e) => return Err(e.into()),
        }
    }

    // Periodic maintenance
    engine.maintain().await?;

    Ok(())
}
```

### Handling Backpressure

When the disk budget is exhausted, `ingest()` returns `QuiverError::StorageAtCapacity`.
The embedding layer should handle this by slowing ingestion:

```rust,no_run
use quiver::QuiverError;

match engine.ingest(&bundle).await {
    Ok(()) => { /* success */ }
    Err(e) if e.is_at_capacity() => {
        // Backpressure: wait for subscribers to catch up and segments to be cleaned
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        engine.maintain().await?;  // Try to clean up completed segments
        // Retry ingestion...
    }
    Err(e) => return Err(e),  // Other errors are fatal
}
```

## Cargo Features

- `mmap` (default): Memory-mapped segment reads for zero-copy access
- `serde`: Serialization support for configuration types
- `otap-dataflow-integrations`: Integration with otap-dataflow types
