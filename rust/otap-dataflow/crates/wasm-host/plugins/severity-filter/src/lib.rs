// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal reference `kernel-processor` guest plugin.
//!
//! Keeps only log records whose `severity_text` equals `"ERROR"` by
//! orchestrating a native host kernel over a host-managed pdata resource.
//! The SDK checks the host ABI before initialization; the plugin logs and
//! records a counter from `initialize`.
//!
//! Built as a Rust-nightly `wasm32-wasip3` component using `std`. It is not a
//! Cargo workspace member.

use wasm_guest_sdk::{
    AttrScope, Lifecycle, LogLevel, Pdata, Plugin, counter_add, export_plugin,
    filter_by_attribute_eq, log,
};

struct SeverityFilter;

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
    async fn initialize() -> Result<(), wasm_guest_sdk::InitError> {
        log(
            LogLevel::Info,
            String::from("severity-filter initialize: configuration loaded"),
        )
        .await;
        counter_add(String::from("severity_filter.initialize"), 1).await;
        Ok(())
    }
}

export_plugin!(SeverityFilter);
