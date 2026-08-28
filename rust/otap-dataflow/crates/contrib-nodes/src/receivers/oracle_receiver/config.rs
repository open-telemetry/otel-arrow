// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration and validation for the Oracle polling receiver.

use crate::receivers::sql_polling::CompoundWatermark;
use oracle::sql_type::Timestamp;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Component, PathBuf};
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_PASSWORD_ENV: &str = "ORACLE_PWD";
const DEFAULT_MAX_ROWS: usize = 100;
const MAX_ROWS_LIMIT: usize = 1_000;
const DEFAULT_MAX_BATCH_BYTES: u64 = 1024 * 1024;
const MAX_BATCH_BYTES_LIMIT: u64 = 64 * 1024 * 1024;
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);
const MAX_POLL_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CALL_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const DEFAULT_NACK_BACKOFF: Duration = Duration::from_secs(1);
const MAX_NACK_BACKOFF: Duration = Duration::from_secs(60 * 60);
const DEFAULT_CHECKPOINT_DIR: &str = "${engine.state_dir}/oracle";
const DEFAULT_MAX_CONSECUTIVE_FAILURES: u32 = 5;
const MAX_CONSECUTIVE_FAILURES_LIMIT: u32 = 100;
const MAX_QUERY_BYTES: usize = 64 * 1024;

fn default_password_env() -> String {
    DEFAULT_PASSWORD_ENV.to_owned()
}

const fn default_max_rows() -> usize {
    DEFAULT_MAX_ROWS
}

const fn default_max_batch_bytes() -> u64 {
    DEFAULT_MAX_BATCH_BYTES
}

const fn default_poll_interval() -> Duration {
    DEFAULT_POLL_INTERVAL
}

const fn default_call_timeout() -> Duration {
    DEFAULT_CALL_TIMEOUT
}

const fn default_nack_backoff() -> Duration {
    DEFAULT_NACK_BACKOFF
}

fn default_checkpoint_directory() -> PathBuf {
    PathBuf::from(DEFAULT_CHECKPOINT_DIR)
}

const fn default_max_consecutive_failures() -> u32 {
    DEFAULT_MAX_CONSECUTIVE_FAILURES
}

/// Timestamp half of the ascending compound watermark.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TimestampWatermarkConfig {
    /// Selected Oracle timestamp column.
    pub(super) column: String,
    /// Named Oracle bind without the leading colon.
    pub(super) bind: String,
    /// Explicit initial Oracle timestamp.
    pub(super) initial: String,
}

/// Integer half of the ascending compound watermark.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TieBreakerWatermarkConfig {
    /// Selected signed 64-bit integer column.
    pub(super) column: String,
    /// Named Oracle bind without the leading colon.
    pub(super) bind: String,
    /// Explicit initial signed 64-bit integer.
    pub(super) initial: i64,
}

/// Compound watermark configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WatermarkConfig {
    /// Timestamp component.
    pub(super) timestamp: TimestampWatermarkConfig,
    /// Unique integer component for timestamp collisions.
    pub(super) tie_breaker: TieBreakerWatermarkConfig,
}

/// Durable checkpoint configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckpointConfig {
    /// Root directory. A stable pipeline/source suffix is appended.
    #[serde(default = "default_checkpoint_directory")]
    pub(super) directory: PathBuf,
    /// Consecutive durable write failures allowed before the receiver fails.
    #[serde(default = "default_max_consecutive_failures")]
    pub(super) max_consecutive_failures: u32,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            directory: default_checkpoint_directory(),
            max_consecutive_failures: default_max_consecutive_failures(),
        }
    }
}

/// User-facing Oracle receiver configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub(super) source_id: String,
    pub(super) connect_string: String,
    pub(super) username: String,
    #[serde(default = "default_password_env")]
    pub(super) password_env: String,
    pub(super) query: String,
    pub(super) watermark: WatermarkConfig,
    #[serde(default)]
    pub(super) checkpoint: CheckpointConfig,
    #[serde(default = "default_nack_backoff", with = "humantime_serde")]
    pub(super) nack_backoff: Duration,
    #[serde(default = "default_poll_interval", with = "humantime_serde")]
    pub(super) poll_interval: Duration,
    #[serde(default = "default_call_timeout", with = "humantime_serde")]
    pub(super) call_timeout: Duration,
    #[serde(default = "default_max_rows")]
    pub(super) max_rows: usize,
    #[serde(
        default = "default_max_batch_bytes",
        deserialize_with = "deserialize_byte_size"
    )]
    pub(super) max_batch_bytes: u64,
}

