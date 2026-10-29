// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Scheduling, publishing, warnings, and shutdown for the single-core receiver.

use super::config::Config;
use super::metrics::WindowsPerfCountersMetrics;
use super::otap_builder::project_points;
use super::pdh::{NativePdh, Report, Stopped, Worker};
use async_trait::async_trait;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::{Error, ReceiverErrorKind, TypedError};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{MessageSourceLocalEffectHandlerExtension, ReceiverFactory};
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tokio::time::MissedTickBehavior;

/// Factory identity for the Windows performance-counter receiver.
pub(super) const WINDOWS_PERF_COUNTERS_RECEIVER_URN: &str =
    "urn:otel:receiver:windows_perf_counters";

const WORKER_SHUTDOWN_MAX_WAIT: Duration = Duration::from_secs(1);

/// Open failure episodes of one configured counter; each warns once when it opens.
#[derive(Debug, Default, Clone, Copy)]
struct Episodes {
    add: bool,
    read: bool,
    collect: bool,
    recreate: bool,
}

/// Warn-once state for the receiver's warnings. Counter episodes are tracked per configured
/// counter, since every counter has its own query.
#[derive(Debug)]
struct Diagnostics {
    counters: Vec<Episodes>,
    overrunning: bool,
    dropping: bool,
}

/// Opens an episode; returns whether it was not already open, which is when to warn.
fn begin(episode: &mut bool) -> bool {
    !std::mem::replace(episode, true)
}

impl Diagnostics {
    fn new(counters: usize) -> Self {
        Self {
            counters: vec![Episodes::default(); counters],
            overrunning: false,
            dropping: false,
        }
    }

    /// Returns whether to warn: once until a scrape completes on time.
    fn overrun(&mut self) -> bool {
        begin(&mut self.overrunning)
    }

    fn completed(&mut self, on_time: bool) {
        if on_time {
            self.overrunning = false;
        }
    }

    /// Returns whether to warn: once until a batch is accepted downstream.
    fn dropped(&mut self) -> bool {
        begin(&mut self.dropping)
    }

    fn sent(&mut self) {
        self.dropping = false;
    }
}

/// The start or scrape report the worker owes.
struct InFlight {
    report: oneshot::Receiver<Report>,
    requested: tokio::time::Instant,
    /// The start report, which is not a scrape.
    start: bool,
    late: bool,
}

struct WindowsPerfCountersReceiver {
    config: Config,
    node: String,
    metrics: MetricSet<WindowsPerfCountersMetrics>,
    diagnostics: Diagnostics,
}

/// Creates the Windows performance-counter local receiver.
pub(super) static WINDOWS_PERF_COUNTERS_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: WINDOWS_PERF_COUNTERS_RECEIVER_URN,
    create: |pipeline, node, node_config, receiver_config, _capabilities| {
        if pipeline.num_cores() > 1 {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error:
                    "host-wide windows_perf_counters collection requires a one-core source pipeline"
                        .to_owned(),
            });
        }
        let config = Config::from_json(&node_config.config)?;
        let receiver = WindowsPerfCountersReceiver {
            diagnostics: Diagnostics::new(config.counters.len()),
            config,
            node: node.name.to_string(),
            metrics: WindowsPerfCountersMetrics::register(&pipeline),
        };
        Ok(ReceiverWrapper::local(
            receiver,
            node,
            node_config,
            receiver_config,
        ))
    },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |value| Config::from_json(value).map(|_| ()),
};

fn receiver_error(effect_handler: &local::EffectHandler<OtapPdata>, error: String) -> Error {
    Error::ReceiverError {
        receiver: effect_handler.receiver_id(),
        kind: ReceiverErrorKind::Other,
        error,
        source_detail: String::new(),
    }
}

/// Requests one scrape; the scheduler depends only on this, so tests can script reports.
trait Scrape {
    fn scrape(&self) -> oneshot::Receiver<Report>;
}

impl Scrape for Worker {
    fn scrape(&self) -> oneshot::Receiver<Report> {
        Worker::scrape(self)
    }
}

