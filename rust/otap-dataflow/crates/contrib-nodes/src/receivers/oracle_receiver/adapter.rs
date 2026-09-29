// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle implementation of the database adapter contract.

use super::worker::{NativeWorker, receive};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Timelike};
use oracle::sql_type::{IntervalDS, IntervalYM, OracleType, Timestamp};
use oracle::{Connection, Row as OracleRow};
use otel_arrow_dfe_engine::error::ReceiverErrorKind;
use otel_arrow_dfe_scraper::database::{
    CellValue, ColumnMetadata, CompiledQuery, CompositeCursor, CursorRow, DatabaseSystem,
    DriverAdapter, DriverCancellation, QueryPage, Row,
};
use secrecy::{ExposeSecret, SecretString, zeroize::Zeroizing};
use std::io::Read;
use std::mem::size_of;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock, Weak};

// Oracle client initialization is process-global. The mutex only serializes
// the one-time directory choice when multiple pipeline instances start.
static ORACLE_CLIENT_DIRECTORY: OnceLock<Mutex<Option<String>>> = OnceLock::new();
const MAX_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const MAX_CREDENTIAL_BYTES: u64 = 64 * 1024;
const MAX_TIMESTAMP_COMPONENT_DIGITS: usize = 9;

#[derive(Clone)]
pub(crate) struct OracleAdapterConfig {
    pub(crate) connect_string: String,
    pub(crate) instant_client_dir: String,
    pub(crate) username_file: String,
    pub(crate) password_file: String,
}

/// Oracle adapter that reuses one connection across non-overlapping polls.
///
/// Dedicated OS workers own query/session work and native cancellation. Neither
/// worker belongs to Tokio's blocking pool, so stuck native work cannot hold up
/// that runtime's destruction. Shutdown confirms both workers' resource cleanup;
/// the shared controller retains source ownership when its deadline expires.
pub struct OracleAdapter {
    config: OracleAdapterConfig,
    worker: Option<NativeWorker<Option<OracleSession>>>,
    cancellation: OracleCancellation,
    stopping_cancellation: Option<NativeWorker<()>>,
}

/// One exclusively owned Oracle connection and the query artifacts prepared on it.
struct OracleSession {
    connection: Arc<Connection>,
    prepared: Option<OraclePreparedQuery>,
}

/// Cached statement and immutable decode plan derived from its first result metadata.
struct OraclePreparedQuery {
    statement: oracle::Statement,
    columns: Vec<ColumnMetadata>,
    types: Vec<OracleType>,
    timestamp_index: usize,
    tie_breaker_index: usize,
    timestamp_bind_type: OracleType,
}

impl OracleAdapter {
    /// Creates an adapter whose connection is opened lazily on first use.
    pub(crate) fn new(config: OracleAdapterConfig) -> Self {
        Self {
            config,
            worker: None,
            cancellation: OracleCancellation::default(),
            stopping_cancellation: None,
        }
    }

    /// Runs one synchronous Oracle operation without blocking the local engine core.
    async fn run_blocking<T>(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
        operation: fn(
            Option<OracleSession>,
            &OracleAdapterConfig,
            &CompiledQuery,
            &CompositeCursor,
            &OracleCancellation,
        ) -> Result<(OracleSession, T), OracleAdapterError>,
    ) -> Result<T, OracleAdapterError>
    where
        T: Send + 'static,
    {
        self.cancellation.ensure_not_requested()?;
        if self.worker.is_none() {
            self.worker = Some(
                NativeWorker::new("oracle-query")
                    .map_err(|error| OracleAdapterError::Worker(error.into()))?,
            );
        }
        let worker = self.worker.as_ref().expect("worker initialized");
        // Clone inputs so queued work owns its data and can outlive this future.
        let config = self.config.clone();
        let query = query.clone();
        let cursor = cursor.clone();
        let cancellation = self.cancellation.clone();
        let result = worker
            .run(move |session| {
                let (next, value) =
                    operation(session.take(), &config, &query, &cursor, &cancellation)?;
                *session = Some(next);
                Ok(value)
            })
            .map_err(|error| OracleAdapterError::Worker(error.into()))?;
        let result = receive(result)
            .await
            .map_err(|error| OracleAdapterError::Worker(error.into()))?;
        self.cancellation.ensure_not_requested()?;
        result
    }
}

/// Cancellation shared only between one Oracle blocking worker and its local receiver.
#[derive(Clone, Default)]
pub struct OracleCancellation {
    state: Arc<Mutex<CancellationState>>,
    generation: u64,
}

#[derive(Default)]
struct CancellationState {
    requested: bool,
    stopped: bool,
    cancelling: bool,
    // Only native workers may acquire/drop strong connection references.
    connection: Option<Weak<Connection>>,
    worker: Option<NativeWorker<()>>,
    generation: u64,
}

struct ActiveConnection {
    cancellation: OracleCancellation,
}

impl ActiveConnection {
    /// Publishes the active connection so cancellation can interrupt only this operation.
    fn register(
        cancellation: &OracleCancellation,
        connection: &Arc<Connection>,
    ) -> Result<Self, OracleAdapterError> {
        let mut state = cancellation
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        if state.requested || state.stopped || state.generation != cancellation.generation {
            return Err(OracleAdapterError::Cancelled);
        }
        state.connection = Some(Arc::downgrade(connection));
        Ok(Self {
            cancellation: cancellation.clone(),
        })
    }
}

