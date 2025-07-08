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
// Reusable constants from the previous JSON (still valid)
// pub const RESOURCE_IP: &str = "192.168.0.1";
// pub const RESOURCE_VERSION: f64 = 1.0;
// pub const RESOURCE_HOSTNAME: &str = "host1.mydomain.com";
// pub const RESOURCE_UP: bool = true;
// pub const RESOURCE_STATUS: i32 = 200;
// pub const RESOURCE_UNIQUE1: &str = "uv1";

/// default trace id
const TRACE_ID: &str = "379bb33d04b406432ab33f9f54176a99";
/// default span id
const SPAN_ID: &str = "0497da84b964f973";

// Unix Time Values (from `timeUnixNano` and `observedTimeUnixNano`)
// pub const TIMESTAMP_1: &str = "1663718400000000100"; // timeUnixNano
// pub const TIMESTAMP_2: &str = "1647648000000000104"; // Example from original JSON
// pub const TIMESTAMP_3: &str = "1647648000000000105"; // Additional Unix timestamps
// pub const OBSERVED_TIMESTAMP_1: &str = "1663718400000000100"; // observedTimeUnixNano

// Log Body Messages (from `body.stringValue`)
// pub const LOG_BODY_1: &str =
//     "Qui voluptates saepe similique ducimus quo rerum eum occaecati praesentium.";
// pub const LOG_BODY_2: &str = "Et neque inventore rerum aut deserunt culpa deleniti rerum et.";
// pub const LOG_BODY_3: &str =
//     "Consequatur autem commodi aspernatur voluptates omnis sed saepe vero aut.";
// pub const LOG_BODY_4: &str =
//     "Magnam doloribus nesciunt quia pariatur sunt excepturi maxime placeat suscipit.";

// // Severity Levels
// pub const SEVERITY_DEBUG: &str = "DEBUG"; // SeverityText: DEBUG (5)
// pub const SEVERITY_INFO: &str = "INFO"; // SeverityText: INFO (9, 13, 17)

// // Status Values (Expanded)
// pub const STATUS_200: i32 = 200;
// pub const STATUS_300: i32 = 300;
// pub const STATUS_400: i32 = 400;
// pub const STATUS_404: i32 = 404;
// pub const STATUS_500: i32 = 500;
// pub const STATUS_503: i32 = 503;

// // Hostnames
// pub const HOSTNAME_1: &str = "host1.mydomain.com";
// pub const HOSTNAME_2: &str = "host2.org";
// pub const HOSTNAME_3: &str = "host3.thedomain.edu";
// pub const HOSTNAME_4: &str = "host4.gov";
// pub const HOSTNAME_5: &str = "host5.retailer.com";

// // Additional Logging Fields (Key-Value)
// pub const ATTR1: &str = "attr1";
// pub const ATTR2: &str = "attr2";

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
    // get random number in range of LOG_BODY_1 - LOG_BODY_4 and select that one
    // For example, get random number between 1 and 4
    // let random_number = rand::thread_rng().gen_range(1..=4);
    // if random_number == 1 {
    //     LOG_BODY_1.to_string()
    // }

    // LOG_BODY_1.to_string()

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
