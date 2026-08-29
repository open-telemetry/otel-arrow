// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle implementation of the database adapter contract.

use crate::receivers::database::{
    CellValue, ColumnMetadata, CompiledQuery, DatabaseSystem, DriverAdapter, QueryResult, Row,
};
use async_trait::async_trait;
use oracle::sql_type::{IntervalDS, IntervalYM, OracleType, Timestamp};
use oracle::{Connection, Row as OracleRow};
use std::sync::{Mutex, OnceLock};

static ORACLE_CLIENT_DIRECTORY: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub(crate) struct OracleAdapterConfig {
    pub(crate) connect_string: String,
    pub(crate) instant_client_dir: String,
    pub(crate) username_file: String,
    pub(crate) password_file: String,
}

/// Oracle adapter that reuses one connection across non-overlapping polls.
pub struct OracleAdapter {
    config: OracleAdapterConfig,
    connection: Option<Connection>,
}

impl OracleAdapter {
    pub(crate) fn new(config: OracleAdapterConfig) -> Self {
        Self {
            config,
            connection: None,
        }
    }
}

#[async_trait(?Send)]
impl DriverAdapter for OracleAdapter {
    type Error = OracleAdapterError;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::Oracle
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        let connection = self.connection.take();
        let config = self.config.clone();
        let query = query.clone();
        let result =
            tokio::task::spawn_blocking(move || validate_blocking(connection, &config, &query))
                .await
                .map_err(OracleAdapterError::Worker)?;
        match result {
            Ok((connection, columns)) => {
                self.connection = Some(connection);
                Ok(columns)
            }
            Err(error) => Err(error),
        }
    }

    async fn execute(&mut self, query: &CompiledQuery) -> Result<QueryResult, Self::Error> {
        let connection = self.connection.take();
        let config = self.config.clone();
        let query = query.clone();
        let result =
            tokio::task::spawn_blocking(move || execute_blocking(connection, &config, &query))
                .await
                .map_err(OracleAdapterError::Worker)?;
        match result {
            Ok((connection, rows)) => {
                self.connection = Some(connection);
                Ok(rows)
            }
            Err(error) => Err(error),
        }
    }

    fn is_batch_error(&self, error: &Self::Error) -> bool {
        matches!(
            error,
            OracleAdapterError::Fetch(_)
                | OracleAdapterError::Convert(_)
                | OracleAdapterError::NonFiniteFloat
                | OracleAdapterError::UnsupportedType(_)
                | OracleAdapterError::RowLimit(_)
                | OracleAdapterError::ByteLimit(_)
        )
    }
}

impl Clone for OracleAdapterConfig {
    fn clone(&self) -> Self {
        Self {
            connect_string: self.connect_string.clone(),
            instant_client_dir: self.instant_client_dir.clone(),
            username_file: self.username_file.clone(),
            password_file: self.password_file.clone(),
        }
    }
}

fn validate_blocking(
    connection: Option<Connection>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
) -> Result<(Connection, Vec<ColumnMetadata>), OracleAdapterError> {
    let connection = match connection {
        Some(connection) => connection,
        None => connect(config, query.timeout())?,
    };
    connection
        .set_call_timeout(Some(query.timeout()))
        .map_err(OracleAdapterError::Configure)?;
    begin_read_only(&connection)?;
    let mut statement = connection
        .statement(query.sql())
        .fetch_array_size(1)
        .prefetch_rows(0)
        .build()
        .map_err(OracleAdapterError::Prepare)?;
    let result_set = statement.query(&[]).map_err(OracleAdapterError::Query)?;
    let columns = result_set
        .column_info()
        .iter()
        .map(column_metadata)
        .collect::<Vec<_>>();
    let types = result_set
        .column_info()
        .iter()
        .map(|column| column.oracle_type().clone())
        .collect::<Vec<_>>();
    validate_types(&types)?;
    drop(result_set);
    drop(statement);
    connection
        .rollback()
        .map_err(OracleAdapterError::Configure)?;
    Ok((connection, columns))
}