impl Drop for ActiveConnection {
    fn drop(&mut self) {
        // The guard scopes cancellation to the operation that registered this connection.
        if let Ok(mut state) = self.cancellation.state.lock()
            && state.generation == self.cancellation.generation
        {
            state.connection = None;
        }
    }
}

impl OracleCancellation {
    /// Checks cancellation before and after one synchronous operation on a native worker.
    fn native_call<T>(
        &self,
        operation: impl FnOnce() -> Result<T, OracleAdapterError>,
    ) -> Result<T, OracleAdapterError> {
        self.ensure_not_requested()?;
        let value = operation()?;
        self.ensure_not_requested()?;
        Ok(value)
    }

    /// Prevents a cancelled operation from opening a connection after cancellation won.
    fn ensure_not_requested(&self) -> Result<(), OracleAdapterError> {
        let state = self
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        if state.requested || state.stopped || state.generation != self.generation {
            Err(OracleAdapterError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn take_worker(&self) -> Result<Option<NativeWorker<()>>, OracleAdapterError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        state.requested = true;
        state.stopped = true;
        Ok(state.worker.take())
    }
}

#[async_trait(?Send)]
impl DriverCancellation for OracleCancellation {
    type Error = OracleAdapterError;

    /// Flags the whole operation and interrupts its native call on a dedicated worker.
    async fn cancel(&self) -> Result<(), Self::Error> {
        let result = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| OracleAdapterError::CancellationState)?;
            if state.stopped {
                return Err(OracleAdapterError::Cancelled);
            }
            if state.generation != self.generation {
                return Ok(());
            }
            state.requested = true;
            if state.cancelling {
                return Ok(());
            }
            let Some(connection) = state.connection.clone() else {
                return Ok(());
            };
            if state.worker.is_none() {
                state.worker = Some(
                    NativeWorker::new("oracle-cancel")
                        .map_err(|error| OracleAdapterError::CancellationWorker(error.into()))?,
                );
            }
            state.cancelling = true;
            let cancellation_state = Arc::clone(&self.state);
            let queued = state
                .worker
                .as_ref()
                .expect("cancellation worker initialized")
                .run(move |_| {
                    // Upgrade on this worker so even the last native reference
                    // is dropped off-core when the query completes concurrently.
                    let result = match connection.upgrade() {
                        Some(connection) => connection
                            .break_execution()
                            .map_err(|error| OracleAdapterError::Cancellation(error.into())),
                        None => Ok(()),
                    };
                    cancellation_state
                        .lock()
                        .map_err(|_| OracleAdapterError::CancellationState)?
                        .cancelling = false;
                    result
                });
            match queued {
                Ok(result) => result,
                Err(error) => {
                    state.cancelling = false;
                    return Err(OracleAdapterError::CancellationWorker(error.into()));
                }
            }
        };
        receive(result)
            .await
            .map_err(|error| OracleAdapterError::CancellationWorker(error.into()))?
    }
}

#[async_trait(?Send)]
impl DriverAdapter for OracleAdapter {
    type Error = OracleAdapterError;
    type Cancellation = OracleCancellation;