impl WindowsPerfCountersReceiver {
    /// Records a start or scrape report and emits any due warnings.
    fn apply(&mut self, report: &Report, duration: Option<Duration>) {
        let node = self.node.as_str();
        let config = |counter: usize| &self.config.counters[counter];
        let episodes = &mut self.diagnostics.counters;
        // Each failure kind warns when its counter's episode opens; the counter's success ends it.
        macro_rules! episode {
            ($failures:ident, $successes:ident, $episode:ident, $event:literal) => {
                for &(counter, status) in &report.$failures {
                    if begin(&mut episodes[counter].$episode) {
                        otel_warn!(
                            $event,
                            node = node,
                            counter_index = counter,
                            metric = config(counter).name.as_ref(),
                            path = config(counter).path.as_str(),
                            status = %format_args!("{status:#010x}")
                        );
                    }
                }
                for &counter in &report.$successes {
                    episodes[counter].$episode = false;
                }
            };
        }
        episode!(
            add_failures,
            added,
            add,
            "windows_perf_counters_receiver.counter_add.fail"
        );
        episode!(
            read_failures,
            read,
            read,
            "windows_perf_counters_receiver.counter_read.fail"
        );
        episode!(
            collect_failures,
            collected,
            collect,
            "windows_perf_counters_receiver.collect.fail"
        );
        episode!(
            recreate_failures,
            recreated,
            recreate,
            "windows_perf_counters_receiver.recreate_query.fail"
        );
        // Both can count in one scrape: some queries collect while others fail.
        if !report.collect_failures.is_empty() {
            self.metrics.scrape_failures.inc();
        }
        if !report.collected.is_empty() {
            self.metrics.scrapes.inc();
        }
        if let Some(duration) = duration {
            self.metrics.scrape_duration.record(duration.as_secs_f64());
        }
        self.metrics
            .counter_add_failures
            .add(report.add_failures.len() as u64);
        self.metrics
            .active_counters
            .set(report.active_counters as u64);
    }