/// Validated runtime configuration.
#[derive(Clone, Debug)]
pub(super) struct RuntimeConfig {
    pub(super) source_id: String,
    pub(super) connect_string: String,
    pub(super) username: String,
    pub(super) password_env: String,
    pub(super) query: String,
    pub(super) watermark: WatermarkConfig,
    pub(super) initial_watermark: CompoundWatermark,
    pub(super) checkpoint: CheckpointConfig,
    pub(super) nack_backoff: Duration,
    pub(super) poll_interval: Duration,
    pub(super) call_timeout: Duration,
    pub(super) max_rows: usize,
    pub(super) max_batch_bytes: u64,
    pub(super) config_fingerprint: String,
}

impl TryFrom<Config> for RuntimeConfig {
    type Error = String;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let source_id = validate_source_id(config.source_id)?;
        let connect_string = required_text("connect_string", config.connect_string)?;
        let username = required_text("username", config.username)?;
        let password_env = validate_env_name(config.password_env)?;
        let query = validate_query(config.query, &config.watermark)?;
        let watermark = validate_watermark(config.watermark)?;
        let initial_timestamp = Timestamp::from_str(&watermark.timestamp.initial)
            .map_err(|error| format!("watermark.timestamp.initial is invalid: {error}"))?
            .to_string();
        let initial_watermark = CompoundWatermark {
            timestamp: initial_timestamp,
            tie_breaker: watermark.tie_breaker.initial,
        };
        validate_checkpoint(&config.checkpoint)?;
        validate_duration("poll_interval", config.poll_interval, MAX_POLL_INTERVAL)?;
        validate_duration("call_timeout", config.call_timeout, MAX_CALL_TIMEOUT)?;
        validate_duration("nack_backoff", config.nack_backoff, MAX_NACK_BACKOFF)?;
        if !(1..=MAX_ROWS_LIMIT).contains(&config.max_rows) {
            return Err(format!("max_rows must be between 1 and {MAX_ROWS_LIMIT}"));
        }
        if !(1..=MAX_BATCH_BYTES_LIMIT).contains(&config.max_batch_bytes) {
            return Err(format!(
                "max_batch_bytes must be between 1 and {MAX_BATCH_BYTES_LIMIT}"
            ));
        }

        let fingerprint = FingerprintInput {
            source_id: &source_id,
            connect_string: &connect_string,
            username: &username,
            query: &query,
            watermark: &watermark,
        };
        let fingerprint_bytes = serde_json::to_vec(&fingerprint)
            .map_err(|error| format!("failed to fingerprint Oracle configuration: {error}"))?;
        let config_fingerprint = blake3::hash(&fingerprint_bytes).to_hex().to_string();

        Ok(Self {
            source_id,
            connect_string,
            username,
            password_env,
            query,
            watermark,
            initial_watermark,
            checkpoint: config.checkpoint,
            nack_backoff: config.nack_backoff,
            poll_interval: config.poll_interval,
            call_timeout: config.call_timeout,
            max_rows: config.max_rows,
            max_batch_bytes: config.max_batch_bytes,
            config_fingerprint,
        })
    }
}

#[derive(Serialize)]
struct FingerprintInput<'a> {
    source_id: &'a str,
    connect_string: &'a str,
    username: &'a str,
    query: &'a str,
    watermark: &'a WatermarkConfig,
}

fn deserialize_byte_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    otap_df_config::byte_units::deserialize_u64(deserializer)?
        .ok_or_else(|| DeError::custom("byte size must not be null"))
}

fn required_text(name: &str, value: String) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{name} must not be empty"))
    } else {
        Ok(value.to_owned())
    }
}

fn validate_source_id(source_id: String) -> Result<String, String> {
    let source_id = required_text("source_id", source_id)?;
    if !source_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err("source_id must contain only ASCII alphanumerics, '_', '-', or '.'".to_owned());
    }
    Ok(source_id)
}

