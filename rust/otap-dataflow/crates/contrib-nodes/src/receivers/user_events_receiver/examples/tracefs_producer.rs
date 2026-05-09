// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Emits a Linux user_events tracefs sample for manual receiver testing.
//!
//! Run on Linux with a kernel that supports user_events:
//!
//! ```bash
//! cargo run -p otap-df-contrib-nodes \
//!   --features user_events-receiver \
//!   --example user_events_tracefs_producer
//! ```

#![allow(unsafe_code)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::CString;
    use std::thread;
    use std::time::Duration;

    let mut args = std::env::args().skip(1);
    let event_name = args
        .next()
        .unwrap_or_else(|| "otap_df_tracefs_demo".to_owned());
    let count = args
        .next()
        .map(|value| value.parse::<u32>())
        .transpose()?
        .unwrap_or(5);
    let interval_ms = args
        .next()
        .map(|value| value.parse::<u64>())
        .transpose()?
        .unwrap_or(250);

    let definition = CString::new(format!("{event_name} u32 ci_answer; char[14] ci_message"))?;
    let tracepoint_state = Box::pin(tracepoint::TracepointState::new(0));
    let register_errno = unsafe { tracepoint_state.as_ref().register(&definition) };
    if register_errno != 0 {
        return Err(format!(
            "failed to register user_events tracepoint `{event_name}`: errno {register_errno}"
        )
        .into());
    }

    // Uncomment for manual progress output while running this example locally.
    // println!("registered user_events:{event_name}");
    // println!("waiting for a listener to enable the tracepoint");

    for _ in 0..100 {
        if tracepoint_state.enabled() {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    if !tracepoint_state.enabled() {
        return Err(format!(
            "tracepoint `user_events:{event_name}` was not enabled within 10 seconds"
        )
        .into());
    }

    let ci_message = *b"hello-from-ci\0";
    for ci_answer in 0..count {
        let mut data = [
            tracepoint::EventDataDescriptor::zero(),
            tracepoint::EventDataDescriptor::from_value(&ci_answer),
            tracepoint::EventDataDescriptor::from_bytes(&ci_message),
        ];
        let write_errno = tracepoint_state.write(&mut data);
        if write_errno != 0 {
            return Err(format!(
                "failed to write user_events sample `{event_name}`: errno {write_errno}"
            )
            .into());
        }
        // println!("wrote user_events:{event_name} ci_answer={ci_answer}");
        thread::sleep(Duration::from_millis(interval_ms));
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    panic!("user_events tracefs producer examples only run on Linux");
}
