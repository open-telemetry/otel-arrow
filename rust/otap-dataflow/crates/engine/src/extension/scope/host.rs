// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime hosting for extensions declared at one configuration scope.

use crate::channel_metrics::ChannelMetricsHandle;
use crate::context::ExtensionContext;
use crate::error::Error;
use crate::extension::ExtensionWrapper;
use crate::extension_lifecycle::{EXTENSION_SHUTDOWN_GRACE, ExtensionLifecycle, LifecycleEvent};
use crate::extension_monitor::ExtensionMetricsMonitor;
use crate::terminal_state::TerminalMetricsDeadline;
use otel_arrow_dfe_config::MetricLevel;
use otel_arrow_dfe_config::extension::ExtensionDeclarationScope;
use otel_arrow_dfe_config::policy::ResolvedPolicies;
use otel_arrow_dfe_telemetry::otel_warn;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Interval, MissedTickBehavior, interval_at};

const EXTENSION_MONITOR_TICK_INTERVAL: Duration = Duration::from_secs(1);
const EXTENSION_MONITOR_COLLECT_TELEMETRY_INTERVAL: Duration = Duration::from_secs(10);

/// Effective runtime policy consumed by an extension scope host.
///
/// This projection is shared with live-control validation so policy reloads are
/// rejected only when they would change an already-running scope host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionHostRuntimePolicy {
    control_node_capacity: usize,
    pipeline_metrics: bool,
    channel_metrics_enabled: bool,
}

impl ExtensionHostRuntimePolicy {
    /// Projects the policy values used when constructing and monitoring an
    /// extension scope host.
    #[must_use]
    pub fn from_resolved(policies: &ResolvedPolicies) -> Self {
        Self {
            control_node_capacity: policies.channel_capacity.control.node,
            pipeline_metrics: policies.telemetry.pipeline_metrics,
            channel_metrics_enabled: policies.telemetry.runtime_metrics >= MetricLevel::Basic,
        }
    }

    #[must_use]
    pub(super) fn control_node_capacity(self) -> usize {
        self.control_node_capacity
    }

    #[must_use]
    fn pipeline_metrics(self) -> bool {
        self.pipeline_metrics
    }

    #[must_use]
    pub(super) fn channel_metrics_enabled(self) -> bool {
        self.channel_metrics_enabled
    }

    fn needs_independent_channel_metrics_tick(self) -> bool {
        self.channel_metrics_enabled && !self.pipeline_metrics
    }
}

pub(super) struct PreparedExtensionScopeHost {
    pub(super) context: ExtensionContext,
    pub(super) extensions: Vec<(
        ExtensionWrapper,
        otel_arrow_dfe_telemetry::registry::EntityKey,
    )>,
    pub(super) channel_metrics: Vec<ChannelMetricsHandle>,
    pub(super) runtime_policy: ExtensionHostRuntimePolicy,
}

impl PreparedExtensionScopeHost {
    pub(super) fn start(self, metrics_reporter: &MetricsReporter) -> RunningExtensionScopeHost {
        let terminal_metrics_deadline = TerminalMetricsDeadline::host_owned();
        let monitor = if self.runtime_policy.pipeline_metrics() {
            ExtensionMetricsMonitor::new(
                self.context.clone(),
                EXTENSION_MONITOR_TICK_INTERVAL,
                EXTENSION_MONITOR_COLLECT_TELEMETRY_INTERVAL,
            )
        } else {
            ExtensionMetricsMonitor::disabled(self.context.clone())
        };
        let lifecycle = ExtensionLifecycle::spawn_current(
            self.extensions,
            metrics_reporter.clone(),
            terminal_metrics_deadline.clone(),
            &self.context,
            monitor,
        );
        let channel_metrics_interval = self
            .runtime_policy
            .needs_independent_channel_metrics_tick()
            .then(|| {
                let start = tokio::time::Instant::now() + EXTENSION_MONITOR_TICK_INTERVAL;
                let mut interval = interval_at(start, EXTENSION_MONITOR_TICK_INTERVAL);
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                interval
            });
        RunningExtensionScopeHost {
            lifecycle,
            channel_metrics: self.channel_metrics,
            channel_metrics_interval,
            metrics_reporter: metrics_reporter.clone(),
            terminal_metrics_deadline,
        }
    }
}

pub(super) struct RunningExtensionScopeHost {
    lifecycle: ExtensionLifecycle,
    channel_metrics: Vec<ChannelMetricsHandle>,
    channel_metrics_interval: Option<Interval>,
    metrics_reporter: MetricsReporter,
    terminal_metrics_deadline: TerminalMetricsDeadline,
}

