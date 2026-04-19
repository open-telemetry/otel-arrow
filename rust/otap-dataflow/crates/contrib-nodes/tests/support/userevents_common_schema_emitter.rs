// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg(target_os = "linux")]

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use core_affinity::CoreId;
use eventheader_dynamic as ehd;

pub const PROVIDER_NAME: &str = "myprovider";
pub const TRACEPOINT_NAME: &str = "user_events:myprovider_L2K1";
pub const EVENT_NAME: &str = "my-event-name";
pub const BODY: &str = "This is a test message";
pub const USER_NAME: &str = "otel user";
pub const USER_EMAIL: &str = "otel@opentelemetry.io";
pub const EVENT_ID: u32 = 20;
pub const SEVERITY_NUMBER: i32 = 17;
pub const SEVERITY_TEXT: &str = "ERROR";

pub struct CommonSchemaEmitter {
    _provider: ehd::Provider,
    event_set: Arc<ehd::EventSet>,
}

impl CommonSchemaEmitter {
    pub fn new() -> Self {
        let mut provider = ehd::Provider::new(PROVIDER_NAME, &ehd::Provider::new_options());
        let event_set = provider.register_set(ehd::Level::Error, 1);
        Self {
            _provider: provider,
            event_set,
        }
    }

    pub fn emit_test_log(&self) -> Result<(), i32> {
        let mut builder = ehd::EventBuilder::new();
        let _ = builder
            .reset(EVENT_NAME, 0)
            .add_value("__csver__", 0x400_u16, ehd::FieldFormat::Default, 0)
            .add_struct("PartB", 6, 0)
            .add_str("_typeName", "Log", ehd::FieldFormat::Default, 0)
            .add_str("name", EVENT_NAME, ehd::FieldFormat::Default, 0)
            .add_str("body", BODY, ehd::FieldFormat::Default, 0)
            .add_value(
                "severityNumber",
                SEVERITY_NUMBER as u16,
                ehd::FieldFormat::Default,
                0,
            )
            .add_str("severityText", SEVERITY_TEXT, ehd::FieldFormat::Default, 0)
            .add_value("eventId", EVENT_ID, ehd::FieldFormat::Default, 0)
            .add_struct("PartC", 2, 0)
            .add_str("user_name", USER_NAME, ehd::FieldFormat::Default, 0)
            .add_str("user_email", USER_EMAIL, ehd::FieldFormat::Default, 0);

        let result = builder.write(&self.event_set, None, None);
        if result == 0 { Ok(()) } else { Err(result) }
    }

    pub fn emit_until_delivered(&self, timeout: Duration) -> Result<(), i32> {
        pin_current_thread_to_cpu0();
        let deadline = Instant::now() + timeout;
        let mut last_error = 0;
        while Instant::now() < deadline {
            match self.emit_test_log() {
                Ok(()) => return Ok(()),
                Err(error) => {
                    last_error = error;
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        Err(last_error)
    }
}

fn pin_current_thread_to_cpu0() {
    assert!(
        core_affinity::set_for_current(CoreId { id: 0 }),
        "failed to pin standalone user_events emitter to CPU 0"
    );
}
