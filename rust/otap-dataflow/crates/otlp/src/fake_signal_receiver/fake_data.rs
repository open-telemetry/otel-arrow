// SPDX-License-Identifier: Apache-2.0

//!
//! provides fake data to use for various fields in a OTLP signal
//!

use crate::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue, any_value::Value},
    logs::v1::LogRecordFlags,
    metrics::v1::AggregationTemporality,
    trace::v1::{SpanFlags, Status, span::SpanKind, status::StatusCode},
};
use rand::Rng;
use rand::distr::{Alphabetic, Alphanumeric};

/// default scope name to use for fake signals
const SCOPE_NAME: &str = "fake_signal";
/// default scope version to use for fake signals
const SCOPE_VERSION: &str = "1.0.0";

/// arrays containing values to choose from
const TRACE_STATES: [&str; 3] = ["started", "ended", "unknown"];
const SEVERITY_TEXT: [&str; 6] = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
const SPAN_NAMES: [&str; 6] = [
    "dns-lookup",
    "message-send",
    "http-close",
    "unknown",
    "http-send",
    "http-close",
];
const EVENT_NAMES: [&str; 5] = [
    "unknown",
    "message-receive",
    "message-send",
    "http-receive",
    "http-send",
];
const SPAN_KIND: [SpanKind; 6] = [
    SpanKind::Unspecified,
    SpanKind::Internal,
    SpanKind::Server,
    SpanKind::Client,
    SpanKind::Producer,
    SpanKind::Consumer,
];
const SPAN_FLAGS: [SpanFlags; 4] = [
    SpanFlags::DoNotUse,
    SpanFlags::TraceFlagsMask,
    SpanFlags::ContextHasIsRemoteMask,
    SpanFlags::ContextIsRemoteMask,
];
const STATUS_CODES: [StatusCode; 3] = [StatusCode::Unset, StatusCode::Error, StatusCode::Ok];
const LOG_RECORD_FLAGS: [LogRecordFlags; 2] =
    [LogRecordFlags::DoNotUse, LogRecordFlags::TraceFlagsMask];
const AGGREGATION_TEMPORALITY: [AggregationTemporality; 3] = [
    AggregationTemporality::Unspecified,
    AggregationTemporality::Delta,
    AggregationTemporality::Cumulative,
];

/// provide attributes for fake signals, attribute field
#[must_use]
pub fn get_attributes(attribute_count: usize) -> Vec<KeyValue> {
    let mut attributes = vec![];
    for attribute_index in 0..attribute_count {
        let rng = rand::rng();
        let attribute_value: String = rng
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();
        attributes.push(KeyValue {
            key: format!("attribute.{attribute_index}"),
            value: Some(AnyValue {
                value: Some(Value::StringValue(attribute_value)),
            }),
        });
    }
    attributes
}
/// provide data for the serverity_text field based on severity number
#[must_use]
pub fn get_severity_text(severity_number: i32) -> String {
    match severity_number {
        1..=4 => SEVERITY_TEXT[0].to_string(),
        5..=8 => SEVERITY_TEXT[1].to_string(),
        9..=12 => SEVERITY_TEXT[2].to_string(),
        13..=16 => SEVERITY_TEXT[3].to_string(),
        17..=20 => SEVERITY_TEXT[4].to_string(),
        _ => SEVERITY_TEXT[5].to_string(),
    }
}

/// generate a random severity_number within [1, 25)
#[must_use]
pub fn get_severity_number() -> i32 {
    let mut rng = rand::rng();
    let severity_number: i32 = rng.random_range(1..25);
    severity_number
}

/// generate a random trace_id
#[must_use]
pub fn get_trace_id() -> Vec<u8> {
    // size 32
    let rng = rand::rng();
    let trace_id: String = rng
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    trace_id.into_bytes()
}

