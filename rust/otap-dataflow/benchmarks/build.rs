// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Selects the request-context benchmark adapter for the checked-out representation.

use std::path::Path;

fn main() {
    const PACKED_CONTEXT: &str = "../crates/otap/src/context_bytes.rs";

    println!("cargo:rustc-check-cfg=cfg(packed_pdata_context)");
    println!("cargo:rerun-if-changed={PACKED_CONTEXT}");
    if Path::new(PACKED_CONTEXT).is_file() {
        println!("cargo:rustc-cfg=packed_pdata_context");
    }
}
