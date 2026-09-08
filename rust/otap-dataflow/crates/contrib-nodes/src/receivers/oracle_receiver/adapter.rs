// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle implementation of the database adapter contract.

use crate::receivers::database::{
    CellValue, ColumnMetadata, CompiledQuery, CompositeCursor, CursorRow, DatabaseSystem,
    DriverAdapter, DriverCancellation, QueryPage, Row,
};
use async_trait::async_trait;
use oracle::sql_type::{IntervalDS, IntervalYM, OracleType, Timestamp};
use oracle::{Connection, Row as OracleRow};
use otel_arrow_dfe_engine::error::ReceiverErrorKind;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};

// Oracle client initialization is process-global. The mutex only serializes
// the one-time directory choice when multiple pipeline instances start.
static ORACLE_CLIENT_DIRECTORY: OnceLock<Mutex<Option<String>>> = OnceLock::new();
const MAX_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Clone)]
pub(crate) struct OracleAdapterConfig {
    pub(crate) connect_string: String,
    pub(crate) instant_client_dir: String,
    pub(crate) username_file: String,
    pub(crate) password_file: String,
}

/// Oracle adapter that reuses one connection across non-overlapping polls.
pub struct OracleAdapter {
    config: OracleAdapterConfig,
    connection: Option<Arc<Connection>>,
    cancellation: OracleCancellation,
}

impl OracleAdapter {
    pub(crate) fn new(config: OracleAdapterConfig) -> Self {
        Self {
            config,
            connection: None,
            cancellation: OracleCancellation::default(),
        }
    }

    async fn run_blocking<T>(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
        operation: fn(
            Option<Arc<Connection>>,
            &OracleAdapterConfig,
            &CompiledQuery,
            &CompositeCursor,
            &OracleCancellation,
        ) -> Result<(Arc<Connection>, T), OracleAdapterError>,
    ) -> Result<T, OracleAdapterError>
    where
        T: Send + 'static,
    {
        // Ownership moves into the blocking worker and returns only after the
        // operation, preventing concurrent use of one Oracle connection.
        let connection = self.connection.take();
        let config = self.config.clone();
        let query = query.clone();
        let cursor = cursor.clone();
        let cancellation = self.cancellation.clone();
        let (connection, value) = tokio::task::spawn_blocking(move || {
            operation(connection, &config, &query, &cursor, &cancellation)
        })
        .await
        .map_err(OracleAdapterError::Worker)??;
        self.connection = Some(connection);
        Ok(value)
    }
}

/// Cancellation shared only between one Oracle blocking worker and its local receiver.
#[derive(Clone, Default)]
pub struct OracleCancellation {
    state: Arc<Mutex<CancellationState>>,
}

#[derive(Default)]
struct CancellationState {
    requested: bool,
    connection: Option<Arc<Connection>>,
}

struct ActiveConnection {
    cancellation: OracleCancellation,
}

impl ActiveConnection {
    fn register(
        cancellation: &OracleCancellation,
        connection: Arc<Connection>,
    ) -> Result<Self, OracleAdapterError> {
        let mut state = cancellation
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        if state.requested {
            return Err(OracleAdapterError::Cancelled);
        }
        state.connection = Some(connection);
        Ok(Self {
            cancellation: cancellation.clone(),
        })
    }
}

impl Drop for ActiveConnection {
    fn drop(&mut self) {
        if let Ok(mut state) = self.cancellation.state.lock() {
            state.connection = None;
        }
    }
}

impl OracleCancellation {
    fn ensure_not_requested(&self) -> Result<(), OracleAdapterError> {
        let state = self
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        if state.requested {
            Err(OracleAdapterError::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[async_trait(?Send)]
impl DriverCancellation for OracleCancellation {
    type Error = OracleAdapterError;

    async fn cancel(&self) -> Result<(), Self::Error> {
        let connection = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| OracleAdapterError::CancellationState)?;
            state.requested = true;
            state.connection.clone()
        };
        let Some(connection) = connection else {
            return Ok(());
        };
        tokio::task::spawn_blocking(move || connection.break_execution())
            .await
            .map_err(OracleAdapterError::CancellationWorker)?
            .map_err(OracleAdapterError::Cancellation)
    }
}

#[async_trait(?Send)]
impl DriverAdapter for OracleAdapter {
    type Error = OracleAdapterError;
    type Cancellation = OracleCancellation;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::Oracle
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        let mut state = self
            .cancellation
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        state.requested = false;
        state.connection = None;
        drop(state);
        Ok(self.cancellation.clone())
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        // rust-oracle is synchronous. Moving native calls to the blocking pool
        // keeps the engine's local async control loop responsive.
        let initial = query.watermark().initial.clone();
        self.run_blocking(query, &initial, validate_blocking).await
    }

    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        self.run_blocking(query, cursor, execute_blocking).await
    }

