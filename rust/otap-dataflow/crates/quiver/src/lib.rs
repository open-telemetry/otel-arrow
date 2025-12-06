// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Quiver is an Arrow-native persistence layer.
//!
//! The current implementation only provides configuration scaffolding and
//! placeholder APIs so downstream crates can start integrating without pulling
//! in incomplete storage logic. Future phases will replace the placeholder
//! implementations with real WAL + segment handling.
//!
//! # Example
//! ```
//! use quiver::{engine::QuiverEngine, config::QuiverConfig};
//! use tempfile::tempdir;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let tmp = tempdir()?;
//! let cfg = QuiverConfig::default().with_data_dir(tmp.path());
//! let engine = QuiverEngine::new(cfg)?;
//! assert_eq!(engine.config().data_dir, tmp.path());
//! assert_eq!(engine.config().segment.target_size_bytes.get(), 32 * 1024 * 1024);
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod engine;
pub mod error;
pub mod record_bundle;
pub mod telemetry;
pub(crate) mod wal;

pub use config::{QuiverConfig, RetentionConfig, SegmentConfig, WalConfig};
pub use engine::QuiverEngine;
pub use error::{QuiverError, Result};
pub use wal::WalError;
