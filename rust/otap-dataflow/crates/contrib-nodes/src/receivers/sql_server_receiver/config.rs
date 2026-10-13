// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validated SQL Server configuration without network or filesystem side effects.

use super::sql::SqlServerStatement;
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use otel_arrow_dfe_scraper::database::{
    CatchUpConfig, CheckpointConfig, CompiledQuery, OutputConfig, PollingConfig, ScalarValue,
    WatermarkConfig,
};
use secrecy::zeroize::Zeroizing;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::{fmt, net::IpAddr, path::Path, time::Duration};

/// Configuration compiled without acquiring credentials, opening a session, or locking state.
pub struct SqlServerReceiverConfig {
    source_id: String,
    connection: SqlServerConnectionConfig,
    query: CompiledQuery,
    statement: SqlServerStatement,
    checkpoint: CheckpointConfig,
    fingerprint: String,
}

impl<'de> Deserialize<'de> for SqlServerReceiverConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawConfig::deserialize(deserializer).map_err(|_| {
            D::Error::custom(
                "invalid SQL Server configuration: check required fields, names, types, and enum values",
            )
        })?;
        Self::compile(raw).map_err(D::Error::custom)
    }
}

impl fmt::Debug for SqlServerReceiverConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SqlServerReceiverConfig(<redacted>)")
    }
}

impl SqlServerReceiverConfig {
    /// Returns the non-secret logical source identifier.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Returns connection options; credentials must come from BasicAuthProvider.
    #[must_use]
    pub const fn connection(&self) -> &SqlServerConnectionConfig {
        &self.connection
    }

    /// Returns the shared query plan with canonical operator SQL and logical binds.
    #[must_use]
    pub const fn query(&self) -> &CompiledQuery {
        &self.query
    }

    /// Returns the separately prepared driver SQL and ordered parameter roles.
    #[must_use]
    pub const fn statement(&self) -> &SqlServerStatement {
        &self.statement
    }

    /// Returns the shared persistence and feedback policy.
    #[must_use]
    pub const fn checkpoint(&self) -> &CheckpointConfig {
        &self.checkpoint
    }

    /// Returns the versioned semantic fingerprint, excluding secrets and operational tuning.
    #[must_use]
    pub fn config_fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Returns the bound TOP value, including the snapshot overflow probe.
    #[must_use]
    pub fn page_limit(&self) -> usize {
        self.query.max_rows()
            + usize::from(matches!(
                self.query.watermark(),
                otel_arrow_dfe_scraper::database::CompiledWatermark::Snapshot
            ))
    }