    /// Identifies rows from this adapter as Oracle data.
    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::Oracle
    }

    /// Resets operation-local cancellation state before starting native work.
    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        let mut state = self
            .cancellation
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        if state.stopped {
            return Err(OracleAdapterError::Cancelled);
        }
        if state.cancelling {
            return Err(OracleAdapterError::CancellationState);
        }
        state.generation = state
            .generation
            .checked_add(1)
            .ok_or(OracleAdapterError::CancellationState)?;
        self.cancellation.generation = state.generation;
        state.requested = false;
        state.connection = None;
        drop(state);
        Ok(self.cancellation.clone())
    }

    fn is_retryable(error: &Self::Error) -> bool {
        match error {
            OracleAdapterError::Connect(error)
            | OracleAdapterError::Configure(error)
            | OracleAdapterError::Prepare(error)
            | OracleAdapterError::Query(error)
            | OracleAdapterError::Fetch(error) => {
                transient_native_error(error.oci_code(), error.dpi_code())
            }
            _ => false,
        }
    }

    async fn reconnect(&mut self, query: &CompiledQuery) -> Result<(), Self::Error> {
        let initial = query.watermark().initial.clone();
        self.run_blocking(query, &initial, reconnect_blocking).await
    }

    /// Prepares the query and validates its live result metadata without fetching rows.
    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        // Native preparation and metadata inspection stay on the query worker.
        let initial = query.watermark().initial.clone();
        self.run_blocking(query, &initial, validate_blocking).await
    }

    /// Fetches and normalizes one bounded page after the supplied cursor.
    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        self.run_blocking(query, cursor, execute_blocking).await
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        if let Some(worker) = self.cancellation.take_worker()? {
            self.stopping_cancellation = Some(worker);
        }
        // Poll both stops so both channels close before either cleanup can wait.
        // The caller owns the shared deadline and quarantines unconfirmed work.
        let (query, cancellation) = tokio::join!(
            async {
                match self.worker.as_mut() {
                    Some(worker) => worker.stop().await,
                    None => Ok(()),
                }
            },
            async {
                match self.stopping_cancellation.as_mut() {
                    Some(worker) => worker.stop().await,
                    None => Ok(()),
                }
            },
        );
        query.map_err(|error| OracleAdapterError::Worker(error.into()))?;
        cancellation.map_err(|error| OracleAdapterError::CancellationWorker(error.into()))?;
        self.worker = None;
        self.stopping_cancellation = None;
        Ok(())
    }

    /// Maps adapter failures into stable receiver error categories.
    fn classify_error(error: &Self::Error) -> ReceiverErrorKind {
        match error {
            OracleAdapterError::Connect(_) => ReceiverErrorKind::Connect,
            OracleAdapterError::Credential { .. }
            | OracleAdapterError::CredentialNotRegularFile(_)
            | OracleAdapterError::CredentialTooLarge(_)
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
            | OracleAdapterError::MissingCursorColumn
            | OracleAdapterError::DuplicateColumns
            | OracleAdapterError::NullableCursorColumn
            | OracleAdapterError::UnsupportedCursorTimestamp
            | OracleAdapterError::UnsupportedCursorTieBreaker
            | OracleAdapterError::InvalidCursorTimestamp
            | OracleAdapterError::NormalizedByteLimit { .. }
            | OracleAdapterError::ResultMetadataChanged
            | OracleAdapterError::UnsupportedType => ReceiverErrorKind::Configuration,
            OracleAdapterError::Configure(_)
            | OracleAdapterError::Prepare(_)
            | OracleAdapterError::Query(_)
            | OracleAdapterError::Fetch(_)
            | OracleAdapterError::Convert(_)
            | OracleAdapterError::NullCursorValue
            | OracleAdapterError::InvalidCursorValue => ReceiverErrorKind::Transport,
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

/// Allow only explicit availability failures; authentication, SQL and cleanup errors stay terminal.
fn transient_native_error(oci: Option<i32>, dpi: Option<i32>) -> bool {
    matches!(
        oci,
        Some(
            28 | 1012
                | 1033
                | 1034
                | 1089
                | 1090
                | 3113
                | 3114
                | 3135
                | 12170
                | 12514
                | 12516
                | 12519
                | 12520
                | 12528
                | 12537
                | 12541
                | 12543
                | 12547
                | 12571
        )
    ) || matches!(dpi, Some(1010 | 1067 | 1080))
}

/// Replace the failed session on its owning worker before shared code revalidates the query.
fn reconnect_blocking(
    session: Option<OracleSession>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    _cursor: &CompositeCursor,
    cancellation: &OracleCancellation,
) -> Result<(OracleSession, ()), OracleAdapterError> {
    cancellation.ensure_not_requested()?;
    drop(session);
    let (session, _active) = prepare_session(None, config, query, cancellation)?;
    cancellation.native_call(|| finish_session(&session.connection))?;
    Ok((session, ()))
}

/// Performs startup preparation and returns the validated public column metadata.
fn validate_blocking(
    session: Option<OracleSession>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &OracleCancellation,
) -> Result<(OracleSession, Vec<ColumnMetadata>), OracleAdapterError> {
    // Executing the prepared SELECT is required because Oracle exposes result
    // metadata on the result set. No row is fetched during startup validation.
    let (mut session, _active) = prepare_session(session, config, query, cancellation)?;
    ensure_prepared(&mut session, query, cursor, cancellation)?;
    let columns = session
        .prepared
        .as_ref()
        .expect("query was prepared")
        .columns
        .clone();
    cancellation.native_call(|| finish_session(&session.connection))?;
    Ok((session, columns))
}

/// Executes one prepared page fetch and enforces row and normalized-byte bounds.
fn execute_blocking(
    session: Option<OracleSession>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &OracleCancellation,
) -> Result<(OracleSession, QueryPage), OracleAdapterError> {
    let (mut session, _active) = prepare_session(session, config, query, cancellation)?;
    ensure_prepared(&mut session, query, cursor, cancellation)?;
    let prepared = session.prepared.as_mut().expect("query was prepared");
    let mut result_set = bind_cursor(
        &mut prepared.statement,
        query,
        cursor,
        &prepared.timestamp_bind_type,
        cancellation,
    )?;
    if !metadata_matches(result_set.column_info(), &prepared.columns, &prepared.types) {
        return Err(OracleAdapterError::ResultMetadataChanged);
    }
    let columns = prepared.columns.clone();

    let mut rows = Vec::new();
    let mut payload_bytes = 0;
    for _ in 0..query.max_rows() {
        cancellation.ensure_not_requested()?;
        let Some(row) = result_set.next() else { break };
        let row = row.map_err(|error| OracleAdapterError::Fetch(error.into()))?;
        cancellation.ensure_not_requested()?;
        let normalized = normalize_row(&row, &prepared.types, cancellation)?;
        cancellation.ensure_not_requested()?;
        let cursor = extract_normalized_cursor(
            &normalized,
            prepared.timestamp_index,
            prepared.tie_breaker_index,
        )?;
        if !push_bounded_row(
            &mut rows,
            &mut payload_bytes,
            CursorRow {
                row: normalized,
                cursor,
            },
            query.max_rows(),
            query.max_normalized_bytes(),
        )? {
            break;
        }
    }
    drop(result_set);
    cancellation.native_call(|| finish_session(&session.connection))?;

    Ok((session, QueryPage { columns, rows }))
}

/// Grows page storage geometrically while charging spare slots to the byte budget.
fn push_bounded_row(
    rows: &mut Vec<CursorRow>,
    payload_bytes: &mut u64,
    row: CursorRow,
    max_rows: usize,
    limit: u64,
) -> Result<bool, OracleAdapterError> {
    let next_payload = payload_bytes
        .saturating_add(
            row.row
                .normalized_size()
                .saturating_sub(size_of::<Row>() as u64),
        )
        .saturating_add(row.cursor.timestamp.capacity() as u64);
    let header = size_of::<Vec<CursorRow>>() as u64;
    let slot = size_of::<CursorRow>() as u64;
    let minimum = header
        .saturating_add(next_payload)
        .saturating_add((rows.capacity().max(rows.len() + 1) as u64).saturating_mul(slot));
    if minimum > limit {
        if rows.is_empty() {
            return Err(OracleAdapterError::NormalizedByteLimit {
                normalized_bytes: minimum,
                limit,
            });
        }
        return Ok(false);
    }
    if rows.len() == rows.capacity() {
        // The first-row check proves enough room for another slot. Reserve no more
        // than the row limit or the remaining byte budget can hold.
        let available_slots =
            ((limit - header - next_payload) / slot).min(max_rows as u64) as usize;
        let target = rows
            .capacity()
            .saturating_mul(2)
            .max(4)
            .min(available_slots);
        rows.reserve_exact(target - rows.len());
        let allocated = header
            .saturating_add(next_payload)
            .saturating_add((rows.capacity() as u64).saturating_mul(slot));
        if allocated > limit {
            return Err(OracleAdapterError::NormalizedByteLimit {
                normalized_bytes: allocated,
                limit,
            });
        }
    }
    *payload_bytes = next_payload;
    rows.push(row);
    Ok(true)
}

/// Binds the committed cursor through Oracle named parameters.
///
/// The cursor is never interpolated into SQL text. Normal polling binds the
/// exact cursor-column type discovered from Oracle metadata, preserving both
/// timezone semantics and index-compatible comparisons.
fn bind_cursor<'a>(
    statement: &'a mut oracle::Statement,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    timestamp_type: &OracleType,
    cancellation: &OracleCancellation,
) -> Result<oracle::ResultSet<'a, OracleRow>, OracleAdapterError> {
    cancellation.ensure_not_requested()?;
    let watermark = query.watermark();
    let timestamp =
        cursor_bind_timestamp(parse_cursor_timestamp(&cursor.timestamp)?, timestamp_type)?;
    let timestamp_bind = (&timestamp, timestamp_type);
    let tie_breaker = cursor.tie_breaker;
    let result = statement
        .query_named(&[
            (watermark.timestamp_bind.as_str(), &timestamp_bind),
            (watermark.tie_breaker_bind.as_str(), &tie_breaker),
        ])
        .map_err(|error| OracleAdapterError::Query(error.into()))?;
    cancellation.ensure_not_requested()?;
    Ok(result)
}

