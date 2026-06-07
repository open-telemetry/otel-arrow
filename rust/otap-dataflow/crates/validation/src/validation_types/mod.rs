// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collection of validation instructions executed by the validation exporter.

pub mod attributes;
mod batch;
mod signal_dropped;
pub mod transport_headers;

use attributes::{
    AttributeDomain, KeyValue, validate_deny_keys, validate_no_duplicate_keys,
    validate_require_key_values, validate_require_keys,
};
use batch::{validate_batch_bytes, validate_batch_items};
use otap_df_config::transport_headers::TransportHeaders;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::validate_equivalent;
use serde::{Deserialize, Serialize};
use signal_dropped::validate_signal_drop;
use std::time::Duration;
use transport_headers::{
    TransportHeaderKeyValue, validate_transport_header_deny_keys,
    validate_transport_header_require_key_values, validate_transport_header_require_keys,
};
/// Supported validation instructions executed by the validation exporter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationInstructions {
    /// Check semantic equivalence between control and suv outputs.
    Equivalence,
    /// Check that after contains fewer signals than before, with optional ratio bounds.
    SignalDrop {
        /// Minimum fraction of signals that must be dropped (0.0-1.0).
        #[serde(default)]
        min_drop_ratio: Option<f64>,
        /// Maximum fraction of signals that may be dropped (0.0-1.0).
        #[serde(default)]
        max_drop_ratio: Option<f64>,
    },
    /// Check that each message meets a minimum and/or maximum batch size.
    BatchItems {
        /// Minimum items required in each message (if set).
        #[serde(default)]
        min_batch_size: Option<usize>,
        /// Optional maximum items allowed in each message.
        #[serde(default)]
        max_batch_size: Option<usize>,
        /// allow messages to get released after a certain time
        #[serde(with = "humantime_serde::option")]
        timeout: Option<Duration>,
    },
    /// Check that encoded byte size of each message is within bounds.
    BatchBytes {
        /// Minimum encoded bytes required in each message (if set).
        #[serde(default)]
        min_bytes: Option<usize>,
        /// Optional maximum encoded bytes allowed in each message.
        #[serde(default)]
        max_bytes: Option<usize>,
        /// allow messages to get released after a certain time
        #[serde(with = "humantime_serde::option")]
        timeout: Option<Duration>,
    },
    /// Forbid specific attribute keys in selected domains.
    AttributeDeny {
        /// Domains to inspect.
        domains: Vec<AttributeDomain>,
        /// Keys that must not appear.
        keys: Vec<String>,
    },
    /// Require specific attribute keys to be present.
    AttributeRequireKey {
        /// Domains to inspect.
        domains: Vec<AttributeDomain>,
        /// Keys that must be present.
        keys: Vec<String>,
    },
    /// Require specific attribute key/value pairs to be present.
    AttributeRequireKeyValue {
        /// Domains to inspect.
        domains: Vec<AttributeDomain>,
        /// Key/value pairs that must be present.
        pairs: Vec<KeyValue>,
    },
    /// Ensure no duplicate attribute keys within all attribute lists.
    AttributeNoDuplicate,
    /// Require specific transport header keys to be present on SUV messages.
    TransportHeaderRequireKey {
        /// Header keys (stored/logical names) that must be present.
        keys: Vec<String>,
    },
    /// Require specific transport header key/value pairs on SUV messages.
    TransportHeaderRequireKeyValue {
        /// Key/value pairs that must be present (values compared as UTF-8 text).
        pairs: Vec<TransportHeaderKeyValue>,
    },
    /// Forbid specific transport header keys on SUV messages.
    TransportHeaderDeny {
        /// Header keys (stored/logical names) that must NOT be present.
        keys: Vec<String>,
    },
}
impl ValidationInstructions {
    /// Evaluate this validation against control and system-under-validation messages.
    #[must_use]
    pub(crate) fn validate(
        &self,
        control: &[OtlpProtoMessage],
        suv_msgs: &[OtlpProtoMessage],
        suv_with_duration: &[(OtlpProtoMessage, Duration)],
        transport_headers: &[Option<TransportHeaders>],
    ) -> bool {
        match self {
            ValidationInstructions::Equivalence => validate_equivalent(control, suv_msgs),
            ValidationInstructions::SignalDrop {
                min_drop_ratio,
                max_drop_ratio,
            } => validate_signal_drop(control, suv_msgs, *min_drop_ratio, *max_drop_ratio),
            ValidationInstructions::BatchItems {
                min_batch_size,
                max_batch_size,
                timeout,
            } => validate_batch_items(suv_with_duration, min_batch_size, max_batch_size, timeout),
            ValidationInstructions::BatchBytes {
                min_bytes,
                max_bytes,
                timeout,
            } => validate_batch_bytes(suv_with_duration, min_bytes, max_bytes, timeout),
            ValidationInstructions::AttributeDeny { domains, keys } => {
                validate_deny_keys(suv_msgs, domains, keys)
            }
            ValidationInstructions::AttributeRequireKey { domains, keys } => {
                validate_require_keys(suv_msgs, domains, keys)
            }
            ValidationInstructions::AttributeRequireKeyValue { domains, pairs } => {
                validate_require_key_values(suv_msgs, domains, pairs)
            }
            ValidationInstructions::AttributeNoDuplicate => validate_no_duplicate_keys(suv_msgs),
            ValidationInstructions::TransportHeaderRequireKey { keys } => {
                validate_transport_header_require_keys(transport_headers, keys)
            }
            ValidationInstructions::TransportHeaderRequireKeyValue { pairs } => {
                validate_transport_header_require_key_values(transport_headers, pairs)
            }
            ValidationInstructions::TransportHeaderDeny { keys } => {
                validate_transport_header_deny_keys(transport_headers, keys)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation_types::attributes::{AnyValue, KeyValue};
    use crate::validation_types::transport_headers::TransportHeaderKeyValue;
    use otap_df_config::transport_headers::{TransportHeader, TransportHeaders};
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue as ProtoAny, KeyValue as ProtoKV, any_value::Value as ProtoVal,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use prost::Message;
    use std::time::Duration;
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
    fn with_duration(msgs: &[OtlpProtoMessage]) -> Vec<(OtlpProtoMessage, Duration)> {
        msgs.iter()
            .map(|m| (m.clone(), Duration::from_secs(0)))
            .collect()
    }

    fn no_headers(count: usize) -> Vec<Option<TransportHeaders>> {
        vec![None; count]
    }

    #[test]
    fn equivalence_true_on_matching() {
        let msgs = vec![logs_with_records(2)];
        let suv = with_duration(&msgs);
        assert!(ValidationInstructions::Equivalence.validate(&msgs, &msgs, &suv, &no_headers(1)));
    }

    #[test]
    fn equivalence_false_on_mismatch() {
        use otap_df_pdata::proto::opentelemetry::common::v1::AnyValue as AV;
        use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
        // left: single log with body "only"
        let left = vec![OtlpProtoMessage::Logs(LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AV {
                            value: Some(ProtoVal::StringValue("only".into())),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })];

        // right: includes an extra distinct log record
        let right = vec![OtlpProtoMessage::Logs(LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        LogRecord {
                            body: Some(AV {
                                value: Some(ProtoVal::StringValue("only".into())),
                            }),
                            ..Default::default()
                        },
                        LogRecord {
                            body: Some(AV {
                                value: Some(ProtoVal::StringValue("extra".into())),
                            }),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })];
        let right_suv = with_duration(&right);
        assert!(!ValidationInstructions::Equivalence.validate(
            &left,
            &right,
            &right_suv,
            &no_headers(1)
        ));
    }
    #[test]
    fn batch_respects_bounds() {
        let msgs = vec![logs_with_records(3)];
        let suv = with_duration(&msgs);
        let headers = no_headers(1);
        let instruction = ValidationInstructions::BatchItems {
            min_batch_size: Some(2),
            max_batch_size: Some(5),
            timeout: None,
        };
        assert!(instruction.validate(&msgs, &msgs, &suv, &headers));
        let failing = ValidationInstructions::BatchItems {
            min_batch_size: Some(4),
            max_batch_size: Some(5),
            timeout: None,
        };
        assert!(!failing.validate(&msgs, &msgs, &suv, &headers));
    }

    #[test]
    fn batch_bytes_respects_bounds() {
        let msgs = vec![logs_with_records(1)];
        let suv = with_duration(&msgs);
        let headers = no_headers(1);
        let mut buf = Vec::new();
        // compute encoded size of the latest SUV message
        let latest = msgs.last().unwrap();
        if let OtlpProtoMessage::Logs(l) = latest {
            l.encode(&mut buf).unwrap();
        }
        let sz = buf.len();

        let pass = ValidationInstructions::BatchBytes {
            min_bytes: Some(sz.saturating_sub(1)),
            max_bytes: Some(sz + 10),
            timeout: None,
        };
        let fail_small = ValidationInstructions::BatchBytes {
            min_bytes: Some(sz + 1),
            max_bytes: None,
            timeout: None,
        };
        let fail_large = ValidationInstructions::BatchBytes {
            min_bytes: None,
            max_bytes: Some(sz - 1),
            timeout: None,
        };

        assert!(pass.validate(&msgs, &msgs, &suv, &headers));
        assert!(!fail_small.validate(&msgs, &msgs, &suv, &headers));
        assert!(!fail_large.validate(&msgs, &msgs, &suv, &headers));
    }

    #[test]
    fn attribute_require_key_value_passes() {
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![ProtoKV {
                            key: "foo".into(),
                            value: Some(ProtoAny {
                                value: Some(ProtoVal::StringValue("bar".into())),
                            }),
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let msg = OtlpProtoMessage::Logs(logs);
        let suv_msgs = vec![msg.clone()];
        let suv = vec![(msg, Duration::from_secs(0))];
        let headers = no_headers(1);
        let check = ValidationInstructions::AttributeRequireKeyValue {
            domains: vec![AttributeDomain::Signal],
            pairs: vec![KeyValue::new("foo".into(), AnyValue::String("bar".into()))],
        };
        assert!(check.validate(&[], &suv_msgs, &suv, &headers));
    }
    #[test]
    fn attribute_deny_blocks_key() {
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![ProtoKV {
                            key: "deny".into(),
                            value: Some(ProtoAny {
                                value: Some(ProtoVal::StringValue("x".into())),
                            }),
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let msg = OtlpProtoMessage::Logs(logs);
        let suv_msgs = vec![msg.clone()];
        let suv = vec![(msg, Duration::from_secs(0))];
        let headers = no_headers(1);
        let check = ValidationInstructions::AttributeDeny {
            domains: vec![AttributeDomain::Signal],
            keys: vec!["deny".into()],
        };
        assert!(!check.validate(&[], &suv_msgs, &suv, &headers));
    }
    #[test]
    fn attribute_no_duplicate_detects_duplicates() {
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![
                            ProtoKV {
                                key: "dup".into(),
                                value: Some(ProtoAny {
                                    value: Some(ProtoVal::StringValue("a".into())),
                                }),
                            },
                            ProtoKV {
                                key: "dup".into(),
                                value: Some(ProtoAny {
                                    value: Some(ProtoVal::StringValue("b".into())),
                                }),
                            },
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let msg = OtlpProtoMessage::Logs(logs);
        let suv_msgs = vec![msg.clone()];
        let suv = vec![(msg, Duration::from_secs(0))];
        let headers = no_headers(1);
        let check = ValidationInstructions::AttributeNoDuplicate;
        assert!(!check.validate(&[], &suv_msgs, &suv, &headers));
    }
    #[test]
    fn signal_drop_with_ratio_bounds() {
        let before = vec![logs_with_records(10)];
        let after_msgs = vec![logs_with_records(4)];
        let after = with_duration(&after_msgs);
        let headers = no_headers(1);
        // drop ratio = 0.6
        let pass = ValidationInstructions::SignalDrop {
            min_drop_ratio: Some(0.5),
            max_drop_ratio: Some(0.7),
        };
        let fail_min = ValidationInstructions::SignalDrop {
            min_drop_ratio: Some(0.7),
            max_drop_ratio: None,
        };
        let fail_max = ValidationInstructions::SignalDrop {
            min_drop_ratio: None,
            max_drop_ratio: Some(0.4),
        };
        assert!(pass.validate(&before, &after_msgs, &after, &headers));
        assert!(!fail_min.validate(&before, &after_msgs, &after, &headers));
        assert!(!fail_max.validate(&before, &after_msgs, &after, &headers));
    }

    #[test]
    fn transport_header_require_key_serialization_check() {
        let instruction = ValidationInstructions::TransportHeaderRequireKey {
            keys: vec!["x-tenant-id".into()],
        };
        let yaml = serde_yaml::to_string(&instruction).expect("serialize");
        let back: ValidationInstructions = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(back, instruction);

        // Validate that both the original and round-tripped instruction
        // produce identical results when executed.
        let mut headers = TransportHeaders::default();
        headers.push(TransportHeader::text("x-tenant-id", "x-tenant-id", b"acme"));
        let transport = vec![Some(headers)];
        let control: Vec<OtlpProtoMessage> = vec![];
        let suv_msgs: Vec<OtlpProtoMessage> = vec![];
        let suv_with_dur: Vec<(OtlpProtoMessage, Duration)> = vec![];

        let result_original = instruction.validate(&control, &suv_msgs, &suv_with_dur, &transport);
        let result_roundtrip = back.validate(&control, &suv_msgs, &suv_with_dur, &transport);
        assert!(result_original);
        assert_eq!(result_original, result_roundtrip);
    }

    #[test]
    fn transport_header_require_key_value_serialization_check() {
        let instruction = ValidationInstructions::TransportHeaderRequireKeyValue {
            pairs: vec![TransportHeaderKeyValue::new("x-tenant-id", "acme")],
        };
        let yaml = serde_yaml::to_string(&instruction).expect("serialize");
        let back: ValidationInstructions = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(back, instruction);
    }

    #[test]
    fn transport_header_deny_serialization_check() {
        let instruction = ValidationInstructions::TransportHeaderDeny {
            keys: vec!["x-secret".into()],
        };
        let yaml = serde_yaml::to_string(&instruction).expect("serialize");
        let back: ValidationInstructions = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(back, instruction);
    }
}