    fn compile(mut raw: RawConfig) -> Result<Self, SqlServerConfigError> {
        if raw.source_id.is_empty()
            || raw.source_id.len() > 256
            || !raw
                .source_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        {
            return Err(error(
                "source_id must contain 1..256 ASCII letters, digits, underscores, hyphens, or dots",
            ));
        }
        let connection = SqlServerConnectionConfig::try_from(raw.connection)?;
        whole_seconds("query.interval", raw.query.interval, 60, 24 * 60 * 60)?;
        whole_seconds("query.timeout", raw.query.timeout, 1, 300)?;
        let polling = raw.query.polling();
        polling
            .validate()
            .map_err(|e| SqlServerConfigError(e.to_string()))?;
        raw.watermark
            .validate()
            .map_err(|e| SqlServerConfigError(e.to_string()))?;
        raw.checkpoint
            .validate()
            .map_err(|e| SqlServerConfigError(e.to_string()))?;

        let output = match &mut raw.watermark {
            WatermarkConfig::Snapshot {} => OutputConfig::default(),
            WatermarkConfig::Scalar {
                column, initial, ..
            } => {
                super::sql::validate_column_name(column)?;
                let timestamp_column = match initial {
                    ScalarValue::Int64(_) => None,
                    ScalarValue::Timestamp(value) => {
                        *value = normalize_timestamp(value)?;
                        Some(column.clone())
                    }
                    ScalarValue::UInt64(_) | ScalarValue::String(_) => {
                        return Err(error(
                            "SQL Server scalar cursors support only int64 and datetime2 timestamps",
                        ));
                    }
                };
                OutputConfig {
                    timestamp_column,
                    validation_columns: vec![column.clone()],
                }
            }
            WatermarkConfig::Composite {
                timestamp,
                tie_breaker,
            } => {
                super::sql::validate_column_name(&timestamp.column)?;
                super::sql::validate_column_name(&tie_breaker.column)?;
                timestamp.initial = normalize_timestamp(&timestamp.initial)?;
                OutputConfig {
                    timestamp_column: Some(timestamp.column.clone()),
                    validation_columns: vec![tie_breaker.column.clone()],
                }
            }
        };
        let statement = SqlServerStatement::compile(&raw.query.statement, &raw.watermark)?;
        let fingerprint_input = Fingerprint {
            version: 1,
            database_system: "microsoft.sql_server",
            source_id: &raw.source_id,
            host: &connection.host,
            port: connection.port,
            database: &connection.database,
            statement: statement.canonical_sql(),
            watermark: FingerprintWatermark::from(&raw.watermark),
        };
        let bytes = Zeroizing::new(
            serde_json::to_vec(&fingerprint_input)
                .map_err(|_| error("failed to fingerprint SQL Server configuration"))?,
        );
        let fingerprint = blake3::hash(&bytes).to_hex().to_string();
        let query = CompiledQuery::compile(
            statement.canonical_sql().to_owned(),
            polling,
            &raw.watermark,
            &raw.checkpoint,
            output,
        )
        .map_err(|e| SqlServerConfigError(e.to_string()))?;
        Ok(Self {
            source_id: raw.source_id,
            connection,
            query,
            statement,
            checkpoint: raw.checkpoint,
            fingerprint,
        })
    }
}

/// SQL Server endpoint and secure-by-default, SQL Server-specific TLS settings.
///
/// No credential, connection-string, or retry override is accepted.
pub struct SqlServerConnectionConfig {
    host: String,
    port: u16,
    database: String,
    connect_timeout: Duration,
    tls: TlsConfig,
}

impl<'de> Deserialize<'de> for SqlServerConnectionConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawConnectionConfig::deserialize(deserializer).map_err(|_| {
            D::Error::custom(
                "invalid SQL Server connection: check required fields, names, and types",
            )
        })?;
        Self::try_from(raw).map_err(D::Error::custom)
    }
}

impl TryFrom<RawConnectionConfig> for SqlServerConnectionConfig {
    type Error = SqlServerConfigError;

    fn try_from(raw: RawConnectionConfig) -> Result<Self, Self::Error> {
        let mut connection = Self {
            host: raw.host,
            port: raw.port,
            database: raw.database,
            connect_timeout: raw.connect_timeout,
            tls: raw.tls,
        };
        connection.validate()?;
        Ok(connection)
    }
}

impl fmt::Debug for SqlServerConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SqlServerConnectionConfig(<redacted>)")
    }
}

impl SqlServerConnectionConfig {
    /// Returns the connection address and default TLS identity when no override is configured.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the explicit TCP port; no SQL Browser discovery is performed.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the explicitly selected database.
    #[must_use]
    pub fn database(&self) -> &str {
        &self.database
    }

    /// Returns the total budget for one credential/DNS/TCP/TLS/login attempt.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// Returns whether TLS is required, with no fallback to a non-TLS connection.
    ///
    /// False requests a non-TLS connection, which SQL Server policy may refuse.
    #[must_use]
    pub const fn tls_enabled(&self) -> bool {
        self.tls.enabled
    }

    /// Returns whether both certificate-chain and hostname validation are bypassed.
    ///
    /// This explicit opt-in is intended only for controlled testing.
    #[must_use]
    pub const fn trust_server_certificate(&self) -> bool {
        self.tls.trust_server_certificate
    }

