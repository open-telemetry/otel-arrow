//! Collection of validation checks.
//!
//! These helpers operate on `&[OtlpProtoMessage]` so the validation exporter
//! can run different assertions (equivalence, batching, attribute presence,
//! signal drops, …) without duplicating traversal logic.

mod attributes;
mod batch;
mod signal_dropped;

use otap_df_config::SignalType;
use otap_df_pdata::proto::OtlpProtoMessage;

pub use attributes::{
    check_attributes, AttributeCheckConfig, AttributeDomain, AttributeValue, ExpectedKeyValue,
};
pub use batch::check_min_batch_size;
pub use signal_dropped::check_signal_drop;

/// Ensure all messages in the slice share the same signal type.
pub(crate) fn uniform_signal_type(messages: &[OtlpProtoMessage]) -> Option<SignalType> {
    let mut iter = messages.iter();
    let first = iter.next()?.signal_type();
    if iter.all(|m| m.signal_type() == first) {
        Some(first)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};

    #[test]
    fn uniform_signal_type_single_type() {
        let logs = OtlpProtoMessage::Logs(LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord::default()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });
        let mix = vec![logs.clone(), logs];
        assert_eq!(uniform_signal_type(&mix), Some(SignalType::Logs));
    }
}