/// Parses cursor text without overflow or lossy numeric narrowing in the driver.
pub(super) fn parse_cursor_timestamp(text: &str) -> Result<Timestamp, OracleAdapterError> {
    // oracle 0.6.3 uses unchecked numeric accumulation and narrowing casts.
    // Nine digits fit every intermediate type and avoid fractional truncation.
    let mut digits = 0;
    for byte in text.bytes() {
        if byte.is_ascii_digit() {
            digits += 1;
            if digits > MAX_TIMESTAMP_COMPONENT_DIGITS {
                return Err(OracleAdapterError::InvalidCursorTimestamp);
            }
        } else {
            digits = 0;
        }
    }
    Timestamp::from_str(text).map_err(|_| OracleAdapterError::InvalidCursorTimestamp)
}

/// Canonicalizes configuration cursors so equivalent offsets share identity.
pub(super) fn normalize_cursor_timestamp(text: &str) -> Result<String, OracleAdapterError> {
    cursor_bind_timestamp(parse_cursor_timestamp(text)?, &OracleType::TimestampTZ(9))
        .map(|timestamp| timestamp.to_string())
}

/// Uses a lossless universal bind only for the one metadata-discovery execution.
fn discovery_cursor_bind_type() -> OracleType {
    OracleType::TimestampTZ(9)
}

/// Reuses Oracle's exact cursor-column type and precision for normal polling.
fn cursor_bind_type(source_type: &OracleType) -> Result<OracleType, OracleAdapterError> {
    match source_type {
        OracleType::Date => Ok(source_type.clone()),
        OracleType::Timestamp(precision)
        | OracleType::TimestampTZ(precision)
        | OracleType::TimestampLTZ(precision)
            if *precision <= 9 =>
        {
            Ok(source_type.clone())
        }
        _ => Err(OracleAdapterError::UnsupportedCursorTimestamp),
    }
}