fn execute_blocking(
    connection: Option<Connection>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
) -> Result<(Connection, QueryResult), OracleAdapterError> {
    let connection = match connection {
        Some(connection) => connection,
        None => connect(config, query.timeout())?,
    };
    connection
        .set_call_timeout(Some(query.timeout()))
        .map_err(OracleAdapterError::Configure)?;
    begin_read_only(&connection)?;
    let mut statement = connection
        .statement(query.sql())
        // OCI fetch buffers are allocated outside normalized-byte accounting.
        // Fetch one native row at a time until row-width admission is introduced.
        .fetch_array_size(1)
        .prefetch_rows(0)
        .build()
        .map_err(OracleAdapterError::Prepare)?;
    let mut result_set = statement.query(&[]).map_err(OracleAdapterError::Query)?;
    let columns = result_set
        .column_info()
        .iter()
        .map(column_metadata)
        .collect::<Vec<_>>();
    let types = result_set
        .column_info()
        .iter()
        .map(|column| column.oracle_type().clone())
        .collect::<Vec<_>>();
    validate_types(&types)?;

    let mut rows = Vec::new();
    let mut normalized_bytes = 0_u64;
    for (index, row) in result_set
        .by_ref()
        .take(query.max_rows().saturating_add(1))
        .enumerate()
    {
        if index == query.max_rows() {
            return Err(OracleAdapterError::RowLimit(query.max_rows()));
        }
        let row = normalize_row(&row.map_err(OracleAdapterError::Fetch)?, &types)?;
        normalized_bytes = normalized_bytes.saturating_add(row.normalized_size());
        if normalized_bytes > query.max_normalized_bytes() {
            return Err(OracleAdapterError::ByteLimit(query.max_normalized_bytes()));
        }
        rows.push(row);
    }
    drop(result_set);
    drop(statement);
    connection
        .rollback()
        .map_err(OracleAdapterError::Configure)?;

    Ok((
        connection,
        QueryResult {
            columns,
            rows,
            normalized_bytes,
        },
    ))
}

fn column_metadata(column: &oracle::ColumnInfo) -> ColumnMetadata {
    ColumnMetadata {
        name: column.name().to_owned(),
        source_type: column.oracle_type().to_string(),
        nullable: column.nullable(),
    }
}

fn connect(
    config: &OracleAdapterConfig,
    timeout: std::time::Duration,
) -> Result<Connection, OracleAdapterError> {
    initialize_client(&config.instant_client_dir)?;
    let username = read_credential(&config.username_file, "username")?;
    let password = read_credential(&config.password_file, "password")?;
    let connect_string = bounded_connect_string(&config.connect_string, timeout)?;
    let connection = Connection::connect(username, password, connect_string)
        .map_err(OracleAdapterError::Connect)?;
    connection
        .set_call_timeout(Some(timeout))
        .map_err(OracleAdapterError::Configure)?;
    connection.ping().map_err(OracleAdapterError::Connect)?;
    _ = connection
        .execute("ALTER SESSION SET TIME_ZONE = 'UTC'", &[])
        .map_err(OracleAdapterError::Configure)?;
    Ok(connection)
}

fn begin_read_only(connection: &Connection) -> Result<(), OracleAdapterError> {
    _ = connection
        .execute("SET TRANSACTION READ ONLY", &[])
        .map_err(OracleAdapterError::Configure)?;
    Ok(())
}

fn bounded_connect_string(
    connect_string: &str,
    timeout: std::time::Duration,
) -> Result<String, OracleAdapterError> {
    let normalized = connect_string.to_ascii_lowercase();
    if connect_string.trim_start().starts_with('(') {
        return Err(OracleAdapterError::ConnectDescriptorUnsupported);
    }
    if normalized.contains("connect_timeout=") || normalized.contains("transport_connect_timeout=")
    {
        return Err(OracleAdapterError::ConnectTimeoutOverride);
    }
    let separator = if connect_string.contains('?') {
        '&'
    } else {
        '?'
    };
    let seconds = timeout.as_secs().max(1);
    Ok(format!(
        "{connect_string}{separator}connect_timeout={seconds}&transport_connect_timeout={seconds}"
    ))
}

fn initialize_client(directory: &str) -> Result<(), OracleAdapterError> {
    let selected = ORACLE_CLIENT_DIRECTORY.get_or_init(|| Mutex::new(None));
    let mut selected = selected
        .lock()
        .map_err(|_| OracleAdapterError::ClientInitializationLock)?;
    if let Some(existing) = selected.as_deref() {
        return if existing == directory {
            Ok(())
        } else {
            Err(OracleAdapterError::ClientDirectoryConflict)
        };
    }
    if oracle::InitParams::is_initialized() {
        return Err(OracleAdapterError::ClientAlreadyInitialized);
    }
    let mut params = oracle::InitParams::new();
    _ = params
        .oracle_client_lib_dir(directory)
        .and_then(|params| params.init())
        .map_err(OracleAdapterError::Initialize)?;
    *selected = Some(directory.to_owned());
    Ok(())
}

