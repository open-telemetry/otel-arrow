// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fan-out processor that clones pdata to multiple downstream outputs.
//!
//! # Delivery Modes
//! - `parallel`: Send to all destinations simultaneously
//! - `sequential`: Send one-by-one, advance on ack
//!
//! # Ack Policies (`await_ack`)
//! - `none`: Fire-and-forget, ack upstream immediately
//! - `primary`: Wait for primary destination (or its fallback chain)
//! - `all`: Wait for all non-fallback destinations; fail-fast on any nack
//!
//! # Fallback
//! Destinations can declare `fallback_for: <port>`. On nack/timeout of the
//! origin, the fallback is triggered. Chains (A→B→C) are supported.
//!
//! See `fanout_processor/README.md` for detailed diagrams and examples.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::PortName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, CallData, Context8u8, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, TypedError};
use otap_df_engine::local::processor::{EffectHandler, Processor};
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::{ConsumerEffectHandlerExtension, Interests, ProducerEffectHandlerExtension};
use otap_df_engine::{ProcessorFactory, processor::ProcessorWrapper};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smallvec::{SmallVec, smallvec};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};

/// URN for the fan-out processor.
pub const FANOUT_PROCESSOR_URN: &str = "urn:otel:fanout:processor";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
enum DeliveryMode {
    /// Send to all active destinations immediately.
    #[default]
    Parallel,
    /// Send to destinations sequentially, moving to the next after an Ack.
    Sequential,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
enum AwaitAck {
    /// Wait for the primary (or its fallback) only.
    #[default]
    Primary,
    /// Wait for all active destinations (with replacement by fallback when configured).
    All,
    /// Do not wait; Ack upstream immediately after dispatch.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DestinationConfig {
    /// Out port name used to reach the destination.
    pub port: PortName,
    /// Whether this destination is the primary one (first wins if not specified).
    #[serde(default)]
    pub primary: bool,
    /// Timeout before treating the destination as failed.
    #[serde(with = "humantime_serde", default)]
    pub timeout: Option<Duration>,
    /// Optional port name this destination will act as a fallback for.
    #[serde(default)]
    pub fallback_for: Option<PortName>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FanoutConfig {
    /// Delivery pattern.
    #[serde(default)]
    pub mode: DeliveryMode,
    /// Ack policy.
    #[serde(default)]
    pub await_ack: AwaitAck,
    /// Destinations. Each must map to a dedicated out port with exactly one downstream edge.
    #[serde(default)]
    pub destinations: Vec<DestinationConfig>,
    /// Interval for timeout checks when any destination declares a timeout.
    #[serde(
        with = "humantime_serde",
        default = "FanoutConfig::default_timeout_interval"
    )]
    pub timeout_check_interval: Duration,
}

impl FanoutConfig {
    const fn default_timeout_interval() -> Duration {
        // Default cadence for checking timeouts; conservative polling interval.
        Duration::from_millis(200)
    }

    fn validate(mut self, node_config: &NodeUserConfig) -> Result<ValidatedConfig, ConfigError> {
        if self.destinations.is_empty() {
            return Err(ConfigError::InvalidUserConfig {
                error: "fanout: at least one destination is required".into(),
            });
        }

        // Default primary to the first destination if none set.
        if !self.destinations.iter().any(|d| d.primary) {
            if let Some(first) = self.destinations.first_mut() {
                first.primary = true;
            }
        }

        let mut primary_seen = false;
        let mut port_index = HashMap::new();
        let mut origins: HashSet<PortName> = HashSet::new();

        for (idx, dest) in self.destinations.iter().enumerate() {
            let out_port_cfg = node_config.out_ports.get(&dest.port).ok_or_else(|| {
                ConfigError::InvalidUserConfig {
                    error: format!("fanout: unknown out_port `{}`", dest.port),
                }
            })?;

            if out_port_cfg.destinations.len() != 1 {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "fanout: out_port `{}` must target exactly one destination to avoid load-balancing",
                        dest.port
                    ),
                });
            }

            if port_index.insert(dest.port.clone(), idx).is_some() {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("fanout: duplicate destination port `{}`", dest.port),
                });
            }

            if dest.primary {
                if dest.fallback_for.is_some() {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "fanout: primary destination `{}` cannot also be a fallback",
                            dest.port
                        ),
                    });
                }
                if primary_seen {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "fanout: only one primary destination is allowed".into(),
                    });
                }
                primary_seen = true;
            }

            if let Some(fb) = &dest.fallback_for {
                if !origins.contains(fb) && !self.destinations.iter().any(|d| &d.port == fb) {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "fanout: fallback_for `{}` does not reference a configured destination",
                            fb
                        ),
                    });
                }
            } else {
                let _ = origins.insert(dest.port.clone());
            }
        }

        let primary_index = self
            .destinations
            .iter()
            .position(|d| d.primary)
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: "fanout: missing primary destination".into(),
            })?;

        // Resolve fallback chains to their ultimate origin and detect cycles.
        let mut origins_vec = vec![0usize; self.destinations.len()];
        for (idx, _dest) in self.destinations.iter().enumerate() {
            let mut seen = HashSet::new();
            let mut current = idx;
            loop {
                if !seen.insert(current) {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "fanout: fallback cycle detected".into(),
                    });
                }
                if let Some(fb) = &self.destinations[current].fallback_for {
                    let next =
                        *port_index
                            .get(fb)
                            .ok_or_else(|| ConfigError::InvalidUserConfig {
                                error: format!("fanout: fallback_for `{}` not found", fb),
                            })?;
                    current = next;
                } else {
                    break;
                }
            }
            origins_vec[idx] = current;
        }

        Ok(ValidatedConfig {
            mode: self.mode,
            await_ack: self.await_ack,
            destinations: self.destinations,
            primary_index,
            origins: origins_vec,
            timeout_check_interval: self.timeout_check_interval,
        })
    }
}

