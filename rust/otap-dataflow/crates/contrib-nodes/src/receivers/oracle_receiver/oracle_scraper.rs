// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle-specific SQL polling and row encoding.

use super::config::{RuntimeConfig, WatermarkConfig};
use crate::receivers::sql_polling::{
    CompoundWatermark, Credentials, ErrorClass, Page, PageRequest, SqlColumn, SqlPollingAdapter,
    SqlRow,
};
use async_trait::async_trait;
use oracle::pool::{GetMode, Pool, PoolBuilder};
use oracle::sql_type::{OracleType, Timestamp};
use otap_df_engine::error::ReceiverErrorKind;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value,
};
use otap_df_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use prost::Message;
use serde::Serialize;
use std::future::Future;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// One encoded page awaiting downstream acknowledgement.
pub(super) struct OracleBatch {
    pub(super) pdata: OtapPdata,
    pub(super) candidate: CompoundWatermark,
    pub(super) row_count: usize,
    pub(super) encoded_bytes: usize,
}

/// OCI source owned by the Oracle receiver.
pub(super) struct OracleScraper {
    adapter: OracleAdapter,
    username: String,
    password_env: String,
    max_rows: usize,
    max_batch_bytes: u64,
    session: Option<Pool>,
}

#[derive(Clone)]
pub(super) struct OracleCancellation {
    state: Arc<Mutex<CancellationState>>,
}

#[derive(Default)]
struct CancellationState {
    requested: bool,
    connection: Option<Arc<oracle::Connection>>,
}

impl OracleScraper {
    pub(super) fn new(config: &RuntimeConfig) -> Self {
        let cancellation = OracleCancellation {
            state: Arc::new(Mutex::new(CancellationState::default())),
        };
        Self {
            adapter: OracleAdapter {
                connect_string: config.connect_string.clone(),
                query: config.query.clone(),
                call_timeout: config.call_timeout,
                watermark: config.watermark.clone(),
                max_batch_bytes: config.max_batch_bytes,
                cancellation,
            },
            username: config.username.clone(),
            password_env: config.password_env.clone(),
            max_rows: config.max_rows,
            max_batch_bytes: config.max_batch_bytes,
            session: None,
        }
    }

    pub(super) fn classify_error(&self, error: &OracleScraperError) -> ReceiverErrorKind {
        match error {
            OracleScraperError::PasswordEnvironment { .. } => ReceiverErrorKind::Configuration,
            OracleScraperError::Adapter(error) => match self.adapter.classify_error(error) {
                ErrorClass::Configuration => ReceiverErrorKind::Configuration,
                ErrorClass::Connection => ReceiverErrorKind::Connect,
                ErrorClass::Query => ReceiverErrorKind::Transport,
                ErrorClass::Internal => ReceiverErrorKind::Other,
            },
            OracleScraperError::ShutdownWorker(_)
            | OracleScraperError::CancellationWorker(_)
            | OracleScraperError::Cancellation(_) => ReceiverErrorKind::Shutdown,
            OracleScraperError::RowEncoding(_)
            | OracleScraperError::OversizedFirstRow { .. }
            | OracleScraperError::MissingCandidate
            | OracleScraperError::NotStarted => ReceiverErrorKind::Other,
        }
    }

    pub(super) fn cancellation(&self) -> OracleCancellation {
        self.adapter.cancellation.clone()
    }

    pub(super) async fn start(&mut self) -> Result<(), OracleScraperError> {
        if self.session.is_some() {
            return Ok(());
        }
        let password = std::env::var(&self.password_env).map_err(|source| {
            OracleScraperError::PasswordEnvironment {
                name: self.password_env.clone(),
                source,
            }
        })?;
        let credentials = Credentials::new(self.username.clone(), password);
        self.session = Some(self.adapter.connect(credentials).await?);
        Ok(())
    }

    pub(super) fn poll<'a>(
        &'a mut self,
        watermark: &'a CompoundWatermark,
    ) -> impl Future<Output = Result<Option<OracleBatch>, OracleScraperError>> + 'a {
        let begin_poll = self.adapter.begin_poll();
        async move {
            begin_poll?;
            let session = self
                .session
                .as_mut()
                .ok_or(OracleScraperError::NotStarted)?;
            let page = self
                .adapter
                .fetch_page(
                    session,
                    PageRequest {
                        watermark: watermark.clone(),
                        limit: self.max_rows,
                    },
                )
                .await?;
            prepare_batch(page, self.max_batch_bytes, unix_time_nanos())
        }
    }

    pub(super) async fn shutdown(&mut self) -> Result<(), OracleScraperError> {
        let Some(pool) = self.session.take() else {
            return Ok(());
        };
        tokio::task::spawn_blocking(move || drop(pool))
            .await
            .map_err(OracleScraperError::ShutdownWorker)?;
        Ok(())
    }
}

