// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime helpers for the engine.

use std::io;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

/// Builds a named local runtime for engine-owned `!Send` tasks.
///
/// The scheduler tuning knobs are intentionally left at Tokio defaults. They
/// should only be changed with benchmark evidence for the target workload.
pub(crate) fn build_local_runtime(name: impl Into<String>) -> io::Result<LocalRuntime> {
    Builder::new_current_thread()
        .enable_all()
        .name(name)
        .build_local(LocalOptions::default())
}
