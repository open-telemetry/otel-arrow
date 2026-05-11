// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime helpers for the engine.

use otap_df_config::engine::LocalRuntimeSettings;
use std::io;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

#[derive(Debug, Default, Eq, PartialEq)]
struct LocalRuntimeSchedulerConfig {
    event_interval: Option<u32>,
}

impl LocalRuntimeSchedulerConfig {
    fn from_settings(settings: &LocalRuntimeSettings) -> io::Result<Self> {
        Ok(Self {
            event_interval: validate_positive_interval(
                "engine.runtime.local_runtime.event_interval",
                settings.event_interval,
            )?,
        })
    }

    fn apply_to(&self, builder: &mut Builder) {
        if let Some(value) = self.event_interval {
            let _ = builder.event_interval(value);
        }
    }
}

fn validate_positive_interval(name: &str, value: Option<u32>) -> io::Result<Option<u32>> {
    match value {
        Some(0) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} must be greater than 0"),
        )),
        value => Ok(value),
    }
}

/// Builds a named local runtime for engine-owned `!Send` tasks.
///
/// Scheduler tuning knobs default to Tokio's values unless configured at the
/// engine level.
pub(crate) fn build_local_runtime(
    name: impl Into<String>,
    settings: &LocalRuntimeSettings,
) -> io::Result<LocalRuntime> {
    let scheduler_config = LocalRuntimeSchedulerConfig::from_settings(settings)?;
    let mut builder = Builder::new_current_thread();
    let _ = builder.enable_all().name(name);
    scheduler_config.apply_to(&mut builder);
    builder.build_local(LocalOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(event_interval: Option<u32>) -> LocalRuntimeSettings {
        LocalRuntimeSettings { event_interval }
    }

    #[test]
    fn scheduler_config_defaults_to_tokio_defaults() {
        assert_eq!(
            LocalRuntimeSchedulerConfig::from_settings(&LocalRuntimeSettings::default()).unwrap(),
            LocalRuntimeSchedulerConfig::default()
        );
    }

    #[test]
    fn scheduler_config_reads_positive_event_interval() {
        assert_eq!(
            LocalRuntimeSchedulerConfig::from_settings(&settings(Some(127))).unwrap(),
            LocalRuntimeSchedulerConfig {
                event_interval: Some(127),
            }
        );
    }

    #[test]
    fn scheduler_config_rejects_zero_event_interval() {
        let err = LocalRuntimeSchedulerConfig::from_settings(&settings(Some(0))).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("must be greater than 0"));
    }

    #[test]
    fn build_local_runtime_accepts_default_settings() {
        let _rt = build_local_runtime("test-runtime", &LocalRuntimeSettings::default()).unwrap();
    }
}
