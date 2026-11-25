// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime helpers for the engine.

use otel_arrow_dfe_config::engine::TokioRuntimeSettings;
use std::io;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

/// Checks pipeline scheduler overrides against this binary before starting any workers.
pub fn validate_local_runtime_settings(settings: &TokioRuntimeSettings) -> io::Result<()> {
    let mut errors = settings.validation_errors("engine.runtime.tokio");
    if settings.task_poll_time_histogram && !cfg!(tokio_unstable) {
        errors.push("engine.runtime.tokio.task_poll_time_histogram requires tokio_unstable".into());
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
    settings: &TokioRuntimeSettings,
) -> io::Result<LocalRuntime> {
    validate_local_runtime_settings(settings)?;
    let mut builder = Builder::new_current_thread();
    let _ = builder.enable_all().name(name);
    if let Some(interval) = settings.event_interval {
        let _ = builder.event_interval(interval);
    }
    if let Some(limit) = settings.max_io_events_per_tick {
        let _ = builder.max_io_events_per_tick(limit);
    }
    if let Some(limit) = settings.max_io_events_per_busy_tick {
        let _ = builder.max_io_events_per_busy_tick(limit);
    }
    #[cfg(tokio_unstable)]
    if settings.task_poll_time_histogram {
        // Keep the operator-visible bucket ranges independent of Tokio's defaults.
        let _ = builder
            .enable_metrics_poll_time_histogram()
            .metrics_poll_time_histogram_configuration(
                tokio::runtime::HistogramConfiguration::linear(
                    std::time::Duration::from_micros(100),
                    10,
                ),
            );
    }
    builder.build_local(LocalOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    /// Scenario: a pipeline builds a local runtime with explicit scheduler and I/O intervals.
    /// Guarantees: the tuned runtime can drive a local task to completion.
    #[test]
    fn build_local_runtime_accepts_scheduler_overrides() {
        let settings = TokioRuntimeSettings {
            event_interval: Some(127),
            max_io_events_per_tick: Some(512),
            ..Default::default()
        };
        let rt = build_local_runtime("scheduler-test", &settings).unwrap();
        rt.block_on(rt.spawn_local(async {})).unwrap();
    }

    /// Scenario: the scheduler event interval is configured as zero.
    /// Guarantees: runtime construction names the invalid setting instead of passing it to Tokio.
    #[test]
    fn build_local_runtime_rejects_zero_event_interval() {
        let settings = TokioRuntimeSettings {
            event_interval: Some(0),
            ..Default::default()
        };
        let err = build_local_runtime("invalid-interval-test", &settings).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(
            err.to_string()
                .contains("engine.runtime.tokio.event_interval must be greater than 0")
        );
    }

    /// Scenario: the per-tick I/O event limit is configured as zero.
    /// Guarantees: runtime construction names the invalid limit instead of allocating a buffer.
    #[test]
    fn build_local_runtime_rejects_zero_max_io_events_per_tick() {
        let settings = TokioRuntimeSettings {
            max_io_events_per_tick: Some(0),
            ..Default::default()
        };
        let err = build_local_runtime("invalid-io-test", &settings).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(
            err.to_string()
                .contains("engine.runtime.tokio.max_io_events_per_tick must be greater than 0")
        );
    }

    /// Scenario: a busy I/O batch limit is configured without other scheduler overrides.
    /// Guarantees: the runtime accepts the independent override and can drive a non-Send task.
    #[test]
    fn build_local_runtime_accepts_busy_io_limit() {
        let settings = TokioRuntimeSettings {
            max_io_events_per_busy_tick: Some(8),
            ..Default::default()
        };
        let rt = build_local_runtime("busy-io-test", &settings).unwrap();
        let value = Rc::new(42);
        let task = rt.spawn_local(async move { *value });
        assert_eq!(rt.block_on(task).unwrap(), 42);
    }

    /// Scenario: the busy I/O batch limit is zero in programmatically constructed settings.
    /// Guarantees: runtime construction returns InvalidInput naming the setting instead of panicking.
    #[test]
    fn build_local_runtime_rejects_zero_busy_io_limit() {
        let settings = TokioRuntimeSettings {
            max_io_events_per_busy_tick: Some(0),
            ..Default::default()
        };
        let err = build_local_runtime("invalid-busy-io-test", &settings).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains(
                "engine.runtime.tokio.max_io_events_per_busy_tick must be greater than 0"
            )
        );
    }

    /// Scenario: a pipeline requests poll-time diagnostics on a build with Tokio metrics.
    /// Guarantees: the constructed runtime actually records poll-time histograms.
    #[cfg(tokio_unstable)]
    #[test]
    fn build_local_runtime_enables_poll_time_histogram() {
        let settings = TokioRuntimeSettings {
            task_poll_time_histogram: true,
            ..Default::default()
        };
        let rt = build_local_runtime("histogram-test", &settings).unwrap();
        assert!(rt.handle().metrics().poll_time_histogram_enabled());
    }

    /// Scenario: a pipeline requests poll-time diagnostics without Tokio metrics support.
    /// Guarantees: runtime construction rejects the unsupported diagnostic as invalid input.
    #[cfg(not(tokio_unstable))]
    #[test]
    fn build_local_runtime_rejects_unsupported_poll_time_histogram() {
        let settings = TokioRuntimeSettings {
            task_poll_time_histogram: true,
            ..Default::default()
        };
        let err = build_local_runtime("unsupported-histogram-test", &settings).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("requires tokio_unstable"));
    }

    /// Scenario: a pipeline runtime is built with default engine settings.
    /// Guarantees: Tokio constructs a local runtime without enabling poll-time diagnostics.
    #[test]
    fn build_local_runtime_accepts_default_settings() {
        let rt = build_local_runtime("test-runtime", &TokioRuntimeSettings::default()).unwrap();
        assert_eq!(rt.handle().metrics().num_workers(), 1);
        #[cfg(tokio_unstable)]
        assert!(!rt.handle().metrics().poll_time_histogram_enabled());
    }

    /// Scenario: a non-Send task is spawned before the pipeline runtime is driven.
    /// Guarantees: the task retains its Rc state and runs on its owning thread across a yield.
    #[test]
    fn local_runtime_drives_non_send_tasks_on_owner_thread() {
        let rt = build_local_runtime("non-send-test", &TokioRuntimeSettings::default()).unwrap();
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
        let rt = build_local_runtime("timer-test", &TokioRuntimeSettings::default()).unwrap();
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
        let rt = build_local_runtime("drop-test", &TokioRuntimeSettings::default()).unwrap();
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
