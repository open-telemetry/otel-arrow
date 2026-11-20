// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only `kernel-processor` guest fixture.
//!
//! This component deliberately contains behavior that normal plugins should
//! not copy: structured initialization failures, panics, infinite loops,
//! oversized allocations and strings, copy floods, and lifecycle clock waits.
//! Host tests select those paths through boolean configuration flags.

use serde::Deserialize;
use std::sync::Mutex;
use wasm_guest_sdk::{
    AttrScope, InitError, Lifecycle, LogLevel, Output, OutputInfoError, Pdata, Plugin, clone_pdata,
    counter_add, export_plugin, filter_by_attribute_eq, get_config, get_output_info, log,
};

struct TestPlugin;

const BALLOON_BYTES: usize = 256 * 1024 * 1024;

struct PluginState {
    spin_in_process: bool,
    sleep_in_process: bool,
    retain_pdata: bool,
    retained_pdata: Option<Pdata>,
    outputs: Option<Vec<Option<String>>>,
    oversized_output_port: bool,
    discover_outputs: Option<ExpectedOutputInfo>,
}

#[derive(Clone, Deserialize)]
struct ExpectedOutputInfo {
    connected_ports: Vec<String>,
    default_port: Option<String>,
    max_outputs_per_call: u32,
}

// Keep state in component memory across distinct wasip3 async task contexts.
// The host serializes calls for each store; the mutex also keeps accesses safe
// if that runtime behavior changes.
static PLUGIN_STATE: Mutex<PluginState> = Mutex::new(PluginState {
    spin_in_process: false,
    sleep_in_process: false,
    retain_pdata: false,
    retained_pdata: None,
    outputs: None,
    oversized_output_port: false,
    discover_outputs: None,
});

#[derive(Deserialize)]
#[serde(default)]
struct PluginConfig {
    fail_init: bool,
    panic_init: bool,
    spin: bool,
    sleep_init: bool,
    sleep_process: bool,
    retain_pdata: bool,
    balloon: bool,
    large_log: bool,
    large_counter: bool,
    at_copy_limit: bool,
    copy_flood: bool,
    outputs: Option<Vec<Option<String>>>,
    oversized_output_port: bool,
    discover_outputs: Option<ExpectedOutputInfo>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            fail_init: false,
            panic_init: false,
            spin: false,
            sleep_init: false,
            sleep_process: false,
            retain_pdata: false,
            balloon: false,
            large_log: false,
            large_counter: false,
            at_copy_limit: false,
            copy_flood: false,
            outputs: None,
            oversized_output_port: false,
            discover_outputs: None,
        }
    }
}

impl Plugin for TestPlugin {
    async fn process(data: Pdata) -> Vec<Output> {
        let (
            spin_in_process,
            sleep_in_process,
            retain_pdata,
            outputs,
            oversized_output_port,
            discovery,
        ) = {
            let state = PLUGIN_STATE.lock().expect("plugin state mutex poisoned");
            (
                state.spin_in_process,
                state.sleep_in_process,
                state.retain_pdata,
                state.outputs.clone(),
                state.oversized_output_port,
                state.discover_outputs.clone(),
            )
        };
        if spin_in_process {
            let mut spin: u64 = 0;
            loop {
                spin = spin.wrapping_add(1);
                core::hint::black_box(spin);
            }
        }
        if sleep_in_process {
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if retain_pdata {
            PLUGIN_STATE
                .lock()
                .expect("plugin state mutex poisoned")
                .retained_pdata = Some(data);
            return Vec::new();
        }
        if oversized_output_port {
            return vec![Output {
                port: Some("p".repeat(32 * 1024)),
                data,
            }];
        }
        if let Some(expected) = discovery {
            let info = get_output_info()
                .await
                .expect("wiring available during process");
            assert_eq!(info.connected_ports, expected.connected_ports);
            assert_eq!(info.default_port, expected.default_port);
            assert_eq!(info.max_outputs_per_call, expected.max_outputs_per_call);
            let mut outputs = Vec::new();
            for port in info
                .connected_ports
                .into_iter()
                .take(info.max_outputs_per_call as usize)
            {
                outputs.push(Output {
                    port: Some(port),
                    data: clone_pdata(&data).await,
                });
            }
            return outputs;
        }
        if let Some(ports) = outputs {
            let mut outputs = Vec::with_capacity(ports.len());
            for port in ports {
                outputs.push(Output {
                    port,
                    data: clone_pdata(&data).await,
                });
            }
            return outputs;
        }
        vec![Output {
            port: None,
            data: filter_by_attribute_eq(
                data,
                AttrScope::Record,
                String::from("severity_text"),
                String::from("ERROR"),
            )
            .await,
        }]
    }
}

impl Lifecycle for TestPlugin {
    async fn initialize() -> Result<(), InitError> {
        let config = get_config::<Option<PluginConfig>>()
            .await
            .map_err(|error| InitError {
                message: format!("test-plugin: {error}"),
            })?
            .unwrap_or_default();
        log(
            LogLevel::Info,
            String::from("test-plugin initialize: configuration loaded"),
        )
        .await;

        if config.fail_init {
            return Err(InitError {
                message: String::from("test-plugin: fail_init=true in plugin config"),
            });
        }
        if config.panic_init {
            panic!("test-plugin: panic_init=true in plugin config");
        }
        if config.sleep_init {
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if config.discover_outputs.is_some() {
            assert!(matches!(
                get_output_info().await,
                Err(OutputInfoError::Unavailable)
            ));
        }
        {
            let mut state = PLUGIN_STATE.lock().expect("plugin state mutex poisoned");
            state.spin_in_process = config.spin;
            state.sleep_in_process = config.sleep_process;
            state.retain_pdata = config.retain_pdata;
            state.outputs = config.outputs;
            state.oversized_output_port = config.oversized_output_port;
            state.discover_outputs = config.discover_outputs;
        }
        if config.balloon {
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

        if config.large_log || config.large_counter {
            let len = 16 * 1024 + usize::from(!config.at_copy_limit);
            let message = "x".repeat(len);
            if config.large_log {
                log(LogLevel::Info, message).await;
            } else {
                counter_add(message, 1).await;
            }
        }
        if config.copy_flood {
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

export_plugin!(TestPlugin);
