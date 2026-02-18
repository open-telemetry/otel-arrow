// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0
//! Collection of validation instructions executed by the validation exporter.

pub mod attributes;
mod batch;
mod signal_dropped;

use attributes::{
    AttributeDomain, KeyValue, validate_deny_keys, validate_no_duplicate_keys,
    validate_require_key_values, validate_require_keys,
};
use batch::{validate_batch_bytes, validate_batch_items};
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::validate_equivalent;
use serde::{Deserialize, Serialize};
use signal_dropped::validate_signal_drop;
use std::time::Duration;
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
}
impl ValidationInstructions {
    /// Evaluate this validation against control and system-under-validation messages.
    #[must_use]
    pub fn validate(
        &self,
        control: &[OtlpProtoMessage],
        suv: &[OtlpProtoMessage],
        received_suv_message: &OtlpProtoMessage,
        time_elapsed: &Duration,
    ) -> bool {
        match self {
            ValidationInstructions::Equivalence => validate_equivalent(control, suv),
            ValidationInstructions::SignalDrop {
                min_drop_ratio,
                max_drop_ratio,
            } => validate_signal_drop(control, suv, *min_drop_ratio, *max_drop_ratio),
            ValidationInstructions::BatchItems {
                min_batch_size,
                max_batch_size,
                timeout,
            } => {
                if let Some(time) = timeout {
                    if time_elapsed >= time {
                        return true;
                    }
                }
                validate_batch_items(received_suv_message, *min_batch_size, *max_batch_size)
            }
            ValidationInstructions::BatchBytes {
                min_bytes,
                max_bytes,
                timeout,
            } => {
                if let Some(time) = timeout {
                    if time_elapsed >= time {
                        return true;
                    }
                }
                validate_batch_bytes(received_suv_message, *min_bytes, *max_bytes)
            }
            ValidationInstructions::AttributeDeny { domains, keys } => {
                validate_deny_keys(received_suv_message, domains, keys)
            }
            ValidationInstructions::AttributeRequireKey { domains, keys } => {
                validate_require_keys(received_suv_message, domains, keys)
            }
            ValidationInstructions::AttributeRequireKeyValue { domains, pairs } => {
                validate_require_key_values(received_suv_message, domains, pairs)
            }
            ValidationInstructions::AttributeNoDuplicate => {
                validate_no_duplicate_keys(received_suv_message)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation_types::attributes::{AnyValue, KeyValue};
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
    #[test]
    fn equivalence_true_on_matching() {
        let msgs = vec![logs_with_records(2)];
        assert!(ValidationInstructions::Equivalence.validate(
            &msgs,
            &msgs,
            msgs.last().unwrap(),
            &Duration::from_secs(0)
        ));
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
        assert!(!ValidationInstructions::Equivalence.validate(
            &left,
            &right,
            right.last().unwrap(),
            &Duration::from_secs(0)
        ));
    }
    #[test]
    fn batch_respects_bounds() {
        let msgs = vec![logs_with_records(3)];
        let instruction = ValidationInstructions::BatchItems {
            min_batch_size: Some(2),
            max_batch_size: Some(5),
            timeout: None,
        };
        assert!(instruction.validate(&msgs, &msgs, msgs.last().unwrap(), &Duration::from_secs(0)));
        let failing = ValidationInstructions::BatchItems {
            min_batch_size: Some(4),
            max_batch_size: Some(5),
            timeout: None,
        };
        assert!(!failing.validate(&msgs, &msgs, msgs.last().unwrap(), &Duration::from_secs(0)));
    }

    #[test]
    fn batch_bytes_respects_bounds() {
        let msgs = vec![logs_with_records(1)];
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

        assert!(pass.validate(&msgs, &msgs, latest, &Duration::from_secs(0)));
        assert!(!fail_small.validate(&msgs, &msgs, latest, &Duration::from_secs(0)));
        assert!(!fail_large.validate(&msgs, &msgs, latest, &Duration::from_secs(0)));
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
        let suv = vec![OtlpProtoMessage::Logs(logs)];
        let check = ValidationInstructions::AttributeRequireKeyValue {
            domains: vec![AttributeDomain::Signal],
            pairs: vec![KeyValue::new("foo".into(), AnyValue::String("bar".into()))],
        };
        assert!(check.validate(&[], &suv, suv.last().unwrap(), &Duration::from_secs(0)));
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
        let suv = vec![OtlpProtoMessage::Logs(logs)];
        let check = ValidationInstructions::AttributeDeny {
            domains: vec![AttributeDomain::Signal],
            keys: vec!["deny".into()],
        };
        assert!(!check.validate(&[], &suv, suv.last().unwrap(), &Duration::from_secs(0)));
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
        let suv = vec![OtlpProtoMessage::Logs(logs)];
        let check = ValidationInstructions::AttributeNoDuplicate {
            domains: vec![AttributeDomain::Signal],
        };
        assert!(!check.validate(&[], &suv, suv.last().unwrap(), &Duration::from_secs(0)));
    }
    #[test]
    fn signal_drop_with_ratio_bounds() {
        let before = vec![logs_with_records(10)];
        let after = vec![logs_with_records(4)];
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
        assert!(pass.validate(
            &before,
            &after,
            after.last().unwrap(),
            &Duration::from_secs(0)
        ));
        assert!(!fail_min.validate(
            &before,
            &after,
            after.last().unwrap(),
            &Duration::from_secs(0)
        ));
        assert!(!fail_max.validate(
            &before,
            &after,
            after.last().unwrap(),
            &Duration::from_secs(0)
        ));
    }
}