    /// Returns an optional TLS identity/SNI override, not a new connection address.
    ///
    /// When certificate trust is bypassed, this can still set SNI but is not verified.
    #[must_use]
    pub fn server_name_override(&self) -> Option<&str> {
        self.tls.server_name_override.as_deref()
    }

    /// Returns an optional CA file that augments verified platform trust.
    #[must_use]
    pub fn ca_file(&self) -> Option<&Path> {
        self.tls.ca_file.as_deref().map(Path::new)
    }

    fn validate(&mut self) -> Result<(), SqlServerConfigError> {
        validate_host(&mut self.host, "connection.host")?;
        if self.port == 0 {
            return Err(error("connection.port must be between 1 and 65535"));
        }
        if self.database.trim().is_empty()
            || self.database.encode_utf16().count() > 128
            || self.database.chars().any(char::is_control)
        {
            return Err(error(
                "connection.database must be a nonempty database name of at most 128 UTF-16 units without control characters",
            ));
        }
        whole_seconds("connection.connect_timeout", self.connect_timeout, 1, 300)?;
        if self
            .tls
            .ca_file
            .as_ref()
            .is_some_and(|path| path.trim().is_empty() || path.contains('\0'))
        {
            return Err(error("connection.tls.ca_file must be a nonempty path"));
        }
        if let Some(name) = &mut self.tls.server_name_override {
            validate_host(name, "connection.tls.server_name_override")?;
        }
        if !self.tls.enabled
            && (self.tls.ca_file.is_some()
                || self.tls.server_name_override.is_some()
                || self.tls.trust_server_certificate)
        {
            return Err(error(
                "connection.tls.enabled=false cannot be combined with ca_file, server_name_override, or trust_server_certificate=true",
            ));
        }
        if self.tls.trust_server_certificate && self.tls.ca_file.is_some() {
            return Err(error(
                "connection.tls.ca_file cannot be combined with trust_server_certificate=true",
            ));
        }
        Ok(())
    }
}

fn validate_host(value: &mut str, field: &'static str) -> Result<(), SqlServerConfigError> {
    if value.parse::<IpAddr>().is_err() {
        if value.len() > 253
            || value.is_empty()
            || value.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || !label.as_bytes()[0].is_ascii_alphanumeric()
                    || !label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                    || !label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return Err(SqlServerConfigError(format!(
                "{field} must be a DNS name or IP address"
            )));
        }
        value.make_ascii_lowercase();
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TlsConfig {
    #[serde(default = "default_tls_enabled")]
    enabled: bool,
    #[serde(default)]
    trust_server_certificate: bool,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    ca_file: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    server_name_override: Option<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: default_tls_enabled(),
            trust_server_certificate: false,
            ca_file: None,
            server_name_override: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    source_id: String,
    connection: RawConnectionConfig,
    query: QueryConfig,
    watermark: WatermarkConfig,
    checkpoint: CheckpointConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConnectionConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    database: String,
    #[serde(default = "default_connect_timeout", with = "humantime_serde")]
    connect_timeout: Duration,
    #[serde(default)]
    tls: TlsConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryConfig {
    statement: String,
    #[serde(default = "default_interval", with = "humantime_serde")]
    interval: Duration,
    #[serde(default = "default_query_timeout", with = "humantime_serde")]
    timeout: Duration,
    #[serde(default, deserialize_with = "deserialize_optional_fetch_size")]
    fetch_size_rows: Option<usize>,
    max_rows_per_poll: usize,
    #[serde(deserialize_with = "deserialize_bytes")]
    max_batch_bytes: u64,
    #[serde(default)]
    catch_up: CatchUpConfig,
}

impl QueryConfig {
    fn polling(&self) -> PollingConfig {
        PollingConfig {
            interval: self.interval,
            timeout: self.timeout,
            fetch_size_rows: self
                .fetch_size_rows
                .unwrap_or_else(|| default_fetch_size().min(self.max_rows_per_poll)),
            max_rows_per_poll: self.max_rows_per_poll,
            max_batch_bytes: self.max_batch_bytes,
            catch_up: self.catch_up,
        }
    }
}

#[derive(Serialize)]
struct Fingerprint<'a> {
    version: u8,
    database_system: &'static str,
    source_id: &'a str,
    host: &'a str,
    port: u16,
    database: &'a str,
    statement: &'a str,
    watermark: FingerprintWatermark<'a>,
}

#[derive(Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
enum FingerprintWatermark<'a> {
    Snapshot,
    Scalar {
        column: &'a str,
        bind: &'a str,
        initial: &'a ScalarValue,
    },
    Composite {
        timestamp_column: &'a str,
        timestamp_bind: &'a str,
        timestamp_initial: &'a str,
        tie_breaker_column: &'a str,
        tie_breaker_bind: &'a str,
        tie_breaker_initial: i64,
    },
}

