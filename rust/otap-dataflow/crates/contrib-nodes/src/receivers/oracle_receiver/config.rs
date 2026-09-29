// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle receiver configuration.
//!
//! The customer supplies the complete SQL statement. This module validates that
//! the statement is a single read-only `SELECT`, references both configured
//! named binds as real bind markers, and ends with the required outer
//! `ORDER BY <timestamp> ASC, <tie_breaker> ASC`. String literals, comments,
//! bind-name prefixes, and orderings nested inside parentheses never satisfy
//! that requirement.

use super::adapter::{OracleAdapter, OracleAdapterConfig, parse_cursor_timestamp};
use otel_arrow_dfe_scraper::database::{
    CatchUpConfig, CheckpointConfig, CompiledQuery, OutputConfig, PollingConfig, WatermarkConfig,
};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;

const MAX_SOURCE_ID_BYTES: usize = 256;
const MIN_ORACLE_INTERVAL: Duration = Duration::from_secs(60);
const MAX_ORACLE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MIN_ORACLE_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_ORACLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

const fn default_interval() -> Duration {
    Duration::from_secs(60)
}

const fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

const fn default_fetch_size_rows() -> usize {
    300
}

/// Validated configuration for one Oracle composite-watermark query.
pub struct OracleReceiverConfig {
    source_id: String,
    connection: OracleConnectionConfig,
    authentication: OracleAuthenticationConfig,
    query: CompiledQuery,
    checkpoint: CheckpointConfig,
    config_fingerprint: String,
}

impl<'de> Deserialize<'de> for OracleReceiverConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Structural Serde errors can quote SQL, credentials, and cursor values.
        let raw = RawOracleConfig::deserialize(deserializer).map_err(|_| {
            D::Error::custom(
                "invalid Oracle receiver configuration: check required fields, field names, types, and enum values",
            )
        })?;
        Self::try_from(raw).map_err(D::Error::custom)
    }
}

impl OracleReceiverConfig {
    /// Returns the stable source identifier attached to every emitted row.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Returns the durable checkpoint policy.
    #[must_use]
    pub const fn checkpoint(&self) -> &CheckpointConfig {
        &self.checkpoint
    }

    /// Returns a stable fingerprint of the semantic configuration.
    ///
    /// Credentials and mounted secret paths are excluded, so rotating a secret
    /// does not invalidate an existing durable checkpoint.
    #[must_use]
    pub fn config_fingerprint(&self) -> &str {
        &self.config_fingerprint
    }

    /// Compiles the shared query plan.
    #[must_use]
    pub fn query(&self) -> CompiledQuery {
        self.query.clone()
    }

    /// Builds the Oracle adapter for this configuration.
    #[must_use]
    pub fn adapter(&self) -> OracleAdapter {
        OracleAdapter::new(OracleAdapterConfig {
            connect_string: self.connection.connect_string.clone(),
            instant_client_dir: self.connection.instant_client_dir.clone(),
            username_file: self.authentication.username_file.clone(),
            password_file: self.authentication.password_file.clone(),
        })
    }
}

impl TryFrom<RawOracleConfig> for OracleReceiverConfig {
    type Error = OracleConfigError;

