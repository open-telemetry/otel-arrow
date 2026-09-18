// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Vendor-neutral scraper runtimes for OTAP receiver nodes.
//!
//! This crate owns engine-facing scrape control, durable checkpointing,
//! exclusive source ownership, and low-cardinality telemetry. Vendor-specific
//! receiver crates provide configuration and adapters, then construct the
//! shared controller.

mod checkpoint;
mod controller;
pub mod database;
mod partition;
mod telemetry;

pub use checkpoint::{CheckpointError, CheckpointState, CheckpointStore, WriteOutcome};
pub use controller::DatabaseReceiver;
pub use partition::{LeaseError, SourceLease};
pub use telemetry::DatabaseReceiverMetrics;
