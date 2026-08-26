// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Definition of errors that could happen when exporting OTAP batches to Clickhouse
#[derive(thiserror::Error, Debug)]
pub enum ClickhouseExporterError {
    // Generic Arrow errors
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    // Pdata / Otap Record related errors
    #[error("Pdata error: {0}")]
    Child(#[from] otel_arrow_dfe_pdata::error::Error),

    // Clickhouse client related errors
    #[error("Table creation error: {error}")]
    TableCreationError { error: String },

    #[error("Clickhouse data insertion request error: {source}")]
    InsertRequestError {
        #[source]
        source: clickhouse::error::Error,
    },

    #[error("Clickhouse data insertion response error: {source}")]
    InsertResponseError {
        #[source]
        source: clickhouse::error::Error,
    },

    #[error("Clickhouse data insertion request error: ClickHouse writer lane {lane} is closed")]
    WriterLaneClosed { lane: usize },

    // Data processing errors
    #[error("Missing column: {name}")]
    MissingColumn { name: String },

    #[error("Coercion Error: {error}")]
    CoercionError { error: String },

    #[error("Unsupported Payload Type: {error}")]
    UnsupportedPayload { error: String },

    #[error("Invalid type for column: {name}, expected: {expected}, found: {found}")]
    InvalidColumnType {
        name: String,
        expected: String,
        found: String,
    },
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::ClickhouseExporterError;

    /// Scenario: the ClickHouse client reports a timeout while completing an insertion.
    /// Guarantees: the exporter error retains the concrete client error in its source chain.
    #[test]
    fn insertion_error_retains_clickhouse_source() {
        let error = ClickhouseExporterError::InsertResponseError {
            source: clickhouse::error::Error::TimedOut,
        };

        assert!(matches!(
            error.source().and_then(|source| source.downcast_ref()),
            Some(clickhouse::error::Error::TimedOut)
        ));
    }
}