/// Normalizes offset-bearing cursors to UTC and the discovered source precision.
fn cursor_bind_timestamp(
    timestamp: Timestamp,
    target_type: &OracleType,
) -> Result<Timestamp, OracleAdapterError> {
    let (precision, zoned) = match target_type {
        OracleType::Date => (0, false),
        OracleType::Timestamp(precision) if *precision <= 9 => (*precision, false),
        OracleType::TimestampTZ(precision) | OracleType::TimestampLTZ(precision)
            if *precision <= 9 =>
        {
            (*precision, true)
        }
        _ => return Err(OracleAdapterError::UnsupportedCursorTimestamp),
    };
    let (year, month, day, hour, minute, second, nanosecond) = if timestamp.with_tz() {
        let local = NaiveDate::from_ymd_opt(timestamp.year(), timestamp.month(), timestamp.day())
            .and_then(|date| {
                date.and_hms_nano_opt(
                    timestamp.hour(),
                    timestamp.minute(),
                    timestamp.second(),
                    timestamp.nanosecond(),
                )
            })
            .ok_or(OracleAdapterError::InvalidCursorTimestamp)?;
        let utc = local
            .checked_sub_signed(chrono::TimeDelta::seconds(i64::from(timestamp.tz_offset())))
            .ok_or(OracleAdapterError::InvalidCursorTimestamp)?;
        (
            utc.year(),
            utc.month(),
            utc.day(),
            utc.hour(),
            utc.minute(),
            utc.second(),
            utc.nanosecond(),
        )
    } else {
        (
            timestamp.year(),
            timestamp.month(),
            timestamp.day(),
            timestamp.hour(),
            timestamp.minute(),
            timestamp.second(),
            timestamp.nanosecond(),
        )
    };
    let scale = 10_u32.pow(u32::from(9 - precision));
    let nanosecond = nanosecond / scale * scale;
    let timestamp = Timestamp::new(year, month, day, hour, minute, second, nanosecond)
        .and_then(|timestamp| timestamp.and_prec(precision))
        .map_err(|_| OracleAdapterError::InvalidCursorTimestamp)?;
    if zoned {
        timestamp
            .and_tz_offset(0)
            .map_err(|_| OracleAdapterError::InvalidCursorTimestamp)
    } else {
        Ok(timestamp)
    }
}

/// Builds and caches the statement and decode plan once per connection.
fn ensure_prepared(
    session: &mut OracleSession,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &OracleCancellation,
) -> Result<(), OracleAdapterError> {
    cancellation.ensure_not_requested()?;
    if session.prepared.is_some() {
        return Ok(());
    }

    // Oracle exposes result metadata only after execution. Discover it with a
    // one-row buffer, then rebuild once with a byte-bounded fetch array.
    let mut discovery = session
        .connection
        .statement(query.sql())
        .fetch_array_size(1)
        .prefetch_rows(0)
        .build()
        .map_err(|error| OracleAdapterError::Prepare(error.into()))?;
    let discovery_type = discovery_cursor_bind_type();
    let result_set = bind_cursor(&mut discovery, query, cursor, &discovery_type, cancellation)?;
    let (columns, types) = result_metadata(result_set.column_info())?;
    let (timestamp_index, tie_breaker_index) =
        validate_cursor_columns(result_set.column_info(), query)?;
    let timestamp_bind_type = cursor_bind_type(&types[timestamp_index])?;
    drop(result_set);
    drop(discovery);

    cancellation.ensure_not_requested()?;
    let fetch_rows = bounded_fetch_array_size(&types, query);
    let statement = session
        .connection
        .statement(query.sql())
        .fetch_array_size(fetch_rows)
        // Prefetch owns a second native row buffer outside the calculated byte budget.
        .prefetch_rows(0)
        .build()
        .map_err(|error| OracleAdapterError::Prepare(error.into()))?;
    cancellation.ensure_not_requested()?;
    session.prepared = Some(OraclePreparedQuery {
        statement,
        columns,
        types,
        timestamp_index,
        tie_breaker_index,
        timestamp_bind_type,
    });
    Ok(())
}

/// Caps native fetch rows by configured row, fetch, and normalized-memory limits.
fn bounded_fetch_array_size(types: &[OracleType], query: &CompiledQuery) -> u32 {
    let row_bytes = types.iter().fold(
        (size_of::<Row>() + types.len() * size_of::<CellValue>()) as u64,
        |total, data_type| total.saturating_add(max_normalized_value_bytes(data_type)),
    );
    let byte_limited = query.max_normalized_bytes() / row_bytes.max(1);
    let rows = (query.max_rows() as u64)
        .min(query.fetch_size_rows() as u64)
        .min(byte_limited)
        .max(1);
    rows as u32
}

/// Returns a conservative maximum normalized payload for one supported Oracle value.
fn max_normalized_value_bytes(data_type: &OracleType) -> u64 {
    match data_type {
        OracleType::Varchar2(bytes) | OracleType::Char(bytes) | OracleType::Raw(bytes) => {
            u64::from(*bytes)
        }
        OracleType::NVarchar2(characters) | OracleType::NChar(characters) => {
            u64::from(*characters).saturating_mul(4)
        }
        OracleType::Rowid => 128,
        OracleType::Number(_, _)
        | OracleType::Float(_)
        | OracleType::Date
        | OracleType::Timestamp(_)
        | OracleType::TimestampTZ(_)
        | OracleType::TimestampLTZ(_)
        | OracleType::IntervalDS(_, _)
        | OracleType::IntervalYM(_) => 128,
        OracleType::BinaryFloat
        | OracleType::BinaryDouble
        | OracleType::Int64
        | OracleType::UInt64 => 8,
        OracleType::Boolean => 1,
        // Unsupported types are rejected before this estimate is used.
        _ => 0,
    }
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
    let (timestamp, tie_breaker) =
        validate_described_cursor_columns(&described, query.watermark())?;
    if columns[timestamp].nullable() || columns[tie_breaker].nullable() {
        return Err(OracleAdapterError::NullableCursorColumn);
    }
    Ok((timestamp, tie_breaker))
}

