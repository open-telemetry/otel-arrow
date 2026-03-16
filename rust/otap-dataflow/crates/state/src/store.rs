// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs defining an event-driven observed state store.

use crate::ObservedEventRingBuffer;
use crate::error::Error;
use crate::phase::PipelinePhase;
use crate::pipeline_rt_status::{ApplyOutcome, PipelineRuntimeStatus};
use crate::pipeline_status::PipelineStatus;
use otap_df_config::PipelineKey;
use otap_df_config::health::HealthPolicy;
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_telemetry::event::{EngineEvent, EventType, ObservedEvent, ObservedEventReporter};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::self_tracing::{AnsiCode, ConsoleWriter, LogContext, StyledBufWriter};
use otap_df_telemetry::{otel_error, otel_info};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

const RECENT_EVENTS_CAPACITY: usize = 10;

/// Event-driven observed state store representing what we know about the state of the
/// pipelines (DAG executors) controlled by the controller.
///
/// This store is cloneable and thread-safe, allowing multiple threads to record events
/// concurrently or read the current state.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateStore {
    #[serde(skip)]
    default_health_policy: HealthPolicy,

    #[serde(skip)]
    health_policies: Arc<Mutex<HashMap<PipelineKey, HealthPolicy>>>,

    /// Bounded channel for log (observational) events — lossy under backpressure.
    #[serde(skip)]
    sender: flume::Sender<ObservedEvent>,

    #[serde(skip)]
    receiver: flume::Receiver<ObservedEvent>,

    /// Unbounded channel for engine (lifecycle) events — reliable, never drops.
    #[serde(skip)]
    engine_sender: flume::Sender<EngineEvent>,

    #[serde(skip)]
    engine_receiver: flume::Receiver<EngineEvent>,

    /// Console is used only for Log events when this component acts
    /// as the ConsoleAsync consumer and logs to the console.
    #[serde(skip)]
    console: ConsoleWriter,

    /// Telemetry registry for resolving entity keys to attributes.
    #[serde(skip)]
    registry: TelemetryRegistryHandle,

    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

/// A handle to the observed state, suitable for serialization and external consumption.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateHandle {
    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

impl ObservedStateHandle {
    /// Returns a cloned snapshot of the current pipeline statuses.
    #[must_use]
    pub fn snapshot(&self) -> HashMap<PipelineKey, PipelineStatus> {
        match self.pipelines.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                otel_error!(
                    "state.mutex_poisoned",
                    action = "continuing with stale snapshot"
                );
                poisoned.into_inner().clone()
            }
        }
    }
}