/// generate a random span_id
#[must_use]
pub fn get_span_id() -> Vec<u8> {
    let rng = rand::rng();
    let span_id: String = rng
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    span_id.into_bytes()
}
/// provide data for scope_name field
#[must_use]
pub fn get_scope_name() -> String {
    SCOPE_NAME.to_string()
}

/// provide data for scope_version field
#[must_use]
pub fn get_scope_version() -> String {
    SCOPE_VERSION.to_string()
}
/// provide data for the body field
#[must_use]
pub fn get_body_text() -> AnyValue {
    let rng = rand::rng();

    let body: String = rng
        .sample_iter(&Alphabetic)
        .take(32)
        .map(char::from)
        .collect();

    AnyValue {
        value: Some(Value::StringValue(body)),
    }
}

/// provide data for the trace_state field
#[must_use]
pub fn get_trace_state() -> String {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..TRACE_STATES.len());
    TRACE_STATES[option].to_string()
}

/// provide data for the span name field
#[must_use]
pub fn get_span_name() -> String {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..SPAN_NAMES.len());
    SPAN_NAMES[option].to_string()
}

/// provide data for the status field
#[must_use]
pub fn get_status() -> Status {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..STATUS_CODES.len());
    let status = STATUS_CODES[option];

    Status {
        code: status as i32,
        message: status.as_str_name().to_string(),
    }
}

/// provide data for the event_name field for spans, can be an empty stream
#[must_use]
pub fn get_event_name() -> String {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..EVENT_NAMES.len());
    EVENT_NAMES[option].to_string()
}

/// provide data for the time_unix_nano field
#[must_use]
pub fn get_time_unix_nano() -> u64 {
    let mut rng = rand::rng();
    let unix_time: u64 = rng.random_range(165030000000000000..165050000000000000);
    unix_time
}

/// provide data for the start_time_unix_nano field
#[must_use]
pub fn get_start_time_unix_nano() -> u64 {
    let mut rng = rand::rng();
    let unix_time: u64 = rng.random_range(165030000000000000..165040000000000000);
    unix_time
}

/// provide data for the end_time_unix_nano field
#[must_use]
pub fn get_end_time_unix_nano() -> u64 {
    let mut rng = rand::rng();
    let unix_time: u64 = rng.random_range(165040000000000000..165050000000000000);
    unix_time
}

/// provide double value
#[must_use]
pub fn get_double_value() -> f64 {
    rand::random()
}

/// returns if a SUM datapoint is monotonic or not
#[must_use]
pub fn get_monotonic() -> bool {
    let is_monotonic: bool = rand::random();
    is_monotonic
}

/// provide data for the log flag field
#[must_use]
pub fn get_log_record_flag() -> LogRecordFlags {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..LOG_RECORD_FLAGS.len());
    LOG_RECORD_FLAGS[option]
}

/// provide data for the span flag field
#[must_use]
pub fn get_span_flag() -> SpanFlags {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..SPAN_FLAGS.len());
    SPAN_FLAGS[option]
}

/// provide data for the span kind field
#[must_use]
pub fn get_span_kind() -> SpanKind {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..SPAN_KIND.len());
    SPAN_KIND[option]
}

/// provide data for the span kind field
#[must_use]
pub fn get_aggregation() -> AggregationTemporality {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..AGGREGATION_TEMPORALITY.len());
    AGGREGATION_TEMPORALITY[option]
}

/// provides random u64 value for various fields
#[must_use]
pub fn get_int_value() -> u64 {
    rand::random()
}

/// provides vec of f64 for the buckets count
#[must_use]
pub fn get_explicit_bounds() -> Vec<f64> {
    let mut rng = rand::rng();
    let count: usize = rng.random_range(1..3);
    rand::random_iter().take(count).collect()
}

/// provides vec of u64 for the buckets count
#[must_use]
pub fn get_buckets_count() -> Vec<u64> {
    let mut rng = rand::rng();
    let count: usize = rng.random_range(1..4);
    rand::random_iter().take(count).collect()
}