impl<'a> From<&'a WatermarkConfig> for FingerprintWatermark<'a> {
    fn from(value: &'a WatermarkConfig) -> Self {
        match value {
            WatermarkConfig::Snapshot {} => Self::Snapshot,
            WatermarkConfig::Scalar {
                column,
                bind,
                initial,
            } => Self::Scalar {
                column,
                bind,
                initial,
            },
            WatermarkConfig::Composite {
                timestamp,
                tie_breaker,
            } => Self::Composite {
                timestamp_column: &timestamp.column,
                timestamp_bind: &timestamp.bind,
                timestamp_initial: &timestamp.initial,
                tie_breaker_column: &tie_breaker.column,
                tie_breaker_bind: &tie_breaker.bind,
                tie_breaker_initial: tie_breaker.initial,
            },
        }
    }
}

pub(super) fn normalize_timestamp(value: &str) -> Result<String, SqlServerConfigError> {
    let invalid =
        || error("watermark timestamp must be an exact UTC datetime2 value in years 0001..9999");
    if value.len() > 64
        || value.split_once('.').is_some_and(|(_, fraction)| {
            fraction.bytes().take_while(u8::is_ascii_digit).count() > 9
        })
    {
        return Err(invalid());
    }
    let timestamp = DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
                .map(|value| value.and_utc())
        })
        .map_err(|_| invalid())?;
    if !(1..=9999).contains(&timestamp.year())
        || timestamp.nanosecond() >= 1_000_000_000
        || timestamp.nanosecond() % 100 != 0
    {
        return Err(invalid());
    }
    Ok(format!(
        "{}.{:07}Z",
        timestamp.format("%Y-%m-%dT%H:%M:%S"),
        timestamp.nanosecond() / 100
    ))
}

const fn default_port() -> u16 {
    1433
}
const fn default_tls_enabled() -> bool {
    true
}
const fn default_connect_timeout() -> Duration {
    Duration::from_secs(10)
}
const fn default_interval() -> Duration {
    Duration::from_secs(60)
}
const fn default_query_timeout() -> Duration {
    Duration::from_secs(30)
}
const fn default_fetch_size() -> usize {
    300
}

fn deserialize_bytes<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    otel_arrow_dfe_config::byte_units::deserialize_u64(deserializer)?
        .ok_or_else(|| D::Error::custom("query.max_batch_bytes must not be null"))
}

fn deserialize_optional_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

fn deserialize_optional_fetch_size<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<usize>, D::Error> {
    usize::deserialize(deserializer).map(Some)
}

fn whole_seconds(
    field: &'static str,
    value: Duration,
    min: u64,
    max: u64,
) -> Result<(), SqlServerConfigError> {
    if !(min..=max).contains(&value.as_secs()) || value.subsec_nanos() != 0 {
        return Err(SqlServerConfigError(format!(
            "{field} must be between {min}s and {max}s in whole seconds"
        )));
    }
    Ok(())
}

/// A configuration failure that never contains SQL, endpoint, or cursor values.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct SqlServerConfigError(String);

pub(super) fn error(message: impl Into<String>) -> SqlServerConfigError {
    SqlServerConfigError(message.into())
}