impl OracleCancellation {
    pub(super) fn cancel(&self) -> impl Future<Output = Result<(), OracleScraperError>> + use<> {
        let cancellation = (|| {
            let mut state = self
                .state
                .lock()
                .map_err(|_| OracleAdapterError::CancellationState)?;
            state.requested = true;
            Ok::<_, OracleScraperError>(state.connection.clone().map(|connection| {
                tokio::task::spawn_blocking(move || connection.break_execution())
            }))
        })();
        async move {
            let Some(cancellation) = cancellation? else {
                return Ok(());
            };
            cancellation
                .await
                .map_err(OracleScraperError::CancellationWorker)?
                .map_err(OracleScraperError::Cancellation)
        }
    }
}

struct OracleAdapter {
    connect_string: String,
    query: String,
    call_timeout: Duration,
    watermark: WatermarkConfig,
    max_batch_bytes: u64,
    cancellation: OracleCancellation,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum OracleAdapterError {
    #[error("Oracle connection failed: {0}")]
    Connect(#[source] oracle::Error),
    #[error("Oracle query failed: {0}")]
    Query(#[source] oracle::Error),
    #[error("Oracle watermark timestamp is invalid: {0}")]
    InvalidWatermark(String),
    #[error("{operation} blocking worker failed")]
    Worker {
        operation: &'static str,
        #[source]
        source: tokio::task::JoinError,
    },
    #[error("page limit must be greater than zero")]
    InvalidPageLimit,
    #[error("Oracle poll cancellation state is unavailable")]
    CancellationState,
    #[error("Oracle poll was cancelled")]
    Cancelled,
    #[error(
        "Oracle column {column} has unbounded type {data_type}; cast it to a bounded character type in the query"
    )]
    UnboundedColumn { column: String, data_type: String },
}

impl OracleAdapter {
    fn begin_poll(&self) -> Result<(), OracleAdapterError> {
        let mut state = self
            .cancellation
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        state.requested = false;
        state.connection = None;
        Ok(())
    }
}

#[async_trait(?Send)]
impl SqlPollingAdapter for OracleAdapter {
    type Session = Pool;
    type Error = OracleAdapterError;

    async fn connect(&self, credentials: Credentials) -> Result<Self::Session, Self::Error> {
        let (username, password) = credentials.into_parts();
        let connect_string = self.connect_string.clone();
        let call_timeout = self.call_timeout;
        tokio::task::spawn_blocking(move || {
            create_pool(username, password, connect_string, call_timeout)
        })
        .await
        .map_err(|source| OracleAdapterError::Worker {
            operation: "Oracle connection",
            source,
        })?
        .map_err(OracleAdapterError::Connect)
    }

    async fn fetch_page(
        &self,
        session: &mut Self::Session,
        request: PageRequest,
    ) -> Result<Page, Self::Error> {
        if request.limit == 0 {
            return Err(OracleAdapterError::InvalidPageLimit);
        }
        let timestamp = Timestamp::from_str(&request.watermark.timestamp)
            .map_err(|error| OracleAdapterError::InvalidWatermark(error.to_string()))?;
        let pool = session.clone();
        let query = self.query.clone();
        let call_timeout = self.call_timeout;
        let watermark = self.watermark.clone();
        let tie_breaker = request.watermark.tie_breaker;
        let max_batch_bytes = self.max_batch_bytes;
        let cancellation = self.cancellation.clone();
        tokio::task::spawn_blocking(move || {
            let connection = Arc::new(pool.get().map_err(OracleAdapterError::Query)?);
            let _active = ActiveConnection::register(&cancellation, connection.clone())?
                .ok_or(OracleAdapterError::Cancelled)?;
            query_rows(
                &connection,
                &query,
                &watermark,
                timestamp,
                tie_breaker,
                request.limit,
                max_batch_bytes,
                call_timeout,
            )
        })
        .await
        .map_err(|source| OracleAdapterError::Worker {
            operation: "Oracle query",
            source,
        })?
    }

