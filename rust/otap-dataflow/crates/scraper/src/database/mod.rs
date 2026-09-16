// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral contracts used by query-polling receiver adapters.
//!
//! Vendor adapters own native connectivity and normalize returned values into
//! [`CellValue`]. Shared OTLP encoding preserves these normalized values.
//! Only composite watermarks are currently accepted.

mod config;
mod driver;
mod otap;
mod page;
mod query;
mod row;

pub use config::{
    CheckpointConfig, ConfigError, OnNack, OutputConfig, PollingConfig, TieBreakerCursorConfig,
    TimestampCursorConfig, WatermarkConfig,
};
pub use driver::{DatabaseSystem, DriverAdapter, DriverCancellation};
pub(crate) use otap::parse_utc_timestamp;
pub use otap::{EncodedPage, OtlpMappingError, encode_page, validate_mapping};
pub use page::{CompositeCursor, CursorRow, QueryPage};
pub use query::{CompiledQuery, CompositeWatermark, QueryError};
pub use row::{CellValue, ColumnMetadata, Row};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mapping_tests;
