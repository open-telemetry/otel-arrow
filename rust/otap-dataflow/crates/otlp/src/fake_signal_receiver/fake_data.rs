// SPDX-License-Identifier: Apache-2.0

//!
//! provides fake data to use for various fields in a OTLP signal
//!

use crate::fake_signal_receiver::config::AttributeValue;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue},
    logs::v1::LogRecordFlags,
    metrics::v1::{
        AggregationTemporality, exponential_histogram_data_point::Buckets,
        summary_data_point::ValueAtQuantile,
    },
    trace::v1::{SpanFlags, Status, span::SpanKind, status::StatusCode},
};
use rand::Rng;
use rand::distr::{Alphabetic, Alphanumeric};
use rand::seq::IndexedRandom;
use std::collections::HashMap;

/// default scope name to use for fake signals
const SCOPE_NAME: &str = "fake_signal";
/// default scope version to use for fake signals
const SCOPE_VERSION: &str = "1.0.0";

const LOG_RECORD_FLAGS: [LogRecordFlags; 2] =
    [LogRecordFlags::DoNotUse, LogRecordFlags::TraceFlagsMask];
const AGGREGATION_TEMPORALITY: [AggregationTemporality; 3] = [
    AggregationTemporality::Unspecified,
    AggregationTemporality::Delta,
    AggregationTemporality::Cumulative,
];



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

/// provide data for the status field
#[must_use]
pub fn get_status() -> Status {
    let mut rng = rand::rng();
    let option: usize = rng.random_range(0..STATUS_CODES.len());
    let status = STATUS_CODES[option];
    Status::new(status.as_str_name(), status)
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
