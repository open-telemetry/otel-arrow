// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native SQL Server binding roles over the shared source-query contract.

use super::config::{SqlServerConfigError, error};
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, CompiledQuery, CompiledWatermark, DatabaseSystem, ScalarValue, SourceQuery,
};
use std::fmt;

/// Source of one native parameter, in positional driver binding order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SqlServerParameter {
    /// The scalar signed-integer cursor.
    ScalarInt64,
    /// The scalar or composite datetime2 cursor.
    Timestamp,
    /// The composite signed-integer tie-breaker.
    TieBreaker,
    /// The incremental row ceiling plus one successor cursor.
    PageLimit,
}

/// Shared source SQL plus native driver binding roles.
pub struct SqlServerStatement {
    source: SourceQuery,
    parameters: Vec<SqlServerParameter>,
}

impl fmt::Debug for SqlServerStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqlServerStatement")
            .field("sql", &"<redacted>")
            .field("parameters", &self.parameters)
            .finish()
    }
}

impl SqlServerStatement {
    /// Returns source SQL for native result discovery and compatibility fingerprints.
    #[must_use]
    pub fn canonical_sql(&self) -> &str {
        self.source.sql()
    }

    pub(super) fn fingerprint_sql(&self) -> &str {
        self.source.fingerprint_sql()
    }

    /// Builds native paging SQL using the database's actual result-column names.
    pub fn driver_sql(
        &self,
        query: &CompiledQuery,
        columns: &[ColumnMetadata],
    ) -> Result<String, SqlServerConfigError> {
        self.source
            .paged_sql(query, columns)
            .map_err(|e| error(e.to_string()))
    }

    /// Returns native parameter roles in binding order.
    #[must_use]
    pub fn parameters(&self) -> &[SqlServerParameter] {
        &self.parameters
    }

    pub(super) fn compile(query: &CompiledQuery) -> Result<Self, SqlServerConfigError> {
        let source = SourceQuery::compile(query, DatabaseSystem::SqlServer)
            .map_err(|e| error(e.to_string()))?;
        let mut parameters = match query.watermark() {
            CompiledWatermark::Snapshot => Vec::new(),
            CompiledWatermark::Scalar(cursor) => vec![match cursor.initial {
                ScalarValue::Int64(_) => SqlServerParameter::ScalarInt64,
                ScalarValue::Timestamp(_) => SqlServerParameter::Timestamp,
                _ => {
                    return Err(error(
                        "SQL Server scalar cursors require int64 or timestamp",
                    ));
                }
            }],
            CompiledWatermark::Composite(_) => vec![
                SqlServerParameter::Timestamp,
                SqlServerParameter::TieBreaker,
            ],
        };
        if !parameters.is_empty() {
            parameters.push(SqlServerParameter::PageLimit);
        }
        Ok(Self { source, parameters })
    }
}

pub(super) fn validate_column_name(value: &str) -> Result<(), SqlServerConfigError> {
    if value.trim().is_empty()
        || value.encode_utf16().count() > 128
        || value.chars().any(char::is_control)
    {
        return Err(error(
            "SQL Server result names must contain 1..128 UTF-16 units without control characters",
        ));
    }
    Ok(())
}
