// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Emits Linux user_events EventHeader samples for manual receiver testing.
//!
//! Run on Linux with a kernel that supports user_events:
//!
//! ```bash
//! cargo run -p otap-df-contrib-nodes \
//!   --features user_events-eventheader \
//!   --example user_events_eventheader_producer
//! ```

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread;
    use std::time::Duration;

    use eventheader_dynamic::{EventBuilder, FieldFormat, Level, Provider};

    let mut args = std::env::args().skip(1);
    let provider_name = args
        .next()
        .unwrap_or_else(|| "otap_df_eventheader_demo".to_owned());
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

    let mut provider = Provider::new(&provider_name, &Provider::new_options());
    let event_set = provider.register_set(Level::Informational, 1);
    if event_set.errno() != 0 {
        return Err(format!(
            "failed to register user_events EventHeader provider `{provider_name}`: errno {}",
            event_set.errno()
        )
        .into());
    }

    // Uncomment for manual progress output while running this example locally.
    // println!("registered user_events:{provider_name}_L4K1");
    // println!("waiting for a listener to enable the tracepoint");

    for _ in 0..100 {
        if event_set.enabled() {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    if !event_set.enabled() {
        return Err(format!(
            "tracepoint `user_events:{provider_name}_L4K1` was not enabled within 10 seconds"
        )
        .into());
    }

    for ci_answer in 0..count {
        let write_errno = EventBuilder::new()
            .reset("CiSmoke", 0)
            .add_str("ci_message", b"hello-from-ci", FieldFormat::Default, 0)
            .add_value("ci_answer", ci_answer, FieldFormat::UnsignedInt, 0)
            .write(&event_set, None, None);
        if write_errno != 0 {
            return Err(format!(
                "failed to write user_events EventHeader sample `{provider_name}`: errno {write_errno}"
            )
            .into());
        }
        // println!("wrote user_events:{provider_name}_L4K1 ci_answer={ci_answer}");
        thread::sleep(Duration::from_millis(interval_ms));
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    panic!("user_events EventHeader producer examples only run on Linux");
}
