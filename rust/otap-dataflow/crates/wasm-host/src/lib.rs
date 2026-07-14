// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! WASM host-kernel runtime for OTAP dataflow processor plugins.
//!
//! **Experimental / unstable.** This crate is under active development. The
//! WIT contract (`wit/plugin.wit`) and the host surface are **not stable** and
//! are subject to breaking changes without notice while the binary plugin
//! system is being built out. The runtime is gated behind the off-by-default
//! `wasm` cargo feature; enable it explicitly to compile the wasmtime-based
//! host and register the `wasm_processor` factory.
//!
//! This crate is a thin, end-to-end vertical slice that proves the
//! host-kernel processor-plugin mechanism works against the real
//! `otap-dataflow` engine. See the crate `README.md` and
//! [open-telemetry/otel-arrow#2973] / [#3227] for the broader design.
//!
//! # Design principles enforced here
//!
//! - **Host-kernel orchestration.** The guest only ever holds an opaque
//!   `batch` handle; bulk Arrow data never crosses the WASM boundary. Kernels
//!   run natively on the host's Arrow arrays.
//! - **Valid OTel data.** The kernel surface is OTel-semantic (filter by
//!   attribute), and the reconstructed batch is validated against OTAP schema
//!   invariants (via [`otap_df_pdata`]'s `set`) before forwarding downstream.
//! - **No hot-path compilation.** The `.wasm` component is compiled once at
//!   startup and instantiated into a long-running, per-core instance.
//!
//! [open-telemetry/otel-arrow#2973]: https://github.com/open-telemetry/otel-arrow/issues/2973
//! [#3227]: https://github.com/open-telemetry/otel-arrow/issues/3227

#[cfg(feature = "wasm")]
mod bridge;
#[cfg(feature = "wasm")]
mod host;
#[cfg(feature = "wasm")]
mod processor;

/// Generated component-model bindings for the `kernel-processor` world.
///
/// The macro-generated code does not satisfy this workspace's strict lints
/// (`missing_docs`, `unsafe_code`, ...), so the whole module opts out.
#[cfg(feature = "wasm")]
#[allow(
    missing_docs,
    unsafe_code,
    unused_qualifications,
    clippy::all,
    clippy::pedantic
)]
mod bindings {
    wasmtime::component::bindgen!({
        world: "kernel-processor",
        path: "wit",
        with: {
            "otel:otap-dataflow-plugin/otel-kernels@0.1.0.batch": crate::host::HostBatchData,
        },
    });
}

#[cfg(feature = "wasm")]
pub use processor::{
    WASM_PROCESSOR_FACTORY, WASM_PROCESSOR_URN, WasmProcessor, WasmProcessorConfig,
};
