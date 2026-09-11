// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic matching and the dynamic topic registry.
//!
//! Compiles per-signal topic and exclude patterns (literal names or
//! `^`-prefixed regex), matches actual Kafka topic names against them, detects
//! the per-message [`MessageFormat`] from a header, and assigns compact `u32`
//! IDs to topic names for CallData routing.

use crate::common::kafka::{MSG_FORMAT_OTAP, MSG_FORMAT_OTLP, MSG_FORMAT_SYSLOG, MessageFormat};
use otel_arrow_dfe_config::error::Error as ConfigError;
use rdkafka::Message as _;
use rdkafka::message::{BorrowedMessage, Headers};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

/// Compile a slice of topic config strings into a parallel [`Vec`] of
/// optional [`Regex`] values. Entries starting with `^` are treated as
/// regex patterns; literal topic names yield `None`.
///
/// Returns an error if any regex pattern is invalid.
pub(super) fn compile_topic_regexes(topics: &[String]) -> Result<Vec<Option<Regex>>, ConfigError> {
    topics
        .iter()
        .map(|t| {
            if t.starts_with('^') {
                Regex::new(t)
                    .map(Some)
                    .map_err(|e| ConfigError::InvalidUserConfig {
                        error: format!("Invalid regex topic pattern '{t}': {e}"),
                    })
            } else {
                Ok(None)
            }
        })
        .collect()
}

/// Check whether an actual topic name matches any configured topic in the
/// given list. Each entry is checked against its parallel regex (if the
/// topic was a pattern), or via exact string equality.
pub(super) fn matches_any_topic(
    config_topics: &[String],
    regexes: &[Option<Regex>],
    actual: &str,
) -> bool {
    config_topics
        .iter()
        .zip(regexes.iter())
        .any(|(topic, regex)| match regex {
            Some(r) => r.is_match(actual),
            None => topic == actual,
        })
}

/// Compile exclude topic patterns into [`Regex`] values.
/// All entries are treated as regex patterns (they must be valid regex per
/// validation). Returns an error if any pattern is invalid.
pub(super) fn compile_exclude_regexes(
    exclude_topics: &[String],
) -> Result<Vec<Regex>, ConfigError> {
    exclude_topics
        .iter()
        .map(|t| {
            Regex::new(t).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Invalid exclude_topics regex pattern '{t}': {e}"),
            })
        })
        .collect()
}

/// Check whether an actual topic name matches any exclude pattern.
pub(super) fn matches_any_exclude(exclude_regexes: &[Regex], actual: &str) -> bool {
    exclude_regexes.iter().any(|r| r.is_match(actual))
}

/// Detect the message format from Kafka headers, falling back to the
/// configured default when the header is absent or unrecognized.
pub(super) fn detect_message_format(
    kafka_message: &BorrowedMessage<'_>,
    header_key: &str,
    default: MessageFormat,
) -> MessageFormat {
    match kafka_message
        .headers()
        .and_then(|hs| hs.iter().find(|h| h.key == header_key))
        .and_then(|h| h.value)
    {
        value if value == Some(MSG_FORMAT_OTLP) => MessageFormat::OtlpProto,
        value if value == Some(MSG_FORMAT_OTAP) => MessageFormat::OtapProto,
        value if value == Some(MSG_FORMAT_SYSLOG) => MessageFormat::Syslog,
        _ => default,
    }
}

/// Dynamically assigns compact `u32` IDs to actual Kafka topic names.
///
/// Used to encode topic identity into [`CallData`] for Ack/Nack routing
/// while supporting regex-matched topic names that aren't known at config
/// time.
pub(super) struct TopicRegistry {
    name_to_id: HashMap<Arc<str>, u32>,
    id_to_name: Vec<Arc<str>>,
}

impl TopicRegistry {
    pub(super) fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }

    /// Get or assign a `u32` ID for the given topic name.
    pub(super) fn get_or_assign(&mut self, topic: &str) -> Option<u32> {
        // if topic hasn't been seen yet then we assign topic a id
        if let Some(&id) = self.name_to_id.get(topic) {
            return Some(id);
        }
        // The next ID is the current count. Refuse if it doesn't fit in `u32`.
        let id = u32::try_from(self.id_to_name.len()).ok()?;
        let name: Arc<str> = Arc::from(topic);
        self.id_to_name.push(Arc::clone(&name));
        let _ = self.name_to_id.insert(name, id);
        Some(id)
    }

    /// Look up a topic name by its assigned ID.
    ///
    /// Returns a cheap `Arc<str>` clone so callers can hold an owned handle to
    /// the topic name without borrowing the registry -- avoiding a borrow
    /// conflict when the same call site also needs `&mut self` (e.g. to mutate
    /// the offset tracker), and without allocating a fresh `String` per ack.
    pub(super) fn name_for(&self, id: u32) -> Option<Arc<str>> {
        self.id_to_name.get(id as usize).map(Arc::clone)
    }
}