    fn classify_error(error: &Self::Error) -> ReceiverErrorKind {
        match error {
            OracleAdapterError::Connect(_) => ReceiverErrorKind::Connect,
            OracleAdapterError::Credential { .. }
            | OracleAdapterError::CredentialNotRegularFile(_)
            | OracleAdapterError::InvalidCredentialEncoding(_)
            | OracleAdapterError::EmptyCredential(_)
            | OracleAdapterError::Initialize(_)
            | OracleAdapterError::ClientAlreadyInitialized
            | OracleAdapterError::ClientDirectoryConflict
            | OracleAdapterError::ClientInitializationLock
            | OracleAdapterError::ConnectDescriptorUnsupported
            | OracleAdapterError::ConnectTimeoutOverride
            | OracleAdapterError::ConnectRetryUnsupported
            | OracleAdapterError::MultipleAddressUnsupported
            | OracleAdapterError::MissingCursorColumn(_)
            | OracleAdapterError::UnsupportedCursorTimestamp { .. }
            | OracleAdapterError::UnsupportedCursorTieBreaker { .. }
            | OracleAdapterError::InvalidCursorTimestamp(_)
            | OracleAdapterError::NormalizedByteLimit { .. }
            | OracleAdapterError::UnsupportedType(_) => ReceiverErrorKind::Configuration,
            OracleAdapterError::Configure(_)
            | OracleAdapterError::Prepare(_)
            | OracleAdapterError::Query(_)
            | OracleAdapterError::Fetch(_)
            | OracleAdapterError::Convert(_)
            | OracleAdapterError::NullCursorValue(_) => ReceiverErrorKind::Transport,
            OracleAdapterError::CancellationState
            | OracleAdapterError::CancellationWorker(_)
            | OracleAdapterError::Cancellation(_)
            | OracleAdapterError::Cancelled => ReceiverErrorKind::Shutdown,
            OracleAdapterError::NonFiniteFloat | OracleAdapterError::Worker(_) => {
                ReceiverErrorKind::Other
            }
        }
    }
}

fn validate_blocking(
    connection: Option<Arc<Connection>>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &OracleCancellation,
) -> Result<(Arc<Connection>, Vec<ColumnMetadata>), OracleAdapterError> {
    // Executing the prepared SELECT is required because Oracle exposes result
    // metadata on the result set. No row is fetched during startup validation.
    let (connection, _active) = prepare_session(connection, config, query, cancellation)?;
    // One row is the only safe native bound before rust-oracle exposes current
    // result metadata.
    let mut statement = connection
        .statement(query.sql())
        .fetch_array_size(1)
        .prefetch_rows(0)
        .build()
        .map_err(OracleAdapterError::Prepare)?;
    let result_set = bind_cursor(&mut statement, query, cursor)?;
    let (columns, _) = result_metadata(result_set.column_info())?;
    _ = validate_cursor_columns(result_set.column_info(), query)?;
    drop(result_set);
    drop(statement);
    finish_session(&connection)?;
    Ok((connection, columns))
}

