// SPDX-License-Identifier: Apache-2.0

//!
//! provides fake data to use for various fields in a OTLP signal
//!

use crate::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue, any_value::Value},
    metrics::v1::{
        exemplar::Value as ExemplarValue, exponential_histogram_data_point::Buckets,
        number_data_point::Value as NumberValue, summary_data_point::ValueAtQuantile,
    },
    trace::v1::Status,
};
use rand::Rng;
use rand::distr::{Alphabetic, Alphanumeric};

// ToDo: metric name, metric unit, metric description

const SCHEMA_URL: &str = "http://schema.opentelemetry.io";
/// default scope name to use for fake signals
const SCOPE_NAME: &str = "fake_signal";
/// default scope version to use for fake signals
const SCOPE_VERSION: &str = "1.0.0";

const TRACE_STATES: [&str; 3] = ["started", "ended", "unknown"];

const SEVERITY_TEXT: [&str; 6] = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];

// const SPAN_NAMES: [&str; 6] =

// const SPAN_EVENT_NAMES: [&str; 6] =

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
        })
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
pub fn get_body_text() -> Option<AnyValue> {
    let rng = rand::rng();

    let body: String = rng
        .sample_iter(&Alphabetic)
        .take(32)
        .map(char::from)
        .collect();

    Some(AnyValue {
        value: Some(Value::StringValue(body)),
    })
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
    "span_name".to_string()
}
/// provide data for the status field
#[must_use]
pub fn get_status() -> Option<Status> {
    Some(Status {
        message: "Ok".to_string(),
        code: 1,
    })
}
/// provide data for the event_name field for spans, can be an empty stream
#[must_use]
pub fn get_event_name() -> String {
    if rand::random() {
        "".to_string()
    } else {
        "event_nmae".to_string()
    }
}
/// provide data for the time_unix_nano field
#[must_use]
pub fn get_time_unix_nano() -> u64 {
    let mut rng = rand::rng();
    let unix_time: u64 = rng.random_range(165030000000000000..165050000000000000);
    unix_time
}
/// provide data for the observed_time_unix_nano field
#[must_use]
pub fn get_observed_time_unix_nano() -> u64 {
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

/// provide value to use for datapoint
#[must_use]
pub fn get_datapoint_value() -> Option<NumberValue> {
    let mut rng = rand::rng();
    if rand::random() {
        let int_value: i64 = rng.random_range(1..101);
        Some(NumberValue::AsInt(int_value))
    } else {
        let double_value: f64 = rng.random_range(1.5..101.5);
        Some(NumberValue::AsDouble(double_value))
    }
}

/// provide value to use for exemplar
#[must_use]
pub fn get_exemplar_value() -> Option<ExemplarValue> {
    let mut rng = rand::rng();
    if rand::random() {
        let int_value: i64 = rng.random_range(1..101);
        Some(ExemplarValue::AsInt(int_value))
    } else {
        let double_value: f64 = rng.random_range(1.5..101.5);
        Some(ExemplarValue::AsDouble(double_value))
    }
}

/// returns if a SUM datapoint is monotonic or not
#[must_use]
pub fn get_monotonic() -> bool {
    let is_monotonic: bool = rand::random();
    is_monotonic
}

/// provides the schema_url string
#[must_use]
pub fn get_schema_url() -> String {
    SCHEMA_URL.to_string()
}