    /// Validates the native schema and compiles the immutable query plan.
    fn try_from(mut config: RawOracleConfig) -> Result<Self, Self::Error> {
        required("source_id", &config.source_id)?;
        if config.source_id.len() > MAX_SOURCE_ID_BYTES {
            return Err(OracleConfigError::new(format!(
                "source_id must not exceed {MAX_SOURCE_ID_BYTES} bytes"
            )));
        }
        required(
            "connection.connect_string",
            &config.connection.connect_string,
        )?;
        required(
            "connection.instant_client_dir",
            &config.connection.instant_client_dir,
        )?;
        required(
            "authentication.username_file",
            &config.authentication.username_file,
        )?;
        required(
            "authentication.password_file",
            &config.authentication.password_file,
        )?;
        required("query.statement", &config.query.statement)?;
        if !(MIN_ORACLE_INTERVAL..=MAX_ORACLE_INTERVAL).contains(&config.query.interval)
            || config.query.interval.subsec_nanos() != 0
        {
            return Err(OracleConfigError::new(
                "query.interval must be between 1 minute and 24 hours in whole seconds",
            ));
        }
        if !(MIN_ORACLE_TIMEOUT..=MAX_ORACLE_TIMEOUT).contains(&config.query.timeout)
            || config.query.timeout.subsec_nanos() != 0
        {
            return Err(OracleConfigError::new(
                "query.timeout must be between 1s and 5m in whole seconds",
            ));
        }
        config.query.polling().validate()?;
        config.watermark.validate()?;
        config.checkpoint.validate()?;
        validate_oracle_identifier(
            "watermark.timestamp.column",
            &config.watermark.timestamp().column,
        )?;
        validate_oracle_identifier(
            "watermark.tie_breaker.column",
            &config.watermark.tie_breaker().column,
        )?;
        let initial = parse_cursor_timestamp(&config.watermark.timestamp().initial).map_err(|_| {
            OracleConfigError::new(
                "watermark.timestamp.initial must be a valid Oracle timestamp with at most nine digits per numeric component",
            )
        })?;
        let WatermarkConfig::Composite { timestamp, .. } = &mut config.watermark;
        timestamp.initial = initial.to_string();
        let statement = validate_statement(&config.query.statement, &config.watermark)?;
        let query = CompiledQuery::compile(
            statement.clone(),
            config.query.polling(),
            &config.watermark,
            &config.checkpoint,
            OutputConfig {
                timestamp_column: Some(config.watermark.timestamp().column.clone()),
                validation_columns: vec![config.watermark.tie_breaker().column.clone()],
            },
        )?;

        let fingerprint = FingerprintInput {
            source_id: &config.source_id,
            connect_string: &config.connection.connect_string,
            statement: &statement,
            timestamp_column: &config.watermark.timestamp().column,
            timestamp_bind: &config.watermark.timestamp().bind,
            timestamp_initial: &config.watermark.timestamp().initial,
            tie_breaker_column: &config.watermark.tie_breaker().column,
            tie_breaker_bind: &config.watermark.tie_breaker().bind,
            tie_breaker_initial: config.watermark.tie_breaker().initial,
        };
        let fingerprint_bytes = serde_json::to_vec(&fingerprint).map_err(|error| {
            OracleConfigError::new(format!(
                "failed to fingerprint Oracle configuration: {error}"
            ))
        })?;
        let config_fingerprint = blake3::hash(&fingerprint_bytes).to_hex().to_string();

        Ok(Self {
            source_id: config.source_id,
            connection: config.connection,
            authentication: config.authentication,
            query,
            checkpoint: config.checkpoint,
            config_fingerprint,
        })
    }
}

