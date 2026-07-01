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

/// Shape of generated logs data with configurable attribute richness, used by
/// the OTAP-flat study to vary the ratio of *shared* resource/scope attributes
/// (which the REE and dictionary layouts store once) to *per-record* log
/// attributes (which every layout stores per row).
///
/// The generated records model realistic telemetry rather than identical rows:
/// each record has a unique pseudo-random `trace_id`/`span_id`, a varying
/// timestamp, a templated body, and mixed-type log attributes that blend
/// low-cardinality enums with high-cardinality per-record values. Resource and
/// scope attributes are distinct per resource/scope but shared across that
/// resource/scope's records, which is the low-cardinality-shared case the
/// run-end and dictionary layouts target.
#[derive(Clone, Copy, Debug)]
pub struct RichGenParams {
    /// A short label for tables.
    pub label: &'static str,
    /// Number of distinct resources.
    pub num_resources: usize,
    /// Number of scopes within each resource.
    pub num_scopes: usize,
    /// Number of log records within each scope.
    pub num_logs: usize,
    /// Attributes attached to each resource (distinct per resource, shared by
    /// that resource's records).
    pub num_resource_attrs: usize,
    /// Attributes attached to each scope.
    pub num_scope_attrs: usize,
    /// Mixed-type attributes attached to each log record.
    pub num_log_attrs: usize,
}

impl RichGenParams {
    /// Total number of log records produced.
    #[must_use]
    pub fn total_logs(&self) -> usize {
        self.num_resources * self.num_scopes * self.num_logs
    }
}

/// splitmix64: a cheap deterministic mixer used to synthesize high-entropy,
/// unique-per-record trace ids and attribute values, so the generated data does
/// not compress like identical rows.
fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A unique, high-entropy 16-byte trace id for a global record index.
fn trace_id_for(idx: u64) -> Vec<u8> {
    let mut b = Vec::with_capacity(16);
    b.extend_from_slice(&mix64(idx).to_be_bytes());
    b.extend_from_slice(&mix64(idx ^ 0xDEAD_BEEF).to_be_bytes());
    b
}

/// A unique 8-byte span id for a global record index.
fn span_id_for(idx: u64) -> Vec<u8> {
    mix64(idx.wrapping_mul(0x100_0001)).to_be_bytes().to_vec()
}

/// Distinct-per-resource, low-cardinality resource attributes.
fn resource_attrs_for(res_idx: usize, n: usize) -> Vec<KeyValue> {
    (0..n)
        .map(|i| {
            let value = if i == 0 {
                format!("service-{res_idx}")
            } else {
                format!("res{res_idx}-attr{i}")
            };
            KeyValue::new(format!("resource_attr{i}"), AnyValue::new_string(value))
        })
        .collect()
}

/// Distinct-per-scope, low-cardinality scope attributes.
fn scope_attrs_for(scope_idx: usize, n: usize) -> Vec<KeyValue> {
    (0..n)
        .map(|i| {
            KeyValue::new(
                format!("scope_attr{i}"),
                AnyValue::new_string(format!("scope{scope_idx}-attr{i}")),
            )
        })
        .collect()
}

const HTTP_METHODS: [&str; 4] = ["GET", "POST", "PUT", "DELETE"];

/// Mixed-type log attributes for a global record index, blending
/// low-cardinality enums with high-cardinality per-record values.
fn log_attrs_for(idx: u64, n: usize) -> Vec<KeyValue> {
    (0..n)
        .map(|i| {
            let key = format!("log_attr{i}");
            let value = match i % 5 {
                // low-cardinality enum string
                0 => AnyValue::new_string(HTTP_METHODS[(idx as usize + i) % 4].to_string()),
                // high-cardinality int
                1 => AnyValue::new_int((mix64(idx + i as u64) % 1_000_000) as i64),
                // high-cardinality double
                2 => AnyValue::new_double((idx as f64) * 1.5 + i as f64),
                // bool
                3 => AnyValue::new_bool((idx as usize + i).is_multiple_of(2)),
                // high-cardinality id-like string
                _ => AnyValue::new_string(format!("id-{:016x}", mix64(idx.wrapping_add(i as u64)))),
            };
            KeyValue::new(key, value)
        })
        .collect()
}

/// Build an OTLP [`LogsData`] with configurable per-scope/resource/log attribute
/// counts. Unlike [`create_logs_data`], attribute richness is a parameter and
/// each record carries realistic high-cardinality content so the study can
/// measure resource-heavy versus log-heavy telemetry without identical rows.
#[must_use]
pub fn create_logs_data_rich(params: &RichGenParams) -> LogsData {
    let base_time = 1_700_000_000_000_000_000u64;
    LogsData::new(
        (0..params.num_resources)
            .map(|res_idx| {
                ResourceLogs::new(
                    Resource::build()
                        .attributes(resource_attrs_for(res_idx, params.num_resource_attrs))
                        .dropped_attributes_count(0u32),
                    (0..params.num_scopes)
                        .map(|scope_idx| {
                            ScopeLogs::new(
                                InstrumentationScope::build()
                                    .name("library")
                                    .version("scopev1")
                                    .attributes(scope_attrs_for(scope_idx, params.num_scope_attrs))
                                    .dropped_attributes_count(0u32)
                                    .finish(),
                                (0..params.num_logs)
                                    .map(|log_idx| {
                                        let idx = ((res_idx * params.num_scopes + scope_idx)
                                            * params.num_logs
                                            + log_idx)
                                            as u64;
                                        LogRecord::build()
                                            .time_unix_nano(base_time + idx * 1_000_000)
                                            .observed_time_unix_nano(
                                                base_time + idx * 1_000_000 + 500,
                                            )
                                            .severity_number(SeverityNumber::Info)
                                            .severity_text("Info")
                                            .trace_id(trace_id_for(idx))
                                            .span_id(span_id_for(idx))
                                            .attributes(log_attrs_for(idx, params.num_log_attrs))
                                            .body(AnyValue::new_string(format!(
                                                "request {idx} handled in {}ms",
                                                idx % 500
                                            )))
                                            .finish()
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

/// Generate an OTAP logs batch for a [`RichGenParams`] shape.
#[must_use]
pub fn gen_logs_otap_rich(params: &RichGenParams) -> OtapArrowRecords {
    let logs_data = create_logs_data_rich(params);
    let mut proto_bytes = Vec::new();
    logs_data
        .encode(&mut proto_bytes)
        .expect("can encode OTLP proto bytes");
    let view = RawLogsData::new(proto_bytes.as_ref());
    encode_logs_otap_batch(&view).expect("can encode OTAP logs batch")
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
