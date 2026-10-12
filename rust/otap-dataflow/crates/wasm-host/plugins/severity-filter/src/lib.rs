// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal reference `kernel-processor` guest plugin.
//!
//! Keeps only log records whose `severity_text` equals `"ERROR"` by
//! orchestrating a native host kernel over a host-managed pdata resource.
//! It also demonstrates the initialization and host-services contract:
//! `initialize` checks the host ABI, logs, and records a counter.
//!
//! Built as a Rust-nightly `wasm32-wasip3` component using `std`. It is not a
//! Cargo workspace member.

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
use otel::otap_dataflow_plugin::host_services::{LogLevel, counter_add, host_abi_version, log};
use otel::otap_dataflow_plugin::otel_kernels::{AttrScope, Pdata, filter_by_attribute_eq};

struct SeverityFilter;

const EXPECTED_HOST_ABI_VERSION: u32 = 2;

impl Plugin for SeverityFilter {
    async fn process(data: Pdata) -> Option<Pdata> {
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

impl Lifecycle for SeverityFilter {
    async fn initialize() -> Result<(), InitError> {
        let abi = host_abi_version().await;
        if abi != EXPECTED_HOST_ABI_VERSION {
            return Err(InitError {
                message: format!(
                    "severity-filter: unsupported host-services ABI version {abi} (expected {EXPECTED_HOST_ABI_VERSION})"
                ),
            });
        }

        log(
            LogLevel::Info,
            String::from("severity-filter initialize: configuration loaded"),
        )
        .await;
        counter_add(String::from("severity_filter.initialize"), 1).await;
        Ok(())
    }
}

export!(SeverityFilter);