#[derive(Debug, Clone)]
struct ValidatedConfig {
    mode: DeliveryMode,
    await_ack: AwaitAck,
    destinations: Vec<DestinationConfig>,
    primary_index: usize,
    origins: Vec<usize>,
    timeout_check_interval: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DestStatus {
    PendingSend,
    InFlight,
    Acked,
    Nacked,
}

#[derive(Debug)]
struct EndpointState {
    origin: usize,
    status: DestStatus,
    timeout_at: Option<Instant>,
    payload: Option<OtapPdata>,
}

#[derive(Debug)]
struct Inflight {
    await_ack: AwaitAck,
    mode: DeliveryMode,
    primary: usize,
    endpoints: Vec<EndpointState>,
    completed_origins: HashSet<usize>,
    required_origins: usize,
    /// Original pdata (before fanout's subscription was added) for upstream ack/nack.
    /// This ensures upstream routing uses the correct context stack.
    original_pdata: OtapPdata,
    next_send_queue: SmallVec<[usize; 4]>,
}

#[metric_set(name = "fanout.processor.metrics")]
#[derive(Debug, Default, Clone)]
struct FanoutMetrics {
    #[metric(unit = "{item}")]
    pub sent: Counter<u64>,
    #[metric(unit = "{item}")]
    pub acked: Counter<u64>,
    #[metric(unit = "{item}")]
    pub nacked: Counter<u64>,
    #[metric(unit = "{item}")]
    pub timed_out: Counter<u64>,
}

/// Fan-out processor implementation.
pub struct FanoutProcessor {
    config: ValidatedConfig,
    metrics: MetricSet<FanoutMetrics>,
    inflight: HashMap<u64, Inflight>,
    next_id: u64,
    timer_started: bool,
}

fn build_calldata(request_id: u64, dest_index: usize) -> CallData {
    smallvec![
        Context8u8::from(request_id),
        Context8u8::from(dest_index as u64)
    ]
}

fn parse_calldata(calldata: &CallData) -> Option<(u64, usize)> {
    if calldata.len() < 2 {
        return None;
    }
    let req = u64::from(calldata[0]);
    let dest = usize::try_from(u64::from(calldata[1])).ok()?;
    Some((req, dest))
}

fn now() -> Instant {
    Instant::now()
}

impl FanoutProcessor {
    fn new(pipeline_ctx: PipelineContext, config: ValidatedConfig) -> Self {
        let metrics = pipeline_ctx.register_metrics::<FanoutMetrics>();
        Self {
            config,
            metrics,
            inflight: HashMap::new(),
            next_id: 1,
            timer_started: false,
        }
    }