fn execute_blocking(
    connection: Option<Arc<Connection>>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &OracleCancellation,
) -> Result<(Arc<Connection>, QueryPage), OracleAdapterError> {
    let (connection, _active) = prepare_session(connection, config, query, cancellation)?;
    let mut statement = connection
        .statement(query.sql())
        // Rebuilding a statement can expose changed view or column widths.
        // A one-row native buffer is the only safe bound before rust-oracle
        // reveals current result metadata.
        .fetch_array_size(1)
        .prefetch_rows(0)
        .build()
        .map_err(OracleAdapterError::Prepare)?;
    let mut result_set = bind_cursor(&mut statement, query, cursor)?;
    let (columns, types) = result_metadata(result_set.column_info())?;
    let (timestamp_index, tie_breaker_index) =
        validate_cursor_columns(result_set.column_info(), query)?;

    let watermark = query.watermark();
    let mut rows = Vec::with_capacity(query.fetch_size().min(query.max_rows()));
    let mut normalized_bytes = 0_u64;
    for row in result_set.by_ref().take(query.max_rows()) {
        let row = row.map_err(OracleAdapterError::Fetch)?;
        let cursor = extract_cursor(&row, timestamp_index, tie_breaker_index, watermark)?;
        let normalized = normalize_row(&row, &types)?;
        let next_bytes = normalized_bytes.saturating_add(normalized.normalized_size());
        if next_bytes > query.max_normalized_bytes() {
            if rows.is_empty() {
                // Skipping the row would silently lose data, so fail instead.
                return Err(OracleAdapterError::NormalizedByteLimit {
                    normalized_bytes: normalized.normalized_size(),
                    limit: query.max_normalized_bytes(),
                });
            }
            // Return the fitting prefix; later valid rows arrive next poll.
            break;
        }
        normalized_bytes = next_bytes;
        rows.push(CursorRow {
            row: normalized,
            cursor,
        });
    }
    drop(result_set);
    drop(statement);
    finish_session(&connection)?;

    Ok((connection, QueryPage { columns, rows }))
}

/// Binds the committed cursor through Oracle named parameters.
///
/// The cursor is never interpolated into SQL text, and the timestamp bind uses
/// an explicit timezone-naive type so a session offset cannot shift the
/// boundary between polls.
fn bind_cursor<'a>(
    statement: &'a mut oracle::Statement,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
) -> Result<oracle::ResultSet<'a, OracleRow>, OracleAdapterError> {
    let watermark = query.watermark();
    let timestamp = Timestamp::from_str(&cursor.timestamp)
        .map_err(|error| OracleAdapterError::InvalidCursorTimestamp(error.to_string()))?;
    // Bind with timezone information even for DATE and timezone-naive
    // TIMESTAMP columns. The session is UTC, so a naive cursor binds as
    // +00:00, while a TIMESTAMP WITH TIME ZONE cursor retains its source
    // offset instead of silently shifting the checkpoint boundary.
    let timestamp_type = cursor_bind_type();
    let timestamp_bind = (&timestamp, &timestamp_type);
    let tie_breaker = cursor.tie_breaker;
    statement
        .query_named(&[
            (watermark.timestamp_bind.as_str(), &timestamp_bind),
            (watermark.tie_breaker_bind.as_str(), &tie_breaker),
        ])
        .map_err(OracleAdapterError::Query)
}

fn cursor_bind_type() -> OracleType {
    OracleType::TimestampTZ(9)
}

/// Validates that both cursor columns exist with deterministic supported types.
fn validate_cursor_columns(
    columns: &[oracle::ColumnInfo],
    query: &CompiledQuery,
) -> Result<(usize, usize), OracleAdapterError> {
    let described = columns
        .iter()
        .map(|column| (column.name().to_owned(), column.oracle_type().clone()))
        .collect::<Vec<_>>();
    validate_described_cursor_columns(&described, query.watermark())
}

/// Pure cursor-metadata validation over adapter-independent column descriptions.
fn validate_described_cursor_columns(
    columns: &[(String, OracleType)],
    watermark: &crate::receivers::database::CompositeWatermark,
) -> Result<(usize, usize), OracleAdapterError> {
    let timestamp_index = cursor_column_index(columns, &watermark.timestamp_column)?;
    let tie_breaker_index = cursor_column_index(columns, &watermark.tie_breaker_column)?;
    let timestamp_type = &columns[timestamp_index].1;
    if !matches!(
        timestamp_type,
        OracleType::Date
            | OracleType::Timestamp(_)
            | OracleType::TimestampTZ(_)
            | OracleType::TimestampLTZ(_)
    ) {
        return Err(OracleAdapterError::UnsupportedCursorTimestamp {
            column: watermark.timestamp_column.clone(),
            data_type: timestamp_type.to_string(),
        });
    }
    let tie_breaker_type = &columns[tie_breaker_index].1;
    if !matches!(
        tie_breaker_type,
        OracleType::Int64 | OracleType::UInt64 | OracleType::Number(_, 0)
    ) {
        return Err(OracleAdapterError::UnsupportedCursorTieBreaker {
            column: watermark.tie_breaker_column.clone(),
            data_type: tie_breaker_type.to_string(),
        });
    }
    Ok((timestamp_index, tie_breaker_index))
}