    fn classify_error(&self, error: &Self::Error) -> ErrorClass {
        match error {
            OracleAdapterError::Connect(_) => ErrorClass::Connection,
            OracleAdapterError::Query(_) => ErrorClass::Query,
            OracleAdapterError::InvalidWatermark(_) | OracleAdapterError::InvalidPageLimit => {
                ErrorClass::Configuration
            }
            OracleAdapterError::Worker { .. }
            | OracleAdapterError::CancellationState
            | OracleAdapterError::Cancelled => ErrorClass::Internal,
            OracleAdapterError::UnboundedColumn { .. } => ErrorClass::Configuration,
        }
    }
}

struct ActiveConnection {
    cancellation: OracleCancellation,
}

impl ActiveConnection {
    fn register(
        cancellation: &OracleCancellation,
        connection: Arc<oracle::Connection>,
    ) -> Result<Option<Self>, OracleAdapterError> {
        let mut state = cancellation
            .state
            .lock()
            .map_err(|_| OracleAdapterError::CancellationState)?;
        if state.requested {
            return Ok(None);
        }
        state.connection = Some(connection);
        Ok(Some(Self {
            cancellation: cancellation.clone(),
        }))
    }
}

impl Drop for ActiveConnection {
    fn drop(&mut self) {
        if let Ok(mut state) = self.cancellation.state.lock() {
            state.connection = None;
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum OracleScraperError {
    #[error("failed to read Oracle password from environment variable {name}")]
    PasswordEnvironment {
        name: String,
        #[source]
        source: std::env::VarError,
    },
    #[error(transparent)]
    Adapter(#[from] OracleAdapterError),
    #[error("Oracle pool shutdown worker failed")]
    ShutdownWorker(#[source] tokio::task::JoinError),
    #[error("Oracle poll cancellation worker failed")]
    CancellationWorker(#[source] tokio::task::JoinError),
    #[error("Oracle poll cancellation failed")]
    Cancellation(#[source] oracle::Error),
    #[error(transparent)]
    RowEncoding(#[from] RowEncodingError),
    #[error("first Oracle row encodes to {encoded_bytes} bytes, above max_batch_bytes {limit}")]
    OversizedFirstRow { encoded_bytes: usize, limit: u64 },
    #[error("Oracle page contained rows without a final watermark candidate")]
    MissingCandidate,
    #[error("Oracle source has not been started")]
    NotStarted,
}

fn create_pool(
    username: String,
    password: String,
    connect_string: String,
    call_timeout: Duration,
) -> oracle::Result<Pool> {
    let mut builder = PoolBuilder::new(username, password, connect_string);
    _ = builder
        .min_connections(1)
        .max_connections(1)
        .connection_increment(0)
        .get_mode(GetMode::TimedWait(call_timeout));
    _ = builder.ping_interval(Some(Duration::from_secs(60)))?;
    _ = builder.ping_timeout(call_timeout)?;
    builder.build()
}

fn query_rows(
    connection: &oracle::Connection,
    query: &str,
    watermark: &WatermarkConfig,
    timestamp: Timestamp,
    tie_breaker: i64,
    max_rows: usize,
    max_batch_bytes: u64,
    call_timeout: Duration,
) -> Result<Page, OracleAdapterError> {
    connection
        .set_call_timeout(Some(call_timeout))
        .map_err(OracleAdapterError::Query)?;
    let mut statement = connection
        .statement(query)
        .fetch_array_size(max_rows.min(100) as u32)
        .build()
        .map_err(OracleAdapterError::Query)?;
    // Timestamp's default ToSql type includes a time zone. Force the configured
    // timezone-naive watermark type so session offsets cannot move the boundary.
    let timestamp_type = OracleType::Timestamp(9);
    let timestamp_bind = (&timestamp, &timestamp_type);
    let mut rows = statement
        .query_named(&[
            (watermark.timestamp.bind.as_str(), &timestamp_bind),
            (watermark.tie_breaker.bind.as_str(), &tie_breaker),
        ])
        .map_err(OracleAdapterError::Query)?;
    for column in rows.column_info() {
        if matches!(
            column.oracle_type(),
            OracleType::CLOB | OracleType::NCLOB | OracleType::Long | OracleType::Json
        ) {
            return Err(OracleAdapterError::UnboundedColumn {
                column: column.name().to_owned(),
                data_type: column.oracle_type().to_string(),
            });
        }
    }
    let column_names = rows
        .column_info()
        .iter()
        .map(|column| column.name().to_owned())
        .collect::<Vec<_>>();
    let mut result = Vec::with_capacity(max_rows.min(32));
    let mut candidate = None;
    let mut selected_value_bytes = 0u64;

    for row in rows.by_ref().take(max_rows) {
        let row = row.map_err(OracleAdapterError::Query)?;
        let row_watermark = CompoundWatermark {
            timestamp: row
                .get::<_, Timestamp>(watermark.timestamp.column.as_str())
                .map_err(OracleAdapterError::Query)?
                .to_string(),
            tie_breaker: row
                .get::<_, i64>(watermark.tie_breaker.column.as_str())
                .map_err(OracleAdapterError::Query)?,
        };
        let mut columns = Vec::with_capacity(column_names.len());
        for (index, name) in column_names.iter().enumerate() {
            let value = row
                .get::<_, Option<String>>(index)
                .map_err(OracleAdapterError::Query)?;
            let value_bytes = value.as_ref().map_or(0, String::len);
            columns.push(SqlColumn {
                name: name.clone(),
                value,
            });
            selected_value_bytes =
                selected_value_bytes.saturating_add(u64::try_from(value_bytes).unwrap_or(u64::MAX));
        }
        if !result.is_empty() && selected_value_bytes > max_batch_bytes {
            break;
        }
        candidate = Some(row_watermark.clone());
        result.push(SqlRow {
            columns,
            watermark: row_watermark,
        });
    }

    Ok(Page {
        rows: result,
        candidate,
    })
}

#[derive(Debug, Serialize)]
struct LogColumn<'a> {
    name: &'a str,
    value: &'a Option<String>,
}

#[derive(Debug, Serialize)]
struct LogRow<'a> {
    columns: Vec<LogColumn<'a>>,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum RowEncodingError {
    #[error("JSON row serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OTLP protobuf encoding failed: {0}")]
    Otlp(#[from] prost::EncodeError),
}

fn prepare_batch(
    page: Page,
    max_batch_bytes: u64,
    observed_time_unix_nano: u64,
) -> Result<Option<OracleBatch>, OracleScraperError> {
    if page.rows.is_empty() {
        debug_assert!(page.candidate.is_none());
        return Ok(None);
    }

    let mut records = Vec::with_capacity(page.rows.len());
    let mut candidate = None;
    let mut encoded_bytes = 0;
    let empty_logs = logs_from_records(Vec::new());
    let empty_resource = &empty_logs.resource_logs[0];
    let empty_scope = &empty_resource.scope_logs[0];
    let scope_static_bytes = empty_scope.encoded_len();
    let mut resource_without_scopes = empty_resource.clone();
    resource_without_scopes.scope_logs.clear();
    let resource_static_bytes = resource_without_scopes.encoded_len();
    let mut records_wire_bytes = 0usize;
    for row in &page.rows {
        let record = row_to_log(row, observed_time_unix_nano).map_err(RowEncodingError::Json)?;
        let record_bytes = record.encoded_len();
        let next_records_wire_bytes = records_wire_bytes
            .saturating_add(1)
            .saturating_add(prost::encoding::encoded_len_varint(record_bytes as u64))
            .saturating_add(record_bytes);
        let scope_bytes = scope_static_bytes.saturating_add(next_records_wire_bytes);
        let resource_bytes = resource_static_bytes
            .saturating_add(1)
            .saturating_add(prost::encoding::encoded_len_varint(scope_bytes as u64))
            .saturating_add(scope_bytes);
        let candidate_size = 1usize
            .saturating_add(prost::encoding::encoded_len_varint(resource_bytes as u64))
            .saturating_add(resource_bytes);
        if u64::try_from(candidate_size).unwrap_or(u64::MAX) > max_batch_bytes {
            if records.is_empty() {
                return Err(OracleScraperError::OversizedFirstRow {
                    encoded_bytes: candidate_size,
                    limit: max_batch_bytes,
                });
            }
            break;
        }
        records.push(record);
        records_wire_bytes = next_records_wire_bytes;
        encoded_bytes = candidate_size;
        candidate = Some(row.watermark.clone());
    }

    let candidate = candidate.ok_or(OracleScraperError::MissingCandidate)?;
    let row_count = records.len();
    let logs = logs_from_records(records);
    debug_assert_eq!(encoded_bytes, logs.encoded_len());
    let payload = OtlpProtoMessage::Logs(logs)
        .try_into()
        .map_err(RowEncodingError::Otlp)?;
    Ok(Some(OracleBatch {
        pdata: OtapPdata::new_todo_context(payload),
        candidate,
        row_count,
        encoded_bytes,
    }))
}

fn row_to_log(row: &SqlRow, observed_time_unix_nano: u64) -> Result<LogRecord, serde_json::Error> {
    let body = LogRow {
        columns: row
            .columns
            .iter()
            .map(|column| LogColumn {
                name: &column.name,
                value: &column.value,
            })
            .collect(),
    };
    Ok(LogRecord {
        observed_time_unix_nano,
        severity_number: SeverityNumber::Info as i32,
        severity_text: "INFO".to_owned(),
        body: Some(string_value(serde_json::to_string(&body)?)),
        event_name: "oracle.query.row".to_owned(),
        ..Default::default()
    })
}

fn logs_from_records(log_records: Vec<LogRecord>) -> LogsData {
    LogsData {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "db.system.name".to_owned(),
                    value: Some(string_value("oracle")),
                }],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: Some(InstrumentationScope {
                    name: "otel-arrow.oracle_receiver".to_owned(),
                    ..Default::default()
                }),
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn string_value(value: impl Into<String>) -> AnyValue {
    AnyValue {
        value: Some(any_value::Value::StringValue(value.into())),
    }
}

fn unix_time_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::oracle_receiver::checkpoint::CheckpointStore;
    use crate::receivers::oracle_receiver::config::Config;
    use serde_json::Value;

    fn row(timestamp: &str, tie_breaker: i64, value: &str) -> SqlRow {
        SqlRow {
            columns: vec![
                SqlColumn {
                    name: "EVENT_TS".to_owned(),
                    value: Some(timestamp.to_owned()),
                },
                SqlColumn {
                    name: "EVENT_ID".to_owned(),
                    value: Some(tie_breaker.to_string()),
                },
                SqlColumn {
                    name: "PAYLOAD".to_owned(),
                    value: Some(value.to_owned()),
                },
            ],
            watermark: CompoundWatermark {
                timestamp: timestamp.to_owned(),
                tie_breaker,
            },
        }
    }

    /// Scenario: Oracle rows are encoded as bounded OTLP logs.
    /// Guarantees: row JSON and the final emitted row's watermark are preserved.
    #[test]
    fn rows_encode_with_final_candidate() {
        let page = Page {
            rows: vec![
                row("2026-01-01 00:00:00", 1, "alpha"),
                row("2026-01-01 00:00:00", 2, "beta"),
            ],
            candidate: Some(CompoundWatermark {
                timestamp: "2026-01-01 00:00:00".to_owned(),
                tie_breaker: 2,
            }),
        };
        let batch = prepare_batch(page, 1024 * 1024, 123)
            .expect("encode")
            .expect("batch");

        assert_eq!(batch.row_count, 2);
        assert_eq!(batch.candidate.tie_breaker, 2);
        assert!(batch.encoded_bytes > 0);
    }

    /// Scenario: adding a later row would exceed the encoded-byte limit.
    /// Guarantees: only the fitting prefix is emitted and its final row supplies the candidate.
    #[test]
    fn encoded_byte_limit_uses_last_emitted_row() {
        let first = row("2026-01-01 00:00:00", 1, "small");
        let first_size = logs_from_records(vec![row_to_log(&first, 123).unwrap()]).encoded_len();
        let page = Page {
            rows: vec![first, row("2026-01-01 00:00:00", 2, &"x".repeat(4096))],
            candidate: Some(CompoundWatermark {
                timestamp: "2026-01-01 00:00:00".to_owned(),
                tie_breaker: 2,
            }),
        };
        let batch = prepare_batch(page, first_size as u64, 123)
            .expect("encode")
            .expect("batch");

        assert_eq!(batch.row_count, 1);
        assert_eq!(batch.candidate.tie_breaker, 1);
    }

    /// Scenario: the first selected row alone exceeds max_batch_bytes.
    /// Guarantees: the receiver errors instead of skipping an unrepresentable row.
    #[test]
    fn oversized_first_row_is_an_error() {
        let page = Page {
            rows: vec![row("2026-01-01 00:00:00", 1, &"x".repeat(4096))],
            candidate: Some(CompoundWatermark {
                timestamp: "2026-01-01 00:00:00".to_owned(),
                tie_breaker: 1,
            }),
        };
        assert!(matches!(
            prepare_batch(page, 1, 123),
            Err(OracleScraperError::OversizedFirstRow { .. })
        ));
    }

    /// Scenario: a SQL NULL appears in an Oracle row body.
    /// Guarantees: JSON encoding preserves SQL NULL instead of converting it to text.
    #[test]
    fn null_column_is_preserved() {
        let row = SqlRow {
            columns: vec![SqlColumn {
                name: "OPTIONAL_VALUE".to_owned(),
                value: None,
            }],
            watermark: CompoundWatermark {
                timestamp: "2026-01-01 00:00:00".to_owned(),
                tie_breaker: 1,
            },
        };
        let record = row_to_log(&row, 123).expect("log");
        let Some(AnyValue {
            value: Some(any_value::Value::StringValue(body)),
        }) = record.body
        else {
            panic!("body");
        };
        let body: Value = serde_json::from_str(&body).expect("json");
        assert!(body["columns"][0]["value"].is_null());
    }

    /// Scenario: shutdown is requested before an Oracle poll worker registers its connection.
    /// Guarantees: cancellation is recorded synchronously and cannot be cleared by select scheduling.
    #[test]
    fn cancellation_request_is_synchronous() {
        let cancellation = OracleCancellation {
            state: Arc::new(Mutex::new(CancellationState::default())),
        };
        let _cancellation = cancellation.cancel();

        assert!(
            cancellation
                .state
                .lock()
                .expect("cancellation state")
                .requested
        );
    }

    /// Scenario: a live Oracle source spans pages, timestamp collisions, a non-UTC session, NACK replay, restart, and a concurrent insert.
    /// Guarantees: timestamp binds preserve timezone-naive boundaries, only ACKed final-row tuples advance durable state, and the final checkpoint drains every ordered row.
    #[tokio::test(flavor = "current_thread")]
    async fn live_composite_watermark_checkpoint_when_configured() {
        if std::env::var_os("OTAP_ORACLE_RECEIVER_E2E").is_none() {
            return;
        }

        let username = std::env::var("ORACLE_USERNAME").expect("ORACLE_USERNAME");
        let password = std::env::var("ORACLE_PWD").expect("ORACLE_PWD");
        let connect_string = std::env::var("ORACLE_CONNECT_STRING").expect("ORACLE_CONNECT_STRING");
        let table = format!("OTAP_ORA_E2E_{}", std::process::id());
        let connection =
            oracle::Connection::connect(&username, &password, &connect_string).expect("connect");
        match connection.execute(
            &format!(
                "CREATE TABLE {table} (
                    EVENT_TS TIMESTAMP(9) NOT NULL,
                    EVENT_ID NUMBER(19) NOT NULL PRIMARY KEY,
                    PAYLOAD VARCHAR2(200) NOT NULL
                )"
            ),
            &[],
        ) {
            Ok(_) => {}
            Err(error)
                if error
                    .db_error()
                    .is_some_and(|db_error| db_error.code() == 955) => {}
            Err(error) => panic!("create fixture: {error}"),
        }
        let _ = connection
            .execute(&format!("DELETE FROM {table}"), &[])
            .expect("clear table");
        for event_id in 1i64..=7 {
            let offset = (event_id - 1) / 2;
            let payload = format!("event-{event_id}");
            let _ = connection
                .execute(
                    &format!(
                        "INSERT INTO {table} (EVENT_TS, EVENT_ID, PAYLOAD)
                         VALUES (
                            TO_TIMESTAMP('2026-01-01 00:00:00', 'YYYY-MM-DD HH24:MI:SS')
                                + NUMTODSINTERVAL(:1, 'SECOND'),
                            :2,
                            :3
                         )"
                    ),
                    &[&offset, &event_id, &payload],
                )
                .expect("insert fixture");
        }
        connection.commit().expect("commit fixture");

        let checkpoint_dir = tempfile::tempdir().expect("checkpoint tempdir");
        let query = format!(
            "SELECT EVENT_TS, EVENT_ID, PAYLOAD FROM {table}
             WHERE (EVENT_TS > :last_ts OR (EVENT_TS = :last_ts AND EVENT_ID > :last_id))
             ORDER BY EVENT_TS ASC, EVENT_ID ASC"
        );
        let config: Config = serde_json::from_value(serde_json::json!({
            "source_id": "oracle-e2e",
            "connect_string": connect_string,
            "username": username,
            "password_env": "ORACLE_PWD",
            "query": query,
            "watermark": {
                "timestamp": {
                    "column": "EVENT_TS",
                    "bind": "last_ts",
                    "initial": "2025-12-31 23:59:59"
                },
                "tie_breaker": {
                    "column": "EVENT_ID",
                    "bind": "last_id",
                    "initial": 0
                }
            },
            "checkpoint": {
                "directory": checkpoint_dir.path(),
                "max_consecutive_failures": 3
            },
            "max_rows": 3,
            "max_batch_bytes": "1 MiB"
        }))
        .expect("config");
        let runtime = RuntimeConfig::try_from(config).expect("runtime config");
        let _ = connection
            .execute("ALTER SESSION SET TIME_ZONE = '-08:00'", &[])
            .expect("set non-UTC session time zone");
        let non_utc_resume = query_rows(
            &connection,
            &runtime.query,
            &runtime.watermark,
            Timestamp::from_str("2026-01-01 00:00:01.000000000").expect("resume timestamp"),
            3,
            3,
            runtime.max_batch_bytes,
            runtime.call_timeout,
        )
        .expect("query from a non-UTC session");
        assert_eq!(
            non_utc_resume
                .candidate
                .expect("non-UTC resume candidate")
                .tie_breaker,
            6
        );
        let checkpoint = CheckpointStore::new(
            checkpoint_dir.path(),
            "e2e-group",
            "e2e-pipeline",
            "oracle",
            &runtime.source_id,
            runtime.config_fingerprint.clone(),
        );

        let mut source = OracleScraper::new(&runtime);
        source.start().await.expect("start");
        let first = source
            .poll(&runtime.initial_watermark)
            .await
            .expect("first page")
            .expect("first batch");
        assert_eq!(first.row_count, 3);
        assert_eq!(first.candidate.tie_breaker, 3);

        let replay = source
            .poll(&runtime.initial_watermark)
            .await
            .expect("NACK replay")
            .expect("replayed batch");
        assert_eq!(replay.row_count, first.row_count);
        assert_eq!(replay.candidate, first.candidate);

        let (first_checkpoint, _) = checkpoint
            .write(0, &first.candidate)
            .expect("ACK first page");
        source.shutdown().await.expect("stop before restart");

        let restored = checkpoint.read().expect("read checkpoint").expect("state");
        assert_eq!(restored, first_checkpoint);
        let mut restarted = OracleScraper::new(&runtime);
        restarted.start().await.expect("restart");
        let second = restarted
            .poll(&restored.watermark)
            .await
            .expect("second page")
            .expect("second batch");
        assert_eq!(second.row_count, 3);
        assert_eq!(second.candidate.tie_breaker, 6);

        let concurrent_id = 8i64;
        let concurrent_offset = 3i64;
        let _ = connection
            .execute(
                &format!(
                    "INSERT INTO {table} (EVENT_TS, EVENT_ID, PAYLOAD)
                     VALUES (
                        TO_TIMESTAMP('2026-01-01 00:00:00', 'YYYY-MM-DD HH24:MI:SS')
                            + NUMTODSINTERVAL(:1, 'SECOND'),
                        :2,
                        'concurrent'
                     )"
                ),
                &[&concurrent_offset, &concurrent_id],
            )
            .expect("concurrent insert");
        connection.commit().expect("commit concurrent insert");

        let (second_checkpoint, _) = checkpoint
            .write(restored.revision, &second.candidate)
            .expect("ACK second page");
        let final_batch = restarted
            .poll(&second_checkpoint.watermark)
            .await
            .expect("final page")
            .expect("final batch");
        assert_eq!(final_batch.row_count, 2);
        assert_eq!(final_batch.candidate.tie_breaker, 8);
        let (final_checkpoint, _) = checkpoint
            .write(second_checkpoint.revision, &final_batch.candidate)
            .expect("ACK final page");
        assert!(
            restarted
                .poll(&final_checkpoint.watermark)
                .await
                .expect("empty page")
                .is_none()
        );
        assert_eq!(
            checkpoint.read().expect("final read"),
            Some(final_checkpoint)
        );

        restarted.shutdown().await.expect("shutdown");
        let _ = connection
            .execute(&format!("DROP TABLE {table} PURGE"), &[])
            .expect("drop fixture");
    }
}
