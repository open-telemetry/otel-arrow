// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Signal drop validation.
//!
//! Compare two slices of messages (control vs. system-under-validation)
//! and verify that the latter contains fewer items.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Validate that the `suv` messages contain strictly fewer items than `control`.
pub(crate) fn validate_signal_drop(
    control: &[OtlpProtoMessage],
    suv: &[OtlpProtoMessage],
    min_drop_ratio: Option<f64>,
    max_drop_ratio: Option<f64>,
) -> bool {
    let control_total: usize = control.iter().map(OtlpProtoMessage::num_items).sum();
    let suv_total: usize = suv.iter().map(OtlpProtoMessage::num_items).sum();

    if suv_total >= control_total {
        return false;
    }

    let drop_ratio = (control_total as f64 - suv_total as f64) / control_total as f64;

    if let Some(min) = min_drop_ratio {
        if drop_ratio < min {
            return false;
        }
    }

    if let Some(max) = max_drop_ratio {
        if drop_ratio > max {
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
    fn detects_drop() {
        let control = [logs_with_records(5)];
        let suv = [logs_with_records(3)];
        assert!(validate_signal_drop(&control, &suv, None, None));
    }

    #[test]
    fn no_drop_returns_false() {
        let control = [logs_with_records(4)];
        let suv = [logs_with_records(4)];
        assert!(!validate_signal_drop(&control, &suv, None, None));
    }
}