async fn next_optional_interval_tick(interval: &mut Option<Interval>) {
    match interval {
        Some(interval) => {
            _ = interval.tick().await;
        }
        None => std::future::pending().await,
    }
}

pub(super) enum ScopeEvent {
    Ready(ExtensionDeclarationScope),
    Failed {
        scope: ExtensionDeclarationScope,
        error: Error,
    },
}

impl RunningExtensionScopeHost {
    async fn wait_ready(&mut self) -> Result<(), Error> {
        self.lifecycle.wait_all_spawned().await?;
        self.lifecycle.wait_all_ready().await
    }

    fn report_channel_metrics(&self, metrics_reporter: &mut MetricsReporter) {
        for metrics in &self.channel_metrics {
            if let Err(error) = metrics.report(metrics_reporter) {
                otel_warn!(
                    "extension_scope.channel_metrics.reporting_failed",
                    error = error.to_string()
                );
            }
        }
    }

    async fn shutdown(&mut self, scope: &ExtensionDeclarationScope) -> Result<(), Error> {
        self.shutdown_until(scope, Instant::now() + EXTENSION_SHUTDOWN_GRACE)
            .await
    }

    async fn shutdown_until(
        &mut self,
        scope: &ExtensionDeclarationScope,
        deadline: Instant,
    ) -> Result<(), Error> {
        self.terminal_metrics_deadline.record(deadline);
        self.lifecycle
            .initiate_shutdown_until(Some("extension scope host shutdown"), deadline);
        let timed_out = self.lifecycle.drain_until_deadline().await;

        let mut reporter = self.metrics_reporter.clone();
        self.report_channel_metrics(&mut reporter);
        let deadline = self.terminal_metrics_deadline.clone().get();
        if let Err(error) = self
            .lifecycle
            .finish_metrics_reporting_until(&reporter, deadline)
            .await
        {
            otel_warn!(
                "extension_scope.metrics.final_reporting_failed",
                error = error.to_string()
            );
        }
        if timed_out > 0 {
            return Err(Error::InternalError {
                message: format!(
                    "{scope} extension shutdown timed out; forcibly aborted {timed_out} task(s)"
                ),
            });
        }
        Ok(())
    }