/// Pure cursor-metadata validation over adapter-independent column descriptions.
fn validate_described_cursor_columns(
    columns: &[(String, OracleType)],
    watermark: &otel_arrow_dfe_scraper::database::CompositeWatermark,
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
        return Err(OracleAdapterError::UnsupportedCursorTimestamp);
    }
    let tie_breaker_type = &columns[tie_breaker_index].1;
    if !matches!(
        tie_breaker_type,
        OracleType::Int64 | OracleType::Number(1..=18, 0)
    ) {
        return Err(OracleAdapterError::UnsupportedCursorTieBreaker);
    }
    Ok((timestamp_index, tie_breaker_index))
}

/// Resolves a configured cursor column using Oracle's case-insensitive identifier rules.
fn cursor_column_index(
    columns: &[(String, OracleType)],
    name: &str,
) -> Result<usize, OracleAdapterError> {
    columns
        .iter()
        .position(|(column, _)| column.eq_ignore_ascii_case(name))
        .ok_or(OracleAdapterError::MissingCursorColumn)
}

/// Extracts the composite cursor from values already decoded for output.
fn extract_normalized_cursor(
    row: &Row,
    timestamp_index: usize,
    tie_breaker_index: usize,
) -> Result<CompositeCursor, OracleAdapterError> {
    let timestamp = match row.values.get(timestamp_index) {
        Some(CellValue::Timestamp(value) | CellValue::TimestampTz(value)) => value.clone(),
        Some(CellValue::Null) => return Err(OracleAdapterError::NullCursorValue),
        _ => return Err(OracleAdapterError::InvalidCursorValue),
    };
    let tie_breaker = match row.values.get(tie_breaker_index) {
        Some(CellValue::Int64(value)) => *value,
        Some(CellValue::Decimal(value)) => value
            .parse::<i64>()
            .map_err(|_| OracleAdapterError::InvalidCursorValue)?,
        Some(CellValue::Null) => return Err(OracleAdapterError::NullCursorValue),
        _ => return Err(OracleAdapterError::InvalidCursorValue),
    };
    Ok(CompositeCursor::new(timestamp, tie_breaker))
}

/// Opens or reuses the single session, publishes it for cancellation, and starts read-only work.
fn prepare_session(
    session: Option<OracleSession>,
    config: &OracleAdapterConfig,
    query: &CompiledQuery,
    cancellation: &OracleCancellation,
) -> Result<(OracleSession, ActiveConnection), OracleAdapterError> {
    cancellation.ensure_not_requested()?;
    let new_connection = session.is_none();
    let session = match session {
        Some(session) => session,
        None => OracleSession {
            connection: Arc::new(connect(config, query.timeout(), cancellation)?),
            prepared: None,
        },
    };
    let active = ActiveConnection::register(cancellation, &session.connection)?;
    cancellation.ensure_not_requested()?;
    session
        .connection
        .set_call_timeout(Some(query.timeout()))
        .map_err(|error| OracleAdapterError::Configure(error.into()))?;
    cancellation.ensure_not_requested()?;
    if new_connection {
        session
            .connection
            .ping()
            .map_err(|error| OracleAdapterError::Connect(error.into()))?;
        cancellation.ensure_not_requested()?;
        _ = session
            .connection
            .execute("ALTER SESSION SET TIME_ZONE = 'UTC'", &[])
            .map_err(|error| OracleAdapterError::Configure(error.into()))?;
        cancellation.ensure_not_requested()?;
    }
    begin_read_only(&session.connection)?;
    cancellation.ensure_not_requested()?;
    Ok((session, active))
}

/// Compiles stable public metadata and the per-column Oracle decode types.
fn result_metadata(
    columns: &[oracle::ColumnInfo],
) -> Result<(Vec<ColumnMetadata>, Vec<OracleType>), OracleAdapterError> {
    validate_column_names(columns.iter().map(oracle::ColumnInfo::name))?;
    let types = columns
        .iter()
        .map(|column| column.oracle_type().clone())
        .collect::<Vec<_>>();
    validate_types(&types)?;
    let metadata = columns.iter().map(column_metadata).collect();
    Ok((metadata, types))
}

fn validate_column_names<'a>(
    names: impl Iterator<Item = &'a str>,
) -> Result<(), OracleAdapterError> {
    let mut seen = std::collections::HashSet::new();
    for name in names {
        if !seen.insert(name.to_ascii_lowercase()) {
            return Err(OracleAdapterError::DuplicateColumns);
        }
    }
    Ok(())
}

/// Checks driver-reported result metadata against the cached decode plan.
///
/// oracle driver caches column metadata on reused statements, so this check
/// cannot reliably detect live schema changes. Such changes are unsupported;
/// restart the receiver instance to rebuild and validate the prepared query.
fn metadata_matches(
    columns: &[oracle::ColumnInfo],
    metadata: &[ColumnMetadata],
    types: &[OracleType],
) -> bool {
    columns.len() == metadata.len()
        && columns
            .iter()
            .zip(metadata.iter().zip(types))
            .all(|(column, (metadata, data_type))| {
                column.name() == metadata.name && column.oracle_type() == data_type
            })
}

/// Ends the read-only transaction without retaining database-side state between polls.
fn finish_session(connection: &Connection) -> Result<(), OracleAdapterError> {
    connection
        .rollback()
        .map_err(|error| OracleAdapterError::Configure(error.into()))
}

/// Converts Oracle metadata into the vendor-neutral scraper representation.
fn column_metadata(column: &oracle::ColumnInfo) -> ColumnMetadata {
    ColumnMetadata {
        name: column.name().to_owned(),
        source_type: column.oracle_type().to_string(),
        nullable: column.nullable(),
    }
}

