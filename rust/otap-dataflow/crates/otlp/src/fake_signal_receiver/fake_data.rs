// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: Apache-2.0

//!
//! provides fake data to use for various fields in a OTLP signal
//!

use otel_arrow_rust::pdata::{SpanID, TraceID};
use rand::Rng;

use std::time::{SystemTime, UNIX_EPOCH};

const MAX_NS_DELAY: u64 = 10000;
/// default scope name to use for fake signals
const SCOPE_NAME: &str = "fake_signal";
/// default scope version to use for fake signals
const SCOPE_VERSION: &str = "1.0.0";

const TRACE_ID_LENGTH: usize = 16;
const SPAN_ID_LENGTH: usize = 8;

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

// /// provide data for the status field
// #[must_use]
// pub fn get_status() -> Status {
//     let mut rng = rand::rng();
//     let option: usize = rng.random_range(0..STATUS_CODES.len());
//     let status = STATUS_CODES[option];

//     Status {
//         code: status as i32,
//         message: status.as_str_name().to_string(),
//     }
// }

/// provide double value
#[must_use]
pub fn get_double_value() -> f64 {
    rand::random()
}

/// provides random u64 value for various fields
#[must_use]
pub fn get_int_value() -> u64 {
    rand::random()
}

/// generate a unique span id
#[must_use]
pub fn gen_span_id() -> SpanID {
    let mut byte_array: [u8; SPAN_ID_LENGTH] = [0; SPAN_ID_LENGTH];

    // generate random byte array
    rand::rng().fill(&mut byte_array);

    SpanID::new(&byte_array)
}

/// generate a unique trace id
#[must_use]
pub fn gen_trace_id() -> TraceID {
    let mut byte_array: [u8; TRACE_ID_LENGTH] = [0; TRACE_ID_LENGTH];

    // generate random byte array
    rand::rng().fill(&mut byte_array);

    TraceID::new(&byte_array)
}

/// provides the current unix nano time
#[must_use]
pub fn current_time() -> u64 {
    let now = SystemTime::now();
    // Calculate the duration since the UNIX epoch
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            // Convert seconds and nanoseconds to a single u64 value
            duration.as_nanos() as u64
        }
        Err(_) => {
            // Return 0 nanoseconds if the system clock is before UNIX epoch
            0u64
        }
    }
}

/// generates a delay in nano seconds
#[must_use]
pub fn delay() -> u64 {
    // generate a random delay
    MAX_NS_DELAY
}
