// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch validation utilities.
//!
//! Validates that each message in a collection meets configured batch size bounds.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Ensure every message size is within `[min_items, max_items]` (if provided).
/// Returns false if the slice is empty or any message violates the bounds.
pub fn check_batch_size(
    messages: &[OtlpProtoMessage],
    min_items: Option<usize>,
    max_items: Option<usize>,
) -> bool {
    if messages.is_empty() {
        return false;
    }

    messages.iter().all(|msg| {
        let batch_size = msg.num_items();
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
    fn empty_slice_fails() {
        assert!(!check_batch_size(&[], Some(1), None));
    }

    #[test]
    fn min_only_passes_within_bounds() {
        let msgs = [logs_with_records(3), logs_with_records(2)];
        assert!(check_batch_size(&msgs, Some(2), None));
        assert!(!check_batch_size(&msgs, Some(4), None));
    }

    #[test]
    fn max_only_passes_within_bounds() {
        let msgs = [logs_with_records(3), logs_with_records(2)];
        assert!(check_batch_size(&msgs, None, Some(3)));
        assert!(!check_batch_size(&msgs, None, Some(2)));
    }

    #[test]
    fn min_and_max_bounds() {
        let msgs = [logs_with_records(3), logs_with_records(2)];
        assert!(check_batch_size(&msgs, Some(2), Some(3)));
        assert!(!check_batch_size(&msgs, Some(2), Some(2)));
    }
}