    pub(super) async fn run(
        mut self,
        scope: ExtensionDeclarationScope,
        events: mpsc::UnboundedSender<ScopeEvent>,
        mut shutdown_rx: oneshot::Receiver<Instant>,
    ) -> Result<(), Error> {
        let readiness = tokio::select! {
            deadline = &mut shutdown_rx => {
                return match deadline {
                    Ok(deadline) => self.shutdown_until(&scope, deadline).await,
                    Err(_) => self.shutdown(&scope).await,
                };
            }
            readiness = self.wait_ready() => readiness,
        };
        let mut failure_reported = false;
        match readiness {
            Ok(()) => {
                if events.send(ScopeEvent::Ready(scope.clone())).is_err() {
                    return self.shutdown(&scope).await;
                }
            }
            Err(error) => {
                failure_reported = true;
                if let Err(send_error) = events.send(ScopeEvent::Failed {
                    scope: scope.clone(),
                    error,
                }) {
                    let _ = self.shutdown(&scope).await;
                    let ScopeEvent::Failed { error, .. } = send_error.0 else {
                        unreachable!("failed readiness notification must contain an error");
                    };
                    return Err(error);
                }
            }
        }

        loop {
            tokio::select! {
                deadline = &mut shutdown_rx => {
                    return match deadline {
                        Ok(deadline) => self.shutdown_until(&scope, deadline).await,
                        Err(_) => self.shutdown(&scope).await,
                    };
                }
                event = self.lifecycle.next_event() => {
                    match event {
                        LifecycleEvent::MonitorTick(now) => {
                            let mut reporter = self.metrics_reporter.clone();
                            self.lifecycle.monitor_tick(now, &mut reporter);
                            self.report_channel_metrics(&mut reporter);
                        }
                        LifecycleEvent::Completion(Ok(Ok(()))) => {
                            unreachable!(
                                "an extension completion before host shutdown must be upgraded to an error"
                            );
                        }
                        LifecycleEvent::Completion(Ok(Err(error))) => {
                            if !failure_reported {
                                failure_reported = true;
                                if let Err(send_error) = events.send(ScopeEvent::Failed {
                                    scope: scope.clone(),
                                    error,
                                }) {
                                    let _ = self.shutdown(&scope).await;
                                    let ScopeEvent::Failed { error, .. } = send_error.0 else {
                                        unreachable!(
                                            "failed lifecycle notification must contain an error"
                                        );
                                    };
                                    return Err(error);
                                }
                            } else {
                                otel_warn!(
                                    "extension_scope.host.additional_failure",
                                    scope = scope.to_string(),
                                    error = error.to_string()
                                );
                            }
                        }
                        LifecycleEvent::Completion(Err(error)) => {
                            let error = Error::JoinTaskError {
                                is_canceled: error.is_cancelled(),
                                is_panic: error.is_panic(),
                                error: error.to_string(),
                            };
                            if !failure_reported {
                                failure_reported = true;
                                if let Err(send_error) = events.send(ScopeEvent::Failed {
                                    scope: scope.clone(),
                                    error,
                                }) {
                                    let _ = self.shutdown(&scope).await;
                                    let ScopeEvent::Failed { error, .. } = send_error.0 else {
                                        unreachable!(
                                            "failed join notification must contain an error"
                                        );
                                    };
                                    return Err(error);
                                }
                            } else {
                                otel_warn!(
                                    "extension_scope.host.additional_failure",
                                    scope = scope.to_string(),
                                    error = error.to_string()
                                );
                            }
                        }
                    }
                }
                _ = next_optional_interval_tick(&mut self.channel_metrics_interval) => {
                    let mut reporter = self.metrics_reporter.clone();
                    self.report_channel_metrics(&mut reporter);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ControllerContext;
    use otel_arrow_dfe_telemetry::InternalTelemetrySystem;

    /// Scenario: an early provider failure uses a metrics fallback before its host begins shutdown.
    /// Guarantees: that local fallback does not constrain the host's later phase deadline.
    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_phase_does_not_inherit_pre_shutdown_metrics_fallback() {
        let telemetry = InternalTelemetrySystem::default();
        let prepared = PreparedExtensionScopeHost {
            context: ControllerContext::new(telemetry.registry()).engine_extension_context(),
            extensions: Vec::new(),
            channel_metrics: Vec::new(),
            runtime_policy: ExtensionHostRuntimePolicy {
                control_node_capacity: 1,
                pipeline_metrics: false,
                channel_metrics_enabled: false,
            },
        };
        tokio::task::LocalSet::new()
            .run_until(async {
                let collector =
                    tokio::task::spawn_local(telemetry.collector().run_collection_loop());
                let mut host = prepared.start(&telemetry.reporter());
                let reporter_deadline = host.terminal_metrics_deadline.clone();
                let early_failure_deadline = reporter_deadline.get();
                let shutdown_deadline = early_failure_deadline + EXTENSION_SHUTDOWN_GRACE;
                host.shutdown_until(&ExtensionDeclarationScope::Engine, shutdown_deadline)
                    .await
                    .expect("host shutdown should complete");
                assert_eq!(
                    reporter_deadline.get(),
                    shutdown_deadline,
                    "remaining providers must not inherit a pre-shutdown failure's cutoff"
                );
                reporter_deadline.record(shutdown_deadline + Duration::from_secs(1));
                assert_eq!(reporter_deadline.get(), shutdown_deadline);
                collector.abort();
                assert!(
                    collector
                        .await
                        .expect_err("collector should run until stopped")
                        .is_cancelled()
                );
            })
            .await;
    }

    /// Scenario: lifecycle metrics are disabled while runtime channel metrics
    /// remain enabled for an extension scope host.
    /// Guarantees: channel metrics receive an independent periodic reporting
    /// tick instead of being silently disabled with lifecycle metrics.
    #[test]
    fn channel_metrics_tick_is_independent_from_pipeline_metrics() {
        let runtime_only = ExtensionHostRuntimePolicy {
            control_node_capacity: 1,
            pipeline_metrics: false,
            channel_metrics_enabled: true,
        };
        let all_metrics = ExtensionHostRuntimePolicy {
            control_node_capacity: 1,
            pipeline_metrics: true,
            channel_metrics_enabled: true,
        };
        let no_metrics = ExtensionHostRuntimePolicy {
            control_node_capacity: 1,
            pipeline_metrics: false,
            channel_metrics_enabled: false,
        };

        assert!(runtime_only.needs_independent_channel_metrics_tick());
        assert!(!all_metrics.needs_independent_channel_metrics_tick());
        assert!(!no_metrics.needs_independent_channel_metrics_tick());
    }
}
