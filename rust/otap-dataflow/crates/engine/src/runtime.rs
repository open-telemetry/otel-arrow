// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime helpers for the engine.

use otel_arrow_dfe_config::engine::LocalRuntimeSettings;
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

    /// Maximum I/O readiness events processed during a nonblocking driver poll.
    ///
    /// Unlike `max_io_events_per_tick`, this only limits polls that do not wait,
    /// normally while runnable tasks remain. A smaller batch leaves excess I/O
    /// events in the kernel and returns to task polling sooner, while idle
    /// polls can still drain larger batches. This may improve responsiveness
    /// under bursts, but can increase driver passes or reduce throughput.
    /// `event_interval` controls when busy polls occur; this controls their
    /// batch size. Leave unset to reuse Tokio's `max_io_events_per_tick` limit.
    max_io_events_per_busy_tick: Option<usize>,

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
        validate_local_runtime_settings(settings)?;

        Ok(Self {
            event_interval: settings.event_interval,
            max_io_events_per_tick: settings.max_io_events_per_tick,
            max_io_events_per_busy_tick: settings.max_io_events_per_busy_tick,
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
        if let Some(value) = self.max_io_events_per_busy_tick {
            let _ = builder.max_io_events_per_busy_tick(value);
        }
        #[cfg(tokio_unstable)]
        if self.poll_time_histogram {
            // Fix the operator-visible bucket labels independently of future Tokio defaults.
            let _ = builder
                .enable_metrics_poll_time_histogram()
                .metrics_poll_time_histogram_configuration(
                    tokio::runtime::HistogramConfiguration::linear(
                        std::time::Duration::from_micros(100),
                        10,
                    ),
                );
        }
    }
}

/// Checks pipeline scheduler overrides against this binary before starting any workers.
pub fn validate_local_runtime_settings(settings: &LocalRuntimeSettings) -> io::Result<()> {
    let mut errors = settings.validation_errors("engine.runtime.local_runtime");
    if settings.poll_time_histogram && !cfg!(tokio_unstable) {
        errors.push(
            "engine.runtime.local_runtime.poll_time_histogram requires tokio_unstable".into(),
        );
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            errors.join("; "),
        ))
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
    use std::rc::Rc;

    fn settings(
        event_interval: Option<u32>,
        max_io_events_per_tick: Option<usize>,
        poll_time_histogram: bool,
    ) -> LocalRuntimeSettings {
        LocalRuntimeSettings {
            event_interval,
            max_io_events_per_tick,
            poll_time_histogram,
            ..Default::default()
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
                max_io_events_per_busy_tick: None,
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

    /// Scenario: a busy I/O batch limit is configured without other scheduler overrides.
    /// Guarantees: validation preserves the independent override and builds a working local runtime.
    #[test]
    fn build_local_runtime_accepts_busy_io_limit() {
        let settings = LocalRuntimeSettings {
            max_io_events_per_busy_tick: Some(8),
            ..Default::default()
        };
        let config = LocalRuntimeSchedulerConfig::from_settings(&settings).unwrap();
        assert_eq!(config.max_io_events_per_busy_tick, Some(8));
        assert_eq!(config.max_io_events_per_tick, None);
        assert_eq!(config.event_interval, None);
        let rt = build_local_runtime("busy-io-test", &settings).unwrap();
        let value = Rc::new(42);
        let task = rt.spawn_local(async move { *value });
        assert_eq!(rt.block_on(task).unwrap(), 42);
    }

    /// Scenario: the busy I/O batch limit is zero in programmatically constructed settings.
    /// Guarantees: runtime construction returns InvalidInput naming the setting instead of panicking.
    #[test]
    fn build_local_runtime_rejects_zero_busy_io_limit() {
        let settings = LocalRuntimeSettings {
            max_io_events_per_busy_tick: Some(0),
            ..Default::default()
        };
        let err = build_local_runtime("invalid-busy-io-test", &settings).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains(
            "engine.runtime.local_runtime.max_io_events_per_busy_tick must be greater than 0"
        ));
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

    /// Scenario: a non-Send task is spawned before the pipeline runtime is driven.
    /// Guarantees: the task retains its Rc state and runs on its owning thread across a yield.
    #[test]
    fn local_runtime_drives_non_send_tasks_on_owner_thread() {
        let rt = build_local_runtime("non-send-test", &LocalRuntimeSettings::default()).unwrap();
        let owner = std::thread::current().id();
        let value = Rc::new(42);
        let task = rt.spawn_local(async move {
            assert_eq!(std::thread::current().id(), owner);
            tokio::task::yield_now().await;
            assert_eq!(std::thread::current().id(), owner);
            *value
        });
        assert_eq!(rt.block_on(task).unwrap(), 42);
    }

    /// Scenario: a local task waits for a Tokio timer with virtual time paused.
    /// Guarantees: the runtime enables its timer driver and completes the joined non-Send task.
    #[test]
    fn local_runtime_drives_timers() {
        let rt = build_local_runtime("timer-test", &LocalRuntimeSettings::default()).unwrap();
        rt.block_on(async {
            tokio::time::pause();
            let value = Rc::new(42);
            let task = tokio::task::spawn_local(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                *value
            });
            assert_eq!(task.await.unwrap(), 42);
        });
    }

    /// Scenario: a local task has started but remains pending when its runtime is dropped.
    /// Guarantees: dropping the runtime cancels the task and releases its task-owned Rc state.
    #[test]
    fn local_runtime_drop_releases_pending_tasks() {
        let rt = build_local_runtime("drop-test", &LocalRuntimeSettings::default()).unwrap();
        let value = Rc::new(42);
        let weak = Rc::downgrade(&value);
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let _task = rt.spawn_local(async move {
            started_tx.send(()).unwrap();
            std::future::pending::<()>().await;
            drop(value);
        });
        rt.block_on(started_rx).unwrap();
        assert!(weak.upgrade().is_some());
        drop(rt);
        assert!(weak.upgrade().is_none());
    }
}