impl ObservedStateStore {
    /// Creates a new `ObservedStateStore` with the given configuration and telemetry registry.
    #[must_use]
    pub fn new(config: &ObservedStateSettings, registry: TelemetryRegistryHandle) -> Self {
        let (sender, receiver) = flume::bounded::<ObservedEvent>(config.reporting_channel_size);
        let (engine_sender, engine_receiver) = flume::unbounded::<EngineEvent>();

        Self {
            default_health_policy: HealthPolicy::default(),
            health_policies: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver,
            engine_sender,
            engine_receiver,
            console: ConsoleWriter::color(),
            registry,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reporter for engine lifecycle events.
    ///
    /// Engine events sent through this reporter use the dedicated engine channel
    /// rather than the bounded observed-event channel. Log events still use the
    /// bounded path.
    #[must_use]
    pub fn engine_reporter(&self, log_policy: SendPolicy) -> ObservedEventReporter {
        ObservedEventReporter::new_with_engine_sender(
            log_policy,
            self.sender.clone(),
            self.engine_sender.clone(),
        )
    }

    /// Returns a reporter that sends observed events through the bounded channel
    /// according to `policy`.
    ///
    /// For engine lifecycle events that require the dedicated engine channel, use
    /// [`engine_reporter`](Self::engine_reporter) instead.
    #[must_use]
    pub fn reporter(&self, policy: SendPolicy) -> ObservedEventReporter {
        ObservedEventReporter::new(policy, self.sender.clone())
    }

    /// Registers or updates the health policy for a specific pipeline.
    pub fn register_pipeline_health_policy(
        &self,
        pipeline_key: PipelineKey,
        health_policy: HealthPolicy,
    ) {
        let mut policies = self.health_policies.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with stale health policy"
            );
            poisoned.into_inner()
        });
        _ = policies.insert(pipeline_key, health_policy);
    }

    /// Returns a handle that can be used to read the current observed state.
    #[must_use]
    pub fn handle(&self) -> ObservedStateHandle {
        ObservedStateHandle {
            pipelines: self.pipelines.clone(),
        }
    }

    /// Reports a new observed event in the store.
    fn report(&self, observed_event: ObservedEvent) -> Result<(), Error> {
        match observed_event {
            ObservedEvent::Engine(engine) => {
                let _ = self.report_engine(engine)?;
            }
            ObservedEvent::Log(log) => {
                let context = &log.record.context;

                self.console.print_log_record(log.time, &log.record, |w| {
                    if !context.is_empty() {
                        w.write_styled(AnsiCode::Magenta, |w| {
                            Self::format_scope_from_registry(w, context, &self.registry);
                        });
                    }
                });
            }
        }
        Ok(())
    }

    /// Format scope attributes by looking up entity keys in the registry.
    /// Appends entity references inline after the log message.
    /// Format: ` entity/schema: key=val key2=val2 entity/schema2: ...`
    ///
    /// TODO(#1907, #1746): This code is too expensive. We would like a lower-cost
    /// way to output human readable text.
    fn format_scope_from_registry(
        w: &mut StyledBufWriter<'_>,
        context: &LogContext,
        registry: &TelemetryRegistryHandle,
    ) {
        for key in context.iter() {
            let visited = registry.visit_entity(*key, |attrs| {
                (
                    attrs
                        .iter_attributes()
                        .map(|(a, b)| (a, b.clone()))
                        .collect::<Vec<_>>(),
                    attrs.schema_name(),
                )
            });
            visited
                .map(|(attrs, schema_name)| {
                    let _ = write!(w, " entity/{}:", schema_name);

                    // TODO(#1907): We should be able to use
                    for (attr_key, attr_value) in attrs {
                        let _ = write!(w, " {}={}", attr_key, attr_value.to_string_value());
                    }
                })
                .unwrap_or_default()
        }
    }

    /// Reports a new observed event in the store.
    fn report_engine(&self, observed_event: EngineEvent) -> Result<ApplyOutcome, Error> {
        match &observed_event.r#type {
            EventType::Request(req) => {
                otel_info!("state.observed_event",
                    pipeline_group_id = %observed_event.key.pipeline_group_id,
                    pipeline_id = %observed_event.key.pipeline_id,
                    core_id = observed_event.key.core_id,
                    event_type = ?req,
                    message = observed_event.message.as_deref().unwrap_or(""),
                );
            }
            EventType::Error(err) => {
                otel_error!("state.observed_error",
                    pipeline_group_id = %observed_event.key.pipeline_group_id,
                    pipeline_id = %observed_event.key.pipeline_id,
                    core_id = observed_event.key.core_id,
                    event_type = ?err,
                    message = observed_event.message.as_deref().unwrap_or(""),
                );
            }
            EventType::Success(_) => {}
        };

        // Events without a pipeline key don't update state.
        let key = &observed_event.key;

        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        let pipeline_key = PipelineKey::new(key.pipeline_group_id.clone(), key.pipeline_id.clone());

        let health_policy = self
            .health_policies
            .lock()
            .ok()
            .and_then(|policies| policies.get(&pipeline_key).cloned())
            .unwrap_or_else(|| self.default_health_policy.clone());
        let ps = pipelines
            .entry(pipeline_key)
            .or_insert_with(|| PipelineStatus::new(health_policy));

        // Upsert the core record and its condition snapshot
        let cs = ps
            .cores
            .entry(key.core_id)
            .or_insert_with(|| PipelineRuntimeStatus {
                phase: PipelinePhase::Pending,
                last_heartbeat_time: observed_event.time,
                recent_events: ObservedEventRingBuffer::new(RECENT_EVENTS_CAPACITY),
                ..Default::default()
            });
        cs.apply_event(observed_event)
    }

    /// Runs the collection loop, receiving observed events and updating the observed store.
    /// This function runs indefinitely until the channel is closed or the cancellation token is
    /// triggered.
    ///
    /// Engine lifecycle events are received from a dedicated unbounded channel
    /// and are prioritized over log events so log traffic cannot delay
    /// lifecycle processing on the shared consumer loop.
    pub async fn run(self, cancel: CancellationToken) -> Result<(), Error> {
        let mut engine_closed = false;
        let mut log_closed = false;

        loop {
            if engine_closed && log_closed {
                break;
            }

            tokio::select! {
                biased;

                _ = cancel.cancelled() => break,

                result = self.engine_receiver.recv_async(), if !engine_closed => {
                    match result {
                        Ok(engine_event) => {
                            if let Err(e) = self.report_engine(engine_event) {
                                otel_error!("state.report_failed", error = ?e);
                            }
                        }
                        Err(_) => engine_closed = true,
                    }
                }

                result = self.receiver.recv_async(), if !log_closed => {
                    match result {
                        Ok(event) => {
                            if let Err(e) = self.report(event) {
                                otel_error!("state.report_failed", error = ?e);
                            }
                        }
                        Err(_) => log_closed = true,
                    }
                }
            }
        }

        Ok(())
    }
}