    fn from_config(
        pipeline_ctx: PipelineContext,
        node_config: &NodeUserConfig,
        user_config: &Value,
    ) -> Result<Self, ConfigError> {
        let cfg: FanoutConfig = serde_json::from_value(user_config.clone()).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("fanout: invalid config: {e}"),
            }
        })?;
        let validated = cfg.validate(node_config)?;
        Ok(Self::new(pipeline_ctx, validated))
    }

    async fn ensure_timer(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if self.timer_started || self.config.destinations.iter().all(|d| d.timeout.is_none()) {
            return Ok(());
        }
        let interval = self.config.timeout_check_interval;
        let _ = effect_handler.start_periodic_timer(interval).await?;
        self.timer_started = true;
        Ok(())
    }

    async fn register_inflight(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<u64, Error> {
        self.ensure_timer(effect_handler).await?;

        let request_id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);

        let mut endpoints = Vec::with_capacity(self.config.destinations.len());
        let mut queue = SmallVec::<[usize; 4]>::new();
        let interests = Interests::ACKS_OR_NACKS | Interests::RETURN_DATA;
        // Deadlines are initialized here and reset at actual dispatch time; a future tightening
        // could defer setting timeout_at until send if needed.
        let now = now();

        for (idx, dest) in self.config.destinations.iter().enumerate() {
            // Create a clone for this destination and subscribe with fanout calldata.
            let mut dest_data = pdata.clone();
            let calldata = build_calldata(request_id, idx);
            effect_handler.subscribe_to(interests, calldata, &mut dest_data);

            let timeout_at = dest.timeout.map(|d| now + d);
            let origin = self.config.origins[idx];

            // Fallback destinations are only sent when triggered.
            let is_fallback = dest.fallback_for.is_some();
            let status = if is_fallback
                || (matches!(self.config.mode, DeliveryMode::Sequential) && !queue.is_empty())
            {
                DestStatus::PendingSend
            } else {
                queue.push(idx);
                DestStatus::InFlight
            };

            endpoints.push(EndpointState {
                origin,
                status,
                timeout_at,
                payload: Some(dest_data),
            });
        }

        let required_origins = self
            .config
            .destinations
            .iter()
            .filter(|d| d.fallback_for.is_none())
            .count();

        let _ = self.inflight.insert(
            request_id,
            Inflight {
                await_ack: self.config.await_ack,
                mode: self.config.mode,
                primary: self.config.primary_index,
                endpoints,
                completed_origins: HashSet::new(),
                required_origins,
                original_pdata: pdata,
                next_send_queue: queue,
            },
        );

        Ok(request_id)
    }

    async fn dispatch_ready(
        inflight: &mut Inflight,
        destinations: &[DestinationConfig],
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), TypedError<OtapPdata>> {
        let mut to_send = SmallVec::<[usize; 4]>::new();
        match inflight.mode {
            DeliveryMode::Parallel => {
                for (idx, ep) in inflight.endpoints.iter().enumerate() {
                    if matches!(ep.status, DestStatus::InFlight) {
                        to_send.push(idx);
                    }
                }
            }
            DeliveryMode::Sequential => {
                if let Some(idx) = inflight.next_send_queue.first().copied() {
                    to_send.push(idx);
                }
            }
        }

        for idx in to_send {
            if let Some(payload) = inflight.endpoints[idx].payload.take() {
                inflight.endpoints[idx].timeout_at = destinations[idx].timeout.map(|d| now() + d);
                effect_handler
                    .send_message_to(destinations[idx].port.clone(), payload)
                    .await?;
            }
        }
        Ok(())
    }

    fn mark_complete(&mut self, request_id: u64, origin: usize) -> Option<OtapPdata> {
        if let Some(inflight) = self.inflight.get_mut(&request_id) {
            let _ = inflight.completed_origins.insert(origin);
            if inflight.completed_origins.len() >= inflight.required_origins {
                let entry = self.inflight.remove(&request_id)?;
                return Some(entry.original_pdata);
            }
        }
        None
    }

    fn handle_failure(
        &mut self,
        request_id: u64,
        dest_index: usize,
        reason: String,
    ) -> Option<NackMsg<OtapPdata>> {
        let inflight = self.inflight.get_mut(&request_id)?;
        let origin = inflight.endpoints[dest_index].origin;
        inflight.endpoints[dest_index].status = DestStatus::Nacked;

        // Trigger fallback if configured.
        if let Some((fb_idx, _)) = inflight.endpoints.iter().enumerate().find(|(idx, ep)| {
            ep.origin == origin
                && ep.status == DestStatus::PendingSend
                && self.config.destinations[*idx].fallback_for.is_some()
        }) {
            inflight.endpoints[fb_idx].status = DestStatus::InFlight;
            inflight.endpoints[fb_idx].timeout_at =
                self.config.destinations[fb_idx].timeout.map(|d| now() + d);
            if matches!(inflight.mode, DeliveryMode::Sequential) {
                inflight.next_send_queue.clear();
                inflight.next_send_queue.push(fb_idx);
            }
            return None;
        }

        // No fallback, produce a nack using original pdata for correct upstream routing.
        Some(NackMsg {
            reason,
            calldata: smallvec![],
            refused: Box::new(inflight.original_pdata.clone()),
        })
    }

    async fn handle_timeout(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<Vec<NackMsg<OtapPdata>>, Error> {
        let now = now();
        let mut expired = Vec::new();
        let mut dispatch_requests = HashSet::new();
        let requests: Vec<u64> = self.inflight.keys().cloned().collect();
        for req in requests {
            let mut timeouts = Vec::new();
            if let Some(inflight) = self.inflight.get_mut(&req) {
                for (idx, ep) in inflight.endpoints.iter().enumerate() {
                    if let Some(deadline) = ep.timeout_at {
                        if matches!(ep.status, DestStatus::InFlight) && deadline <= now {
                            timeouts.push(idx);
                        }
                    }
                }
            }
            for idx in timeouts {
                let (await_ack, primary, origin) = {
                    let inflight = match self.inflight.get(&req) {
                        Some(inflight) => inflight,
                        None => continue,
                    };
                    (
                        inflight.await_ack,
                        inflight.primary,
                        inflight.endpoints[idx].origin,
                    )
                };
                self.metrics.timed_out.add(1);
                match self.handle_failure(
                    req,
                    idx,
                    format!("fanout: timeout on {}", self.config.destinations[idx].port),
                ) {
                    Some(nack) => {
                        // Ignore non-primary timeouts when awaiting primary only.
                        if matches!(await_ack, AwaitAck::Primary) && origin != primary {
                            continue;
                        }
                        expired.push(nack);
                        let _ = self.inflight.remove(&req);
                    }
                    None => {
                        let _ = dispatch_requests.insert(req);
                    }
                }
            }
        }

        for req in dispatch_requests {
            if let Some(inflight) = self.inflight.get_mut(&req) {
                Self::dispatch_ready(inflight, &self.config.destinations, effect_handler).await?;
            }
        }

        Ok(expired)
    }

    async fn process_ack(
        &mut self,
        ack: AckMsg<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let Some((request_id, dest_index)) = parse_calldata(&ack.calldata) else {
            return Ok(());
        };
        let (origin, await_ack, primary, mode) = {
            let Some(inflight) = self.inflight.get_mut(&request_id) else {
                return Ok(());
            };
            if dest_index >= inflight.endpoints.len() {
                return Ok(());
            }
            inflight.endpoints[dest_index].status = DestStatus::Acked;
            let origin = inflight.endpoints[dest_index].origin;

            if matches!(inflight.mode, DeliveryMode::Sequential) {
                inflight.next_send_queue.retain(|idx| *idx != dest_index);
                // Advance to the next pending send for this request.
                if inflight.next_send_queue.is_empty() {
                    if let Some(next_idx) = inflight
                        .endpoints
                        .iter()
                        .enumerate()
                        .find(|(_, ep)| {
                            ep.status == DestStatus::PendingSend && ep.payload.is_some()
                        })
                        .map(|(idx, _)| idx)
                    {
                        inflight.endpoints[next_idx].status = DestStatus::InFlight;
                        inflight.next_send_queue.push(next_idx);
                    }
                }
            }
            (origin, inflight.await_ack, inflight.primary, inflight.mode)
        };

        if matches!(await_ack, AwaitAck::None) {
            return Ok(());
        }

        // Await primary: if this ack corresponds to primary origin (or its fallback) we can finish.
        if matches!(await_ack, AwaitAck::Primary) && origin == primary {
            let entry = self.inflight.remove(&request_id);
            self.metrics.acked.add(1);
            if let Some(inflight) = entry {
                // Use original_pdata for correct upstream routing
                let ack_to_return = AckMsg {
                    accepted: Box::new(inflight.original_pdata),
                    calldata: smallvec![],
                };
                effect_handler.notify_ack(ack_to_return).await?;
            }
            return Ok(());
        }

        if matches!(mode, DeliveryMode::Sequential) {
            if let Some(inflight) = self.inflight.get_mut(&request_id) {
                Self::dispatch_ready(inflight, &self.config.destinations, effect_handler).await?;
            }
        }

        if matches!(await_ack, AwaitAck::All) {
            let maybe_ack = self.mark_complete(request_id, origin);
            if let Some(original_pdata) = maybe_ack {
                self.metrics.acked.add(1);
                // Use original_pdata for correct upstream routing
                let ackmsg = AckMsg {
                    accepted: Box::new(original_pdata),
                    calldata: smallvec![],
                };
                effect_handler.notify_ack(ackmsg).await?;
            }
        }
        Ok(())
    }

    async fn process_nack(
        &mut self,
        nack: NackMsg<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let Some((request_id, dest_index)) = parse_calldata(&nack.calldata) else {
            return Ok(());
        };
        let (origin, await_ack, primary) = {
            let Some(inflight) = self.inflight.get_mut(&request_id) else {
                return Ok(());
            };
            if dest_index >= inflight.endpoints.len() {
                return Ok(());
            }
            (
                inflight.endpoints[dest_index].origin,
                inflight.await_ack,
                inflight.primary,
            )
        };

        if matches!(await_ack, AwaitAck::Primary) && origin != primary {
            return Ok(());
        }

        if let Some(nackmsg) = self.handle_failure(request_id, dest_index, nack.reason.clone()) {
            self.metrics.nacked.add(1);
            let _ = self.inflight.remove(&request_id);
            effect_handler.notify_nack(nackmsg).await?;
            return Ok(());
        }

        // Fallback triggered: try dispatch immediately.
        if let Some(inflight) = self.inflight.get_mut(&request_id) {
            Self::dispatch_ready(inflight, &self.config.destinations, effect_handler).await?;
            // Wait for fallback outcome instead of nacking now.
            return Ok(());
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl Processor<OtapPdata> for FanoutProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::Control(NodeControlMsg::Ack(ack)) => {
                self.process_ack(ack, effect_handler).await
            }
            Message::Control(NodeControlMsg::Nack(nack)) => {
                self.process_nack(nack, effect_handler).await
            }
            Message::Control(NodeControlMsg::TimerTick { .. }) => {
                for nack in self.handle_timeout(effect_handler).await? {
                    effect_handler.notify_nack(nack).await?;
                }
                Ok(())
            }
            Message::Control(NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            }) => {
                _ = metrics_reporter.report(&mut self.metrics);
                Ok(())
            }
            // Shutdown and other control messages are ignored: we drop inflight state on drop,
            // mirroring other stateless processors. Follow-up could proactively nack inflight on shutdown.
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                // Fast-path for await_ack = None: Ack immediately after dispatch.
                let await_ack_none = matches!(self.config.await_ack, AwaitAck::None);
                let request_id = self
                    .register_inflight(pdata.clone(), effect_handler)
                    .await?;
                let inflight = self
                    .inflight
                    .get_mut(&request_id)
                    .expect("inflight just inserted");
                Self::dispatch_ready(inflight, &self.config.destinations, effect_handler).await?;
                self.metrics.sent.add(1);

                if await_ack_none {
                    // Use original pdata for correct upstream routing
                    let entry = self.inflight.remove(&request_id);
                    self.metrics.acked.add(1);
                    if let Some(inflight) = entry {
                        effect_handler
                            .notify_ack(AckMsg::new(inflight.original_pdata))
                            .await?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// Factory to create a fan-out processor.
pub fn create_fanout_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let fanout =
        FanoutProcessor::from_config(pipeline_ctx.clone(), &node_config, &node_config.config)?;
    Ok(ProcessorWrapper::local(
        fanout,
        node,
        node_config,
        processor_config,
    ))
}

/// Register the fan-out processor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static FANOUT_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: FANOUT_PROCESSOR_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig| {
        create_fanout_processor(pipeline_ctx, node, node_config, proc_cfg)
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::Context;
    use otap_df_config::SignalType;
    use otap_df_config::node::{DispatchStrategy, HyperEdgeConfig, NodeKind, NodeUserConfig};
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{NodeControlMsg, PipelineControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::processor::EffectHandler;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::processor::TEST_OUT_PORT_NAME;
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
    use otap_df_telemetry::InternalTelemetrySystem;
    use serde_json::{Value, json};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::sleep;

    fn make_dest(
        port: &str,
        primary: bool,
        fallback_for: Option<&str>,
        timeout: Option<&str>,
    ) -> Value {
        let mut obj = json!({ "port": port, "primary": primary });
        if let Some(fb) = fallback_for {
            obj["fallback_for"] = json!(fb);
        }
        if let Some(to) = timeout {
            obj["timeout"] = json!(to);
        }
        obj
    }

    struct FanoutHarness {
        fanout: FanoutProcessor,
        effect: EffectHandler<OtapPdata>,
        outputs: HashMap<String, LocalReceiver<OtapPdata>>,
        pipeline_rx: otap_df_engine::shared::message::SharedReceiver<PipelineControlMsg<OtapPdata>>,
    }

    fn build_harness(destinations: Value, mode: &str, await_ack: &str) -> FanoutHarness {
        let metrics_system = InternalTelemetrySystem::default();
        let controller_ctx = ControllerContext::new(metrics_system.registry());
        let destinations_cfg = destinations.clone();
        let mut out_ports = std::collections::HashMap::new();
        let destinations_for_ports = destinations_cfg.clone();
        if let Some(arr) = destinations_for_ports.as_array() {
            for dest in arr {
                let port = dest
                    .get("port")
                    .and_then(|v| v.as_str())
                    .expect("port string")
                    .to_string();
                let port_name: PortName = port.clone().into();
                let _ = out_ports.insert(
                    port_name.clone(),
                    HyperEdgeConfig {
                        destinations: [test_node(format!("{}_dst", port)).name.clone()]
                            .into_iter()
                            .collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                );
            }
        }
        let node_cfg = NodeUserConfig {
            kind: NodeKind::Processor,
            plugin_urn: FANOUT_PROCESSOR_URN.into(),
            description: None,
            out_ports,
            default_out_port: None,
            config: json!({
                "mode": mode,
                "await_ack": await_ack,
                "destinations": destinations_cfg,
            }),
        };

        let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
        let fanout = FanoutProcessor::from_config(pipeline_ctx, &node_cfg, &node_cfg.config)
            .expect("valid config");

        let mut outputs = HashMap::new();
        let mut senders = HashMap::new();
        for port in node_cfg.out_ports.keys() {
            let (tx, rx) = otap_df_channel::mpsc::Channel::new(4);
            let _ = senders.insert(port.clone(), LocalSender::mpsc(tx));
            let _ = outputs.insert(port.to_string(), LocalReceiver::mpsc(rx));
        }

        let mut effect = EffectHandler::new(
            test_node("fanout"),
            senders,
            node_cfg.default_out_port.clone(),
            metrics_system.reporter(),
        );
        let (pipeline_tx, pipeline_rx) = pipeline_ctrl_msg_channel(10);
        effect.set_pipeline_ctrl_msg_sender(pipeline_tx);

        FanoutHarness {
            fanout,
            effect,
            outputs,
            pipeline_rx,
        }
    }

    fn drain(receiver: &mut LocalReceiver<OtapPdata>) -> Vec<OtapPdata> {
        let mut items = Vec::new();
        while let Ok(msg) = receiver.try_recv() {
            items.push(msg);
        }
        items
    }

    fn make_node_config() -> NodeUserConfig {
        NodeUserConfig {
            kind: NodeKind::Processor,
            plugin_urn: FANOUT_PROCESSOR_URN.into(),
            description: None,
            out_ports: [(
                TEST_OUT_PORT_NAME.into(),
                HyperEdgeConfig {
                    destinations: [test_node("downstream").name.clone()].into_iter().collect(),
                    dispatch_strategy: DispatchStrategy::Broadcast,
                },
            )]
            .into(),
            default_out_port: None,
            config: json!({
                "destinations": [
                    { "port": TEST_OUT_PORT_NAME, "primary": true }
                ],
                "await_ack": "primary"
            }),
        }
    }

    /// Simulated upstream node id for tests (e.g., a receiver before the fanout).
    const TEST_UPSTREAM_NODE_ID: usize = 12345;

    fn make_pdata() -> OtapPdata {
        let payload = OtapPayload::OtlpBytes(OtlpProtoBytes::empty(SignalType::Logs));
        // Simulate an upstream subscriber (e.g., receiver) so acks/nacks route correctly.
        OtapPdata::new(Context::default(), payload).test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            smallvec![],
            TEST_UPSTREAM_NODE_ID,
        )
    }

    #[test]
    fn config_requires_destination() {
        let cfg = FanoutConfig {
            destinations: Vec::new(),
            ..Default::default()
        };
        let node_cfg = make_node_config();
        assert!(cfg.validate(&node_cfg).is_err());
    }

    #[test]
    fn config_rejects_fallback_to_unknown_port() {
        let cfg = FanoutConfig {
            destinations: vec![
                DestinationConfig {
                    port: "p1".into(),
                    primary: true,
                    timeout: None,
                    fallback_for: None,
                },
                DestinationConfig {
                    port: "p2".into(),
                    primary: false,
                    timeout: None,
                    fallback_for: Some("missing".into()),
                },
            ],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            kind: NodeKind::Processor,
            plugin_urn: FANOUT_PROCESSOR_URN.into(),
            description: None,
            out_ports: [
                (
                    "p1".into(),
                    HyperEdgeConfig {
                        destinations: [test_node("d1").name.clone()].into_iter().collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                ),
                (
                    "p2".into(),
                    HyperEdgeConfig {
                        destinations: [test_node("d2").name.clone()].into_iter().collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                ),
            ]
            .into(),
            default_out_port: None,
            config: json!({}),
        };
        assert!(cfg.validate(&node_cfg).is_err());
    }

    #[test]
    fn config_rejects_multiple_primary() {
        let cfg = FanoutConfig {
            destinations: vec![
                DestinationConfig {
                    port: "p1".into(),
                    primary: true,
                    timeout: None,
                    fallback_for: None,
                },
                DestinationConfig {
                    port: "p2".into(),
                    primary: true,
                    timeout: None,
                    fallback_for: None,
                },
            ],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            kind: NodeKind::Processor,
            plugin_urn: FANOUT_PROCESSOR_URN.into(),
            description: None,
            out_ports: [
                (
                    "p1".into(),
                    HyperEdgeConfig {
                        destinations: [test_node("d1").name.clone()].into_iter().collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                ),
                (
                    "p2".into(),
                    HyperEdgeConfig {
                        destinations: [test_node("d2").name.clone()].into_iter().collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                ),
            ]
            .into(),
            default_out_port: None,
            config: json!({}),
        };
        assert!(cfg.validate(&node_cfg).is_err());
    }

    #[test]
    fn config_rejects_primary_marked_as_fallback() {
        let cfg = FanoutConfig {
            destinations: vec![DestinationConfig {
                port: "p1".into(),
                primary: true,
                timeout: None,
                fallback_for: Some("p0".into()),
            }],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            kind: NodeKind::Processor,
            plugin_urn: FANOUT_PROCESSOR_URN.into(),
            description: None,
            out_ports: [(
                "p1".into(),
                HyperEdgeConfig {
                    destinations: [test_node("d1").name.clone()].into_iter().collect(),
                    dispatch_strategy: DispatchStrategy::Broadcast,
                },
            )]
            .into(),
            default_out_port: None,
            config: json!({}),
        };
        assert!(cfg.validate(&node_cfg).is_err());
    }

    #[tokio::test]
    async fn processor_sends_and_acks_primary() {
        let mut h = build_harness(
            json!([make_dest(TEST_OUT_PORT_NAME, true, None, None)]),
            "parallel",
            "primary",
        );
        let data = make_pdata();
        h.fanout
            .process(Message::PData(data), &mut h.effect)
            .await
            .expect("process ok");
        let mut sent = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("out port"));
        assert_eq!(sent.len(), 1);
        let mut ack = AckMsg::new(sent.pop().unwrap());
        ack.calldata = ack.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");
        assert!(h.fanout.inflight.is_empty());
    }

    #[test]
    fn duplicate_ports_are_rejected() {
        let node_cfg = NodeUserConfig {
            kind: NodeKind::Processor,
            plugin_urn: FANOUT_PROCESSOR_URN.into(),
            description: None,
            out_ports: [
                (
                    "p1".into(),
                    HyperEdgeConfig {
                        destinations: [test_node("d1").name.clone()].into_iter().collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                ),
                (
                    "p2".into(),
                    HyperEdgeConfig {
                        destinations: [test_node("d2").name.clone()].into_iter().collect(),
                        dispatch_strategy: DispatchStrategy::Broadcast,
                    },
                ),
            ]
            .into(),
            default_out_port: None,
            config: json!({
                "destinations": [
                    { "port": "p1", "primary": true },
                    { "port": "p1", "primary": false }
                ],
                "await_ack": "primary"
            }),
        };

        let metrics_system = InternalTelemetrySystem::default();
        let controller_ctx = ControllerContext::new(metrics_system.registry());
        let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
        let err = FanoutProcessor::from_config(pipeline_ctx, &node_cfg, &node_cfg.config)
            .err()
            .expect("duplicate ports should fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("duplicate destination port"),
            "unexpected error: {msg}"
        );
    }

    #[tokio::test]
    async fn awaits_all_across_multiple_destinations() {
        let mut h = build_harness(
            json!([
                make_dest("p1", true, None, None),
                make_dest("p2", false, None, None)
            ]),
            "parallel",
            "all",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        let mut all = Vec::new();
        for port in ["p1", "p2"] {
            let r = h.outputs.get_mut(port).expect("port");
            all.push((port, drain(r)));
        }
        assert_eq!(all[0].1.len(), 1);
        assert_eq!(all[1].1.len(), 1);

        for (_, mut msgs) in all {
            let mut ack = AckMsg::new(msgs.pop().unwrap());
            ack.calldata = ack.accepted.current_calldata().unwrap();
            h.fanout
                .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
                .await
                .expect("ack ok");
        }

        assert!(h.fanout.inflight.is_empty());
        // Only one upstream ack is emitted.
        let mut delivered = 0;
        while let Ok(msg) = h.pipeline_rx.try_recv() {
            if matches!(msg, PipelineControlMsg::DeliverAck { .. }) {
                delivered += 1;
            }
        }
        assert_eq!(delivered, 1);
    }

    #[tokio::test]
    async fn sequential_mode_sends_to_next_destination_after_ack() {
        let mut h = build_harness(
            json!([
                make_dest("s1", true, None, None),
                make_dest("s2", false, None, None)
            ]),
            "sequential",
            "all",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        let first = drain(h.outputs.get_mut("s1").expect("s1"));
        let second = drain(h.outputs.get_mut("s2").expect("s2"));
        assert_eq!(first.len(), 1);
        assert!(second.is_empty());

        let mut ack_first = AckMsg::new(first.into_iter().next().unwrap());
        ack_first.calldata = ack_first.accepted.current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Ack(ack_first)),
                &mut h.effect,
            )
            .await
            .expect("ack ok");

        let next = drain(h.outputs.get_mut("s2").expect("s2"));
        assert_eq!(next.len(), 1);

        let mut ack_second = AckMsg::new(next.into_iter().next().unwrap());
        ack_second.calldata = ack_second.accepted.current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Ack(ack_second)),
                &mut h.effect,
            )
            .await
            .expect("ack ok");
        assert!(h.fanout.inflight.is_empty());
    }

    #[tokio::test]
    async fn fallback_on_nack_dispatches_to_backup() {
        let mut h = build_harness(
            json!([
                make_dest("primary", true, None, None),
                make_dest("backup", false, Some("primary"), Some("50ms"))
            ]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        let mut primary_msg = drain(h.outputs.get_mut("primary").expect("primary"));
        assert_eq!(primary_msg.len(), 1);
        let mut nack = NackMsg::new("fail", primary_msg.pop().unwrap());
        nack.calldata = nack.refused.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        let backup = drain(h.outputs.get_mut("backup").expect("backup"));
        assert_eq!(backup.len(), 1);
        let mut ack = AckMsg::new(backup.into_iter().next().unwrap());
        ack.calldata = ack.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");
        assert!(h.fanout.inflight.is_empty());

        let mut delivered_ack = 0;
        let mut delivered_nack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            match msg {
                PipelineControlMsg::DeliverAck { .. } => delivered_ack += 1,
                PipelineControlMsg::DeliverNack { .. } => delivered_nack += 1,
                _ => {}
            }
        }
        assert_eq!(delivered_ack, 1);
        assert_eq!(delivered_nack, 0);
    }

    #[tokio::test]
    async fn timeout_triggers_fallback_then_ack() {
        let mut h = build_harness(
            json!([
                make_dest("primary", true, None, Some("30ms")),
                make_dest("backup", false, Some("primary"), None)
            ]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        sleep(Duration::from_millis(60)).await;
        h.fanout
            .process(
                Message::Control(NodeControlMsg::TimerTick {}),
                &mut h.effect,
            )
            .await
            .expect("timer tick ok");

        let backup = drain(h.outputs.get_mut("backup").expect("backup"));
        assert_eq!(backup.len(), 1);
        let mut ack = AckMsg::new(backup.into_iter().next().unwrap());
        ack.calldata = ack.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");

        let mut delivered_ack = 0;
        let mut delivered_nack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            match msg {
                PipelineControlMsg::DeliverAck { .. } => delivered_ack += 1,
                PipelineControlMsg::DeliverNack { .. } => delivered_nack += 1,
                _ => {}
            }
        }
        assert_eq!(delivered_ack, 1);
        assert_eq!(delivered_nack, 0);
    }

    #[tokio::test]
    async fn chained_fallback_succeeds() {
        let mut h = build_harness(
            json!([
                make_dest("a", true, None, None),
                make_dest("b", false, Some("a"), None),
                make_dest("c", false, Some("b"), None)
            ]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Nack a, nack b, ack c.
        let mut a_msg = drain(h.outputs.get_mut("a").expect("a"));
        let mut nack_a = NackMsg::new("a failed", a_msg.pop().unwrap());
        nack_a.calldata = nack_a.refused.current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Nack(nack_a)),
                &mut h.effect,
            )
            .await
            .expect("nack ok");

        let mut b_msg = drain(h.outputs.get_mut("b").expect("b"));
        let mut nack_b = NackMsg::new("b failed", b_msg.pop().unwrap());
        nack_b.calldata = nack_b.refused.current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Nack(nack_b)),
                &mut h.effect,
            )
            .await
            .expect("nack ok");

        let c_msg = drain(h.outputs.get_mut("c").expect("c"));
        assert_eq!(c_msg.len(), 1);
        let mut ack_c = AckMsg::new(c_msg.into_iter().next().unwrap());
        ack_c.calldata = ack_c.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack_c)), &mut h.effect)
            .await
            .expect("ack ok");

        let mut delivered_ack = 0;
        let mut delivered_nack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            match msg {
                PipelineControlMsg::DeliverAck { .. } => delivered_ack += 1,
                PipelineControlMsg::DeliverNack { .. } => delivered_nack += 1,
                _ => {}
            }
        }
        assert_eq!(delivered_ack, 1);
        assert_eq!(delivered_nack, 0);
    }

    #[tokio::test]
    async fn timeout_triggers_nack() {
        let mut h = build_harness(
            json!([make_dest("t1", true, None, Some("50ms"))]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        sleep(Duration::from_millis(100)).await;
        h.fanout
            .process(
                Message::Control(NodeControlMsg::TimerTick {}),
                &mut h.effect,
            )
            .await
            .expect("timer tick ok");
        sleep(Duration::from_millis(1)).await;

        let mut delivered_nacks = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if matches!(msg, PipelineControlMsg::DeliverNack { .. }) {
                delivered_nacks += 1;
            }
        }
        assert_eq!(delivered_nacks, 1);
        assert!(h.fanout.inflight.is_empty());
    }

    #[tokio::test]
    async fn fallback_also_nacks() {
        let mut h = build_harness(
            json!([
                make_dest("primary", true, None, None),
                make_dest("backup", false, Some("primary"), None)
            ]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Nack primary.
        let mut primary_msg = drain(h.outputs.get_mut("primary").expect("primary"));
        assert_eq!(primary_msg.len(), 1);
        let mut nack = NackMsg::new("fail primary", primary_msg.pop().unwrap());
        nack.calldata = nack.refused.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        // Nack fallback.
        let mut backup = drain(h.outputs.get_mut("backup").expect("backup"));
        assert_eq!(backup.len(), 1);
        let mut nack_fb = NackMsg::new("fail backup", backup.pop().unwrap());
        nack_fb.calldata = nack_fb.refused.current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Nack(nack_fb)),
                &mut h.effect,
            )
            .await
            .expect("nack ok");

        let mut delivered_ack = 0;
        let mut delivered_nack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            match msg {
                PipelineControlMsg::DeliverAck { .. } => delivered_ack += 1,
                PipelineControlMsg::DeliverNack { .. } => delivered_nack += 1,
                _ => {}
            }
        }
        assert_eq!(delivered_ack, 0);
        assert_eq!(delivered_nack, 1);
        assert!(h.fanout.inflight.is_empty());
    }

    #[tokio::test]
    async fn fire_and_forget_ack_none() {
        let mut h = build_harness(
            json!([make_dest("ff", true, None, None)]),
            "parallel",
            "none",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");
        let mut delivered = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if matches!(msg, PipelineControlMsg::DeliverAck { .. }) {
                delivered += 1;
            }
        }
        assert_eq!(delivered, 1);
        assert!(h.fanout.inflight.is_empty());
    }

    #[tokio::test]
    async fn non_primary_nack_does_not_abort_primary() {
        let mut h = build_harness(
            json!([
                make_dest("prim", true, None, None),
                make_dest("sec", false, None, None)
            ]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");
        let mut prim = drain(h.outputs.get_mut("prim").expect("prim"));
        let mut sec = drain(h.outputs.get_mut("sec").expect("sec"));
        assert_eq!(prim.len(), 1);
        assert_eq!(sec.len(), 1);

        let mut nack = NackMsg::new("secondary fail", sec.pop().unwrap());
        nack.calldata = nack.refused.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");
        assert!(!h.fanout.inflight.is_empty());

        let mut ack = AckMsg::new(prim.pop().unwrap());
        ack.calldata = ack.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");
        assert!(h.fanout.inflight.is_empty());

        let mut delivered_ack = 0;
        let mut delivered_nack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            match msg {
                PipelineControlMsg::DeliverAck { .. } => delivered_ack += 1,
                PipelineControlMsg::DeliverNack { .. } => delivered_nack += 1,
                _ => {}
            }
        }
        assert_eq!(delivered_ack, 1);
        assert_eq!(delivered_nack, 0);
    }

    #[tokio::test]
    async fn non_primary_timeout_does_not_abort_primary() {
        let mut h = build_harness(
            json!([
                make_dest("prim", true, None, None),
                make_dest("sec", false, None, Some("20ms"))
            ]),
            "parallel",
            "primary",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");
        let mut prim = drain(h.outputs.get_mut("prim").expect("prim"));
        assert_eq!(prim.len(), 1);

        // Let secondary timeout
        sleep(Duration::from_millis(50)).await;
        h.fanout
            .process(
                Message::Control(NodeControlMsg::TimerTick {}),
                &mut h.effect,
            )
            .await
            .expect("timer tick ok");
        // Drain any timeout nacks (should be none upstream in primary-only mode).
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(10), h.pipeline_rx.recv()).await
        {
            if matches!(msg, PipelineControlMsg::DeliverNack { .. }) {
                panic!("unexpected nack upstream");
            }
        }

        // Primary still acks successfully
        let mut ack = AckMsg::new(prim.pop().unwrap());
        ack.calldata = ack.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");

        let mut delivered_ack = 0;
        let mut delivered_nack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            match msg {
                PipelineControlMsg::DeliverAck { .. } => delivered_ack += 1,
                PipelineControlMsg::DeliverNack { .. } => delivered_nack += 1,
                _ => {}
            }
        }
        assert_eq!(delivered_ack, 1);
        assert_eq!(delivered_nack, 0);
    }
    #[tokio::test]
    async fn sequential_multiple_requests_inflight() {
        let mut h = build_harness(
            json!([
                make_dest("s1", true, None, None),
                make_dest("s2", false, None, None)
            ]),
            "sequential",
            "all",
        );
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("first process ok");
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("second process ok");

        let s1_msgs = drain(h.outputs.get_mut("s1").expect("s1"));
        let s2_msgs = drain(h.outputs.get_mut("s2").expect("s2"));
        assert_eq!(s1_msgs.len(), 2);
        assert!(s2_msgs.is_empty());

        let mut ack_first = AckMsg::new(s1_msgs[0].clone());
        ack_first.calldata = s1_msgs[0].current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Ack(ack_first)),
                &mut h.effect,
            )
            .await
            .expect("ack ok");
        let s2_after_first = drain(h.outputs.get_mut("s2").expect("s2"));
        assert_eq!(s2_after_first.len(), 1);

        let mut ack_second = AckMsg::new(s1_msgs[1].clone());
        ack_second.calldata = s1_msgs[1].current_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Ack(ack_second)),
                &mut h.effect,
            )
            .await
            .expect("ack ok");
        let s2_after_second = drain(h.outputs.get_mut("s2").expect("s2"));
        assert_eq!(s2_after_second.len(), 1);

        for msg in s2_after_first
            .into_iter()
            .chain(s2_after_second.into_iter())
        {
            let mut ack = AckMsg::new(msg.clone());
            ack.calldata = msg.current_calldata().unwrap();
            h.fanout
                .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
                .await
                .expect("ack ok");
        }
        assert!(h.fanout.inflight.is_empty());
    }

    /// Test that verifies upstream ack is routed to the correct node with correct calldata.
    ///
    /// This test simulates the real pipeline scenario where:
    /// 1. A receiver subscribes to acks with its own calldata
    /// 2. Fanout receives the pdata and fans out to destinations
    /// 3. Downstream acks back to fanout
    /// 4. Fanout should propagate the ack to the original receiver (not to itself)
    #[tokio::test]
    async fn upstream_ack_routes_to_receiver_not_fanout() {
        use crate::testing::TestCallData;

        let mut h = build_harness(
            json!([make_dest(TEST_OUT_PORT_NAME, true, None, None)]),
            "parallel",
            "primary",
        );

        // Simulate a receiver's subscription: create pdata with upstream context frame
        const UPSTREAM_RECEIVER_NODE_ID: usize = 999;
        let upstream_calldata = TestCallData::new_with(42, 7);
        let data_with_upstream = make_pdata().test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            upstream_calldata.clone().into(),
            UPSTREAM_RECEIVER_NODE_ID,
        );

        // Process through fanout - fanout will add its own subscription frame
        h.fanout
            .process(Message::PData(data_with_upstream), &mut h.effect)
            .await
            .expect("process ok");

        // Get the sent message (now has both receiver and fanout frames)
        let mut sent = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("out port"));
        assert_eq!(sent.len(), 1);

        // Simulate downstream exporter acking - in real pipeline, Context::next_ack
        // would pop the fanout frame and route to fanout
        let mut ack = AckMsg::new(sent.pop().unwrap());
        ack.calldata = ack.accepted.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");

        // Verify fanout's inflight is cleared
        assert!(h.fanout.inflight.is_empty());

        // Now check the upstream delivery - should go to UPSTREAM_RECEIVER_NODE_ID
        let mut delivered_to_receiver = false;
        let mut wrong_node_id = None;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if let PipelineControlMsg::DeliverAck { node_id, ack } = msg {
                if node_id == UPSTREAM_RECEIVER_NODE_ID {
                    // Also verify calldata matches the upstream receiver's calldata
                    let received_calldata: Result<TestCallData, _> = ack.calldata.try_into();
                    assert_eq!(
                        received_calldata.expect("valid calldata"),
                        upstream_calldata,
                        "Ack calldata should match upstream receiver's calldata"
                    );
                    delivered_to_receiver = true;
                } else {
                    wrong_node_id = Some(node_id);
                }
            }
        }

        assert!(
            delivered_to_receiver,
            "Ack should be delivered to upstream receiver (node_id={}), but was delivered to node_id={:?}",
            UPSTREAM_RECEIVER_NODE_ID, wrong_node_id
        );
    }

    /// Test that verifies upstream nack is routed to the correct node with correct calldata.
    #[tokio::test]
    async fn upstream_nack_routes_to_receiver_not_fanout() {
        use crate::testing::TestCallData;

        let mut h = build_harness(
            json!([make_dest(TEST_OUT_PORT_NAME, true, None, None)]),
            "parallel",
            "primary",
        );

        // Simulate a receiver's subscription
        const UPSTREAM_RECEIVER_NODE_ID: usize = 888;
        let upstream_calldata = TestCallData::new_with(99, 11);
        let data_with_upstream = make_pdata().test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            upstream_calldata.clone().into(),
            UPSTREAM_RECEIVER_NODE_ID,
        );

        h.fanout
            .process(Message::PData(data_with_upstream), &mut h.effect)
            .await
            .expect("process ok");

        let mut sent = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("out port"));
        assert_eq!(sent.len(), 1);

        // Simulate downstream exporter nacking
        let mut nack = NackMsg::new("downstream failed", sent.pop().unwrap());
        nack.calldata = nack.refused.current_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        assert!(h.fanout.inflight.is_empty());

        // Check the upstream delivery - should go to UPSTREAM_RECEIVER_NODE_ID
        let mut delivered_to_receiver = false;
        let mut wrong_node_id = None;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if let PipelineControlMsg::DeliverNack { node_id, nack } = msg {
                if node_id == UPSTREAM_RECEIVER_NODE_ID {
                    let received_calldata: Result<TestCallData, _> = nack.calldata.try_into();
                    assert_eq!(
                        received_calldata.expect("valid calldata"),
                        upstream_calldata,
                        "Nack calldata should match upstream receiver's calldata"
                    );
                    delivered_to_receiver = true;
                } else {
                    wrong_node_id = Some(node_id);
                }
            }
        }

        assert!(
            delivered_to_receiver,
            "Nack should be delivered to upstream receiver (node_id={}), but was delivered to node_id={:?}",
            UPSTREAM_RECEIVER_NODE_ID, wrong_node_id
        );
    }
}
