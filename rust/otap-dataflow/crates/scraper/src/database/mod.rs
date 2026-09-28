// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral contracts used by query-polling receiver adapters.
//!
//! Vendor adapters own native connectivity and normalize returned values into
//! [`CellValue`]. Polling, OTLP encoding, and durable persistence are introduced
//! separately. Only composite watermarks are currently accepted.

mod config;
mod driver;
mod page;
mod query;
mod row;

pub use config::{
    CheckpointConfig, ConfigError, OnNack, OutputConfig, PollingConfig, TieBreakerCursorConfig,
    TimestampCursorConfig, WatermarkConfig,
};
pub use driver::{DatabaseSystem, DriverAdapter, DriverCancellation};
pub use page::{CompositeCursor, CursorRow, QueryPage};
pub use query::{CompiledQuery, CompositeWatermark, QueryError};
pub use row::{CellValue, ColumnMetadata, Row};

#[cfg(test)]
mod tests;