fn read_credential(path: &str, kind: &'static str) -> Result<String, OracleAdapterError> {
    let mut value = std::fs::read_to_string(path)
        .map_err(|source| OracleAdapterError::Credential { kind, source })?;
    while value.ends_with(['\r', '\n']) {
        _ = value.pop();
    }
    if value.is_empty() {
        return Err(OracleAdapterError::EmptyCredential(kind));
    }
    Ok(value)
}

fn validate_types(types: &[OracleType]) -> Result<(), OracleAdapterError> {
    for source_type in types {
        match source_type {
            OracleType::Varchar2(_)
            | OracleType::NVarchar2(_)
            | OracleType::Char(_)
            | OracleType::NChar(_)
            | OracleType::Rowid
            | OracleType::Raw(_)
            | OracleType::BinaryFloat
            | OracleType::BinaryDouble
            | OracleType::Number(_, _)
            | OracleType::Float(_)
            | OracleType::Date
            | OracleType::Timestamp(_)
            | OracleType::TimestampTZ(_)
            | OracleType::TimestampLTZ(_)
            | OracleType::IntervalDS(_, _)
            | OracleType::IntervalYM(_)
            | OracleType::Int64
            | OracleType::UInt64
            | OracleType::Boolean => {}
            unsupported => {
                return Err(OracleAdapterError::UnsupportedType(unsupported.to_string()));
            }
        }
    }
    Ok(())
}

fn normalize_row(row: &OracleRow, types: &[OracleType]) -> Result<Row, OracleAdapterError> {
    let values = types
        .iter()
        .enumerate()
        .map(|(index, source_type)| normalize_cell(row, index, source_type))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Row { values })
}

fn normalize_cell(
    row: &OracleRow,
    index: usize,
    source_type: &OracleType,
) -> Result<CellValue, OracleAdapterError> {
    macro_rules! optional {
        ($rust_type:ty, $variant:expr) => {
            row.get::<_, Option<$rust_type>>(index)
                .map(|value| value.map_or(CellValue::Null, $variant))
                .map_err(OracleAdapterError::Convert)
        };
    }

    match source_type {
        OracleType::Varchar2(_)
        | OracleType::NVarchar2(_)
        | OracleType::Char(_)
        | OracleType::NChar(_)
        | OracleType::Rowid => optional!(String, CellValue::String),
        OracleType::Raw(_) => optional!(Vec<u8>, CellValue::Bytes),
        OracleType::BinaryFloat => {
            optional!(f32, |value| CellValue::Float64(f64::from(value))).and_then(finite_float)
        }
        OracleType::BinaryDouble => optional!(f64, CellValue::Float64).and_then(finite_float),
        OracleType::Number(_, _) | OracleType::Float(_) => {
            optional!(String, CellValue::Decimal)
        }
        OracleType::Date | OracleType::Timestamp(_) => {
            optional!(Timestamp, |value: Timestamp| CellValue::Timestamp(
                format_oracle_timestamp(&value, false)
            ))
        }
        OracleType::TimestampTZ(_) | OracleType::TimestampLTZ(_) => {
            optional!(Timestamp, |value: Timestamp| CellValue::TimestampTz(
                format_oracle_timestamp(&value, true)
            ))
        }
        OracleType::IntervalDS(_, _) => {
            optional!(IntervalDS, |value: IntervalDS| CellValue::Interval(
                value.to_string()
            ))
        }
        OracleType::IntervalYM(_) => {
            optional!(IntervalYM, |value: IntervalYM| CellValue::Interval(
                value.to_string()
            ))
        }
        OracleType::Int64 => optional!(i64, CellValue::Int64),
        OracleType::UInt64 => optional!(u64, CellValue::UInt64),
        OracleType::Boolean => optional!(bool, CellValue::Bool),
        unsupported => Err(OracleAdapterError::UnsupportedType(unsupported.to_string())),
    }
}

fn format_oracle_timestamp(value: &Timestamp, with_timezone: bool) -> String {
    let base = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}",
        value.year(),
        value.month(),
        value.day(),
        value.hour(),
        value.minute(),
        value.second(),
        value.nanosecond()
    );
    if !with_timezone {
        return base;
    }
    let sign = if value.tz_offset() < 0 { '-' } else { '+' };
    format!(
        "{base}{sign}{:02}:{:02}",
        value.tz_hour_offset().unsigned_abs(),
        value.tz_minute_offset().unsigned_abs()
    )
}

