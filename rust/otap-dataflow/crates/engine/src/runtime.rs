// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime helpers for the engine.

use otap_df_config::engine::LocalRuntimeSettings;
use std::io;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

#[derive(Debug, Default, Eq, PartialEq)]
struct LocalRuntimeSchedulerConfig {
    event_interval: Option<u32>,
    max_io_events_per_tick: Option<usize>,
    poll_time_histogram: bool,
}

impl LocalRuntimeSchedulerConfig {
    fn from_settings(settings: &LocalRuntimeSettings) -> io::Result<Self> {
        if settings.poll_time_histogram && !cfg!(tokio_unstable) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "engine.runtime.local_runtime.poll_time_histogram requires tokio_unstable",
            ));
        }

        Ok(Self {
            event_interval: validate_positive_interval(
                "engine.runtime.local_runtime.event_interval",
                settings.event_interval,
            )?,
            max_io_events_per_tick: validate_positive_count(
                "engine.runtime.local_runtime.max_io_events_per_tick",
                settings.max_io_events_per_tick,
            )?,
            poll_time_histogram: settings.poll_time_histogram,
        })
    }

    fn apply_to(&self, builder: &mut Builder) {
        if let Some(value) = self.event_interval {
            let _ = builder.event_interval(value);
        }
        if let Some(value) = self.max_io_events_per_tick {
            let _ = builder.max_io_events_per_tick(value);
        }
        #[cfg(tokio_unstable)]
        if self.poll_time_histogram {
            let _ = builder.enable_metrics_poll_time_histogram();
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

fn validate_positive_count(name: &str, value: Option<usize>) -> io::Result<Option<usize>> {
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

    fn settings(
        event_interval: Option<u32>,
        max_io_events_per_tick: Option<usize>,
        poll_time_histogram: bool,
    ) -> LocalRuntimeSettings {
        LocalRuntimeSettings {
            event_interval,
            max_io_events_per_tick,
            poll_time_histogram,
        }
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
            LocalRuntimeSchedulerConfig::from_settings(&settings(Some(127), Some(512), false))
                .unwrap(),
            LocalRuntimeSchedulerConfig {
                event_interval: Some(127),
                max_io_events_per_tick: Some(512),
                poll_time_histogram: false,
            }
        );
    }

    #[test]
    fn scheduler_config_rejects_zero_event_interval() {
        let err = LocalRuntimeSchedulerConfig::from_settings(&settings(Some(0), None, false))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("must be greater than 0"));
    }

    #[test]
    fn scheduler_config_rejects_zero_max_io_events_per_tick() {
        let err = LocalRuntimeSchedulerConfig::from_settings(&settings(None, Some(0), false))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("must be greater than 0"));
    }

    #[cfg(tokio_unstable)]
    #[test]
    fn scheduler_config_accepts_poll_time_histogram_with_tokio_unstable() {
        let config = LocalRuntimeSchedulerConfig::from_settings(&settings(None, None, true))
            .expect("tokio_unstable build should accept histogram config");
        assert!(config.poll_time_histogram);
    }

    #[cfg(not(tokio_unstable))]
    #[test]
    fn scheduler_config_rejects_poll_time_histogram_without_tokio_unstable() {
        let err =
            LocalRuntimeSchedulerConfig::from_settings(&settings(None, None, true)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("requires tokio_unstable"));
    }

    #[test]
    fn build_local_runtime_accepts_default_settings() {
        let _rt = build_local_runtime("test-runtime", &LocalRuntimeSettings::default()).unwrap();
    }
}
