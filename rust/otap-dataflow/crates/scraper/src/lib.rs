// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral scraper for receiver scraping.
//!
//! Database-neutral contracts and runtime behavior belong here. Vendor drivers,
//! receiver factory registration, executable startup, and deployment assets do
//! not. This layer defines validated configuration, values, cursors, pages, and
//! local async driver contracts, durable filesystem checkpoints, and exclusive
//! source leases. The polling controller integrates these with OTLP encoding
//! and downstream acknowledgements.

mod checkpoint;
mod controller;
pub mod database;
mod partition;
mod telemetry;

pub use checkpoint::{CheckpointError, CheckpointState, CheckpointStore, WriteOutcome};
pub use controller::DatabaseReceiver;
pub use partition::{LeaseError, SourceLease};
pub use telemetry::DatabaseReceiverMetrics;
