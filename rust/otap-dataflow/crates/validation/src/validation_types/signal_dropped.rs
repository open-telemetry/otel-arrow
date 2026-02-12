// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Signal drop validation.
//!
//! Compare two slices of messages (typically control vs. system-under-validation)
//! and verify that the latter contains fewer items.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Validate that the `after` messages contain strictly fewer items than `before`.
pub fn check_signal_drop(before: &[OtlpProtoMessage], after: &[OtlpProtoMessage]) -> bool {
    let before_total: usize = before.iter().map(OtlpProtoMessage::num_items).sum();
    let after_total: usize = after.iter().map(OtlpProtoMessage::num_items).sum();

    after_total < before_total
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
    fn detects_drop() {
        let before = [logs_with_records(5)];
        let after = [logs_with_records(3)];
        assert!(check_signal_drop(&before, &after));
    }

    #[test]
    fn no_drop_returns_false() {
        let before = [logs_with_records(4)];
        let after = [logs_with_records(4)];
        assert!(!check_signal_drop(&before, &after));
    }
}
