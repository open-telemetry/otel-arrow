// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

pub fn build_local_runtime(name: &str) -> LocalRuntime {
    Builder::new_current_thread()
        .enable_all()
        .name(name)
        .build_local(LocalOptions::default())
        .expect("failed to build local Tokio runtime")
}
