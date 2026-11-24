// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local runtime for benchmarks with optional environment-based scheduler overrides.

use std::env;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

const EVENT_INTERVAL_ENV: &str = "OTAP_LOCAL_RUNTIME_EVENT_INTERVAL";
const GLOBAL_QUEUE_INTERVAL_ENV: &str = "OTAP_LOCAL_RUNTIME_GLOBAL_QUEUE_INTERVAL";

fn read_positive_u32(key: &str) -> Option<u32> {
    let raw = match env::var(key) {
        Ok(value) if value.is_empty() => return None,
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return None,
        Err(err) => panic!("{key} is not valid UTF-8: {err}"),
    };
    let value = raw
        .parse::<u32>()
        .unwrap_or_else(|err| panic!("{key} must be a positive u32, got {raw:?}: {err}"));
    assert!(value > 0, "{key} must be greater than zero");
    Some(value)
}

/// Builds a named benchmark runtime. Unset or empty overrides leave Tokio's defaults intact.
#[must_use]
pub fn build_local_runtime(name: &str) -> LocalRuntime {
    let mut builder = Builder::new_current_thread();
    let _ = builder.enable_all().name(name);
    if let Some(interval) = read_positive_u32(EVENT_INTERVAL_ENV) {
        let _ = builder.event_interval(interval);
    }
    if let Some(interval) = read_positive_u32(GLOBAL_QUEUE_INTERVAL_ENV) {
        let _ = builder.global_queue_interval(interval);
    }
    builder
        .build_local(LocalOptions::default())
        .expect("failed to build local Tokio runtime")
}