fn cursor_column_index(
    columns: &[(String, OracleType)],
    name: &str,
) -> Result<usize, OracleAdapterError> {
    columns
        .iter()
        .position(|(column, _)| column.eq_ignore_ascii_case(name))
        .ok_or_else(|| OracleAdapterError::MissingCursorColumn(name.to_owned()))
}

/// Extracts the composite cursor of one row, rejecting null components.
fn extract_cursor(
    row: &OracleRow,
    timestamp_index: usize,
    tie_breaker_index: usize,
    watermark: &crate::receivers::database::CompositeWatermark,
) -> Result<CompositeCursor, OracleAdapterError> {
    let timestamp = row
        .get::<_, Option<Timestamp>>(timestamp_index)
        .map_err(OracleAdapterError::Convert)?
        .ok_or_else(|| OracleAdapterError::NullCursorValue(watermark.timestamp_column.clone()))?;
    let tie_breaker = row
        .get::<_, Option<i64>>(tie_breaker_index)
        .map_err(OracleAdapterError::Convert)?
        .ok_or_else(|| OracleAdapterError::NullCursorValue(watermark.tie_breaker_column.clone()))?;
    // The text form round-trips through Timestamp::from_str on the next bind,
    // so the durable checkpoint keeps full source precision.
    Ok(CompositeCursor::new(timestamp.to_string(), tie_breaker))
}

fn prepare_session(
    connection: Option<Arc<Connection>>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    cancellation: &OracleCancellation,
) -> Result<(Arc<Connection>, ActiveConnection), OracleAdapterError> {
    let connection = match connection {
        Some(connection) => connection,
        None => {
            cancellation.ensure_not_requested()?;
            Arc::new(connect(config, query.timeout())?)
        }
    };
    let active = ActiveConnection::register(cancellation, Arc::clone(&connection))?;
    connection
        .set_call_timeout(Some(query.timeout()))
        .map_err(OracleAdapterError::Configure)?;
    begin_read_only(&connection)?;
    Ok((connection, active))
}

fn result_metadata(
    columns: &[oracle::ColumnInfo],
) -> Result<(Vec<ColumnMetadata>, Vec<OracleType>), OracleAdapterError> {
    let metadata = columns.iter().map(column_metadata).collect();
    let types = columns
        .iter()
        .map(|column| column.oracle_type().clone())
        .collect::<Vec<_>>();
    validate_types(&types)?;
    Ok((metadata, types))
}

fn finish_session(connection: &Connection) -> Result<(), OracleAdapterError> {
    connection.rollback().map_err(OracleAdapterError::Configure)
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
    // Mounted files are read for each new connection so secret rotation takes
    // effect after a reconnect without placing credentials in configuration.
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
    // Static SQL inspection is intentionally conservative but cannot classify
    // every Oracle function; the database enforces the final read-only boundary.
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
    if normalized.contains("retry_count=") || normalized.contains("retry_delay=") {
        return Err(OracleAdapterError::ConnectRetryUnsupported);
    }
    if connect_string
        .split('?')
        .next()
        .is_some_and(|address| address.contains(','))
    {
        return Err(OracleAdapterError::MultipleAddressUnsupported);
    }
    let separator = if connect_string.contains('?') {
        '&'
    } else {
        '?'
    };
    let seconds = timeout.min(MAX_CONNECT_TIMEOUT).as_secs().max(1);
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
    let path = Path::new(path);
    let metadata = std::fs::metadata(path)
        .map_err(|source| OracleAdapterError::Credential { kind, source })?;
    if !metadata.is_file() {
        return Err(OracleAdapterError::CredentialNotRegularFile(kind));
    }
    let mut file = std::fs::File::open(path)
        .map_err(|source| OracleAdapterError::Credential { kind, source })?;
    if !file
        .metadata()
        .map_err(|source| OracleAdapterError::Credential { kind, source })?
        .is_file()
    {
        return Err(OracleAdapterError::CredentialNotRegularFile(kind));
    }
    let mut bytes = Vec::new();
    _ = file
        .read_to_end(&mut bytes)
        .map_err(|source| OracleAdapterError::Credential { kind, source })?;
    let mut value = String::from_utf8(bytes)
        .map_err(|_| OracleAdapterError::InvalidCredentialEncoding(kind))?;
    while value.ends_with(['\r', '\n']) {
        _ = value.pop();
    }
    if value.is_empty() {
        return Err(OracleAdapterError::EmptyCredential(kind));
    }
    Ok(value)
}