fn validate_env_name(value: String) -> Result<String, String> {
    let value = required_text("password_env", value)?;
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return Err("password_env must not be empty".to_owned());
    };
    if !(first.is_ascii_alphabetic() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("password_env must be a valid environment variable name".to_owned());
    }
    Ok(value)
}

fn validate_watermark(mut watermark: WatermarkConfig) -> Result<WatermarkConfig, String> {
    watermark.timestamp.column =
        validate_oracle_identifier("watermark.timestamp.column", watermark.timestamp.column)?;
    watermark.timestamp.bind =
        validate_bind_name("watermark.timestamp.bind", watermark.timestamp.bind)?;
    watermark.timestamp.initial =
        required_text("watermark.timestamp.initial", watermark.timestamp.initial)?;
    watermark.tie_breaker.column =
        validate_oracle_identifier("watermark.tie_breaker.column", watermark.tie_breaker.column)?;
    watermark.tie_breaker.bind =
        validate_bind_name("watermark.tie_breaker.bind", watermark.tie_breaker.bind)?;

    if watermark
        .timestamp
        .column
        .eq_ignore_ascii_case(&watermark.tie_breaker.column)
    {
        return Err("watermark columns must be distinct".to_owned());
    }
    if watermark
        .timestamp
        .bind
        .eq_ignore_ascii_case(&watermark.tie_breaker.bind)
    {
        return Err("watermark bind names must be distinct".to_owned());
    }
    Ok(watermark)
}

fn validate_oracle_identifier(name: &str, value: String) -> Result<String, String> {
    let value = required_text(name, value)?;
    let mut bytes = value.bytes();
    let first = bytes.next().expect("required text checked above");
    if !first.is_ascii_alphabetic()
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'#'))
    {
        return Err(format!(
            "{name} must be an unquoted Oracle identifier beginning with an ASCII letter"
        ));
    }
    Ok(value)
}

fn validate_bind_name(name: &str, value: String) -> Result<String, String> {
    let value = required_text(name, value)?;
    let mut bytes = value.bytes();
    let first = bytes.next().expect("required text checked above");
    if !(first.is_ascii_alphabetic() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(format!(
            "{name} must omit ':' and contain only ASCII alphanumerics or '_'"
        ));
    }
    Ok(value)
}

fn validate_query(query: String, watermark: &WatermarkConfig) -> Result<String, String> {
    let query = required_text("query", query)?;
    if query.len() > MAX_QUERY_BYTES {
        return Err(format!("query must be at most {MAX_QUERY_BYTES} bytes"));
    }
    let query = query.strip_suffix(';').unwrap_or(&query).trim().to_owned();
    if query.contains(';') || query.contains("--") || query.contains("/*") {
        return Err("query must be one SELECT/WITH statement without SQL comments".to_owned());
    }
    let first = query.split_whitespace().next().unwrap_or_default();
    if !first.eq_ignore_ascii_case("SELECT") && !first.eq_ignore_ascii_case("WITH") {
        return Err("query must start with SELECT or WITH".to_owned());
    }

    let upper = query.to_ascii_uppercase();
    let tokens = sql_tokens(&upper)?;
    for bind in [&watermark.timestamp.bind, &watermark.tie_breaker.bind] {
        let marker = format!(":{}", bind.to_ascii_uppercase());
        if !tokens.iter().any(|token| token.text == marker) {
            return Err(format!("query must reference Oracle bind {marker}"));
        }
    }

    let timestamp_column = watermark.timestamp.column.to_ascii_uppercase();
    let tie_breaker_column = watermark.tie_breaker.column.to_ascii_uppercase();
    let expected = [
        "ORDER",
        "BY",
        timestamp_column.as_str(),
        "ASC",
        ",",
        tie_breaker_column.as_str(),
        "ASC",
    ];
    let last_order = tokens.windows(2).rposition(|window| {
        window[0].depth == 0
            && window[1].depth == 0
            && window[0].text == "ORDER"
            && window[1].text == "BY"
    });
    let matching_order = last_order.is_some_and(|index| {
        tokens[index..]
            .iter()
            .take(expected.len())
            .filter(|token| token.depth == 0)
            .map(|token| token.text.as_str())
            .eq(expected.iter().copied())
    });
    if !matching_order {
        return Err(format!(
            "query's final ORDER BY must be {} ASC, {} ASC",
            watermark.timestamp.column, watermark.tie_breaker.column
        ));
    }
    Ok(query)
}