/// Initializes the client, reads mounted credentials, and opens a connection.
fn connect(
    config: &OracleAdapterConfig,
    timeout: std::time::Duration,
    cancellation: &OracleCancellation,
) -> Result<Connection, OracleAdapterError> {
    cancellation.native_call(|| initialize_client(&config.instant_client_dir))?;
    // Mounted files are read for each new connection so secret rotation takes
    // effect after a reconnect without placing credentials in configuration.
    let username =
        cancellation.native_call(|| read_credential(&config.username_file, "username"))?;
    let password =
        cancellation.native_call(|| read_credential(&config.password_file, "password"))?;
    let connect_string = bounded_connect_string(&config.connect_string, timeout)?;
    cancellation.native_call(|| {
        Connection::connect(
            username.expose_secret(),
            password.expose_secret(),
            connect_string,
        )
        .map_err(|error| OracleAdapterError::Connect(error.into()))
    })
}

/// Makes the database enforce the receiver's read-only query contract.
fn begin_read_only(connection: &Connection) -> Result<(), OracleAdapterError> {
    // Static SQL inspection is intentionally conservative but cannot classify
    // every Oracle function; the database enforces the final read-only boundary.
    _ = connection
        .execute("SET TRANSACTION READ ONLY", &[])
        .map_err(|error| OracleAdapterError::Configure(error.into()))?;
    Ok(())
}

/// Injects fixed startup timeouts while rejecting options that could multiply attempts.
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

/// Applies the process-global Instant Client directory exactly once.
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
        .map_err(|error| OracleAdapterError::Initialize(error.into()))?;
    *selected = Some(directory.to_owned());
    Ok(())
}

/// Reads one bounded UTF-8 credential and zeroizes its storage on drop.
fn read_credential(path: &str, kind: &'static str) -> Result<SecretString, OracleAdapterError> {
    let path = Path::new(path);
    let metadata = std::fs::metadata(path).map_err(|source| OracleAdapterError::Credential {
        kind,
        failure: source.into(),
    })?;
    if !metadata.is_file() {
        return Err(OracleAdapterError::CredentialNotRegularFile(kind));
    }
    if metadata.len() > MAX_CREDENTIAL_BYTES {
        return Err(OracleAdapterError::CredentialTooLarge(kind));
    }
    let file = std::fs::File::open(path).map_err(|source| OracleAdapterError::Credential {
        kind,
        failure: source.into(),
    })?;
    if !file
        .metadata()
        .map_err(|source| OracleAdapterError::Credential {
            kind,
            failure: source.into(),
        })?
        .is_file()
    {
        return Err(OracleAdapterError::CredentialNotRegularFile(kind));
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(metadata.len() as usize));
    _ = file
        .take(MAX_CREDENTIAL_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| OracleAdapterError::Credential {
            kind,
            failure: source.into(),
        })?;
    if bytes.len() as u64 > MAX_CREDENTIAL_BYTES {
        return Err(OracleAdapterError::CredentialTooLarge(kind));
    }
    let value = std::str::from_utf8(&bytes)
        .map_err(|_| OracleAdapterError::InvalidCredentialEncoding(kind))?
        .trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        return Err(OracleAdapterError::EmptyCredential(kind));
    }
    Ok(value.to_owned().into())
}

/// Rejects result types that lack a precision-preserving normalization path.
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
            _ => {
                return Err(OracleAdapterError::UnsupportedType);
            }
        }
    }
    Ok(())
}

/// Applies the cached type plan to every cell in one Oracle row.
fn normalize_row(
    row: &OracleRow,
    types: &[OracleType],
    cancellation: &OracleCancellation,
) -> Result<Row, OracleAdapterError> {
    let values = types
        .iter()
        .enumerate()
        .map(|(index, source_type)| {
            cancellation.native_call(|| normalize_cell(row, index, source_type))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Row { values })
}

/// Converts one nullable Oracle scalar into the closed neutral value model.
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
                .map_err(|error| OracleAdapterError::Convert(error.into()))
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
        _ => Err(OracleAdapterError::UnsupportedType),
    }
}

/// Formats full timestamp precision and includes an offset only for zoned source types.
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

/// Rejects NaN and infinity because OTLP cannot preserve them portably.
fn finite_float(value: CellValue) -> Result<CellValue, OracleAdapterError> {
    match value {
        CellValue::Float64(value) if !value.is_finite() => Err(OracleAdapterError::NonFiniteFloat),
        value => Ok(value),
    }
}

/// Numeric Oracle diagnostics without retained native messages or source chains.
#[derive(Clone, Copy, Debug)]
pub struct OracleErrorCodes {
    oci: Option<i32>,
    dpi: Option<i32>,
}

impl From<oracle::Error> for OracleErrorCodes {
    fn from(error: oracle::Error) -> Self {
        Self {
            oci: error.oci_code(),
            dpi: error.dpi_code(),
        }
    }
}

impl OracleErrorCodes {
    /// Returns the Oracle database error code, when available.
    pub const fn oci_code(&self) -> Option<i32> {
        self.oci
    }

    /// Returns the ODPI-C error code, when available.
    pub const fn dpi_code(&self) -> Option<i32> {
        self.dpi
    }
}

/// OS diagnostics without retained filenames, custom error text or source chains.
#[derive(Clone, Copy, Debug)]
pub struct OracleIoError {
    kind: std::io::ErrorKind,
    code: Option<i32>,
}