fn finite_float(value: CellValue) -> Result<CellValue, OracleAdapterError> {
    match value {
        CellValue::Float64(value) if !value.is_finite() => Err(OracleAdapterError::NonFiniteFloat),
        value => Ok(value),
    }
}

/// Oracle connection, query, or conversion failure.
#[derive(Debug, thiserror::Error)]
pub enum OracleAdapterError {
    /// A mounted credential file could not be read.
    #[error("failed to read Oracle {kind} file")]
    Credential {
        /// Credential kind without its configured path.
        kind: &'static str,
        /// Underlying file error.
        #[source]
        source: std::io::Error,
    },
    /// A mounted credential file was empty.
    #[error("Oracle {0} file must not be empty")]
    EmptyCredential(&'static str),
    /// Oracle client initialization failed.
    #[error("Oracle client initialization failed")]
    Initialize(#[source] oracle::Error),
    /// Oracle was already initialized outside this adapter.
    #[error("Oracle client was initialized before instant_client_dir was applied")]
    ClientAlreadyInitialized,
    /// Another adapter selected a different process-global client directory.
    #[error("instant_client_dir conflicts with the initialized Oracle client")]
    ClientDirectoryConflict,
    /// Oracle client initialization state was poisoned.
    #[error("Oracle client initialization lock was poisoned")]
    ClientInitializationLock,
    /// Connection establishment or validation failed.
    #[error("Oracle connection failed")]
    Connect(#[source] oracle::Error),
    /// The first slice cannot safely inject bounds into a connect descriptor.
    #[error("Oracle connect descriptors are not supported; use an Easy Connect string")]
    ConnectDescriptorUnsupported,
    /// Connection timeout properties are owned by the receiver's query timeout.
    #[error("Oracle connect string must not override receiver connection timeouts")]
    ConnectTimeoutOverride,
    /// Session or timeout setup failed.
    #[error("Oracle session configuration failed")]
    Configure(#[source] oracle::Error),
    /// Statement preparation failed.
    #[error("Oracle query preparation failed")]
    Prepare(#[source] oracle::Error),
    /// Query execution failed.
    #[error("Oracle query execution failed")]
    Query(#[source] oracle::Error),
    /// Row fetching failed.
    #[error("Oracle row fetch failed")]
    Fetch(#[source] oracle::Error),
    /// Native value conversion failed.
    #[error("Oracle value conversion failed")]
    Convert(#[source] oracle::Error),
    /// A floating-point result cannot be represented faithfully.
    #[error("Oracle returned a non-finite floating-point value")]
    NonFiniteFloat,
    /// The result type does not have bounded conversion support.
    #[error("Oracle result type '{0}' is not supported")]
    UnsupportedType(String),
    /// The query returned more than the configured row ceiling.
    #[error("Oracle query exceeded the configured {0}-row poll limit")]
    RowLimit(usize),
    /// Normalized data exceeded the fixed first-slice byte ceiling.
    #[error("Oracle query exceeded the configured {0}-byte poll limit")]
    ByteLimit(u64),
    /// Blocking Oracle execution could not be joined.
    #[error("Oracle worker failed")]
    Worker(#[source] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use super::{CellValue, OracleAdapterError, bounded_connect_string, finite_float};
    use std::time::Duration;

    /// Scenario: Oracle returns a non-finite binary floating-point value.
    /// Guarantees: Conversion fails the poll instead of emitting a lossy or invalid value.
    #[test]
    fn rejects_non_finite_float() {
        let error = finite_float(CellValue::Float64(f64::NAN))
            .expect_err("non-finite value should fail conversion");
        assert!(matches!(error, OracleAdapterError::NonFiniteFloat));
    }

    /// Scenario: An Easy Connect string omits network timeout properties.
    /// Guarantees: The adapter derives bounded connect and transport timeouts from query timeout.
    #[test]
    fn adds_bounded_network_timeouts() {
        let connect_string =
            bounded_connect_string("database.contoso.com:1521/ORCL", Duration::from_secs(120))
                .expect("Easy Connect string should be supported");

        assert_eq!(
            connect_string,
            "database.contoso.com:1521/ORCL?connect_timeout=120&transport_connect_timeout=120"
        );
    }
}