struct SqlToken {
    text: String,
    depth: usize,
}

fn sql_tokens(sql: &str) -> Result<Vec<SqlToken>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut characters = sql.chars().peekable();
    while let Some(ch) = characters.next() {
        if in_string {
            if ch == '\'' {
                if characters.peek() == Some(&'\'') {
                    let _ = characters.next();
                } else {
                    in_string = false;
                }
            }
            continue;
        }
        if ch == '\'' {
            push_sql_token(&mut tokens, &mut current, depth);
            in_string = true;
            continue;
        }
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '#' | ':') {
            current.push(ch);
        } else {
            push_sql_token(&mut tokens, &mut current, depth);
            match ch {
                '(' => depth = depth.saturating_add(1),
                ')' => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or_else(|| "query contains unbalanced parentheses".to_owned())?;
                }
                ',' => tokens.push(SqlToken {
                    text: ",".to_owned(),
                    depth,
                }),
                _ => {}
            }
        }
    }
    if in_string {
        return Err("query contains an unterminated string literal".to_owned());
    }
    if depth != 0 {
        return Err("query contains unbalanced parentheses".to_owned());
    }
    push_sql_token(&mut tokens, &mut current, depth);
    Ok(tokens)
}

fn push_sql_token(tokens: &mut Vec<SqlToken>, current: &mut String, depth: usize) {
    if !current.is_empty() {
        tokens.push(SqlToken {
            text: std::mem::take(current),
            depth,
        });
    }
}

fn validate_checkpoint(checkpoint: &CheckpointConfig) -> Result<(), String> {
    if checkpoint.directory.as_os_str().is_empty() {
        return Err("checkpoint.directory must not be empty".to_owned());
    }
    if checkpoint
        .directory
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("checkpoint.directory must not contain '..'".to_owned());
    }
    if !(1..=MAX_CONSECUTIVE_FAILURES_LIMIT).contains(&checkpoint.max_consecutive_failures) {
        return Err(format!(
            "checkpoint.max_consecutive_failures must be between 1 and {MAX_CONSECUTIVE_FAILURES_LIMIT}"
        ));
    }
    Ok(())
}