/// Semantic fields identifying one checkpoint stream.
///
/// Credential file paths and the Instant Client directory are excluded so
/// rotating a mounted secret cannot invalidate durable state.
#[derive(Serialize)]
struct FingerprintInput<'a> {
    source_id: &'a str,
    connect_string: &'a str,
    statement: &'a str,
    timestamp_column: &'a str,
    timestamp_bind: &'a str,
    timestamp_initial: &'a str,
    tie_breaker_column: &'a str,
    tie_breaker_bind: &'a str,
    tie_breaker_initial: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOracleConfig {
    source_id: String,
    connection: OracleConnectionConfig,
    authentication: OracleAuthenticationConfig,
    query: OracleQueryConfig,
    watermark: WatermarkConfig,
    checkpoint: CheckpointConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleConnectionConfig {
    connect_string: String,
    instant_client_dir: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleAuthenticationConfig {
    username_file: String,
    password_file: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleQueryConfig {
    statement: String,
    #[serde(default = "default_interval", with = "humantime_serde")]
    interval: Duration,
    #[serde(default = "default_fetch_size_rows")]
    fetch_size_rows: usize,
    max_rows_per_poll: usize,
    #[serde(deserialize_with = "deserialize_byte_size")]
    max_batch_bytes: u64,
    #[serde(default = "default_timeout", with = "humantime_serde")]
    timeout: Duration,
    #[serde(default)]
    catch_up: CatchUpConfig,
}

impl OracleQueryConfig {
    /// Converts Oracle query settings into the vendor-neutral polling contract.
    fn polling(&self) -> PollingConfig {
        PollingConfig {
            interval: self.interval,
            timeout: self.timeout,
            fetch_size_rows: self.fetch_size_rows,
            max_rows_per_poll: self.max_rows_per_poll,
            max_batch_bytes: self.max_batch_bytes,
            catch_up: self.catch_up,
        }
    }
}

/// Accepts human-readable byte sizes while storing an exact byte count.
fn deserialize_byte_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    otel_arrow_dfe_config::byte_units::deserialize_u64(deserializer)?
        .ok_or_else(|| DeError::custom("byte size must not be null"))
}

/// Validates the customer-authored statement's binds and final ordering.
fn validate_statement(
    statement: &str,
    watermark: &WatermarkConfig,
) -> Result<String, OracleConfigError> {
    let trimmed = statement.trim();
    let statement = trimmed
        .strip_suffix(';')
        .unwrap_or(trimmed)
        .trim()
        .to_owned();
    if statement.contains(';') || statement.contains("--") || statement.contains("/*") {
        return Err(OracleConfigError::new(
            "query.statement must be one SELECT statement without SQL comments",
        ));
    }
    if !statement
        .split_whitespace()
        .next()
        .is_some_and(|keyword| keyword.eq_ignore_ascii_case("SELECT"))
    {
        return Err(OracleConfigError::new(
            "query.statement must start with SELECT",
        ));
    }

    let upper = statement.to_ascii_uppercase();
    let tokens = sql_tokens(&upper)?;
    if tokens.iter().skip(1).any(|token| {
        matches!(
            token.text.as_str(),
            "SELECT" | "UNION" | "INTERSECT" | "MINUS" | "EXCEPT"
        )
    }) {
        return Err(OracleConfigError::new(
            "query.statement must be a single SELECT without subqueries or set operations",
        ));
    }
    let timestamp = watermark.timestamp();
    let tie_breaker = watermark.tie_breaker();
    // Exact token equality, so ':last_ts_extra' never satisfies ':last_ts' and
    // a bind marker inside a string literal is not a token at all.
    for bind in [&timestamp.bind, &tie_breaker.bind] {
        let marker = format!(":{}", bind.to_ascii_uppercase());
        if !tokens.iter().any(|token| token.text == marker) {
            return Err(OracleConfigError::new(format!(
                "query.statement must reference Oracle bind {marker}"
            )));
        }
    }
    for (column, operator, bind) in [
        (&timestamp.column, ">", &timestamp.bind),
        (&timestamp.column, "=", &timestamp.bind),
        (&tie_breaker.column, ">", &tie_breaker.bind),
    ] {
        if !contains_comparison(&tokens, column, operator, bind) {
            return Err(OracleConfigError::new(format!(
                "query.statement must compare {column} {operator} :{bind}"
            )));
        }
    }

    let timestamp_column = timestamp.column.to_ascii_uppercase();
    let tie_breaker_column = tie_breaker.column.to_ascii_uppercase();
    let expected = [
        "ORDER",
        "BY",
        timestamp_column.as_str(),
        "ASC",
        ",",
        tie_breaker_column.as_str(),
        "ASC",
    ];
    // Only a top-level ORDER BY orders the returned rows, so nested orderings
    // inside a subquery cannot satisfy the paging contract.
    let last_order = tokens.windows(2).rposition(|window| {
        window[0].depth == 0
            && window[1].depth == 0
            && window[0].text == "ORDER"
            && window[1].text == "BY"
    });
    let matches_ordering = last_order.is_some_and(|index| {
        tokens[index..]
            .iter()
            .map(|token| token.text.as_str())
            .eq(expected.iter().copied())
    });
    if !matches_ordering {
        return Err(OracleConfigError::new(format!(
            "query.statement must end with ORDER BY {} ASC, {} ASC",
            timestamp.column, tie_breaker.column
        )));
    }
    let order = last_order.expect("ordering was validated");
    let where_index = tokens
        .iter()
        .position(|token| token.text == "WHERE" && token.depth == 0)
        .ok_or_else(|| {
            OracleConfigError::new("query.statement requires a top-level WHERE predicate")
        })?;
    if where_index >= order {
        return Err(OracleConfigError::new(
            "query.statement requires a top-level WHERE predicate before ORDER BY",
        ));
    }
    let predicate = tokens[where_index + 1..order]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<Vec<_>>()
        .join("");
    let expected_predicate = format!(
        "{}>:{}OR({}=:{}AND{}>:{})",
        timestamp.column.to_ascii_uppercase(),
        timestamp.bind.to_ascii_uppercase(),
        timestamp.column.to_ascii_uppercase(),
        timestamp.bind.to_ascii_uppercase(),
        tie_breaker.column.to_ascii_uppercase(),
        tie_breaker.bind.to_ascii_uppercase(),
    );
    if predicate != expected_predicate && predicate != format!("({expected_predicate})") {
        return Err(OracleConfigError::new(
            "query.statement requires exactly timestamp > :timestamp OR (timestamp = :timestamp AND tie_breaker > :tie_breaker)",
        ));
    }

    /// Finds a required cursor comparison at any parenthesis depth.
    fn contains_comparison(tokens: &[SqlToken], column: &str, operator: &str, bind: &str) -> bool {
        let expected_column = column.to_ascii_uppercase();
        let expected_bind = format!(":{}", bind.to_ascii_uppercase());
        tokens.windows(3).any(|window| {
            window[0].text == expected_column
                && window[1].text == operator
                && window[2].text == expected_bind
        })
    }
    Ok(statement)
}

struct SqlToken {
    text: String,
    depth: usize,
}

/// Splits uppercased SQL into identifier tokens, discarding string literals.
fn sql_tokens(sql: &str) -> Result<Vec<SqlToken>, OracleConfigError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut characters = sql.chars().peekable();
    while let Some(character) = characters.next() {
        if in_string {
            if character == '\'' {
                if characters.peek() == Some(&'\'') {
                    _ = characters.next();
                } else {
                    in_string = false;
                }
            }
            continue;
        }
        if character == '\'' {
            push_token(&mut tokens, &mut current, depth);
            tokens.push(SqlToken {
                text: "'literal'".to_owned(),
                depth,
            });
            in_string = true;
            continue;
        }
        if character.is_ascii_alphanumeric() || matches!(character, '_' | '$' | '#' | ':') {
            current.push(character);
            continue;
        }
        push_token(&mut tokens, &mut current, depth);
        match character {
            '(' => {
                tokens.push(SqlToken {
                    text: "(".to_owned(),
                    depth,
                });
                depth = depth.saturating_add(1);
            }
            ')' => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    OracleConfigError::new("query.statement contains unbalanced parentheses")
                })?;
                tokens.push(SqlToken {
                    text: ")".to_owned(),
                    depth,
                });
            }
            ',' => tokens.push(SqlToken {
                text: ",".to_owned(),
                depth,
            }),
            '>' | '=' => tokens.push(SqlToken {
                text: character.to_string(),
                depth,
            }),
            value if value.is_whitespace() => {}
            value => tokens.push(SqlToken {
                text: value.to_string(),
                depth,
            }),
        }
    }

    if in_string {
        return Err(OracleConfigError::new(
            "query.statement contains an unterminated string literal",
        ));
    }
    if depth != 0 {
        return Err(OracleConfigError::new(
            "query.statement contains unbalanced parentheses",
        ));
    }
    push_token(&mut tokens, &mut current, depth);
    Ok(tokens)
}

