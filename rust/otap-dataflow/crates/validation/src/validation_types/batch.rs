// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch validation utilities.
//!
//! Validates that each message in a collection meets configured batch size bounds.

use otap_df_pdata::proto::OtlpProtoMessage;
use std::time::Duration;

/// Ensure every message size is within `[min_items, max_items]` (if provided).
/// Messages that arrived after the timeout are exempt from the check.
pub(crate) fn validate_batch_items(
    messages: &[(OtlpProtoMessage, Duration)],
    min_items: &Option<usize>,
    max_items: &Option<usize>,
    timeout: &Option<Duration>,
) -> bool {
    messages.iter().all(|(message, elapsed)| {
        if let Some(t) = timeout {
            if elapsed >= t {
                return true;
            }
        }
        let batch_size = message.num_items();
        if let Some(min) = min_items {
            if &batch_size < min {
                return false;
            }
        }
        if let Some(max) = max_items {
            if &batch_size > max {
                return false;
            }
        }
        true
    })
}

/// Ensure every message encoded size in bytes is within bounds.
/// Messages that arrived after the timeout are exempt from the check.
pub(crate) fn validate_batch_bytes(
    messages: &[(OtlpProtoMessage, Duration)],
    min_bytes: &Option<usize>,
    max_bytes: &Option<usize>,
    timeout: &Option<Duration>,
) -> bool {
    messages.iter().all(|(message, elapsed)| {
        if let Some(t) = timeout {
            if elapsed >= t {
                return true;
            }
        }
        let mut buf = Vec::new();
        let _ = message.encode(&mut buf);
        let byte_size = buf.len();
        if let Some(min) = min_bytes {
            if &byte_size < min {
                return false;
            }
        }
        if let Some(max) = max_bytes {
            if &byte_size > max {
                return false;
            }
        }
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };

    fn logs_with_records(count: usize) -> OtlpProtoMessage {
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord::default(); count],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        OtlpProtoMessage::Logs(logs)
    }

    #[test]
    fn single_message_items_bounds() {
        let msgs = vec![(logs_with_records(3), Duration::from_secs(0))];
        assert!(validate_batch_items(&msgs, &Some(1), &Some(3), &None));
        assert!(!validate_batch_items(&msgs, &Some(4), &None, &None));
        assert!(!validate_batch_items(&msgs, &None, &Some(2), &None));
    }

    #[test]
    fn multiple_messages_all_must_pass() {
        let msgs = vec![
            (logs_with_records(3), Duration::from_secs(0)),
            (logs_with_records(5), Duration::from_secs(0)),
        ];
        assert!(validate_batch_items(&msgs, &Some(2), &Some(5), &None));
        assert!(!validate_batch_items(&msgs,& Some(4), &None, &None));
    }

    #[test]
    fn timeout_exempts_messages() {
        let timeout = Some(Duration::from_secs(5));
        let msgs = vec![
            (logs_with_records(3), Duration::from_secs(0)),
            (logs_with_records(1), Duration::from_secs(6)), // past timeout, exempt
        ];
        // Without timeout, min=2 would fail for the second message (1 item).
        // With timeout, the second message is exempt.
        assert!(validate_batch_items(&msgs, &Some(2), &None, &timeout));
    }
}
