// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch validation utilities.
//!
//! Validates that each message in a collection meets configured batch size bounds.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Ensure every message size is within `[min_items, max_items]` (if provided).
pub(crate) fn validate_batch_items(
    message: &OtlpProtoMessage,
    min_items: Option<usize>,
    max_items: Option<usize>,
) -> bool {
    let batch_size = message.num_items();
    if let Some(min) = min_items {
        if batch_size < min {
            return false;
        }
    }
    if let Some(max) = max_items {
        if batch_size > max {
            return false;
        }
    }
    true
}

/// Ensure every message encoded size in bytes is within bounds.
pub(crate) fn validate_batch_bytes(
    message: &OtlpProtoMessage,
    min_bytes: Option<usize>,
    max_bytes: Option<usize>,
) -> bool {
    let mut buf = Vec::new();
    let _ = message.encode(&mut buf);
    let byte_size = buf.len();
    if let Some(min) = min_bytes {
        if byte_size < min {
            return false;
        }
    }
    if let Some(max) = max_bytes {
        if byte_size > max {
            return false;
        }
    }
    true
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
        let msg = logs_with_records(3);
        assert!(validate_batch_items(&msg, Some(1), Some(3)));
        assert!(!validate_batch_items(&msg, Some(4), None));
        assert!(!validate_batch_items(&msg, None, Some(2)));
    }
}
