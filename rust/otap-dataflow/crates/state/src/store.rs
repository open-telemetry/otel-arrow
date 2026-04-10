// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs defining an event-driven observed state store.

use crate::ObservedEventRingBuffer;
use crate::error::Error;
use crate::phase::PipelinePhase;
use crate::pipeline_rt_status::{ApplyOutcome, PipelineRuntimeStatus};
use crate::pipeline_status::{PipelineRolloutSummary, PipelineStatus, RuntimeInstanceKey};
use otap_df_config::PipelineKey;
use otap_df_config::health::HealthPolicy;
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_telemetry::event::{EngineEvent, EventType, ObservedEvent, ObservedEventReporter};
use otap_df_telemetry::log_tap::InternalLogTapHandle;
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

    #[serde(skip)]
    log_tap: Option<InternalLogTapHandle>,

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
        Self::new_with_log_tap(config, registry, None)
    }

    /// Creates a new `ObservedStateStore` with an optional retained-log sink.
    #[must_use]
    pub fn new_with_log_tap(
        config: &ObservedStateSettings,
        registry: TelemetryRegistryHandle,
        log_tap: Option<InternalLogTapHandle>,
    ) -> Self {
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
            log_tap,
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
        let reporter = ObservedEventReporter::new_with_engine_sender(
            log_policy,
            self.sender.clone(),
            self.engine_sender.clone(),
        );
        if let Some(log_tap) = &self.log_tap {
            reporter.with_drop_counter(log_tap.ingest_drop_counter())
        } else {
            reporter
        }
    }

    /// Returns a reporter that sends observed events through the bounded channel
    /// according to `policy`.
    ///
    /// For engine lifecycle events that require the dedicated engine channel, use
    /// [`engine_reporter`](Self::engine_reporter) instead.
    #[must_use]
    pub fn reporter(&self, policy: SendPolicy) -> ObservedEventReporter {
        let reporter = ObservedEventReporter::new(policy, self.sender.clone());
        if let Some(log_tap) = &self.log_tap {
            reporter.with_drop_counter(log_tap.ingest_drop_counter())
        } else {
            reporter
        }
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

    /// Returns the health policy currently configured for one logical pipeline.
    fn health_policy_for_pipeline(&self, pipeline_key: &PipelineKey) -> HealthPolicy {
        self.health_policies
            .lock()
            .ok()
            .and_then(|policies| policies.get(pipeline_key).cloned())
            .unwrap_or_else(|| self.default_health_policy.clone())
    }

    /// Records the committed active generation for a logical pipeline.
    pub fn set_pipeline_active_generation(&self, pipeline_key: PipelineKey, generation: u64) {
        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        let status = pipelines
            .entry(pipeline_key.clone())
            .or_insert_with(|| PipelineStatus::new(self.health_policy_for_pipeline(&pipeline_key)));
        status.set_active_generation(generation);
    }

    /// Records which generation is serving traffic for the given logical core.
    pub fn set_pipeline_serving_generation(
        &self,
        pipeline_key: PipelineKey,
        core_id: otap_df_config::CoreId,
        generation: u64,
    ) {
        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        let status = pipelines
            .entry(pipeline_key.clone())
            .or_insert_with(|| PipelineStatus::new(self.health_policy_for_pipeline(&pipeline_key)));
        status.set_serving_generation(core_id, generation);
    }

    /// Removes the serving-generation marker for a logical core.
    pub fn clear_pipeline_serving_generation(
        &self,
        pipeline_key: PipelineKey,
        core_id: otap_df_config::CoreId,
    ) {
        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        if let Some(status) = pipelines.get_mut(&pipeline_key) {
            status.clear_serving_generation(core_id);
        }
    }

    /// Updates the rollout summary exposed in `/status`.
    pub fn set_pipeline_rollout_summary(
        &self,
        pipeline_key: PipelineKey,
        rollout: PipelineRolloutSummary,
    ) {
        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        let status = pipelines
            .entry(pipeline_key.clone())
            .or_insert_with(|| PipelineStatus::new(self.health_policy_for_pipeline(&pipeline_key)));
        status.set_rollout_summary(rollout);
    }

    /// Clears any rollout summary for the logical pipeline.
    pub fn clear_pipeline_rollout_summary(&self, pipeline_key: PipelineKey) {
        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        if let Some(status) = pipelines.get_mut(&pipeline_key) {
            status.clear_rollout_summary();
        }
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
                if let Some(log_tap) = &self.log_tap {
                    log_tap.record(log.clone());
                }
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
        if ps.active_generation().is_none() {
            ps.set_active_generation(key.deployment_generation);
        }

        // Upsert the runtime-instance record and its condition snapshot
        let cs = ps
            .instances
            .entry(RuntimeInstanceKey {
                core_id: key.core_id,
                deployment_generation: key.deployment_generation,
            })
            .or_insert_with(|| PipelineRuntimeStatus {
                phase: PipelinePhase::Pending,
                last_heartbeat_time: observed_event.time,
                recent_events: ObservedEventRingBuffer::new(RECENT_EVENTS_CAPACITY),
                ..Default::default()
            });
        cs.apply_event(observed_event)
    }

    fn drain_remaining(&self, engine_closed: &mut bool, log_closed: &mut bool) {
        loop {
            let mut made_progress = false;

            if !*engine_closed {
                loop {
                    match self.engine_receiver.try_recv() {
                        Ok(engine_event) => {
                            made_progress = true;
                            if let Err(e) = self.report_engine(engine_event) {
                                otel_error!("state.report_failed", error = ?e);
                            }
                        }
                        Err(flume::TryRecvError::Empty) => break,
                        Err(flume::TryRecvError::Disconnected) => {
                            *engine_closed = true;
                            break;
                        }
                    }
                }
            }

            if !*log_closed {
                loop {
                    match self.receiver.try_recv() {
                        Ok(event) => {
                            made_progress = true;
                            if let Err(e) = self.report(event) {
                                otel_error!("state.report_failed", error = ?e);
                            }
                        }
                        Err(flume::TryRecvError::Empty) => break,
                        Err(flume::TryRecvError::Disconnected) => {
                            *log_closed = true;
                            break;
                        }
                    }
                }
            }

            if !made_progress {
                break;
            }
        }
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

                _ = cancel.cancelled() => {
                    self.drain_remaining(&mut engine_closed, &mut log_closed);
                    break;
                },

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
    use otap_df_telemetry::log_tap;
    use otap_df_telemetry::otel_info;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::self_tracing::{LogContext, LogRecord};
    use std::borrow::Cow;
    use std::time::Duration;
    use std::time::SystemTime;
    use tokio_util::sync::CancellationToken;
    use tracing::{Event, Subscriber};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::layer::{Context, Layer};
    use tracing_subscriber::registry::LookupSpan;

    fn make_key(core_id: usize) -> otap_df_config::DeployedPipelineKey {
        otap_df_config::DeployedPipelineKey {
            pipeline_group_id: Cow::Borrowed("group"),
            pipeline_id: Cow::Borrowed("pipeline"),
            core_id,
            deployment_generation: 0,
        }
    }

    #[derive(Clone)]
    struct ReporterLayer {
        reporter: ObservedEventReporter,
    }

    impl<S> Layer<S> for ReporterLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            self.reporter.log(otap_df_telemetry::event::LogEvent {
                time: SystemTime::now(),
                record: LogRecord::new(event, LogContext::default()),
            });
        }
    }

    fn emit_log_via_async_reporter(reporter: ObservedEventReporter, message: &str) {
        let dispatch = tracing::Dispatch::new(
            tracing_subscriber::Registry::default().with(ReporterLayer { reporter }),
        );
        tracing::dispatcher::with_default(&dispatch, || {
            otel_info!("state.test.log", message = message);
        });
    }

    /// Validates that `send_timeout(1ms)` on a full bounded channel drops
    /// events the same way `try_send` does.  The timeout is too short to
    /// survive a burst without a consumer.
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

    /// Validates that engine lifecycle events delivered through
    /// `engine_reporter()` are never dropped, even with a tiny log-channel
    /// buffer and high concurrency.  All cores must reach `Running`.
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
                .per_instance()
                .values()
                .filter(|c| matches!(c.phase, PipelinePhase::Pending))
                .count(),
        );
    }

    /// Validates that saturation of the bounded channel does not interfere
    /// with engine event delivery through the dedicated engine channel.
    /// Engine events must all arrive regardless of bounded-channel
    /// backpressure.
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
                .per_instance()
                .values()
                .filter(|c| matches!(c.phase, PipelinePhase::Pending))
                .count(),
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn console_async_path_fans_out_to_tap() {
        let config = ObservedStateSettings {
            reporting_channel_size: 8,
            engine_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        };
        let log_tap = log_tap::build(
            &otap_df_config::settings::telemetry::logs::InternalLogTapConfig {
                enabled: true,
                max_entries: 8,
                max_bytes: 1024 * 1024,
            },
        );
        let store = ObservedStateStore::new_with_log_tap(
            &config,
            TelemetryRegistryHandle::new(),
            Some(log_tap.clone()),
        );
        let reporter = store.reporter(config.logging_events.clone());

        let cancel = CancellationToken::new();
        let consumer = tokio::spawn(store.clone().run(cancel.clone()));

        emit_log_via_async_reporter(reporter, "tap fanout");

        for _ in 0..50 {
            let result = log_tap.query(log_tap::LogQuery {
                after: None,
                limit: 10,
            });
            if result.logs.len() == 1 {
                assert!(result.logs[0].event.to_string().contains("tap fanout"));
                cancel.cancel();
                consumer.await.expect("join").expect("run succeeds");
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        cancel.cancel();
        consumer.await.expect("join").expect("run succeeds");
        panic!("log was not retained via console_async path");
    }

    #[tokio::test]
    async fn cancel_drains_console_async_logs_into_tap() {
        let config = ObservedStateSettings {
            reporting_channel_size: 8,
            engine_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        };
        let log_tap = log_tap::build(
            &otap_df_config::settings::telemetry::logs::InternalLogTapConfig {
                enabled: true,
                max_entries: 8,
                max_bytes: 1024 * 1024,
            },
        );
        let store = ObservedStateStore::new_with_log_tap(
            &config,
            TelemetryRegistryHandle::new(),
            Some(log_tap.clone()),
        );
        let reporter = store.reporter(config.logging_events.clone());

        emit_log_via_async_reporter(reporter, "shutdown drain");

        let cancel = CancellationToken::new();
        cancel.cancel();
        store.run(cancel).await.expect("run succeeds");

        let result = log_tap.query(log_tap::LogQuery {
            after: None,
            limit: 10,
        });
        assert_eq!(result.logs.len(), 1);
        assert!(result.logs[0].event.to_string().contains("shutdown drain"));
    }
}
