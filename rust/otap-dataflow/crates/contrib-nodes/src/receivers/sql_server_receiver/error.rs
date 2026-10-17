// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Safe error categories; never retain a driver message or source chain.

/// SQL Server operation failure with sensitive diagnostics discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
// The numeric server code is retained without allocating a diagnostic string.
#[allow(variant_size_differences)]
pub enum SqlServerError {
    /// The configured credential capability could not provide a usable credential.
    #[error("SQL Server credentials are unavailable or expired")]
    Credentials,
    /// Connection transport failed.
    #[error("SQL Server transport failed ({0:?})")]
    Transport(std::io::ErrorKind),
    /// Transport failed after submission and server-side cleanup could not be confirmed.
    #[error("SQL Server transport failed ({0:?}); response cleanup is unconfirmed")]
    UnconfirmedTransport(std::io::ErrorKind),
    /// The server reported a numeric error code.
    #[error("SQL Server rejected the operation (code {0})")]
    Server(u32),
    /// TLS verification or negotiation failed.
    #[error("SQL Server TLS verification or negotiation failed")]
    Tls,
    /// Contradictory SQL Server TLS options cannot be applied safely.
    #[error("SQL Server TLS options are contradictory")]
    TlsConfiguration,
    /// An operation reached its absolute deadline.
    #[error("SQL Server operation deadline expired")]
    Deadline,
    /// A metadata/fetch deadline expired and the server acknowledged its cancellation.
    #[error("SQL Server query deadline expired; cancellation was acknowledged")]
    QueryTimeout,
    /// The current operation was cancelled.
    #[error("SQL Server operation cancelled")]
    Cancelled,
    /// A result does not match the validated metadata.
    #[error("SQL Server result metadata is unsupported, ambiguous, or changed")]
    Metadata,
    /// The driver decoded an incompatible or invalid value.
    #[error("SQL Server value cannot be converted without loss")]
    Conversion,
    /// A source cursor cannot be bound losslessly.
    #[error("SQL Server cursor type, range, or precision is incompatible")]
    Cursor,
    /// No validated query/session is available.
    #[error("SQL Server operation requires validation of the configured query")]
    NotValidated,
    /// A normalized incremental row cannot fit the configured budget.
    #[error("SQL Server row exceeds the normalized page budget")]
    OversizeRow,
    /// A snapshot cannot fit as one complete result.
    #[error("SQL Server snapshot exceeds its row or byte budget")]
    SnapshotLimit,
    /// The driver response is invalid or outside the supported protocol subset.
    #[error("SQL Server driver returned an unsupported or invalid response")]
    Driver,
}

impl From<tiberius::error::Error> for SqlServerError {
    fn from(error: tiberius::error::Error) -> Self {
        match error {
            tiberius::error::Error::Io { kind, .. } => Self::Transport(kind),
            tiberius::error::Error::Server(error) => Self::Server(error.code()),
            tiberius::error::Error::Tls(_) => Self::Tls,
            tiberius::error::Error::Encoding(_)
            | tiberius::error::Error::Conversion(_)
            | tiberius::error::Error::Utf8
            | tiberius::error::Error::Utf16 => Self::Conversion,
            _ => Self::Driver,
        }
    }
}