/// Restricts cursor columns and binds to simple Oracle identifiers.
fn validate_oracle_identifier(
    field: &'static str,
    identifier: &str,
) -> Result<(), OracleConfigError> {
    let mut bytes = identifier.bytes();
    let first = bytes.next().unwrap_or(b'0');
    if !first.is_ascii_alphabetic()
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'#'))
    {
        return Err(OracleConfigError::new(format!(
            "{field} must be an unquoted Oracle identifier"
        )));
    }
    Ok(())
}

/// Finishes the current SQL token while retaining its parenthesis depth.
fn push_token(tokens: &mut Vec<SqlToken>, current: &mut String, depth: usize) {
    if !current.is_empty() {
        tokens.push(SqlToken {
            text: std::mem::take(current),
            depth,
        });
    }
}

/// Produces a field-specific error for an empty required string.
fn required(field: &'static str, value: &str) -> Result<(), OracleConfigError> {
    if value.trim().is_empty() {
        Err(OracleConfigError::new(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

/// Invalid Oracle receiver configuration.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct OracleConfigError {
    message: String,
}

impl OracleConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<otel_arrow_dfe_scraper::database::ConfigError> for OracleConfigError {
    fn from(error: otel_arrow_dfe_scraper::database::ConfigError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<otel_arrow_dfe_scraper::database::QueryError> for OracleConfigError {
    fn from(error: otel_arrow_dfe_scraper::database::QueryError) -> Self {
        Self::new(error.to_string())
    }
}
