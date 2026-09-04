// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared, database-neutral contracts used by SQL receiver implementations.
//!
//! Vendor adapters own native connectivity and normalize every returned value
//! into [`CellValue`]. The shared runtime then owns scheduling, live mapping
//! validation, exact OTLP page batching, ACK-driven composite watermark
//! progression, and durable checkpoints. Keeping that boundary here prevents
//! Oracle details from leaking into future PostgreSQL or SQL Server receivers.
//!
//! Only `watermark.mode: composite` is implemented. Scalar and repeating
//! snapshot modes are deliberately unsupported and deferred to follow-up work.

mod checkpoint;
mod config;
mod driver;
mod metrics;
mod otlp;
mod page;
mod query;
mod receiver;
mod row;

pub use checkpoint::{
    CheckpointError, CheckpointState, CheckpointStore, LeaseError, SourceLease, WriteOutcome,
};
pub use config::{
    CheckpointConfig, ConfigError, OnNack, OutputConfig, PollingConfig, TieBreakerCursorConfig,
    TimestampCursorConfig, WatermarkConfig,
};
pub use driver::{DatabaseSystem, DriverAdapter, DriverCancellation};
pub use metrics::DatabaseReceiverMetrics;
pub use otlp::{EncodedPage, OtlpMappingError, encode_page, validate_mapping};
pub use page::{CompositeCursor, CursorRow, QueryPage};
pub use query::{CompiledQuery, CompositeWatermark, QueryError};
pub use receiver::DatabaseReceiver;
pub use row::{CellValue, ColumnMetadata, Row};

#[cfg(test)]
mod tests;
