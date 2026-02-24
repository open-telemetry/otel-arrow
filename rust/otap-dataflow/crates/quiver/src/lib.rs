// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Quiver is an Arrow-native persistence layer for durable buffering of
//! telemetry data.
//!
//! The crate provides:
//!
//! - **Durability**: Write-ahead log (WAL) for crash recovery
//! - **Segment Storage**: Immutable Arrow IPC-based Quiver segment files with zero-copy reads
//! - **Subscription**: Multi-subscriber consumption with progress tracking
//! - **Maintenance**: Automatic cleanup of completed segments
//!
//! # Async Runtime
//!
//! Quiver uses [Tokio](https://tokio.rs) for async I/O operations. The primary
//! async APIs are:
//!
//! - [`QuiverEngine::open`] / [`QuiverEngineBuilder::build`] - async engine initialization
//! - [`QuiverEngine::ingest`] - async bundle ingestion with WAL persistence
//! - [`QuiverEngine::next_bundle`] - async bundle consumption with timeout and cancellation
//! - [`QuiverEngine::flush`] / [`QuiverEngine::shutdown`] - async segment finalization
//!
//! Synchronous alternatives like [`QuiverEngine::poll_next_bundle`] are available
//! for non-blocking polling patterns.
//!
//! # Architecture
//!
//! The [`QuiverEngine`] is the primary entry point. It coordinates:
//!
//! 1. **Ingestion**: Bundles are appended to the WAL, then accumulated in memory
//! 2. **Finalization**: When thresholds are exceeded, segments are written to disk
//! 3. **Subscription**: Subscribers consume bundles with at-least-once delivery
//! 4. **Cleanup**: Completed segments are deleted after all subscribers finish
//!
//! # Example
//!
//! This example demonstrates the consumer-side API with graceful shutdown via
//! cancellation token. The token can be triggered from a signal handler or
//! another task to wake waiting consumers immediately.
//!
//! ```no_run
//! use quiver::{QuiverEngine, QuiverConfig, DiskBudget, RetentionPolicy, SubscriberId, CancellationToken};
//! use std::sync::Arc;
//! use std::time::Duration;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Use a durable filesystem path (not /tmp, which may be tmpfs on Linux)
//!     let data_dir = PathBuf::from("/var/lib/quiver/data");
//!     let cfg = QuiverConfig::default().with_data_dir(&data_dir);
//!
//!     // Configure disk budget (10 GB cap with backpressure).
//!     // for_config() reads segment/WAL sizes from the config and validates
//!     // that hard_cap >= wal_max + 2 * segment_target.
//!     let budget = Arc::new(DiskBudget::for_config(
//!         10 * 1024 * 1024 * 1024,  // 10 GB hard cap
//!         &cfg,
//!         RetentionPolicy::Backpressure,
//!     )?);
//!     let engine = QuiverEngine::open(cfg, budget).await?;
//!
//!     // Register and activate a subscriber
//!     let sub_id = SubscriberId::new("my-exporter")?;
//!     engine.register_subscriber(sub_id.clone())?;
//!     engine.activate_subscriber(&sub_id)?;
//!
//!     // Create a cancellation token for graceful shutdown.
//!     // In production, clone this token and trigger it from a signal handler:
//!     //   let shutdown_clone = shutdown.clone();
//!     //   tokio::spawn(async move {
//!     //       tokio::signal::ctrl_c().await.unwrap();
//!     //       shutdown_clone.cancel();
//!     //   });
//!     let shutdown = CancellationToken::new();
//!
//!     // Producer task would call: engine.ingest(&bundle).await?
//!
//!     // Consumer loop with timeout and cancellation support.
//!     // When shutdown.cancel() is called, next_bundle returns Err(Cancelled)
//!     // immediately, even if waiting for the timeout.
//!     let mut processed = 0u64;
//!     loop {
//!         match engine.next_bundle(&sub_id, Some(Duration::from_secs(5)), Some(&shutdown)).await {
//!             Ok(Some(handle)) => {
//!                 // Process the bundle payload...
//!                 processed += 1;
//!                 handle.ack();  // Acknowledge successful processing
//!             }
//!             Ok(None) => {
//!                 // Timeout - good time to run periodic maintenance.
//!                 // This cleans up segments that all subscribers have completed.
//!                 engine.maintain().await?;
//!                 eprintln!("Processed {} bundles so far", processed);
//!             }
//!             Err(e) if e.is_cancelled() => {
//!                 eprintln!("Shutdown requested, processed {} bundles", processed);
//!                 break;
//!             }
//!             Err(e) => return Err(e.into()),
//!         }
//!     }
//!
//!     // Graceful shutdown: finalize any pending segment and cleanup
//!     engine.shutdown().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - `mmap` (default): Enable memory-mapped segment reads for zero-copy access
//! - `serde`: Enable serialization for configuration types
//! - `otap-dataflow-integrations`: Enable integration with otap-dataflow types

// Declare logging module first so macros are available to subsequent modules
pub(crate) mod logging;

pub mod budget;
pub mod config;
pub mod engine;
pub mod error;
pub mod record_bundle;
pub mod segment;
pub mod segment_store;
pub mod subscriber;
pub mod telemetry;
pub(crate) mod wal;

pub use budget::{BudgetConfigError, DiskBudget};
pub use config::{
    DurabilityMode, QuiverConfig, RetentionConfig, RetentionPolicy, SegmentConfig, WalConfig,
};
pub use engine::{MaintenanceStats, QuiverEngine, QuiverEngineBuilder};
pub use error::{QuiverError, Result};

pub use segment::SegmentError;
pub use segment_store::{ScanResult, SegmentReadMode, SegmentStore};
pub use subscriber::{
    BundleHandle, BundleIndex, BundleRef, RegistryCallback, RegistryConfig, SegmentProvider,
    SubscriberError, SubscriberId, SubscriberRegistry,
};
// Re-export CancellationToken for convenient use by consumers.
// This is the standard tokio-util type used for cooperative cancellation.
pub use tokio_util::sync::CancellationToken;
pub use wal::WalError;