fn validate_types(types: &[OracleType]) -> Result<(), OracleAdapterError> {
    // There is no catch-all string fallback. Every admitted vendor type has an
    // explicit, precision-preserving CellValue conversion below.
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
    // rust-oracle returns conversion failures, including invalid text decoding,
    // as explicit errors. The receiver's query error policy then scopes them.
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
    /// A mounted credential path is not a regular file.
    #[error("Oracle {0} path must reference a regular file")]
    CredentialNotRegularFile(&'static str),
    /// A mounted credential file is not UTF-8.
    #[error("Oracle {0} file must contain valid UTF-8")]
    InvalidCredentialEncoding(&'static str),
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
    /// Connection retry controls would defeat the bounded startup attempt.
    #[error("Oracle connect string must not configure retry_count or retry_delay")]
    ConnectRetryUnsupported,
    /// Multiple addresses would multiply the per-attempt startup timeout.
    #[error("Oracle connect string must contain exactly one database address")]
    MultipleAddressUnsupported,
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
    /// A configured cursor column is absent from live result metadata.
    #[error("watermark cursor column '{0}' is not present in the query result")]
    MissingCursorColumn(String),
    /// The timestamp cursor column is not an Oracle date or timestamp type.
    #[error(
        "watermark timestamp column '{column}' has unsupported type '{data_type}'; DATE and TIMESTAMP family types are required"
    )]
    UnsupportedCursorTimestamp {
        /// Configured cursor column.
        column: String,
        /// Live Oracle type name.
        data_type: String,
    },
    /// The tie-breaker cursor column is not an integral Oracle type.
    #[error(
        "watermark tie-breaker column '{column}' has unsupported type '{data_type}'; a scale-zero integral type is required"
    )]
    UnsupportedCursorTieBreaker {
        /// Configured cursor column.
        column: String,
        /// Live Oracle type name.
        data_type: String,
    },
    /// A row's cursor component was SQL NULL.
    #[error("watermark cursor column '{0}' returned NULL; composite cursors must be non-null")]
    NullCursorValue(String),
    /// The committed cursor timestamp cannot be bound to Oracle.
    #[error("committed watermark timestamp is not a valid Oracle timestamp: {0}")]
    InvalidCursorTimestamp(String),
    /// The first row alone exceeds the normalized in-memory ceiling.
    #[error(
        "the first database row normalizes to {normalized_bytes} bytes, exceeding the {limit}-byte query.max_normalized_bytes limit"
    )]
    NormalizedByteLimit {
        /// Normalized size of the single row.
        normalized_bytes: u64,
        /// Configured normalized-byte ceiling.
        limit: u64,
    },
    /// Blocking Oracle execution could not be joined.
    #[error("Oracle worker failed")]
    Worker(#[source] tokio::task::JoinError),
    /// Cancellation state could not be synchronized with the blocking worker.
    #[error("Oracle cancellation state is unavailable")]
    CancellationState,
    /// Native Oracle cancellation could not be joined.
    #[error("Oracle cancellation worker failed")]
    CancellationWorker(#[source] tokio::task::JoinError),
    /// Oracle rejected a request to interrupt the active call.
    #[error("Oracle cancellation failed")]
    Cancellation(#[source] oracle::Error),
    /// An operation was cancelled before it registered its connection.
    #[error("Oracle operation was cancelled")]
    Cancelled,
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod tests;
