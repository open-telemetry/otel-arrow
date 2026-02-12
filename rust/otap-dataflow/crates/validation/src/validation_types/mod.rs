//! Collection of validation checks.
//!
//! These helpers operate on `&[OtlpProtoMessage]` so the validation exporter
//! can run different assertions (equivalence, batching, attribute presence,
//! signal drops, …) without duplicating traversal logic.

pub mod attributes;
mod batch;
mod signal_dropped;

use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;

use attributes::AttributeCheck;
use batch::check_batch_size;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::assert_equivalent;
use signal_dropped::check_signal_drop;

/// Supported validation kinds executed by the validation exporter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationKind {
    /// Check semantic equivalence between control and suv outputs.
    Equivalence,
    /// Check that after contains fewer signals than before.
    SignalDrop,
    /// Check that each message meets a minimum and/or maximum batch size.
    Batch {
        /// Minimum items required in each message (if set).
        #[serde(default)]
        min_batch_size: Option<usize>,
        /// Optional maximum items allowed in each message.
        #[serde(default)]
        max_batch_size: Option<usize>,
    },
    /// Check attribute presence/absence rules (applied to SUV messages).
    Attributes {
        /// Attribute rules to enforce.
        config: AttributeCheck,
    },
}

impl ValidationKind {
    /// Evaluate this validation against control and system-under-validation messages.
    pub fn evaluate(&self, control: &[OtlpProtoMessage], suv: &[OtlpProtoMessage]) -> bool {
        match self {
            ValidationKind::Equivalence => {
                std::panic::catch_unwind(AssertUnwindSafe(|| assert_equivalent(control, suv)))
                    .is_ok()
            }
            ValidationKind::SignalDrop => check_signal_drop(control, suv),
            ValidationKind::Batch {
                min_batch_size,
                max_batch_size,
            } => check_batch_size(suv, *min_batch_size, *max_batch_size),
            ValidationKind::Attributes { config } => config.check(suv),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation_types::attributes::{AttributeCheck, AttributeDomain};
    use crate::validation_types::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue as ProtoAny, KeyValue as ProtoKV, any_value::Value as ProtoVal,
    };
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
    fn equivalence_true_on_matching() {
        let msgs = vec![logs_with_records(2)];
        assert!(ValidationKind::Equivalence.evaluate(&msgs, &msgs));
    }

    #[test]
    fn batch_respects_bounds() {
        let msgs = vec![logs_with_records(3)];
        let kind = ValidationKind::Batch {
            min_batch_size: Some(2),
            max_batch_size: Some(5),
        };
        assert!(kind.evaluate(&msgs, &msgs));
        let failing = ValidationKind::Batch {
            min_batch_size: Some(4),
            max_batch_size: Some(5),
        };
        assert!(!failing.evaluate(&msgs, &msgs));
    }

    #[test]
    fn attributes_check_passes_when_required_key_present() {
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
        let check = AttributeCheck {
            domains: vec![AttributeDomain::Signal],
            require: vec![KeyValue::new("foo".into(), AnyValue::String("bar".into()))],
            ..Default::default()
        };
        assert!(ValidationKind::Attributes { config: check }.evaluate(&[], &suv));
    }
}
