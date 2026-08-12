// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::env;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

const EVENT_INTERVAL_ENV: &str = "OTAP_LOCAL_RUNTIME_EVENT_INTERVAL";
const GLOBAL_QUEUE_INTERVAL_ENV: &str = "OTAP_LOCAL_RUNTIME_GLOBAL_QUEUE_INTERVAL";

#[derive(Clone, Copy, Debug, Default)]
struct LocalRuntimeSchedulerConfig {
    event_interval: Option<u32>,
    global_queue_interval: Option<u32>,
}

impl LocalRuntimeSchedulerConfig {
    fn from_env() -> Self {
        Self {
            event_interval: read_positive_u32(EVENT_INTERVAL_ENV),
            global_queue_interval: read_positive_u32(GLOBAL_QUEUE_INTERVAL_ENV),
        }
    }

    fn apply_to(self, builder: &mut Builder) {
        if let Some(value) = self.event_interval {
            let _ = builder.event_interval(value);
        }
        if let Some(value) = self.global_queue_interval {
            let _ = builder.global_queue_interval(value);
        }
    }
}

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

pub fn build_local_runtime(name: &str) -> LocalRuntime {
    let config = LocalRuntimeSchedulerConfig::from_env();
    let mut builder = Builder::new_current_thread();
    let _ = builder.enable_all().name(name);
    config.apply_to(&mut builder);
    builder
        .build_local(LocalOptions::default())
        .expect("failed to build local Tokio runtime")
}
