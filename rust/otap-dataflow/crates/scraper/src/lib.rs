// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral scraper for receiver scraping.
//!
//! Database-neutral contracts and runtime behavior belong here. Vendor drivers,
//! receiver factory registration, executable startup, and deployment assets do
//! not. This layer defines validated configuration, values, cursors, pages, and
//! local async driver contracts, durable filesystem checkpoints, and exclusive
//! source leases. Polling and OTLP encoding are introduced separately.

mod checkpoint;
pub mod database;
mod partition;

pub use checkpoint::{CheckpointError, CheckpointState, CheckpointStore, WriteOutcome};
pub use partition::{LeaseError, SourceLease};
