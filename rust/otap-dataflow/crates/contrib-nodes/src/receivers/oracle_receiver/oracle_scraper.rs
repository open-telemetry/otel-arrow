// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle-specific implementation of the SQL adapter and scraper lifecycle.

use super::{DEFAULT_MAX_ROWS, OracleScraperConfig};
use crate::receivers::scraper::{Scraper, ScraperPhase};
use crate::receivers::sql_polling::{
    Credentials, ErrorClass, Page, PageRequest, SqlColumn, SqlPollingAdapter, SqlRow,
};
use async_trait::async_trait;
use oracle::pool::{GetMode, Pool, PoolBuilder};
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
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Oracle scraper backed by the narrow SQL polling adapter.
pub(super) struct OracleScraper {
    adapter: OracleAdapter,
    username: String,
    password_env: String,
    max_rows: usize,
    session: Option<Pool>,
}

impl OracleScraper {
    /// Creates an Oracle scraper. OCI resources are opened by [`Scraper::start`].
    pub(super) fn new(config: OracleScraperConfig) -> Self {
        Self {
            adapter: OracleAdapter {
                connect_string: config.connect_string,
                query: config.query,
                call_timeout: config.call_timeout,
            },
            username: config.username,
            password_env: config.password_env,
            max_rows: config.max_rows,
            session: None,
        }
    }
}

struct OracleAdapter {
    connect_string: String,
    query: String,
    call_timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum OracleAdapterError {
    #[error("Oracle connection failed: {0}")]
    Connect(#[source] oracle::Error),
    #[error("Oracle query failed: {0}")]
    Query(#[source] oracle::Error),
    #[error("{operation} blocking worker failed")]
    Worker {
        operation: &'static str,
        #[source]
        source: tokio::task::JoinError,
    },
    #[error("page limit must be greater than zero")]
    InvalidPageLimit,
    #[error(
        "Oracle compound-watermark polling is not implemented for timestamp {timestamp} and tie-breaker {tie_breaker}"
    )]
    UnsupportedWatermark {
        timestamp: String,
        tie_breaker: String,
    },
}

#[async_trait(?Send)]
impl SqlPollingAdapter for OracleAdapter {
    type Session = Pool;
    type Error = OracleAdapterError;

    async fn connect(&self, credentials: Credentials) -> Result<Self::Session, Self::Error> {
        let (username, password) = credentials.into_parts();
        let connect_string = self.connect_string.clone();
        let call_timeout = self.call_timeout;

        // The driver exposes synchronous OCI calls. Keeping them in a blocking
        // worker preserves the local pipeline core while the one-session pool
        // and sequential lifecycle bound work to one operation per receiver.
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
        if let Some(watermark) = request.watermark {
            return Err(OracleAdapterError::UnsupportedWatermark {
                timestamp: watermark.timestamp,
                tie_breaker: watermark.tie_breaker,
            });
        }

        let pool = session.clone();
        let query = self.query.clone();
        let call_timeout = self.call_timeout;
        let rows = tokio::task::spawn_blocking(move || {
            query_rows(&pool, &query, request.limit, call_timeout)
        })
        .await
        .map_err(|source| OracleAdapterError::Worker {
            operation: "Oracle query",
            source,
        })?
        .map_err(OracleAdapterError::Query)?;

        Ok(Page { rows })
    }

