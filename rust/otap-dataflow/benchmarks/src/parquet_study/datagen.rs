// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generates OTAP logs batches for the parquet study.
//!
//! The generated data mirrors the `otap_encoder` benchmark: a configurable
//! resource x scope x log fan-out where every log record carries a fixed set of
//! mixed-type attributes (string, bool, int, double, bytes, empty, array,
//! kvlist). This exercises every attribute value column (`str/int/double/bool/
//! bytes/ser`) so the flatten/unflatten round-trips are meaningfully tested.

use otap_df_pdata::encode::encode_logs_otap_batch;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use prost::Message;

/// Shape of the generated logs data.
#[derive(Clone, Copy, Debug)]
pub struct LogsGenParams {
    /// Number of distinct resources.
    pub num_resources: usize,
    /// Number of scopes within each resource.
    pub num_scopes: usize,
    /// Number of log records within each scope.
    pub num_logs: usize,
}

impl LogsGenParams {
    /// A short id usable as a Criterion benchmark parameter label.
    #[must_use]
    pub fn label(&self) -> String {
        format!(
            "r{}_s{}_l{}_n{}",
            self.num_resources,
            self.num_scopes,
            self.num_logs,
            self.total_logs(),
        )
    }

    /// Total number of log records produced.
    #[must_use]
    pub fn total_logs(&self) -> usize {
        self.num_resources * self.num_scopes * self.num_logs
    }
}

fn sample_log_attributes() -> Vec<KeyValue> {
    let attr_values = vec![
        AnyValue::new_string("terry"),
        AnyValue::new_bool(true),
        AnyValue::new_int(5),
        AnyValue::new_double(2.0),
        AnyValue::new_bytes(b"hi"),
        AnyValue { value: None },
        AnyValue::new_array(vec![AnyValue::new_bool(true)]),
        AnyValue::new_kvlist(vec![KeyValue::new("key1", AnyValue::new_bool(true))]),
    ];
    let mut log_attributes = attr_values
        .into_iter()
        .enumerate()
        .map(|(i, val)| KeyValue {
            key: format!("attr{i}"),
            value: Some(val),
        })
        .collect::<Vec<_>>();

    // an attribute whose value is absent
    log_attributes.push(KeyValue {
        key: "noneval".to_string(),
        value: None,
    });
    log_attributes
}

/// Build an OTLP [`LogsData`] proto message with the requested shape.
#[must_use]
pub fn create_logs_data(params: &LogsGenParams) -> LogsData {
    let log_attributes = sample_log_attributes();

    LogsData::new(
        (0..params.num_resources)
            .map(|_| {
                ResourceLogs::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new(
                            "resource_attr1",
                            AnyValue::new_string("resource_value"),
                        )])
                        .dropped_attributes_count(1u32),
                    (0..params.num_scopes)
                        .map(|_| {
                            ScopeLogs::new(
                                InstrumentationScope::build()
                                    .name("library")
                                    .version("scopev1")
                                    .attributes(vec![
                                        KeyValue::new(
                                            "scope_attr1",
                                            AnyValue::new_string("scope_val1"),
                                        ),
                                        KeyValue::new(
                                            "scope_attr2",
                                            AnyValue::new_string("scope_val2"),
                                        ),
                                    ])
                                    .dropped_attributes_count(2u32)
                                    .finish(),
                                (0..params.num_logs)
                                    .map(|_| {
                                        LogRecord::build()
                                            .time_unix_nano(2_000_000_000u64)
                                            .severity_number(SeverityNumber::Info)
                                            .event_name("event1")
                                            .observed_time_unix_nano(3_000_000_000u64)
                                            .trace_id(vec![
                                                0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
                                            ])
                                            .span_id(vec![0, 0, 0, 0, 1, 1, 1, 1])
                                            .severity_text("Info")
                                            .attributes(log_attributes.clone())
                                            .dropped_attributes_count(3u32)
                                            .flags(LogRecordFlags::TraceFlagsMask)
                                            .body(AnyValue::new_string("log_body"))
                                            .finish()
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .set_schema_url("https://schema.opentelemetry.io/scope_schema")
                        })
                        .collect::<Vec<_>>(),
                )
                .set_schema_url("https://schema.opentelemetry.io/resource_schema")
            })
            .collect::<Vec<_>>(),
    )
}

/// Generate an OTAP logs batch ([`OtapArrowRecords::Logs`]) for the given shape,
/// alongside the size in bytes of the equivalent OTLP protobuf encoding (a
/// useful reference point for the "on the wire" comparison).
#[must_use]
pub fn gen_logs_otap(params: &LogsGenParams) -> (OtapArrowRecords, usize) {
    let logs_data = create_logs_data(params);
    let mut proto_bytes = Vec::new();
    logs_data
        .encode(&mut proto_bytes)
        .expect("can encode OTLP proto bytes");

    let view = RawLogsData::new(proto_bytes.as_ref());
    let otap = encode_logs_otap_batch(&view).expect("can encode OTAP logs batch");
    (otap, proto_bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

    #[test]
    fn generates_expected_record_counts() {
        let params = LogsGenParams {
            num_resources: 2,
            num_scopes: 3,
            num_logs: 4,
        };
        let (otap, proto_len) = gen_logs_otap(&params);
        assert!(proto_len > 0);

        let logs = otap
            .get(ArrowPayloadType::Logs)
            .expect("logs payload present");
        assert_eq!(logs.num_rows(), params.total_logs());

        // Every payload type we expect to flatten should be present.
        for pt in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
        ] {
            assert!(otap.get(pt).is_some(), "missing payload {pt:?}");
        }
    }
}
