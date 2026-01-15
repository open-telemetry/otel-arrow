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
//! ```
//! use quiver::{QuiverEngine, QuiverConfig, DiskBudget, RetentionPolicy};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let data_dir = tempfile::tempdir()?; // Use real path in production!
//! # let path = data_dir.path();
//! // Use a durable filesystem path, not /tmp (which may be tmpfs)
//! // let path = PathBuf::from("/var/lib/quiver/data");
//! let cfg = QuiverConfig::default().with_data_dir(path);
//!
//! // Configure disk budget (10 GB cap with backpressure)
//! let budget = Arc::new(DiskBudget::new(10 * 1024 * 1024 * 1024, RetentionPolicy::Backpressure));
//! let engine = QuiverEngine::new(cfg, budget)?;
//!
//! // Or use the builder pattern (uses unlimited budget by default):
//! // let engine = QuiverEngine::builder(cfg).with_budget(budget).build()?;
//!
//! // Ingest bundles via engine.ingest(bundle)
//! // Register subscribers via engine.register_subscriber(id)
//! // Consume bundles via engine.next_bundle(id)
//! // Periodic maintenance via engine.maintain()
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `mmap` (default): Enable memory-mapped segment reads for zero-copy access
//! - `serde`: Enable serialization for configuration types
//! - `otap-dataflow-integrations`: Enable integration with otap-dataflow types

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

pub use budget::{BudgetConfigError, DiskBudget, PendingWrite};
pub use config::{
    DurabilityMode, QuiverConfig, RetentionConfig, RetentionPolicy, SegmentConfig, WalConfig,
};
pub use engine::{MaintenanceStats, QuiverEngine, QuiverEngineBuilder};
pub use error::{QuiverError, Result};
pub use segment::SegmentError;
pub use segment_store::{SegmentReadMode, SegmentStore};
pub use subscriber::{
    BundleHandle, BundleIndex, BundleRef, RegistryCallback, RegistryConfig, SegmentProvider,
    SubscriberError, SubscriberId, SubscriberRegistry,
};
pub use wal::WalError;
