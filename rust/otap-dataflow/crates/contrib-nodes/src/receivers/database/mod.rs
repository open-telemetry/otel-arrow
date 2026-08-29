// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared, database-neutral contracts used by SQL receiver implementations.

mod config;
mod driver;
mod otlp;
mod query;
mod receiver;
mod row;
mod scheduler;

pub use config::{ConfigError, ErrorPolicy, OutputConfig, PollingConfig};
pub use driver::{DatabaseSystem, DriverAdapter, QueryResult};
pub use otlp::{OtlpMappingError, rows_to_pdata, validate_mapping};
pub use query::{CompiledQuery, QueryError};
pub use receiver::DatabaseReceiver;
pub use row::{CellValue, ColumnMetadata, Row};
