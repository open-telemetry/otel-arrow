// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch validation utilities.
//!
//! Validates that each message in a collection meets configured batch size bounds.

use otap_df_pdata::proto::OtlpProtoMessage;
use prost::Message;
/// Ensure every message size is within `[min_items, max_items]` (if provided).
/// Returns true for an empty slice (no violations).
pub fn validate_batch_items(
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

/// Ensure every message request count (always 1 per message) is within bounds.
/// Returns true for an empty slice (no violations).
pub fn validate_batch_requests(
    _message: &OtlpProtoMessage,
    min_reqs: Option<usize>,
    max_reqs: Option<usize>,
) -> bool {
    let val = 1usize; // one OTLP message = one request
    if let Some(min) = min_reqs {
        if val < min {
            return false;
        }
    }
    if let Some(max) = max_reqs {
        if val > max {
            return false;
        }
    }
    true
}

/// Ensure every message encoded size in bytes is within bounds.
/// Returns true for an empty slice (no violations).
pub fn validate_batch_bytes(
    message: &OtlpProtoMessage,
    min_bytes: Option<usize>,
    max_bytes: Option<usize>,
) -> bool {
    let size = match encoded_size(message) {
        Some(s) => s,
        None => return false,
    };
    if let Some(min) = min_bytes {
        if size < min {
            return false;
        }
    }
    if let Some(max) = max_bytes {
        if size > max {
            return false;
        }
    }
    true
}

fn encoded_size(msg: &OtlpProtoMessage) -> Option<usize> {
    let mut buf = Vec::new();
    match msg {
        OtlpProtoMessage::Logs(l) => l.encode(&mut buf).ok()?,
        OtlpProtoMessage::Metrics(m) => m.encode(&mut buf).ok()?,
        OtlpProtoMessage::Traces(t) => t.encode(&mut buf).ok()?,
    }
    Some(buf.len())
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
        assert!(!validate_batch_items(&[], Some(1), None));
    }

    #[test]
    fn min_only_passes_within_bounds() {
        let msgs = [logs_with_records(3), logs_with_records(2)];
        assert!(validate_batch_items(&msgs, Some(2), None));
        assert!(!validate_batch_items(&msgs, Some(4), None));
    }

    #[test]
    fn max_only_passes_within_bounds() {
        let msgs = [logs_with_records(3), logs_with_records(2)];
        assert!(validate_batch_items(&msgs, None, Some(3)));
        assert!(!validate_batch_items(&msgs, None, Some(2)));
    }

    #[test]
    fn min_and_max_bounds() {
        let msgs = [logs_with_records(3), logs_with_records(2)];
        assert!(validate_batch_items(&msgs, Some(2), Some(3)));
        assert!(!validate_batch_items(&msgs, Some(2), Some(2)));
    }
}
