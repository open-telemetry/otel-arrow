// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only `kernel-processor` guest fixture.
//!
//! This component deliberately contains behavior that normal plugins should
//! not copy: structured initialization failures, panics, infinite loops,
//! oversized allocations and strings, copy floods, and lifecycle clock waits.
//! Host tests select those paths through boolean configuration flags.

#![allow(unexpected_cfgs)]

wit_bindgen::generate!({
    world: "kernel-processor",
    path: "../../wit",
    async: true,
    std_feature,
});

use exports::otel::otap_dataflow_plugin::lifecycle::Guest as Lifecycle;
use exports::otel::otap_dataflow_plugin::lifecycle::InitError;
use exports::otel::otap_dataflow_plugin::processor::Guest as Plugin;
use otel::otap_dataflow_plugin::host_services::{
    LogLevel, counter_add, get_config, host_abi_version, log,
};
use otel::otap_dataflow_plugin::otel_kernels::{AttrScope, Pdata, filter_by_attribute_eq};

struct TestPlugin;

const EXPECTED_HOST_ABI_VERSION: u32 = 2;
const BALLOON_BYTES: usize = 256 * 1024 * 1024;

static mut SPIN_IN_PROCESS: bool = false;
static mut SLEEP_IN_PROCESS: bool = false;
static mut RETAIN_PDATA: bool = false;
static mut RETAINED_PDATA: Option<Pdata> = None;

fn config_flag_set(config: &str, flag: &str) -> bool {
    let compact = format!("\"{flag}\":true");
    let spaced = format!("\"{flag}\": true");
    config.contains(&compact) || config.contains(&spaced)
}

impl Plugin for TestPlugin {
    async fn process(data: Pdata) -> Option<Pdata> {
        if unsafe { SPIN_IN_PROCESS } {
            let mut spin: u64 = 0;
            loop {
                spin = spin.wrapping_add(1);
                core::hint::black_box(spin);
            }
        }
        if unsafe { SLEEP_IN_PROCESS } {
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if unsafe { RETAIN_PDATA } {
            unsafe { RETAINED_PDATA = Some(data) };
            return None;
        }
        Some(
            filter_by_attribute_eq(
                data,
                AttrScope::Record,
                String::from("severity_text"),
                String::from("ERROR"),
            )
            .await,
        )
    }
}

impl Lifecycle for TestPlugin {
    async fn initialize() -> Result<(), InitError> {
        let abi = host_abi_version().await;
        if abi != EXPECTED_HOST_ABI_VERSION {
            return Err(InitError {
                message: format!(
                    "test-plugin: unsupported host-services ABI version {abi} (expected {EXPECTED_HOST_ABI_VERSION})"
                ),
            });
        }

        let config = get_config().await;
        log(
            LogLevel::Info,
            String::from("test-plugin initialize: configuration loaded"),
        )
        .await;

        if config_flag_set(&config, "fail_init") {
            return Err(InitError {
                message: String::from("test-plugin: fail_init=true in plugin config"),
            });
        }
        if config_flag_set(&config, "panic_init") {
            panic!("test-plugin: panic_init=true in plugin config");
        }
        if config_flag_set(&config, "spin") {
            unsafe { SPIN_IN_PROCESS = true };
        }
        if config_flag_set(&config, "sleep_init") {
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if config_flag_set(&config, "sleep_process") {
            unsafe { SLEEP_IN_PROCESS = true };
        }
        if config_flag_set(&config, "retain_pdata") {
            unsafe { RETAIN_PDATA = true };
        }
        if config_flag_set(&config, "balloon") {
            let mut balloon: Vec<u8> = Vec::new();
            return match balloon.try_reserve(BALLOON_BYTES) {
                Err(_) => Err(InitError {
                    message: String::from("test-plugin: host refused an over-cap allocation"),
                }),
                Ok(()) => Err(InitError {
                    message: String::from("test-plugin: host allowed an over-cap allocation"),
                }),
            };
        }

        let large_log = config_flag_set(&config, "large_log");
        let large_counter = config_flag_set(&config, "large_counter");
        if large_log || large_counter {
            let len = 16 * 1024 + usize::from(!config_flag_set(&config, "at_copy_limit"));
            let message = "x".repeat(len);
            if large_log {
                log(LogLevel::Info, message).await;
            } else {
                counter_add(message, 1).await;
            }
        }
        if config_flag_set(&config, "copy_flood") {
            for _ in 0..2_001 {
                counter_add(String::new(), 0).await;
            }
            let name = "x".repeat(16 * 1024);
            for _ in 0..129 {
                counter_add(name.clone(), 0).await;
            }
        }

        counter_add(String::from("test_plugin.initialize"), 1).await;
        Ok(())
    }
}

export!(TestPlugin);
