// SPDX-License-Identifier: Apache-2.0

//!
//! provides fake data to use for various fields in a OTLP signal
//!

use crate::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue, any_value::Value},
    metrics::v1::{
        Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
        HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        Sum, Summary, SummaryDataPoint, exemplar::Value as ExemplarValue,
        exponential_histogram_data_point::Buckets, number_data_point::Value as NumberValue,
        summary_data_point::ValueAtQuantile,
    },
    trace::v1::Status,
};

// everything

// randomize the following values

// randomize the following values
/*

resource_attributes

 */

// LOGS

// todo randomize the following values
/*
time_unix_nano
observed_time_unix_nano
serveirty_text
severity_number
event_name
log_record attributes

body value
flags
dropped_attribute_count
 */

// Traces

/*
randomize the following
Start time unix nano
end time unix nano
span name
trace id
span id
span status
span attributes
parent span id
 */

// Metrics
/*
metric name
metric unit
metric value
metric description
datapoint attributes

 */

/*

need to randomize for each datatype
 */

// Profiles

/// default scope name to use for fake signals
const SCOPE_NAME: &str = "fake_signal";
/// default scope version to use for fake signals
const SCOPE_VERSION: &str = "1.0.0";
/// default trace id
const TRACE_ID: &str = "379bb33d04b406432ab33f9f54176a99";
/// default span id
const SPAN_ID: &str = "0497da84b964f973";

/// provide attributes for fake signals, attribute field
#[must_use]
pub fn get_attributes() -> Vec<KeyValue> {
    vec![KeyValue {
        key: "hostname".to_string(),
        value: Some(AnyValue {
            value: Some(Value::StringValue("host3.thedomain.edu".to_string())),
        }),
    }]
}
/// provide data for the serverity_text field
#[must_use]
pub fn get_severity_text() -> String {
    // random number between 1 and 3
    "INFO".to_string()
}

/// generate a random trace_id or use an existing one from constants
#[must_use]
pub fn get_trace_id() -> Vec<u8> {
    Vec::from(TRACE_ID.as_bytes())
}

/// generate a random span_id or use an existing one from constants
#[must_use]
pub fn get_span_id() -> Vec<u8> {
    Vec::from(SPAN_ID.as_bytes())
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
    Some(AnyValue {
        value: Some(Value::StringValue(
            "Sint impedit non ut eligendi nisi neque harum maxime adipisci.".to_string(),
        )),
    })
}
/// provide data for the trace_state field
#[must_use]
pub fn get_trace_state() -> String {
    "trace_state".to_string()
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
    "".to_string()
}
/// provide data for the time_unix_nano field
#[must_use]
pub fn get_time_unix_nano() -> u64 {
    1650499200000000000
}
/// provide data for the observed_time_unix_nano field
#[must_use]
pub fn get_observed_time_unix_nano() -> u64 {
    1650499200000000000
}
/// provide data for the start_time_unix_nano field
#[must_use]
pub fn get_start_time_unix_nano() -> u64 {
    1650499200000000000
}
/// provide data for the end_time_unix_nano field
#[must_use]
pub fn get_end_time_unix_nano() -> u64 {
    1650499200000000000
}