fn validate_duration(name: &str, value: Duration, maximum: Duration) -> Result<(), String> {
    if value.is_zero() {
        return Err(format!("{name} must be greater than zero"));
    }
    if value > maximum {
        return Err(format!("{name} must be <= {}s", maximum.as_secs()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_config() -> serde_json::Value {
        json!({
            "source_id": "orders",
            "connect_string": "//localhost:1521/FREEPDB1",
            "username": "PDBADMIN",
            "query": "SELECT EVENT_TS, EVENT_ID, PAYLOAD FROM EVENTS WHERE (EVENT_TS > :last_ts OR (EVENT_TS = :last_ts AND EVENT_ID > :last_id)) ORDER BY EVENT_TS ASC, EVENT_ID ASC",
            "watermark": {
                "timestamp": {
                    "column": "EVENT_TS",
                    "bind": "last_ts",
                    "initial": "2026-01-01 00:00:00.000000000"
                },
                "tie_breaker": {
                    "column": "EVENT_ID",
                    "bind": "last_id",
                    "initial": 0
                }
            }
        })
    }

    /// Scenario: a complete Oracle watermark configuration uses safe defaults.
    /// Guarantees: the runtime has explicit initial state and bounded polling values.
    #[test]
    fn valid_config_builds_runtime_defaults() {
        let config: Config = serde_json::from_value(valid_config()).expect("config");
        let runtime = RuntimeConfig::try_from(config).expect("runtime");

        assert_eq!(runtime.initial_watermark.tie_breaker, 0);
        assert_eq!(runtime.max_rows, DEFAULT_MAX_ROWS);
        assert_eq!(runtime.max_batch_bytes, DEFAULT_MAX_BATCH_BYTES);
        assert_eq!(runtime.poll_interval, DEFAULT_POLL_INTERVAL);
        assert_eq!(runtime.nack_backoff, DEFAULT_NACK_BACKOFF);
    }

    /// Scenario: a query omits a bind or reverses the configured composite order.
    /// Guarantees: unsafe paging SQL is rejected before opening Oracle.
    #[test]
    fn query_requires_binds_and_matching_ascending_order() {
        let mut missing_bind = valid_config();
        missing_bind["query"] = json!(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS WHERE EVENT_TS > :last_ts ORDER BY EVENT_TS ASC, EVENT_ID ASC"
        );
        assert!(serde_json::from_value::<Config>(missing_bind)
            .and_then(|config| RuntimeConfig::try_from(config).map_err(serde::de::Error::custom))
            .is_err());

        let mut descending = valid_config();
        descending["query"] = json!(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS WHERE EVENT_TS > :last_ts AND EVENT_ID > :last_id ORDER BY EVENT_TS DESC, EVENT_ID ASC"
        );
        let config: Config = serde_json::from_value(descending).expect("shape");
        assert!(RuntimeConfig::try_from(config).is_err());

        let mut bind_prefix = valid_config();
        bind_prefix["query"] = json!(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS WHERE EVENT_TS > :last_ts_extra AND EVENT_ID > :last_id ORDER BY EVENT_TS ASC, EVENT_ID ASC"
        );
        let config: Config = serde_json::from_value(bind_prefix).expect("shape");
        assert!(RuntimeConfig::try_from(config).is_err());

        let mut nested_order = valid_config();
        nested_order["query"] = json!(
            "SELECT EVENT_TS, EVENT_ID FROM (SELECT EVENT_TS, EVENT_ID FROM EVENTS WHERE EVENT_TS > :last_ts OR (EVENT_TS = :last_ts AND EVENT_ID > :last_id) ORDER BY EVENT_TS ASC, EVENT_ID ASC) ORDER BY EVENT_ID DESC"
        );
        let config: Config = serde_json::from_value(nested_order).expect("shape");
        assert!(RuntimeConfig::try_from(config).is_err());

        let mut nested_only_order = valid_config();
        nested_only_order["query"] = json!(
            "SELECT EVENT_TS, EVENT_ID FROM (SELECT EVENT_TS, EVENT_ID FROM EVENTS WHERE EVENT_TS > :last_ts OR (EVENT_TS = :last_ts AND EVENT_ID > :last_id) ORDER BY EVENT_TS ASC, EVENT_ID ASC)"
        );
        let config: Config = serde_json::from_value(nested_only_order).expect("shape");
        assert!(RuntimeConfig::try_from(config).is_err());

        let mut bind_literal = valid_config();
        bind_literal["query"] = json!(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS WHERE ':last_ts' = ':last_ts' AND EVENT_ID > :last_id ORDER BY EVENT_TS ASC, EVENT_ID ASC"
        );
        let config: Config = serde_json::from_value(bind_literal).expect("shape");
        assert!(RuntimeConfig::try_from(config).is_err());
    }

    /// Scenario: a watermark timestamp cannot be represented by the Oracle driver.
    /// Guarantees: invalid initial state fails closed during configuration validation.
    #[test]
    fn invalid_initial_timestamp_fails_closed() {
        let mut value = valid_config();
        value["watermark"]["timestamp"]["initial"] = json!("not-a-timestamp");
        let config: Config = serde_json::from_value(value).expect("shape");
        assert!(RuntimeConfig::try_from(config).is_err());
    }

    /// Scenario: limits, identifiers, and unknown fields violate the receiver contract.
    /// Guarantees: strict validation rejects unbounded or ambiguous configuration.
    #[test]
    fn strict_validation_rejects_invalid_fields_and_limits() {
        for (field, value) in [
            ("max_rows", json!(0)),
            ("max_rows", json!(MAX_ROWS_LIMIT + 1)),
            ("max_batch_bytes", json!(0)),
            ("nack_backoff", json!("0s")),
        ] {
            let mut config = valid_config();
            config[field] = value;
            let config: Config = serde_json::from_value(config).expect("shape");
            assert!(RuntimeConfig::try_from(config).is_err(), "{field}");
        }

        let mut unknown = valid_config();
        unknown["unknown"] = json!(true);
        assert!(serde_json::from_value::<Config>(unknown).is_err());
    }
}
