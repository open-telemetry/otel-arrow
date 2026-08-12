// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime helpers for the engine.

use otap_df_config::engine::LocalRuntimeSettings;
use std::io;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

/// Validated scheduler settings applied to each pipeline-local Tokio runtime.
///
/// Optional values deliberately remain unset unless an operator overrides
/// them. This lets [`Builder`] select the defaults appropriate for the Tokio
/// version in use. These controls have no universally optimal values: tune
/// them only against a representative workload while observing throughput,
/// CPU use, and tail latency.
#[derive(Debug, Default, Eq, PartialEq)]
struct LocalRuntimeSchedulerConfig {
    /// Number of scheduler ticks between checks for external events.
    ///
    /// A tick is approximately one task `poll`. Smaller values give timers and
    /// I/O readiness higher priority when the ready-task queue stays busy, at
    /// the cost of more frequent driver synchronization and syscalls. Larger
    /// values can reduce that overhead when tasks poll quickly and yield
    /// cooperatively, but may delay external wakeups under sustained load.
    /// Leave unset for Tokio's default; override only when measurements show a
    /// throughput-versus-readiness-latency tradeoff worth changing.
    event_interval: Option<u32>,

    /// Maximum number of I/O readiness events processed in one scheduler tick.
    ///
    /// Larger values can drain I/O bursts in fewer driver passes, but may keep
    /// the runtime in the I/O driver longer before it resumes polling ready
    /// tasks. Smaller values return to task polling sooner, while potentially
    /// requiring more driver passes to drain a burst. Leave unset for Tokio's
    /// default and tune only for measured burst-drain or task-latency issues.
    max_io_events_per_tick: Option<usize>,

    /// Whether Tokio records the distribution of time spent in task polls.
    ///
    /// Enable this temporarily to identify long-running polls and tasks that
    /// do not yield cooperatively. Tokio reads the clock twice per task poll,
    /// so collection can add measurable overhead and is not intended as a
    /// default production setting. It also requires a `tokio_unstable` build;
    /// runtime construction rejects the setting otherwise.
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

    /// Scenario: default engine settings are translated into scheduler configuration.
    /// Guarantees: every override remains disabled so Tokio selects its own defaults.
    #[test]
    fn scheduler_config_defaults_to_tokio_defaults() {
        assert_eq!(
            LocalRuntimeSchedulerConfig::from_settings(&LocalRuntimeSettings::default()).unwrap(),
            LocalRuntimeSchedulerConfig::default()
        );
    }

    /// Scenario: positive scheduler values and a disabled histogram are configured.
    /// Guarantees: validation preserves each explicit value without modification.
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

    /// Scenario: the scheduler event interval is configured as zero.
    /// Guarantees: validation returns `InvalidInput` before passing the value to Tokio.
    #[test]
    fn scheduler_config_rejects_zero_event_interval() {
        let err = LocalRuntimeSchedulerConfig::from_settings(&settings(Some(0), None, false))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("must be greater than 0"));
    }

    /// Scenario: the per-tick I/O event limit is configured as zero.
    /// Guarantees: validation returns `InvalidInput` before runtime construction.
    #[test]
    fn scheduler_config_rejects_zero_max_io_events_per_tick() {
        let err = LocalRuntimeSchedulerConfig::from_settings(&settings(None, Some(0), false))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("must be greater than 0"));
    }

    /// Scenario: poll-time histograms are enabled in a `tokio_unstable` build.
    /// Guarantees: validation accepts and retains the diagnostic setting.
    #[cfg(tokio_unstable)]
    #[test]
    fn scheduler_config_accepts_poll_time_histogram_with_tokio_unstable() {
        let config = LocalRuntimeSchedulerConfig::from_settings(&settings(None, None, true))
            .expect("tokio_unstable build should accept histogram config");
        assert!(config.poll_time_histogram);
    }

    /// Scenario: poll-time histograms are enabled without `tokio_unstable` support.
    /// Guarantees: validation rejects the unsupported diagnostic setting as invalid input.
    #[cfg(not(tokio_unstable))]
    #[test]
    fn scheduler_config_rejects_poll_time_histogram_without_tokio_unstable() {
        let err =
            LocalRuntimeSchedulerConfig::from_settings(&settings(None, None, true)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("requires tokio_unstable"));
    }

    /// Scenario: a pipeline runtime is built with default engine settings.
    /// Guarantees: Tokio successfully constructs a `LocalRuntime` without overrides.
    #[test]
    fn build_local_runtime_accepts_default_settings() {
        let _rt = build_local_runtime("test-runtime", &LocalRuntimeSettings::default()).unwrap();
    }
}