impl ObservedStateHandle {
    /// Retrieves the current status of a pipeline by its key.
    /// Returns a snapshot clone of the status if present.
    #[must_use]
    pub fn pipeline_status(&self, pipeline_key: &PipelineKey) -> Option<PipelineStatus> {
        let pipelines = self.pipelines.lock().ok()?;
        pipelines.get(pipeline_key).cloned()
    }

    /// Checks if a pipeline is considered live based on its observed status.
    #[must_use]
    pub fn liveness(&self, pipeline_key: &PipelineKey) -> bool {
        self.pipelines
            .lock()
            .ok()
            .is_some_and(|pipelines| pipelines.get(pipeline_key).is_some_and(|ps| ps.liveness()))
    }

    /// Checks if a pipeline is considered ready based on its observed status.
    #[must_use]
    pub fn readiness(&self, pipeline_key: &PipelineKey) -> bool {
        self.pipelines
            .lock()
            .ok()
            .is_some_and(|pipelines| pipelines.get(pipeline_key).is_some_and(|ps| ps.readiness()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::observed_state::SendPolicy;
    use otap_df_telemetry::event::{EngineEvent, ObservedEvent};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::borrow::Cow;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;

    fn make_key(core_id: usize) -> otap_df_config::DeployedPipelineKey {
        otap_df_config::DeployedPipelineKey {
            pipeline_group_id: Cow::Borrowed("group"),
            pipeline_id: Cow::Borrowed("pipeline"),
            core_id,
        }
    }

    /// Verifies that a size-1 channel with `try_send` drops almost all
    /// events from a burst of 64 Admitted sends.
    #[test]
    fn try_send_drops_admitted_events_when_channel_full() {
        let config = ObservedStateSettings {
            reporting_channel_size: 1,
            engine_events: SendPolicy {
                blocking_timeout: None, // try_send: instant fail when full
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        };
        let store = ObservedStateStore::new(&config, TelemetryRegistryHandle::new());
        let reporter = store.reporter(config.engine_events.clone());

        // Blast 64 Admitted events without a consumer. Channel size 1 means
        // at most 1 event can be buffered; the rest are silently dropped.
        for core in 0..64 {
            reporter.report(EngineEvent::admitted(make_key(core), None));
        }

        // Drain the channel directly — only 1 event should be present.
        let mut buffered = 0;
        while store.receiver.try_recv().is_ok() {
            buffered += 1;
        }
        assert_eq!(
            buffered, 1,
            "Only 1 event should survive in a size-1 channel, got {buffered}"
        );
    }

    /// Exercises the shipped `send_timeout(1ms)` default path (P2 gap).
    /// With a size-1 channel and no consumer, `send_timeout(1ms)` should
    /// still drop almost all events from a burst of 64 Admitted sends.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_timeout_1ms_drops_admitted_events() {
        let config = ObservedStateSettings {
            reporting_channel_size: 1,
            engine_events: SendPolicy {
                blocking_timeout: Some(Duration::from_millis(1)), // real default
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        };
        let store = ObservedStateStore::new(&config, TelemetryRegistryHandle::new());
        let reporter = store.reporter(config.engine_events.clone());

        // Blast 64 Admitted events from a blocking thread (send_timeout
        // blocks the calling OS thread).  No consumer is running, so the
        // channel stays full after the first event and the rest time out.
        let reporter_clone = reporter.clone();
        tokio::task::spawn_blocking(move || {
            for core in 0..64 {
                reporter_clone.report(EngineEvent::admitted(make_key(core), None));
            }
        })
        .await
        .unwrap();

        // Drain the channel — only 1 event should be present.
        let mut buffered = 0;
        while store.receiver.try_recv().is_ok() {
            buffered += 1;
        }
        assert_eq!(
            buffered, 1,
            "Only 1 event should survive in a size-1 channel with send_timeout(1ms), \
             got {buffered}"
        );
    }

    /// End-to-end reproduction of issue #2328: dropped `Admitted` events
    /// leave cores stuck in `Pending`, and subsequent `Ready` events for
    /// those cores are rejected as invalid transitions.
    ///
    /// Uses `multi_thread` so the blocking `send_timeout` in `spawn_blocking`
    /// does not starve the async consumer task.
    ///
    /// To avoid the P3 reliability gap (cores appearing stuck only because
    /// events were undrained at cancellation time), we poll until all cores
    /// appear in the state map before inspecting phases.  All 64 Ready events
    /// are delivered reliably via `send_timeout(5s)`, so once `total_cores()`
    /// reaches 64, the consumer has processed every Ready event.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropped_admitted_leaves_cores_stuck_in_pending() {
        let num_cores: usize = 64;
        let config = ObservedStateSettings {
            reporting_channel_size: 1,
            engine_events: SendPolicy {
                blocking_timeout: None, // try_send
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        };

        let store = ObservedStateStore::new(&config, TelemetryRegistryHandle::new());
        let handle = store.handle();
        let reporter = store.reporter(config.engine_events.clone());

        // Phase 1: Blast Admitted events WITHOUT a consumer running.
        // Channel size 1 + try_send ⇒ only 1 event is buffered.
        for core in 0..num_cores {
            reporter.report(EngineEvent::admitted(make_key(core), None));
        }

        // Phase 2: Start the consumer so it drains the single buffered
        // Admitted and then processes incoming Ready events.
        let store_clone = store.clone();
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let consumer = tokio::spawn(async move { store_clone.run(cancel_clone).await });

        // Give the consumer time to drain the queued Admitted event.
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Phase 3: Send Ready for every core.  Use `spawn_blocking` +
        // `send_timeout` so we don't block the tokio runtime.  The consumer
        // is actively draining, so each send should succeed quickly.
        let sender = store.sender.clone();
        tokio::task::spawn_blocking(move || {
            for core in 0..num_cores {
                let event = ObservedEvent::Engine(EngineEvent::ready(make_key(core), None));
                sender
                    .send_timeout(event, Duration::from_secs(5))
                    .expect("Ready event should be delivered while consumer is running");
            }
        })
        .await
        .unwrap();

        // Phase 4: Wait until the consumer has processed all 64 Ready events.
        // Each Ready creates or updates a core entry, so total_cores() == 64
        // means every Ready has been applied (whether it succeeded or hit
        // InvalidTransition).  This avoids the P3 gap of relying on fixed
        // sleeps where cores might appear stuck only because they were undrained.
        let pipeline_key = PipelineKey::new(Cow::Borrowed("group"), Cow::Borrowed("pipeline"));
        for _ in 0..200 {
            if let Some(status) = handle.pipeline_status(&pipeline_key) {
                if status.total_cores() == num_cores {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        cancel.cancel();
        let _ = consumer.await;

        // Phase 5: Inspect state — the consumer has processed all events.
        let status = handle
            .pipeline_status(&pipeline_key)
            .expect("pipeline should exist");

        assert_eq!(
            status.total_cores(),
            num_cores,
            "Consumer should have observed all {num_cores} cores"
        );

        let stuck_in_pending = status
            .per_core()
            .values()
            .filter(|c| matches!(c.phase, PipelinePhase::Pending))
            .count();
        let running = status.running_cores();

        // At most 1 Admitted made it through, so at most 1 core reached
        // Running.  The other 63 are stuck in Pending because Ready was
        // rejected as an invalid transition (Pending → Ready is not valid).
        assert!(
            stuck_in_pending > 0,
            "Expected some cores stuck in Pending due to dropped Admitted events, \
             but all {num_cores} cores progressed (running={running}). \
             This means the channel-drop bug did not manifest.",
        );
        assert!(
            running < num_cores,
            "Expected fewer than {num_cores} running cores, got {running}. \
             Stuck in Pending: {stuck_in_pending}.",
        );
    }

    /// Verifies the fix for issue #2328: using `engine_reporter()`, engine
    /// lifecycle events are delivered through the dedicated unbounded channel
    /// and are never dropped, even with a tiny log-channel buffer and high
    /// concurrency.  All 64 cores should reach Running.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn engine_reporter_never_drops_lifecycle_events() {
        let num_cores: usize = 64;
        let config = ObservedStateSettings {
            // Intentionally tiny log channel — must NOT affect engine events.
            reporting_channel_size: 1,
            engine_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        };

        let store = ObservedStateStore::new(&config, TelemetryRegistryHandle::new());
        let handle = store.handle();
        // Use the reliable engine_reporter — this is what production code uses.
        let reporter = store.engine_reporter(config.engine_events.clone());

        // Start the consumer first.
        let store_clone = store.clone();
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let consumer = tokio::spawn(async move { store_clone.run(cancel_clone).await });

        // Blast Admitted + Ready for every core from a blocking thread.
        let reporter_clone = reporter.clone();
        tokio::task::spawn_blocking(move || {
            for core in 0..num_cores {
                reporter_clone.report(EngineEvent::admitted(make_key(core), None));
            }
            for core in 0..num_cores {
                reporter_clone.report(EngineEvent::ready(make_key(core), None));
            }
        })
        .await
        .unwrap();

        // Wait until all cores appear in the state map.
        let pipeline_key = PipelineKey::new(Cow::Borrowed("group"), Cow::Borrowed("pipeline"));
        for _ in 0..200 {
            if let Some(status) = handle.pipeline_status(&pipeline_key) {
                if status.total_cores() == num_cores && status.running_cores() == num_cores {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        cancel.cancel();
        let _ = consumer.await;

        let status = handle
            .pipeline_status(&pipeline_key)
            .expect("pipeline should exist");

        assert_eq!(
            status.total_cores(),
            num_cores,
            "All {num_cores} cores should be observed"
        );
        assert_eq!(
            status.running_cores(),
            num_cores,
            "All {num_cores} cores should reach Running when engine events are reliable. \
             Stuck in Pending: {}",
            status
                .per_core()
                .values()
                .filter(|c| matches!(c.phase, PipelinePhase::Pending))
                .count(),
        );
    }

    /// Verifies that flooding the bounded log channel does not interfere with
    /// engine event delivery.  The log channel is size-1 with `try_send`, so
    /// almost all log events are dropped.  Engine events go through the
    /// dedicated unbounded channel and must all arrive.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bounded_channel_contention_does_not_block_engine_events() {
        let num_cores: usize = 64;
        let config = ObservedStateSettings {
            reporting_channel_size: 1, // tiny log channel
            engine_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None, // try_send → instant drop when full
                console_fallback: false,
            },
        };

        let store = ObservedStateStore::new(&config, TelemetryRegistryHandle::new());
        let handle = store.handle();
        let engine_reporter = store.engine_reporter(config.engine_events.clone());

        // Start the consumer.
        let store_clone = store.clone();
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let consumer = tokio::spawn(async move { store_clone.run(cancel_clone).await });

        // Flood the bounded (log/fallback) channel directly via try_send.
        // Most events will be dropped (size-1 + try_send), which is expected.
        //
        // We use StartRequested engine events as filler because constructing a
        // real LogEvent requires tracing callsite infrastructure.  The purpose
        // of this test is channel-level isolation: proving that saturation of
        // the bounded channel cannot block or delay the unbounded engine path.
        // Log-specific processing (console print) is exercised separately by
        // the report() dispatch in the consumer.
        let log_sender = store.sender.clone();
        let log_flood = tokio::task::spawn_blocking(move || {
            for i in 0..1000 {
                let _ = log_sender.try_send(ObservedEvent::Engine(EngineEvent::start_requested(
                    make_key(i % num_cores),
                    None,
                )));
            }
        });

        // Concurrently send engine lifecycle events via the reliable path.
        let engine_reporter_clone = engine_reporter.clone();
        let engine_send = tokio::task::spawn_blocking(move || {
            for core in 0..num_cores {
                engine_reporter_clone.report(EngineEvent::admitted(make_key(core), None));
            }
            for core in 0..num_cores {
                engine_reporter_clone.report(EngineEvent::ready(make_key(core), None));
            }
        });

        log_flood.await.unwrap();
        engine_send.await.unwrap();

        // Wait until all cores reach Running.
        let pipeline_key = PipelineKey::new(Cow::Borrowed("group"), Cow::Borrowed("pipeline"));
        for _ in 0..200 {
            if let Some(status) = handle.pipeline_status(&pipeline_key) {
                if status.total_cores() == num_cores && status.running_cores() == num_cores {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        cancel.cancel();
        let _ = consumer.await;

        let status = handle
            .pipeline_status(&pipeline_key)
            .expect("pipeline should exist");

        assert_eq!(
            status.total_cores(),
            num_cores,
            "All {num_cores} cores should be observed despite log flood"
        );
        assert_eq!(
            status.running_cores(),
            num_cores,
            "All {num_cores} cores should reach Running despite log channel contention. \
             Stuck in Pending: {}",
            status
                .per_core()
                .values()
                .filter(|c| matches!(c.phase, PipelinePhase::Pending))
                .count(),
        );
    }
}