    fn classify_error(&self, error: &Self::Error) -> ErrorClass {
        match error {
            OracleAdapterError::Connect(_) => ErrorClass::Connection,
            OracleAdapterError::Query(_) => ErrorClass::Query,
            OracleAdapterError::Worker { .. } => ErrorClass::Internal,
            OracleAdapterError::InvalidPageLimit
            | OracleAdapterError::UnsupportedWatermark { .. } => ErrorClass::Configuration,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum OracleScraperError {
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
    #[error(transparent)]
    RowEncoding(#[from] RowEncodingError),
    #[error("Oracle scraper has not been started")]
    NotStarted,
}

#[async_trait(?Send)]
impl Scraper for OracleScraper {
    type Error = OracleScraperError;

    fn name(&self) -> &'static str {
        "Oracle"
    }

    fn classify_error(&self, phase: ScraperPhase, error: &Self::Error) -> ReceiverErrorKind {
        match error {
            OracleScraperError::PasswordEnvironment { .. } => ReceiverErrorKind::Configuration,
            OracleScraperError::Adapter(error) => match self.adapter.classify_error(error) {
                ErrorClass::Configuration => ReceiverErrorKind::Configuration,
                ErrorClass::Connection => ReceiverErrorKind::Connect,
                ErrorClass::Query => ReceiverErrorKind::Transport,
                ErrorClass::Internal => ReceiverErrorKind::Other,
            },
            OracleScraperError::ShutdownWorker(_) => ReceiverErrorKind::Shutdown,
            OracleScraperError::RowEncoding(_) | OracleScraperError::NotStarted => {
                if matches!(phase, ScraperPhase::Shutdown) {
                    ReceiverErrorKind::Shutdown
                } else {
                    ReceiverErrorKind::Other
                }
            }
        }
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
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

    async fn scrape(&mut self) -> Result<Option<OtapPdata>, Self::Error> {
        let session = self
            .session
            .as_mut()
            .ok_or(OracleScraperError::NotStarted)?;
        let page = self
            .adapter
            .fetch_page(
                session,
                PageRequest {
                    watermark: None,
                    limit: self.max_rows,
                },
            )
            .await?;

        if page.rows.is_empty() {
            return Ok(None);
        }

        Ok(Some(rows_to_pdata(page.rows, unix_time_nanos())?))
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        let Some(pool) = self.session.take() else {
            return Ok(());
        };

        tokio::task::spawn_blocking(move || drop(pool))
            .await
            .map_err(OracleScraperError::ShutdownWorker)?;
        Ok(())
    }
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
    pool: &Pool,
    query: &str,
    max_rows: usize,
    call_timeout: Duration,
) -> oracle::Result<Vec<SqlRow>> {
    let connection = pool.get()?;
    connection.set_call_timeout(Some(call_timeout))?;

    let mut statement = connection
        .statement(query)
        .fetch_array_size(max_rows.min(DEFAULT_MAX_ROWS) as u32)
        .build()?;
    let mut rows = statement.query(&[])?;
    let column_names = rows
        .column_info()
        .iter()
        .map(|column| column.name().to_owned())
        .collect::<Vec<_>>();
    let mut result = Vec::with_capacity(max_rows.min(32));

    for row in rows.by_ref().take(max_rows) {
        let row = row?;
        let mut columns = Vec::with_capacity(column_names.len());
        for (index, name) in column_names.iter().enumerate() {
            columns.push(SqlColumn {
                name: name.clone(),
                value: row.get::<_, Option<String>>(index)?,
            });
        }
        result.push(SqlRow { columns });
    }

    Ok(result)
}

#[derive(Debug, Serialize)]
struct LogColumn {
    name: String,
    value: Option<String>,
}

#[derive(Debug, Serialize)]
struct LogRow {
    columns: Vec<LogColumn>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RowEncodingError {
    #[error("JSON row serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OTLP protobuf encoding failed: {0}")]
    Otlp(#[from] prost::EncodeError),
}

fn rows_to_pdata(
    rows: Vec<SqlRow>,
    observed_time_unix_nano: u64,
) -> Result<OtapPdata, RowEncodingError> {
    let logs = rows_to_logs(rows, observed_time_unix_nano)?;
    let payload = OtlpProtoMessage::Logs(logs).try_into()?;
    Ok(OtapPdata::new_todo_context(payload))
}

fn rows_to_logs(
    rows: Vec<SqlRow>,
    observed_time_unix_nano: u64,
) -> Result<LogsData, serde_json::Error> {
    let log_records = rows
        .into_iter()
        .map(|row| {
            let row = LogRow {
                columns: row
                    .columns
                    .into_iter()
                    .map(|column| LogColumn {
                        name: column.name,
                        value: column.value,
                    })
                    .collect(),
            };
            Ok(LogRecord {
                observed_time_unix_nano,
                severity_number: SeverityNumber::Info as i32,
                severity_text: "INFO".to_owned(),
                body: Some(string_value(serde_json::to_string(&row)?)),
                event_name: "oracle.query.row".to_owned(),
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>, serde_json::Error>>()?;

    Ok(LogsData {
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
    })
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
    use otap_df_pdata::proto::opentelemetry::common::v1::any_value;
    use serde_json::Value;

    fn test_adapter() -> OracleAdapter {
        OracleAdapter {
            connect_string: "//localhost:1521/FREEPDB1".to_owned(),
            query: "SELECT 1 FROM DUAL".to_owned(),
            call_timeout: Duration::from_secs(10),
        }
    }

    /// Scenario: an Oracle query returns string and NULL column values in one row.
    /// Guarantees: one OTLP log is produced and its JSON body preserves names and NULL values.
    #[test]
    fn rows_are_encoded_as_otlp_logs() {
        let logs = rows_to_logs(
            vec![SqlRow {
                columns: vec![
                    SqlColumn {
                        name: "NAME".to_owned(),
                        value: Some("alpha".to_owned()),
                    },
                    SqlColumn {
                        name: "OPTIONAL_VALUE".to_owned(),
                        value: None,
                    },
                ],
            }],
            123,
        )
        .expect("rows should encode");

        let record = &logs.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(record.observed_time_unix_nano, 123);
        let Some(AnyValue {
            value: Some(any_value::Value::StringValue(body)),
        }) = record.body.as_ref()
        else {
            panic!("row body should be a string");
        };
        let body: Value = serde_json::from_str(body).expect("body should be JSON");
        assert_eq!(body["columns"][0]["name"], "NAME");
        assert_eq!(body["columns"][0]["value"], "alpha");
        assert!(body["columns"][1]["value"].is_null());
    }

    /// Scenario: the Oracle password environment variable is missing during startup.
    /// Guarantees: the scraper classifies the failure as actionable configuration.
    #[test]
    fn missing_password_is_a_configuration_error() {
        let scraper = OracleScraper::new(OracleScraperConfig {
            connect_string: "//localhost:1521/FREEPDB1".to_owned(),
            username: "PDBADMIN".to_owned(),
            password_env: "MISSING_ORACLE_PASSWORD".to_owned(),
            query: "SELECT 1 FROM DUAL".to_owned(),
            call_timeout: Duration::from_secs(10),
            max_rows: 10,
        });
        let error = OracleScraperError::PasswordEnvironment {
            name: "MISSING_ORACLE_PASSWORD".to_owned(),
            source: std::env::VarError::NotPresent,
        };

        assert_eq!(
            scraper.classify_error(ScraperPhase::Start, &error),
            ReceiverErrorKind::Configuration
        );
    }

    /// Scenario: Oracle reports that compound-watermark paging is not implemented.
    /// Guarantees: the adapter classifies the unsupported request as configuration.
    #[test]
    fn unsupported_watermark_is_a_configuration_error() {
        let adapter = test_adapter();
        let error = OracleAdapterError::UnsupportedWatermark {
            timestamp: "2026-08-18T00:00:00Z".to_owned(),
            tie_breaker: "42".to_owned(),
        };

        assert_eq!(adapter.classify_error(&error), ErrorClass::Configuration);
    }
}
