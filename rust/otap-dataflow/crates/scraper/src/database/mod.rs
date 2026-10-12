// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral contracts used by query-polling receiver adapters.
//!
//! Vendor adapters own native connectivity and normalize returned values into
//! [`CellValue`]. Shared OTLP encoding preserves these normalized values.
//! Snapshot, scalar, and composite modes share the durable delivery runtime.

mod config;
mod driver;
mod otap;
mod page;
mod query;
mod row;
mod scalar;

pub use config::{
    CatchUpConfig, CheckpointConfig, ConfigError, OnNack, OnPermanentNack, OutputConfig,
    PollingConfig, TieBreakerCursorConfig, TimestampCursorConfig, WatermarkConfig,
};
pub use driver::{DatabaseSystem, DriverAdapter, DriverCancellation};
pub(crate) use otap::OtlpPageEncoder;
pub use otap::{EncodedPage, OtlpMappingError, encode_page, validate_mapping};
pub use page::{CompositeCursor, Cursor, CursorRow, QueryPage};
pub use query::{
    CompiledQuery, CompiledWatermark, CompositeWatermark, QueryError, ScalarWatermark,
};
pub use row::{CellValue, ColumnMetadata, Row};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mapping_tests;

pub use scalar::{CursorError, ScalarValue};

#[cfg(test)]
mod scalar_tests;

#[cfg(test)]
mod snapshot_tests;
