// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reference `kernel-processor` guest plugin.
//!
//! Keeps only log records whose `severity_text` equals `"ERROR"`, by
//! orchestrating the host `filter-by-attribute-eq` kernel over a host-managed
//! pdata resource handle. No bulk Arrow data crosses the WASM boundary.
//!
//! Built as a `no_std`, `wasm32-wasip2` component. Not a Cargo workspace
//! member; compiled on demand by the wasm-host integration test.

#![no_std]

extern crate alloc;

// A tiny global allocator so the generated bindings (which use `alloc`) work in
// a `no_std` component.
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

wit_bindgen::generate!({
    world: "kernel-processor",
    path: "../../wit",
});

// wit-bindgen names the exported-interface trait `Guest`; alias it to `Plugin`
// to model the author-facing naming a plugin implementer would expect.
use exports::otel::otap_dataflow_plugin::processor::Guest as Plugin;
use otel::otap_dataflow_plugin::otel_kernels::{AttrScope, Pdata, filter_by_attribute_eq};

struct SeverityFilter;

impl Plugin for SeverityFilter {
    fn process(data: Pdata) -> Option<Pdata> {
        Some(filter_by_attribute_eq(
            data,
            AttrScope::Record,
            "severity_text",
            "ERROR",
        ))
    }
}

export!(SeverityFilter);