impl From<std::io::Error> for OracleIoError {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: error.kind(),
            code: error.raw_os_error(),
        }
    }
}

/// Oracle connection, query, or conversion failure.
///
/// Native error messages may contain SQL, endpoint, row, or cursor data.
/// Expose only the operation and numeric codes, never native Debug/source chains.
#[derive(thiserror::Error)]
pub enum OracleAdapterError {
    /// A mounted credential file could not be read.
    #[error("failed to read Oracle {kind} file (kind {io_kind:?}, OS {code:?})", io_kind = .failure.kind, code = .failure.code)]
    Credential {
        /// Credential kind without its configured path.
        kind: &'static str,
        /// Sanitized file error category and numeric code.
        failure: OracleIoError,
    },
    /// A mounted credential path is not a regular file.
    #[error("Oracle {0} path must reference a regular file")]
    CredentialNotRegularFile(&'static str),
    /// A mounted credential file exceeds the fixed allocation bound.
    #[error("Oracle {0} file must not exceed 64 KiB")]
    CredentialTooLarge(&'static str),
    /// A mounted credential file is not UTF-8.
    #[error("Oracle {0} file must contain valid UTF-8")]
    InvalidCredentialEncoding(&'static str),
    /// A mounted credential file was empty.
    #[error("Oracle {0} file must not be empty")]
    EmptyCredential(&'static str),
    /// Oracle client initialization failed.
    #[error("Oracle client initialization failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Initialize(OracleErrorCodes),
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
    #[error("Oracle connection failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Connect(OracleErrorCodes),
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
    #[error("Oracle session configuration failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Configure(OracleErrorCodes),
    /// Statement preparation failed.
    #[error("Oracle query preparation failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Prepare(OracleErrorCodes),
    /// Query execution failed.
    #[error("Oracle query execution failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Query(OracleErrorCodes),
    /// Row fetching failed.
    #[error("Oracle row fetch failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Fetch(OracleErrorCodes),
    /// Native value conversion failed.
    #[error("Oracle value conversion failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Convert(OracleErrorCodes),
    /// A floating-point result cannot be represented faithfully.
    #[error("Oracle returned a non-finite floating-point value")]
    NonFiniteFloat,
    /// The result type does not have bounded conversion support.
    #[error("Oracle result contains an unsupported type")]
    UnsupportedType,
    /// A cached statement was invalidated with a different result shape.
    #[error("Oracle query result metadata changed after startup validation")]
    ResultMetadataChanged,
    /// A configured cursor column is absent from live result metadata.
    #[error("a configured watermark cursor column is not present in the query result")]
    MissingCursorColumn,
    /// Duplicate output names would make the shared row mapping ambiguous.
    #[error("Oracle query result column names must be distinct, ignoring ASCII case")]
    DuplicateColumns,
    /// Cursor nullability must be excluded by the source schema.
    #[error("watermark cursor columns must be declared NOT NULL")]
    NullableCursorColumn,
    /// The timestamp cursor column is not an Oracle date or timestamp type.
    #[error(
        "watermark timestamp column has an unsupported type; DATE and TIMESTAMP family types are required"
    )]
    UnsupportedCursorTimestamp,
    /// The tie-breaker cursor column is not an integral Oracle type.
    #[error(
        "watermark tie-breaker column has an unsupported type; a scale-zero integral type is required"
    )]
    UnsupportedCursorTieBreaker,
    /// A row's cursor component was SQL NULL.
    #[error("a watermark cursor column returned NULL; composite cursors must be non-null")]
    NullCursorValue,
    /// A cursor value failed its already-validated timestamp/integer contract.
    #[error("a watermark cursor value cannot be represented by the composite cursor")]
    InvalidCursorValue,
    /// The committed cursor timestamp cannot be bound to Oracle.
    #[error("committed watermark timestamp is not a valid Oracle timestamp")]
    InvalidCursorTimestamp,
    /// The first row alone exceeds the normalized in-memory ceiling.
    #[error(
        "the first database row normalizes to {normalized_bytes} bytes, exceeding the {limit}-byte budget from query.max_batch_bytes"
    )]
    NormalizedByteLimit {
        /// Normalized size of the single row.
        normalized_bytes: u64,
        /// Configured normalized-byte ceiling.
        limit: u64,
    },
    /// The Oracle query worker could not start, accept work, or confirm completion.
    #[error("Oracle worker failed (kind {kind:?}, OS {code:?})", kind = .0.kind, code = .0.code)]
    Worker(OracleIoError),
    /// Cancellation state could not be synchronized with the blocking worker.
    #[error("Oracle cancellation state is unavailable")]
    CancellationState,
    /// The native cancellation worker could not start, accept work, or confirm completion.
    #[error("Oracle cancellation worker failed (kind {kind:?}, OS {code:?})", kind = .0.kind, code = .0.code)]
    CancellationWorker(OracleIoError),
    /// Oracle rejected a request to interrupt the active call.
    #[error("Oracle cancellation failed (OCI {oci:?}, DPI {dpi:?})", oci = .0.oci_code(), dpi = .0.dpi_code())]
    Cancellation(OracleErrorCodes),
    /// An operation was cancelled before it registered its connection.
    #[error("Oracle operation was cancelled")]
    Cancelled,
}

impl std::fmt::Debug for OracleAdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, formatter)
    }
}

#[cfg(test)]
oracle_module_tests!(adapter);