    /// Runs until drain or shutdown; returns the control deadline and whether ingress drained.
    /// The start report is awaited here too, so control stays responsive while PDH starts.
    async fn collect(
        &mut self,
        worker: &impl Scrape,
        started: oneshot::Receiver<Report>,
        control: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(Instant, bool), Error> {
        let start = tokio::time::Instant::now() + self.config.initial_delay;
        let mut ticks = tokio::time::interval_at(start, self.config.collection_interval);
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        // Without a timeout, a scrape is late once the next tick is due.
        let threshold = self
            .config
            .timeout
            .unwrap_or(self.config.collection_interval);
        let mut in_flight = Some(InFlight {
            report: started,
            requested: tokio::time::Instant::now(),
            start: true,
            late: false,
        });
        loop {
            // With a timeout, the overrun warning fires when it expires instead of at a skipped tick.
            let overdue_at = match (&in_flight, self.config.timeout) {
                (Some(pending), Some(timeout)) if !pending.start && !pending.late => {
                    Some(pending.requested + timeout)
                }
                _ => None,
            };
            tokio::select! {
                biased;
                message = control.recv() => match message.map_err(Error::ChannelRecvError)? {
                    NodeControlMsg::CollectTelemetry { mut metrics_reporter } => {
                        let _ = metrics_reporter.report(&mut self.metrics);
                    }
                    NodeControlMsg::DrainIngress { deadline, .. } => return Ok((deadline, true)),
                    NodeControlMsg::Shutdown { deadline, .. } => return Ok((deadline, false)),
                    _ => {}
                },
                // Ticks are not polled during start, so the first tick stays due until start
                // completes. Ticks come before reports so a due tick marks a scrape late first.
                _ = ticks.tick(), if !in_flight.as_ref().is_some_and(|p| p.start) => {
                    match in_flight.as_mut() {
                        Some(pending) => {
                            self.metrics.scrape_overruns.inc();
                            if self.config.timeout.is_none() {
                                pending.late = true;
                                self.warn_overrun();
                            }
                        }
                        None => {
                            in_flight = Some(InFlight {
                                report: worker.scrape(),
                                requested: tokio::time::Instant::now(),
                                start: false,
                                late: false,
                            });
                        }
                    }
                }
                Some(report) = async {
                    match in_flight.as_mut() {
                        Some(pending) => Some((&mut pending.report).await),
                        None => None,
                    }
                }, if in_flight.is_some() => {
                    let Some(pending) = in_flight.take() else {
                        continue;
                    };
                    let report = report.map_err(|_| {
                        receiver_error(effect_handler, "PDH worker stopped".to_owned())
                    })?;
                    if pending.start {
                        self.apply(&report, None);
                        continue;
                    }
                    let elapsed = pending.requested.elapsed();
                    // The threshold may have passed before this loop saw the expired timer.
                    let late = pending.late || elapsed >= threshold;
                    if late && !pending.late {
                        self.warn_overrun();
                    }
                    self.apply(&report, Some(elapsed));
                    self.diagnostics.completed(!late);
                    // Late results are forwarded like any other.
                    if let Some(records) = project_points(&self.config.counters, &report.points)
                        .map_err(|err| receiver_error(effect_handler, err.to_string()))?
                    {
                        self.send(effect_handler, OtapPdata::new(Context::default(), records.into()))?;
                    }
                }
                () = tokio::time::sleep_until(overdue_at.unwrap_or(start)), if overdue_at.is_some() => {
                    if let Some(pending) = in_flight.as_mut() {
                        pending.late = true;
                    }
                    self.warn_overrun();
                }
            }
        }
    }

    /// Forwards a batch without waiting; when downstream is full or closed, drops it.
    fn send(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        data: OtapPdata,
    ) -> Result<(), Error> {
        match effect_handler.try_send_message_with_source_node(data) {
            Ok(()) => self.diagnostics.sent(),
            Err(TypedError::ChannelSendError(_)) => {
                self.metrics.batches_dropped.inc();
                if self.diagnostics.dropped() {
                    otel_warn!(
                        "windows_perf_counters_receiver.batch.dropped",
                        node = self.node.as_str()
                    );
                }
            }
            Err(other) => return Err(receiver_error(effect_handler, other.to_string())),
        }
        Ok(())
    }

    fn warn_overrun(&mut self) {
        if self.diagnostics.overrun() {
            otel_warn!(
                "windows_perf_counters_receiver.scrape.overrun",
                node = self.node.as_str(),
                collection_interval_seconds = self.config.collection_interval.as_secs_f64(),
                timeout_seconds = self
                    .config
                    .timeout
                    .map_or(0.0, |timeout| timeout.as_secs_f64())
            );
        }
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for WindowsPerfCountersReceiver {
    async fn start(
        mut self: Box<Self>,
        mut control: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let telemetry = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        let (worker, started) = Worker::start(self.config.counters.clone(), NativePdh::open)
            .map_err(|err| receiver_error(&effect_handler, err.to_string()))?;
        let mut result = self
            .collect(&worker, started, &mut control, &effect_handler)
            .await;
        let worker_deadline = match &result {
            Ok((deadline, _)) => (*deadline).min(Instant::now() + WORKER_SHUTDOWN_MAX_WAIT),
            Err(_) => Instant::now() + WORKER_SHUTDOWN_MAX_WAIT,
        };
        match worker.shutdown(worker_deadline.into()).await {
            Stopped::Cleanly => {}
            Stopped::TimedOut => otel_warn!(
                "windows_perf_counters_receiver.shutdown.timeout",
                node = self.node.as_str()
            ),
            Stopped::Failed => {
                if result.is_ok() {
                    result = Err(receiver_error(
                        &effect_handler,
                        "PDH worker panicked".to_owned(),
                    ));
                }
            }
        }
        let _ = telemetry.cancel().await;
        let (deadline, drained) = result?;
        if drained {
            effect_handler.notify_receiver_drained().await?;
        }
        Ok(TerminalState::new(
            deadline,
            self.metrics.terminal_snapshots(),
        ))
    }
}

#[cfg(test)]
mod tests;
