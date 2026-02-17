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
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
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
    /// Destinations. Each must map to a dedicated output port with exactly one downstream edge.
    #[serde(default)]
    pub destinations: Vec<DestinationConfig>,
    /// Interval for timeout checks when any destination declares a timeout.
    #[serde(
        with = "humantime_serde",
        default = "FanoutConfig::default_timeout_interval"
    )]
    pub timeout_check_interval: Duration,
    /// Maximum number of in-flight messages tracked by the processor.
    /// When exceeded, new messages are nacked to apply backpressure.
    /// Only applies when await_ack is "primary" or "all" (not "none").
    /// Default: 10000. Set to 0 for unlimited (not recommended for production).
    #[serde(default = "FanoutConfig::default_max_inflight")]
    pub max_inflight: usize,
}

impl FanoutConfig {
    const fn default_timeout_interval() -> Duration {
        // Default cadence for checking timeouts; conservative polling interval.
        Duration::from_millis(200)
    }

    const fn default_max_inflight() -> usize {
        // Default limit for in-flight messages to bound internal state.
        10_000
    }

    fn validate(mut self, node_config: &NodeUserConfig) -> Result<ValidatedConfig, ConfigError> {
        if self.destinations.is_empty() {
            return Err(ConfigError::InvalidUserConfig {
                error: "fanout: at least one destination is required".into(),
            });
        }

        // Limit to 64 destinations because completed_origins uses a u64 bitset.
        if self.destinations.len() > 64 {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "fanout: at most 64 destinations supported, got {}",
                    self.destinations.len()
                ),
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
        let declared_outputs: HashSet<PortName> = node_config.outputs.iter().cloned().collect();

        for (idx, dest) in self.destinations.iter().enumerate() {
            if !declared_outputs.contains(&dest.port) {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("fanout: unknown output `{}`", dest.port),
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
                // Check if the fallback target exists in any destination.
                if !self.destinations.iter().any(|d| &d.port == fb) {
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

        // Precompute fallback_for_dest: for each destination, find the immediate fallback (if any).
        // A fallback for dest[i] is a destination whose fallback_for points to dest[i].port.
        // Reject ambiguous configs where multiple destinations declare fallback_for the same port.
        let mut fallback_for_dest = vec![None; self.destinations.len()];
        for (fb_idx, fb_dest) in self.destinations.iter().enumerate() {
            if let Some(fb_for_port) = &fb_dest.fallback_for {
                if let Some(&origin_idx) = port_index.get(fb_for_port) {
                    if fallback_for_dest[origin_idx].is_some() {
                        return Err(ConfigError::InvalidUserConfig {
                            error: format!(
                                "fanout: multiple fallbacks declared for port `{}`",
                                fb_for_port
                            ),
                        });
                    }
                    fallback_for_dest[origin_idx] = Some(fb_idx);
                }
            }
        }

        // Compute fast-path eligibility flags.
        let has_any_fallback = self.destinations.iter().any(|d| d.fallback_for.is_some());
        let has_any_timeout = self.destinations.iter().any(|d| d.timeout.is_some());

        // Reject incompatible combinations.
        if matches!(self.await_ack, AwaitAck::None) && has_any_fallback {
            return Err(ConfigError::InvalidUserConfig {
                error: "fanout: fallback destinations are incompatible with await_ack: none (fire-and-forget mode ignores fallbacks)".into(),
            });
        }
        if matches!(self.await_ack, AwaitAck::None) && has_any_timeout {
            return Err(ConfigError::InvalidUserConfig {
                error: "fanout: timeouts are incompatible with await_ack: none (fire-and-forget mode does not track responses)".into(),
            });
        }

        // Fire-and-forget: await_ack == None
        let use_fire_and_forget = matches!(self.await_ack, AwaitAck::None);

        // Slim primary-only: parallel + primary + no fallback + no timeout
        let use_slim_primary = matches!(self.mode, DeliveryMode::Parallel)
            && matches!(self.await_ack, AwaitAck::Primary)
            && !has_any_fallback
            && !has_any_timeout;

        Ok(ValidatedConfig {
            mode: self.mode,
            await_ack: self.await_ack,
            destinations: self.destinations,
            primary_index,
            origins: origins_vec,
            fallback_for_dest,
            timeout_check_interval: self.timeout_check_interval,
            max_inflight: self.max_inflight,
            use_fire_and_forget,
            use_slim_primary,
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
    /// Precomputed map: dest_index -> immediate fallback index (if any).
    /// Avoids linear scan in handle_failure when triggering fallbacks.
    fallback_for_dest: Vec<Option<usize>>,
    timeout_check_interval: Duration,
    /// Maximum in-flight messages; 0 means unlimited.
    max_inflight: usize,
    /// Fast-path: await_ack == None, no inflight tracking needed.
    use_fire_and_forget: bool,
    /// Fast-path: parallel + primary + no fallback + no timeout.
    /// Uses slim inflight (request_id → original_pdata) instead of full DestinationVec.
    use_slim_primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DestinationStatus {
    PendingSend,
    InFlight,
    Acked,
    Nacked,
    /// Endpoint timed out - late acks/nacks should be ignored.
    TimedOut,
    /// Fallback skipped because its origin succeeded.
    Skipped,
}

#[derive(Debug)]
struct DestinationState {
    origin: usize,
    status: DestinationStatus,
    timeout_at: Option<Instant>,
    payload: Option<OtapPdata>,
}

/// Most fanout configurations have 2-4 destinations; inline storage avoids heap allocation.
type DestinationVec = SmallVec<[DestinationState; 4]>;
/// Queue of destination indices for sequential dispatch or pending sends.
type DestinationIndexQueue = SmallVec<[usize; 4]>;
/// Collection of deadlines returned from dispatch operations.
type DeadlineVec = SmallVec<[Deadline; 4]>;
/// Request IDs pending dispatch after timeout handling.
type PendingDispatchQueue = SmallVec<[u64; 8]>;

#[derive(Debug)]
struct Inflight {
    await_ack: AwaitAck,
    mode: DeliveryMode,
    primary: usize,
    /// Inline storage for up to 4 destinations to avoid heap allocation in common cases.
    destinations: DestinationVec,
    /// Bitset of origins (destination indices without fallback_for) that have completed (acked).
    /// Used with `await_ack: all` to track when all required destinations have responded.
    /// Supports up to 64 destinations; typical configs have 2-4.
    completed_origins: u64,
    /// Number of non-fallback destinations that must complete for `await_ack: all`.
    /// Fallbacks don't count toward this total since they replace their origin's outcome.
    required_origins: usize,
    /// Original pdata (before fanout's subscription was added) for upstream ack/nack.
    /// This ensures upstream routing uses the correct context stack.
    original_pdata: OtapPdata,
    /// Queue of destination indices to send next (used in sequential mode).
    /// In parallel mode, all non-fallback destinations are sent immediately.
    /// In sequential mode, only the head of this queue is dispatched at a time.
    next_send_queue: DestinationIndexQueue,
}

#[metric_set(name = "fanout.processor.metrics")]
#[derive(Debug, Default, Clone)]
struct FanoutMetrics {
    /// Requests dispatched. Note: This is a convenience metric that overlaps with
    /// channel-level send metrics. Consider removing if metric bloat is a concern.
    #[metric(unit = "{item}")]
    pub sent: Counter<u64>,
    /// Requests acked upstream (after await_ack/fallback aggregation).
    #[metric(unit = "{item}")]
    pub acked: Counter<u64>,
    /// Requests nacked upstream (after await_ack/fallback aggregation).
    #[metric(unit = "{item}")]
    pub nacked: Counter<u64>,
    #[metric(unit = "{item}")]
    pub timed_out: Counter<u64>,
    /// Messages rejected due to max_inflight limit (backpressure).
    #[metric(unit = "{item}")]
    pub rejected_max_inflight: Counter<u64>,
}

/// Entry in the deadline min-heap for efficient timeout checking.
/// Wrapped in Reverse<> when inserted to make BinaryHeap a min-heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Deadline {
    at: Instant,
    request_id: u64,
    dest_index: usize,
}

/// Fan-out processor implementation.
pub struct FanoutProcessor {
    config: ValidatedConfig,
    metrics: MetricSet<FanoutMetrics>,
    /// Full inflight tracking for complex scenarios (sequential, await_all, fallback, timeout).
    inflight: HashMap<u64, Inflight>,
    /// Slim inflight for primary-only fast path: just request_id -> original_pdata.
    slim_inflight: HashMap<u64, OtapPdata>,
    /// Min-heap of deadlines for O(log n) timeout checking.
    /// Uses Reverse<Deadline> to make BinaryHeap behave as a min-heap.
    deadline_heap: BinaryHeap<Reverse<Deadline>>,
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
            slim_inflight: HashMap::new(),
            deadline_heap: BinaryHeap::new(),
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

        let mut destinations_state = DestinationVec::new();
        let mut queue = DestinationIndexQueue::new();
        // We only need ack/nack notifications; the payload is not used since we keep
        // original_pdata for upstream routing and use calldata for destination lookup.
        let interests = Interests::ACKS_OR_NACKS;
        // Deadlines are initialized here and reset at actual dispatch time; a future tightening
        // could defer setting timeout_at until send if needed.
        let now = now();

        // TODO(optimization): Currently we clone pdata for ALL destinations upfront. In sequential
        // mode or with fallbacks, we could defer clone+subscribe until dispatch_ready to avoid
        // cloning payloads for destinations that are never sent (e.g., fallbacks when primary succeeds).
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
                DestinationStatus::PendingSend
            } else {
                queue.push(idx);
                DestinationStatus::InFlight
            };

            destinations_state.push(DestinationState {
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
                destinations: destinations_state,
                completed_origins: 0,
                required_origins,
                original_pdata: pdata,
                next_send_queue: queue,
            },
        );

        Ok(request_id)
    }

    /// Dispatch ready destinations and return any new deadlines for the heap.
    async fn dispatch_ready(
        request_id: u64,
        inflight: &mut Inflight,
        destinations: &[DestinationConfig],
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<DeadlineVec, TypedError<OtapPdata>> {
        let mut to_send = DestinationIndexQueue::new();
        let mut new_deadlines = DeadlineVec::new();
        match inflight.mode {
            DeliveryMode::Parallel => {
                for (idx, ep) in inflight.destinations.iter().enumerate() {
                    if matches!(ep.status, DestinationStatus::InFlight) {
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
            if let Some(payload) = inflight.destinations[idx].payload.take() {
                let timeout_at = destinations[idx].timeout.map(|d| now() + d);
                inflight.destinations[idx].timeout_at = timeout_at;
                if let Some(at) = timeout_at {
                    new_deadlines.push(Deadline {
                        at,
                        request_id,
                        dest_index: idx,
                    });
                }
                effect_handler
                    .send_message_to(destinations[idx].port.clone(), payload)
                    .await?;
            }
        }
        Ok(new_deadlines)
    }

    fn mark_complete(&mut self, request_id: u64, origin: usize) -> Option<OtapPdata> {
        if let Some(inflight) = self.inflight.get_mut(&request_id) {
            inflight.completed_origins |= 1u64 << origin;
            if inflight.completed_origins.count_ones() as usize >= inflight.required_origins {
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
        is_timeout: bool,
    ) -> Option<NackMsg<OtapPdata>> {
        let inflight = self.inflight.get_mut(&request_id)?;
        // Mark as TimedOut for timeouts, Nacked for explicit nacks.
        // TimedOut destinations will ignore late acks/nacks.
        inflight.destinations[dest_index].status = if is_timeout {
            DestinationStatus::TimedOut
        } else {
            DestinationStatus::Nacked
        };

        // Trigger fallback if configured (O(1) lookup via precomputed map).
        if let Some(fb_idx) = self.config.fallback_for_dest[dest_index] {
            if inflight.destinations[fb_idx].status == DestinationStatus::PendingSend {
                inflight.destinations[fb_idx].status = DestinationStatus::InFlight;
                let timeout_at = self.config.destinations[fb_idx].timeout.map(|d| now() + d);
                inflight.destinations[fb_idx].timeout_at = timeout_at;
                // Push fallback deadline to the heap if timeout is configured.
                if let Some(at) = timeout_at {
                    self.deadline_heap.push(Reverse(Deadline {
                        at,
                        request_id,
                        dest_index: fb_idx,
                    }));
                }
                if matches!(inflight.mode, DeliveryMode::Sequential) {
                    inflight.next_send_queue.clear();
                    inflight.next_send_queue.push(fb_idx);
                }
                return None;
            }
        }

        // No fallback, produce a nack using original pdata for correct upstream routing.
        Some(NackMsg {
            reason,
            calldata: CallData::new(),
            refused: Box::new(inflight.original_pdata.clone()),
            permanent: false, // Timeout is retriable
        })
    }

    async fn handle_timeout(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<Vec<NackMsg<OtapPdata>>, Error> {
        let now = now();
        let mut expired = Vec::new();
        let mut dispatch_requests = PendingDispatchQueue::new();

        // Pop expired deadlines from the min-heap (O(log n) per pop).
        while let Some(&Reverse(deadline)) = self.deadline_heap.peek() {
            if deadline.at > now {
                break; // No more expired deadlines.
            }
            let _ = self.deadline_heap.pop();

            let req = deadline.request_id;
            let idx = deadline.dest_index;

            // Validate the deadline is still relevant (request exists, destination still InFlight).
            let (await_ack, primary, origin, is_valid) = {
                let Some(inflight) = self.inflight.get(&req) else {
                    continue; // Request already completed.
                };
                if idx >= inflight.destinations.len() {
                    continue;
                }
                let ep = &inflight.destinations[idx];
                // Only process if still InFlight and deadline matches (guards against stale heap entries).
                let is_valid = matches!(ep.status, DestinationStatus::InFlight)
                    && ep.timeout_at == Some(deadline.at);
                (inflight.await_ack, inflight.primary, ep.origin, is_valid)
            };

            if !is_valid {
                continue;
            }

            self.metrics.timed_out.add(1);
            match self.handle_failure(
                req,
                idx,
                format!("fanout: timeout on {}", self.config.destinations[idx].port),
                true, // is_timeout
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
                    // Fallback triggered; dispatch it.
                    if !dispatch_requests.contains(&req) {
                        dispatch_requests.push(req);
                    }
                }
            }
        }

        for req in dispatch_requests {
            if let Some(inflight) = self.inflight.get_mut(&req) {
                let deadlines =
                    Self::dispatch_ready(req, inflight, &self.config.destinations, effect_handler)
                        .await?;
                for d in deadlines {
                    self.deadline_heap.push(Reverse(d));
                }
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
            if dest_index >= inflight.destinations.len() {
                return Ok(());
            }

            // Ignore late acks from destinations that already timed out or were nacked.
            // This prevents a late response from overriding fallback outcomes.
            let current_status = inflight.destinations[dest_index].status;
            if matches!(
                current_status,
                DestinationStatus::TimedOut | DestinationStatus::Nacked
            ) {
                return Ok(());
            }

            inflight.destinations[dest_index].status = DestinationStatus::Acked;
            let origin = inflight.destinations[dest_index].origin;

            // When an origin succeeds, mark its fallback(s) as Skipped so they won't be dispatched.
            for (idx, ep) in inflight.destinations.iter_mut().enumerate() {
                if ep.origin == origin
                    && ep.status == DestinationStatus::PendingSend
                    && self.config.destinations[idx].fallback_for.is_some()
                {
                    ep.status = DestinationStatus::Skipped;
                    ep.payload = None; // Release the payload
                }
            }

            if matches!(inflight.mode, DeliveryMode::Sequential) {
                inflight.next_send_queue.retain(|idx| *idx != dest_index);
                // Advance to the next pending send for this request (skip Skipped destinations).
                if inflight.next_send_queue.is_empty() {
                    if let Some(next_idx) = inflight
                        .destinations
                        .iter()
                        .enumerate()
                        .find(|(_, dest)| {
                            dest.status == DestinationStatus::PendingSend && dest.payload.is_some()
                        })
                        .map(|(idx, _)| idx)
                    {
                        inflight.destinations[next_idx].status = DestinationStatus::InFlight;
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
                    calldata: CallData::new(),
                };
                effect_handler.notify_ack(ack_to_return).await?;
            }
            return Ok(());
        }

        if matches!(mode, DeliveryMode::Sequential) {
            if let Some(inflight) = self.inflight.get_mut(&request_id) {
                let deadlines = Self::dispatch_ready(
                    request_id,
                    inflight,
                    &self.config.destinations,
                    effect_handler,
                )
                .await?;
                for d in deadlines {
                    self.deadline_heap.push(Reverse(d));
                }
            }
        }

        if matches!(await_ack, AwaitAck::All) {
            let maybe_ack = self.mark_complete(request_id, origin);
            if let Some(original_pdata) = maybe_ack {
                self.metrics.acked.add(1);
                // Use original_pdata for correct upstream routing
                let ackmsg = AckMsg {
                    accepted: Box::new(original_pdata),
                    calldata: CallData::new(),
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
            if dest_index >= inflight.destinations.len() {
                return Ok(());
            }

            // Ignore late nacks from destinations that already timed out.
            // This prevents a late response from overriding fallback outcomes.
            let current_status = inflight.destinations[dest_index].status;
            if matches!(current_status, DestinationStatus::TimedOut) {
                return Ok(());
            }

            (
                inflight.destinations[dest_index].origin,
                inflight.await_ack,
                inflight.primary,
            )
        };

        if matches!(await_ack, AwaitAck::Primary) && origin != primary {
            return Ok(());
        }

        if let Some(nackmsg) =
            self.handle_failure(request_id, dest_index, nack.reason.clone(), false)
        {
            self.metrics.nacked.add(1);
            let _ = self.inflight.remove(&request_id);
            effect_handler.notify_nack(nackmsg).await?;
            return Ok(());
        }

        // Fallback triggered: try dispatch immediately.
        if let Some(inflight) = self.inflight.get_mut(&request_id) {
            let deadlines = Self::dispatch_ready(
                request_id,
                inflight,
                &self.config.destinations,
                effect_handler,
            )
            .await?;
            for d in deadlines {
                self.deadline_heap.push(Reverse(d));
            }
            // Wait for fallback outcome instead of nacking now.
            return Ok(());
        }

        Ok(())
    }

    // =========================================================================
    // FAST PATH: Fire-and-forget (await_ack = none)
    // =========================================================================
    // No inflight tracking. Clone/send to each destination, ack upstream immediately.
    // Downstream acks/nacks are ignored (no subscription with ACKS/NACKS interests).

    async fn process_fire_and_forget(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Send to all destinations (parallel, no fallback tracking).
        for dest in &self.config.destinations {
            // Skip fallback destinations in fire-and-forget mode.
            if dest.fallback_for.is_some() {
                continue;
            }
            let dest_data = pdata.clone();
            // No subscription - we don't care about downstream acks/nacks.
            effect_handler
                .send_message_to(dest.port.clone(), dest_data)
                .await?;
        }
        self.metrics.sent.add(1);

        // Ack upstream immediately with original pdata.
        self.metrics.acked.add(1);
        effect_handler.notify_ack(AckMsg::new(pdata)).await?;
        Ok(())
    }

    // =========================================================================
    // FAST PATH: Slim primary-only (parallel + primary + no fallback/timeout)
    // =========================================================================
    // Minimal state: request_id → original_pdata. Ignore non-primary acks/nacks.
    //
    // Note: An optimization was considered to eliminate slim_inflight by keeping the original
    // context on primary (so acks route directly upstream) and using empty context on non-primary
    // (so their acks are dropped). However, this doesn't work with current engine wiring:
    // - Fanout must receive primary acks to update metrics, clear slim_inflight, and enforce
    //   max_inflight. If acks bypass fanout, state accumulates and backpressure breaks.
    // - The slim path already uses cheap clones (context stack is tiny), so gains would be minimal.
    // - A correct optimization would require engine changes (e.g., "passthrough observer" pattern
    //   to observe acks without being in the routing path).

    async fn process_slim_primary(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Check max_inflight limit before accepting new message.
        if self.config.max_inflight > 0 && self.slim_inflight.len() >= self.config.max_inflight {
            self.metrics.rejected_max_inflight.add(1);
            self.metrics.nacked.add(1);
            let nack = NackMsg {
                reason: format!(
                    "fanout: max_inflight limit ({}) exceeded",
                    self.config.max_inflight
                ),
                calldata: CallData::new(),
                refused: Box::new(pdata),
                permanent: false, // Backpressure is retriable
            };
            effect_handler.notify_nack(nack).await?;
            return Ok(());
        }

        let request_id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);

        // Store only the original pdata for upstream routing on primary ack.
        let _ = self.slim_inflight.insert(request_id, pdata.clone());

        // Send to all destinations with subscription so we receive acks.
        let interests = Interests::ACKS_OR_NACKS;
        for (idx, dest) in self.config.destinations.iter().enumerate() {
            let mut dest_data = pdata.clone();
            let calldata = build_calldata(request_id, idx);
            effect_handler.subscribe_to(interests, calldata, &mut dest_data);
            effect_handler
                .send_message_to(dest.port.clone(), dest_data)
                .await?;
        }
        self.metrics.sent.add(1);
        Ok(())
    }

    async fn process_ack_slim_primary(
        &mut self,
        ack: AckMsg<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let Some((request_id, dest_index)) = parse_calldata(&ack.calldata) else {
            return Ok(());
        };

        // Only care about primary acks.
        if dest_index != self.config.primary_index {
            return Ok(());
        }

        // Primary acked - forward upstream and clean up.
        if let Some(original_pdata) = self.slim_inflight.remove(&request_id) {
            self.metrics.acked.add(1);
            effect_handler
                .notify_ack(AckMsg::new(original_pdata))
                .await?;
        }
        Ok(())
    }

    async fn process_nack_slim_primary(
        &mut self,
        nack: NackMsg<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let Some((request_id, dest_index)) = parse_calldata(&nack.calldata) else {
            return Ok(());
        };

        // Only care about primary nacks (no fallback in slim path).
        if dest_index != self.config.primary_index {
            return Ok(());
        }

        // Primary nacked - forward upstream and clean up.
        if let Some(original_pdata) = self.slim_inflight.remove(&request_id) {
            self.metrics.nacked.add(1);
            let nackmsg = NackMsg {
                reason: nack.reason,
                calldata: CallData::new(),
                refused: Box::new(original_pdata),
                permanent: nack.permanent, // Propagate from downstream
            };
            effect_handler.notify_nack(nackmsg).await?;
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
                // Fire-and-forget never receives acks (no subscription).
                // Slim primary path uses dedicated handler.
                if self.config.use_slim_primary {
                    self.process_ack_slim_primary(ack, effect_handler).await
                } else {
                    self.process_ack(ack, effect_handler).await
                }
            }
            Message::Control(NodeControlMsg::Nack(nack)) => {
                // Fire-and-forget never receives nacks (no subscription).
                // Slim primary path uses dedicated handler.
                if self.config.use_slim_primary {
                    self.process_nack_slim_primary(nack, effect_handler).await
                } else {
                    self.process_nack(nack, effect_handler).await
                }
            }
            Message::Control(NodeControlMsg::TimerTick { .. }) => {
                // Fire-and-forget and slim primary don't start timers, so TimerTick
                // should not arrive. Early return as defensive measure.
                if self.config.use_fire_and_forget || self.config.use_slim_primary {
                    return Ok(());
                }
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
                // === FAST PATH 1: Fire-and-forget (await_ack = none) ===
                // No inflight tracking at all. Clone, send, ack upstream immediately.
                if self.config.use_fire_and_forget {
                    self.process_fire_and_forget(pdata, effect_handler).await?;
                    return Ok(());
                }

                // === FAST PATH 2: Slim primary-only (parallel + primary + no fallback/timeout) ===
                // Minimal state: just request_id → original_pdata.
                if self.config.use_slim_primary {
                    self.process_slim_primary(pdata, effect_handler).await?;
                    return Ok(());
                }

                // === FULL PATH: Sequential, await_all, fallback, or timeout ===
                // Full inflight tracking with DestinationVec.

                // Check max_inflight limit before accepting new message.
                if self.config.max_inflight > 0 && self.inflight.len() >= self.config.max_inflight {
                    self.metrics.rejected_max_inflight.add(1);
                    self.metrics.nacked.add(1);
                    let nack = NackMsg {
                        reason: format!(
                            "fanout: max_inflight limit ({}) exceeded",
                            self.config.max_inflight
                        ),
                        calldata: CallData::new(),
                        refused: Box::new(pdata),
                        permanent: false, // Backpressure is retriable
                    };
                    effect_handler.notify_nack(nack).await?;
                    return Ok(());
                }

                let request_id = self.register_inflight(pdata, effect_handler).await?;
                let inflight = self
                    .inflight
                    .get_mut(&request_id)
                    .expect("inflight just inserted");
                let deadlines = Self::dispatch_ready(
                    request_id,
                    inflight,
                    &self.config.destinations,
                    effect_handler,
                )
                .await?;
                for d in deadlines {
                    self.deadline_heap.push(Reverse(d));
                }
                self.metrics.sent.add(1);
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
    wiring_contract: otap_df_engine::wiring_contract::WiringContract {
        output_fanout: otap_df_engine::wiring_contract::OutputFanoutRule::AtMostPerOutput(1),
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::Context;
    use otap_df_config::SignalType;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{NodeControlMsg, PipelineControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::processor::EffectHandler;
    use otap_df_engine::message::Message;
    use otap_df_engine::message::Sender;
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
        let outputs = destinations_cfg
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|dest| dest.get("port").and_then(|v| v.as_str()))
            .map(|port| PortName::from(port.to_string()))
            .collect::<Vec<_>>();
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: outputs.clone(),
            default_output: None,
            config: json!({
                "mode": mode,
                "await_ack": await_ack,
                "destinations": destinations_cfg,
            }),
        };

        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipe".into(), 0, 0, 0);
        let fanout = FanoutProcessor::from_config(pipeline_ctx, &node_cfg, &node_cfg.config)
            .expect("valid config");

        let mut outputs = HashMap::new();
        let mut senders = HashMap::new();
        for port in &node_cfg.outputs {
            let (tx, rx) = otap_df_channel::mpsc::Channel::new(4);
            let _ = senders.insert(port.clone(), Sender::Local(LocalSender::mpsc(tx)));
            let _ = outputs.insert(port.to_string(), LocalReceiver::mpsc(rx));
        }

        let mut effect = EffectHandler::new(
            test_node("fanout"),
            senders,
            node_cfg.default_output.clone(),
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
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec![TEST_OUT_PORT_NAME.into()],
            default_output: None,
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
    fn config_rejects_more_than_64_destinations() {
        // The completed_origins bitset is a u64, so we can't support > 64 destinations.
        let destinations: Vec<DestinationConfig> = (0..65)
            .map(|i| DestinationConfig {
                port: format!("p{i}").into(),
                primary: i == 0,
                timeout: None,
                fallback_for: None,
            })
            .collect();
        let cfg = FanoutConfig {
            destinations,
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: (0..65).map(|i| PortName::from(format!("p{i}"))).collect(),
            default_output: None,
            config: json!({}),
        };
        let err = cfg
            .validate(&node_cfg)
            .expect_err("should reject >64 destinations");
        let msg = format!("{err}");
        assert!(
            msg.contains("64"),
            "expected 64 destination limit error, got: {msg}"
        );
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
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["p1".into(), "p2".into()],
            default_output: None,
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
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["p1".into(), "p2".into()],
            default_output: None,
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
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            outputs: vec!["p1".into()],
            telemetry_attributes: HashMap::new(),
            default_output: None,
            config: json!({}),
        };
        assert!(cfg.validate(&node_cfg).is_err());
    }

    #[test]
    fn config_rejects_fallback_with_fire_and_forget() {
        // Fallback destinations are incompatible with await_ack: none because
        // fire-and-forget mode ignores downstream acks/nacks and never triggers fallbacks.
        let cfg = FanoutConfig {
            await_ack: AwaitAck::None,
            destinations: vec![
                DestinationConfig {
                    port: "primary".into(),
                    primary: true,
                    timeout: None,
                    fallback_for: None,
                },
                DestinationConfig {
                    port: "backup".into(),
                    primary: false,
                    timeout: None,
                    fallback_for: Some("primary".into()),
                },
            ],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["primary".into(), "backup".into()],
            default_output: None,
            config: json!({}),
        };
        let err = cfg
            .validate(&node_cfg)
            .expect_err("should reject fallback with fire-and-forget");
        let msg = format!("{err}");
        assert!(
            msg.contains("fallback") && msg.contains("await_ack: none"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn config_rejects_timeout_with_fire_and_forget() {
        // Timeouts are incompatible with await_ack: none because fire-and-forget
        // mode doesn't track responses or use timers.
        let cfg = FanoutConfig {
            await_ack: AwaitAck::None,
            destinations: vec![DestinationConfig {
                port: "dest".into(),
                primary: true,
                timeout: Some(Duration::from_secs(5)),
                fallback_for: None,
            }],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["dest".into()],
            default_output: None,
            config: json!({}),
        };
        let err = cfg
            .validate(&node_cfg)
            .expect_err("should reject timeout with fire-and-forget");
        let msg = format!("{err}");
        assert!(
            msg.contains("timeout") && msg.contains("await_ack: none"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn config_rejects_fallback_cycle() {
        // Fallback cycles must be detected and rejected to prevent infinite loops.
        // Setup: primary (no fallback), a (fallback_for b), b (fallback_for a) = cycle a->b->a
        let cfg = FanoutConfig {
            destinations: vec![
                DestinationConfig {
                    port: "primary".into(),
                    primary: true,
                    timeout: None,
                    fallback_for: None,
                },
                DestinationConfig {
                    port: "a".into(),
                    primary: false,
                    timeout: None,
                    fallback_for: Some("b".into()),
                },
                DestinationConfig {
                    port: "b".into(),
                    primary: false,
                    timeout: None,
                    fallback_for: Some("a".into()),
                },
            ],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["primary".into(), "a".into(), "b".into()],
            default_output: None,
            config: json!({}),
        };
        let err = cfg
            .validate(&node_cfg)
            .expect_err("should reject fallback cycle");
        let msg = format!("{err}");
        assert!(msg.contains("cycle"), "expected cycle error, got: {msg}");
    }

    #[test]
    fn config_rejects_multiple_fallbacks_for_same_port() {
        // Multiple fallbacks pointing at the same origin port is ambiguous.
        let cfg = FanoutConfig {
            destinations: vec![
                DestinationConfig {
                    port: "primary".into(),
                    primary: true,
                    timeout: None,
                    fallback_for: None,
                },
                DestinationConfig {
                    port: "fb1".into(),
                    primary: false,
                    timeout: None,
                    fallback_for: Some("primary".into()),
                },
                DestinationConfig {
                    port: "fb2".into(),
                    primary: false,
                    timeout: None,
                    fallback_for: Some("primary".into()), // duplicate!
                },
            ],
            ..Default::default()
        };
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["primary".into(), "fb1".into(), "fb2".into()],
            default_output: None,
            config: json!({}),
        };
        let err = cfg
            .validate(&node_cfg)
            .expect_err("should reject duplicate fallback");
        let msg = format!("{err}");
        assert!(
            msg.contains("multiple fallbacks"),
            "expected multiple fallbacks error, got: {msg}"
        );
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
        let mut sent = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("output port"));
        assert_eq!(sent.len(), 1);
        let mut ack = AckMsg::new(sent.pop().unwrap());
        ack.calldata = ack.accepted.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");
        // This config uses slim_inflight (parallel + primary + no fallback/timeout).
        assert!(h.fanout.slim_inflight.is_empty());
    }

    #[test]
    fn duplicate_ports_are_rejected() {
        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: vec!["p1".into(), "p2".into()],
            default_output: None,
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
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipe".into(), 0, 0, 0);
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
            ack.calldata = ack.accepted.source_calldata().unwrap();
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
    async fn await_all_fail_fast_on_nack() {
        // Verify that await_ack: all fails fast when any destination nacks without fallback.
        // The upstream should receive nack immediately without waiting for other destinations.
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

        let mut p1 = drain(h.outputs.get_mut("p1").expect("p1"));
        let _p2 = drain(h.outputs.get_mut("p2").expect("p2"));
        assert_eq!(p1.len(), 1);

        // Nack from p1 - should trigger fail-fast without waiting for p2.
        let mut nack = NackMsg::new("p1 failed", p1.pop().unwrap());
        nack.calldata = nack.refused.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        // Fail-fast: inflight should be removed immediately.
        assert!(
            h.fanout.inflight.is_empty(),
            "fail-fast should remove inflight without waiting for p2"
        );

        // Upstream should receive nack immediately.
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
        assert_eq!(delivered_ack, 0, "should not ack upstream");
        assert_eq!(delivered_nack, 1, "should nack upstream immediately");
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
        ack_first.calldata = ack_first.accepted.source_calldata().unwrap();
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
        ack_second.calldata = ack_second.accepted.source_calldata().unwrap();
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
        nack.calldata = nack.refused.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        let backup = drain(h.outputs.get_mut("backup").expect("backup"));
        assert_eq!(backup.len(), 1);
        let mut ack = AckMsg::new(backup.into_iter().next().unwrap());
        ack.calldata = ack.accepted.source_calldata().unwrap();
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
        ack.calldata = ack.accepted.source_calldata().unwrap();
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
        nack_a.calldata = nack_a.refused.source_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Nack(nack_a)),
                &mut h.effect,
            )
            .await
            .expect("nack ok");

        let mut b_msg = drain(h.outputs.get_mut("b").expect("b"));
        let mut nack_b = NackMsg::new("b failed", b_msg.pop().unwrap());
        nack_b.calldata = nack_b.refused.source_calldata().unwrap();
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
        ack_c.calldata = ack_c.accepted.source_calldata().unwrap();
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
        nack.calldata = nack.refused.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        // Nack fallback.
        let mut backup = drain(h.outputs.get_mut("backup").expect("backup"));
        assert_eq!(backup.len(), 1);
        let mut nack_fb = NackMsg::new("fail backup", backup.pop().unwrap());
        nack_fb.calldata = nack_fb.refused.source_calldata().unwrap();
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
        // Verify fire-and-forget flag is set.
        assert!(h.fanout.config.use_fire_and_forget);

        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Fire-and-forget uses no inflight state at all.
        assert!(h.fanout.inflight.is_empty());
        assert!(h.fanout.slim_inflight.is_empty());

        let mut delivered = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if matches!(msg, PipelineControlMsg::DeliverAck { .. }) {
                delivered += 1;
            }
        }
        assert_eq!(delivered, 1);
    }

    #[tokio::test]
    async fn slim_primary_path_used_when_eligible() {
        // parallel + primary + no fallback + no timeout = slim path
        let mut h = build_harness(
            json!([
                make_dest("p1", true, None, None),
                make_dest("p2", false, None, None)
            ]),
            "parallel",
            "primary",
        );

        // Verify slim primary flag is set and fire-and-forget is not.
        assert!(h.fanout.config.use_slim_primary);
        assert!(!h.fanout.config.use_fire_and_forget);

        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Slim path uses slim_inflight, not full inflight.
        assert!(h.fanout.inflight.is_empty());
        assert!(!h.fanout.slim_inflight.is_empty());

        // Drain outputs.
        let mut p1 = drain(h.outputs.get_mut("p1").expect("p1"));
        let _p2 = drain(h.outputs.get_mut("p2").expect("p2"));

        // Ack from primary clears slim_inflight.
        let mut ack = AckMsg::new(p1.pop().unwrap());
        ack.calldata = ack.accepted.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");
        assert!(h.fanout.slim_inflight.is_empty());

        // Verify upstream ack delivered.
        let mut delivered_ack = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if matches!(msg, PipelineControlMsg::DeliverAck { .. }) {
                delivered_ack += 1;
            }
        }
        assert_eq!(delivered_ack, 1);
    }

    #[tokio::test]
    async fn slim_path_disabled_when_fallback_present() {
        // Adding fallback should disable slim path.
        let h = build_harness(
            json!([
                make_dest("primary", true, None, None),
                make_dest("backup", false, Some("primary"), None)
            ]),
            "parallel",
            "primary",
        );

        // Fallback disables slim path.
        assert!(!h.fanout.config.use_slim_primary);
        assert!(!h.fanout.config.use_fire_and_forget);
    }

    #[tokio::test]
    async fn slim_path_disabled_when_timeout_present() {
        // Adding timeout should disable slim path.
        let h = build_harness(
            json!([make_dest("p1", true, None, Some("100ms"))]),
            "parallel",
            "primary",
        );

        // Timeout disables slim path.
        assert!(!h.fanout.config.use_slim_primary);
        assert!(!h.fanout.config.use_fire_and_forget);
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
        nack.calldata = nack.refused.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");
        // This config uses slim_inflight (parallel + primary + no fallback/timeout).
        assert!(!h.fanout.slim_inflight.is_empty());

        let mut ack = AckMsg::new(prim.pop().unwrap());
        ack.calldata = ack.accepted.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");
        assert!(h.fanout.slim_inflight.is_empty());

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
        ack.calldata = ack.accepted.source_calldata().unwrap();
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
        ack_first.calldata = s1_msgs[0].source_calldata().unwrap();
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
        ack_second.calldata = s1_msgs[1].source_calldata().unwrap();
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
            ack.calldata = msg.source_calldata().unwrap();
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
        let mut sent = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("output port"));
        assert_eq!(sent.len(), 1);

        // Simulate downstream exporter acking - in real pipeline, Context::next_ack
        // would pop the fanout frame and route to fanout
        let mut ack = AckMsg::new(sent.pop().unwrap());
        ack.calldata = ack.accepted.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack)), &mut h.effect)
            .await
            .expect("ack ok");

        // Verify fanout's slim_inflight is cleared (this config uses slim path).
        assert!(h.fanout.slim_inflight.is_empty());

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

        let mut sent = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("output port"));
        assert_eq!(sent.len(), 1);

        // Simulate downstream exporter nacking
        let mut nack = NackMsg::new("downstream failed", sent.pop().unwrap());
        nack.calldata = nack.refused.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Nack(nack)), &mut h.effect)
            .await
            .expect("nack ok");

        // This config uses slim_inflight (parallel + primary + no fallback/timeout).
        assert!(h.fanout.slim_inflight.is_empty());

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

    fn build_harness_with_config(
        destinations: Value,
        mode: &str,
        await_ack: &str,
        extra_config: Value,
    ) -> FanoutHarness {
        let metrics_system = InternalTelemetrySystem::default();
        let controller_ctx = ControllerContext::new(metrics_system.registry());
        let destinations_cfg = destinations.clone();
        let outputs = destinations_cfg
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|dest| dest.get("port").and_then(|v| v.as_str()))
            .map(|port| PortName::from(port.to_string()))
            .collect::<Vec<_>>();

        let mut config = json!({
            "mode": mode,
            "await_ack": await_ack,
            "destinations": destinations_cfg,
        });
        // Merge extra_config into config
        if let (Some(base), Some(extra)) = (config.as_object_mut(), extra_config.as_object()) {
            for (k, v) in extra {
                let _ = base.insert(k.clone(), v.clone());
            }
        }

        let node_cfg = NodeUserConfig {
            r#type: FANOUT_PROCESSOR_URN.into(),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: outputs.clone(),
            default_output: None,
            config,
        };

        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipe".into(), 0, 0, 0);
        let fanout = FanoutProcessor::from_config(pipeline_ctx, &node_cfg, &node_cfg.config)
            .expect("valid config");

        let mut outputs = HashMap::new();
        let mut senders = HashMap::new();
        for port in &node_cfg.outputs {
            let (tx, rx) = otap_df_channel::mpsc::Channel::new(4);
            let _ = senders.insert(port.clone(), Sender::Local(LocalSender::mpsc(tx)));
            let _ = outputs.insert(port.to_string(), LocalReceiver::mpsc(rx));
        }

        let mut effect = EffectHandler::new(
            test_node("fanout"),
            senders,
            node_cfg.default_output.clone(),
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

    /// Test that max_inflight limits are enforced and messages are nacked when exceeded.
    #[tokio::test]
    async fn max_inflight_rejects_when_limit_exceeded() {
        // Use full path (with timeout) to test inflight map, max_inflight = 2
        let mut h = build_harness_with_config(
            json!([make_dest(TEST_OUT_PORT_NAME, true, None, Some("10s"))]),
            "parallel",
            "primary",
            json!({ "max_inflight": 2 }),
        );

        // Send 2 messages - should succeed (at limit)
        for _ in 0..2 {
            h.fanout
                .process(Message::PData(make_pdata()), &mut h.effect)
                .await
                .expect("process ok");
        }
        assert_eq!(h.fanout.inflight.len(), 2, "should have 2 inflight");

        // Send 3rd message - should be nacked due to max_inflight
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Inflight should still be 2
        assert_eq!(
            h.fanout.inflight.len(),
            2,
            "inflight should not grow beyond limit"
        );

        // Check that a nack was sent upstream for the rejected message
        let mut nack_received = false;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if let PipelineControlMsg::DeliverNack { nack, .. } = msg {
                assert!(
                    nack.reason.contains("max_inflight"),
                    "nack reason should mention max_inflight: {}",
                    nack.reason
                );
                nack_received = true;
            }
        }
        assert!(
            nack_received,
            "should have received a nack for rejected message"
        );
    }

    /// Test that max_inflight limits work for slim primary path.
    #[tokio::test]
    async fn max_inflight_slim_path_rejects_when_limit_exceeded() {
        // Use slim path (no timeout, no fallback) with max_inflight = 2
        let mut h = build_harness_with_config(
            json!([make_dest(TEST_OUT_PORT_NAME, true, None, None)]),
            "parallel",
            "primary",
            json!({ "max_inflight": 2 }),
        );

        // Verify slim path is used
        assert!(
            h.fanout.config.use_slim_primary,
            "should use slim primary path"
        );

        // Send 2 messages - should succeed
        for _ in 0..2 {
            h.fanout
                .process(Message::PData(make_pdata()), &mut h.effect)
                .await
                .expect("process ok");
        }
        assert_eq!(
            h.fanout.slim_inflight.len(),
            2,
            "should have 2 slim_inflight"
        );

        // Send 3rd message - should be nacked
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        assert_eq!(
            h.fanout.slim_inflight.len(),
            2,
            "slim_inflight should not grow beyond limit"
        );

        // Check nack was sent
        let mut nack_received = false;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if let PipelineControlMsg::DeliverNack { nack, .. } = msg {
                assert!(nack.reason.contains("max_inflight"));
                nack_received = true;
            }
        }
        assert!(nack_received, "should have received a nack");
    }

    /// Test that max_inflight = 0 means unlimited.
    #[tokio::test]
    async fn max_inflight_zero_means_unlimited() {
        let mut h = build_harness_with_config(
            json!([make_dest(TEST_OUT_PORT_NAME, true, None, Some("10s"))]),
            "parallel",
            "primary",
            json!({ "max_inflight": 0 }),
        );

        // Send messages in batches, draining the output channel between batches
        // to avoid blocking on the bounded channel (capacity 4).
        let total_messages = 20;
        for i in 0..total_messages {
            h.fanout
                .process(Message::PData(make_pdata()), &mut h.effect)
                .await
                .expect("process ok");

            // Drain outputs periodically to prevent channel backpressure
            if (i + 1) % 3 == 0 {
                let _ = drain(h.outputs.get_mut(TEST_OUT_PORT_NAME).expect("output port"));
            }
        }
        assert_eq!(
            h.fanout.inflight.len(),
            total_messages,
            "all messages should be inflight"
        );
    }

    /// Test behavior when late ack arrives from original destination after timeout triggered fallback.
    ///
    /// Sequence:
    /// 1. Primary times out → fallback dispatched, primary marked TimedOut
    /// 2. Late ack arrives from primary → IGNORED (destination is TimedOut)
    /// 3. Fallback acks → request completes
    /// 4. Upstream receives exactly one ack (from fallback outcome)
    #[tokio::test]
    async fn late_ack_from_timed_out_primary_is_ignored() {
        let mut h = build_harness(
            json!([
                make_dest("primary", true, None, Some("30ms")),
                make_dest("backup", false, Some("primary"), None)
            ]),
            "parallel",
            "primary",
        );

        // Send a message
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Drain primary output and save for late ack
        let primary_msgs = drain(h.outputs.get_mut("primary").expect("primary"));
        assert_eq!(primary_msgs.len(), 1);
        let primary_pdata = primary_msgs.into_iter().next().unwrap();

        // Wait for timeout and trigger timer tick
        sleep(Duration::from_millis(60)).await;
        h.fanout
            .process(
                Message::Control(NodeControlMsg::TimerTick {}),
                &mut h.effect,
            )
            .await
            .expect("timer tick ok");

        // Fallback should now be dispatched
        let backup_msgs = drain(h.outputs.get_mut("backup").expect("backup"));
        assert_eq!(
            backup_msgs.len(),
            1,
            "backup should be dispatched after timeout"
        );

        // Now send a LATE ack from the original primary (after timeout triggered fallback)
        // This should be IGNORED because the destination is marked TimedOut
        let mut late_ack = AckMsg::new(primary_pdata);
        late_ack.calldata = late_ack.accepted.source_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Ack(late_ack)),
                &mut h.effect,
            )
            .await
            .expect("late ack handled gracefully");

        // Request should still be inflight (waiting for backup - late ack was ignored)
        assert_eq!(
            h.fanout.inflight.len(),
            1,
            "request still inflight - late ack was ignored"
        );

        // Now backup acks - this should complete the request
        let mut backup_ack = AckMsg::new(backup_msgs.into_iter().next().unwrap());
        backup_ack.calldata = backup_ack.accepted.source_calldata().unwrap();
        h.fanout
            .process(
                Message::Control(NodeControlMsg::Ack(backup_ack)),
                &mut h.effect,
            )
            .await
            .expect("backup ack ok");

        // Request should be complete now
        assert!(
            h.fanout.inflight.is_empty(),
            "request complete after backup acks"
        );

        // Verify exactly one ack was delivered upstream (from backup)
        let mut ack_count = 0;
        while let Ok(Ok(msg)) =
            tokio::time::timeout(Duration::from_millis(50), h.pipeline_rx.recv()).await
        {
            if let PipelineControlMsg::DeliverAck { .. } = msg {
                ack_count += 1;
            }
        }
        assert_eq!(ack_count, 1, "exactly one ack delivered (from backup)");
    }

    /// Test sequential mode does NOT dispatch fallback when origin succeeds.
    ///
    /// Sequence:
    /// 1. Sequential mode with A (has fallback B) and C
    /// 2. A acks successfully → B is marked Skipped
    /// 3. C is dispatched next (B is skipped)
    #[tokio::test]
    async fn sequential_mode_skips_fallback_when_origin_succeeds() {
        let mut h = build_harness(
            json!([
                make_dest("a", true, None, None),
                make_dest("b", false, Some("a"), None), // fallback for a
                make_dest("c", false, None, None)       // regular destination
            ]),
            "sequential",
            "all", // await all to ensure we process all destinations
        );

        // Send a message - in sequential mode, only 'a' should be sent first
        h.fanout
            .process(Message::PData(make_pdata()), &mut h.effect)
            .await
            .expect("process ok");

        // Only 'a' should be dispatched initially
        let a_msgs = drain(h.outputs.get_mut("a").expect("a"));
        assert_eq!(a_msgs.len(), 1, "a should be dispatched first");
        assert!(
            drain(h.outputs.get_mut("b").expect("b")).is_empty(),
            "b not dispatched yet"
        );
        assert!(
            drain(h.outputs.get_mut("c").expect("c")).is_empty(),
            "c not dispatched yet"
        );

        // Ack 'a' - this should mark 'b' as Skipped and dispatch 'c'
        let mut ack_a = AckMsg::new(a_msgs.into_iter().next().unwrap());
        ack_a.calldata = ack_a.accepted.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack_a)), &mut h.effect)
            .await
            .expect("ack a ok");

        // 'b' (fallback) should NOT be dispatched because 'a' succeeded
        let b_msgs = drain(h.outputs.get_mut("b").expect("b"));
        assert!(
            b_msgs.is_empty(),
            "b (fallback) should NOT be dispatched when a succeeds"
        );

        // 'c' should now be dispatched
        let c_msgs = drain(h.outputs.get_mut("c").expect("c"));
        assert_eq!(c_msgs.len(), 1, "c dispatched after a (skipping b)");

        // Ack 'c' to complete
        let mut ack_c = AckMsg::new(c_msgs.into_iter().next().unwrap());
        ack_c.calldata = ack_c.accepted.source_calldata().unwrap();
        h.fanout
            .process(Message::Control(NodeControlMsg::Ack(ack_c)), &mut h.effect)
            .await
            .expect("ack c ok");

        // Request should be complete
        assert!(h.fanout.inflight.is_empty(), "request should be complete");
    }
}
