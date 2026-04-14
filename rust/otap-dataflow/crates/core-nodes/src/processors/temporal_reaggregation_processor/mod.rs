// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Temporal reaggregation processor for OTAP metrics.
//!
//! This processor decreases telemetry volume by reaggregating metrics collected
//! at a higher frequency into a lower one.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use hashbrown::HashMap;
use hashbrown::hash_map::EntryRef::{Occupied, Vacant};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::ProducerEffectHandlerExtension;
use otap_df_engine::WakeupError;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{
    AckMsg, CallData, NackMsg, NodeControlMsg, WakeupRevision, WakeupSlot,
};
use otap_df_engine::error::{Error, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::{ProcessorRuntimeRequirements, ProcessorWrapper};
use otap_df_engine::{ConsumerEffectHandlerExtension, Interests};
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::accessory::slots::{Key as SlotKey, State as SlotState};
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::views::otap::OtapMetricsView;
use otap_df_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otap_df_pdata::{OtapPayload, OtapPayloadHelpers};
use otap_df_pdata_views::views::common::InstrumentationScopeView;
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, DataType, DataView, ExponentialHistogramDataPointView,
    ExponentialHistogramView, GaugeView, HistogramDataPointView, HistogramView, MetricView,
    MetricsView, NumberDataPointView, ResourceMetricsView, ScopeMetricsView, SumView,
    SummaryDataPointView, SummaryView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_warn;

mod builder;
mod config;
mod identity;
mod telemetry;

use self::builder::{Checkpoint, MetricSignalBuilder, StreamMeta};
use self::config::Config;
use self::identity::{
    HashBuffer, MetricId, MetricIdRef, ResourceId, ScopeId, ScopeIdRef, StreamId, StreamIdRef,
    metric_id_of, resource_id_of, scope_id_of, stream_id_of,
};
use self::telemetry::TemporalReaggregationMetrics;

/// Errors that can occur during view processing.
#[derive(thiserror::Error, Debug)]
enum ProcessingError {
    #[error("Engine: {source:?}")]
    Engine {
        #[source]
        source: Error,
    },

    #[error("Data failed to aggregate on retry: {source:?}")]
    AggregationRetryFailed {
        #[source]
        source: AggregationError,
    },
}

impl From<Error> for ProcessingError {
    fn from(value: Error) -> Self {
        ProcessingError::Engine { source: value }
    }
}

impl From<AggregationError> for ProcessingError {
    fn from(value: AggregationError) -> Self {
        ProcessingError::AggregationRetryFailed { source: value }
    }
}

/// Result of processing a single metrics pdata payload.
#[allow(clippy::large_enum_variant)]
enum AggregationResult {
    /// Nothing in the batch was aggregatable. The original pdata should be
    /// forwarded downstream unchanged.
    NoAggregations,
    /// Some metrics were non-aggregatable and have been assembled into a new
    /// [`OtapArrowRecords`] batch that should be sent downstream immediately.
    SomeAggregations(OtapArrowRecords),
    /// All metrics were aggregatable (buffered for later flush). Nothing to
    /// send downstream right now.
    AllAggregated,
}

/// Errors that can occur during view processing.
#[derive(thiserror::Error, Debug)]
enum AggregationError {
    /// An ID counter would exceed its maximum value.
    #[error("Id overflow")]
    IdOverflow,

    /// The number of unique metric streams in the current batch would exceed
    /// the configured [`Config::max_stream_cardinality`] limit.
    #[error("Stream cardinality exceeded")]
    StreamCardinalityExceeded,
}

/// The URN for the temporal reaggregation processor.
pub const TEMPORAL_REAGGREGATION_PROCESSOR_URN: &str = "urn:otel:processor:temporal_reaggregation";

/// Register the temporal reaggregation processor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: TEMPORAL_REAGGREGATION_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_temporal_reaggregation_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
    };

/// Factory function to create a [`TemporalReaggregationProcessor`].
pub fn create_temporal_reaggregation_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        TemporalReaggregationProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// A grouping of counters used to track the current otap id for each table.
/// This is split out from [`IdentityState`] because the passthrough batch doesn't
/// have a need for all of the hashmaps used to aggregate, but both batches need
/// to track these ids.
#[derive(Default)]
struct OtapIdState {
    resource: u16,
    scope: u16,
    metric: u16,
    ndp: u32,
    hdp: u32,
    ehdp: u32,
    sdp: u32,
    ndp_exemplar: u32,
    hdp_exemplar: u32,
    ehdp_exemplar: u32,
}

/// State for the current in-progress batch. This is all the stuff that has to
/// be cleared between batches
#[derive(Default)]
struct IdentityState {
    resources: HashMap<ResourceId, u16>,
    scopes: HashMap<ScopeId<'static>, u16>,
    metrics: HashMap<MetricId<'static>, u16>,
    streams: HashMap<StreamId<'static>, StreamMeta>,
    next: OtapIdState,
}

impl IdentityState {
    fn new() -> Self {
        Self::default()
    }

    fn clear(&mut self) {
        self.resources.clear();
        self.scopes.clear();
        self.metrics.clear();
        self.streams.clear();
        self.next = OtapIdState::default();
    }
}

const FLUSH_WAKEUP_SLOT: WakeupSlot = WakeupSlot(0);

/// The temporal reaggregation processor.
///
/// Accumulates metrics data over a collection interval, deduplicates
/// resources/scopes/metrics by identity, and tracks the latest data point per
/// stream. On each wakeup it flushes the accumulated state as an
/// [`OtapArrowRecords`] batch.
pub struct TemporalReaggregationProcessor {
    /// Processor metrics
    metrics: MetricSet<TemporalReaggregationMetrics>,

    /// The collection period for aggregating metrics before emitting a batch
    collection_period: Duration,

    /// Current wakeup revision.`None` when no wakeup is pending or we've
    /// cancelled any pending wakeup.
    wakeup_revision: Option<WakeupRevision>,

    /// Maximum number of unique streams allowed in a single aggregating batch.
    max_stream_cardinality: u16,

    // Reusable byte buffer for computing attribute hashes.
    hash_buf: HashBuffer,

    /// Identity related state for dedplucating metrics and assigning them
    /// integer Ids. It is outside the scope of [Self::builder] to figure out
    /// what ids to assign, so we do that here. Like the builder, this is reset
    /// on every flush trigger.
    identities: IdentityState,

    /// The in progress aggregated metrics builder. All the data which can be
    /// aggregated for each inbound batch is accumulated here until a flush
    /// trigger is hit and the builder is reset.
    builder: MetricSignalBuilder,

    /// Contexts for all of the inbound batches that have not been fully
    /// acked or nacked
    inbound_batches: SlotState<InboundTracker>,

    /// A list of CallData which are keys for [Self::inbound_batches] and
    /// point to the batches contibuting to aggregated data which has not been
    /// flushed.
    pending_flush: Vec<CallData>,

    /// A map of all passthrough and aggregated batches sent by this processor.
    /// The CallData returned to this processor in ack/nack messages can be
    /// casted to a [SlotKey] and points into this map.
    ///
    /// The CallData inside the entry points to every associated otap batch in
    /// [Self::inbound_batches] so that we can operate on the ref counts there.
    outbound_batches: SlotState<Vec<CallData>>,
}

struct InboundTracker {
    // Original request context
    ctx: Context,
    // Number of pending acks that we're waiting for
    pending_acks: usize,
    // Whether the batch containing the corresponding input data has been flushed
    flushed: bool,
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for TemporalReaggregationProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        self.debug_assert_invariants();
        let result = match msg {
            Message::PData(pdata) => {
                match pdata.signal_type() {
                    SignalType::Metrics => {
                        self.process_metric_pdata(effect_handler, pdata).await?;
                    }
                    // Non-metrics signals pass through unchanged.
                    SignalType::Logs | SignalType::Traces => {
                        effect_handler.send_message_with_source_node(pdata).await?;
                    }
                }
                Ok(())
            }
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::Wakeup { revision, .. } => {
                    if self.wakeup_revision == Some(revision) {
                        self.wakeup_revision = None;
                        self.metrics.flushes_timer.inc();
                        self.flush(effect_handler, None).await?;
                    }

                    Ok(())
                }
                NodeControlMsg::Ack(msg) => self.handle_ack(effect_handler, msg).await,
                NodeControlMsg::Nack(msg) => self.handle_nack(effect_handler, msg).await,
                NodeControlMsg::Shutdown { .. } => self.flush(effect_handler, None).await,
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    _ = metrics_reporter.report(&mut self.metrics);
                    Ok(())
                }
                _ => Ok(()),
            },
        };
        self.debug_assert_invariants();
        result
    }

    fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        ProcessorRuntimeRequirements::with_local_wakeups(1)
    }

    fn accept_pdata(&self) -> bool {
        // We may need up to one inbound slot and two outbound slots.
        //
        // The inbound slot is used to track the inbound request context and is
        // needed if either there are ack/nack interests on the context OR if some of
        // the data is aggregable meaning we won't just pass the whole batch
        // through untouched.
        //
        // One outbound slot may be needed to flush the existing batch if we
        // cannot accommodate the aggregable portion of the inbound payload.
        //
        // A second outbound slot may be needed for the passthrough portion
        // of the inbound data, if any.
        self.inbound_batches.has_capacity() && self.outbound_batches.remaining_capacity() >= 2
    }
}

impl TemporalReaggregationProcessor {
    /// Creates a new processor from a configuration JSON value.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<TemporalReaggregationMetrics>();
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        config.validate()?;
        Ok(Self {
            metrics,
            collection_period: config.period,
            wakeup_revision: None,
            max_stream_cardinality: config.max_stream_cardinality.get(),
            hash_buf: HashBuffer::new(),
            identities: IdentityState::new(),
            builder: MetricSignalBuilder::new(),
            inbound_batches: SlotState::new(config.inbound_request_limit.get()),
            pending_flush: Vec::new(),
            outbound_batches: SlotState::new(config.outbound_request_limit.get()),
        })
    }

    fn debug_assert_invariants(&self) {
        if !cfg!(debug_assertions) {
            return;
        }

        // pending_flush contains no duplicates
        for (i, a) in self.pending_flush.iter().enumerate() {
            for b in self.pending_flush[i + 1..].iter() {
                assert!(a != b, "pending_flush contains duplicate entries");
            }
        }

        // Every pending_flush entry that still has a valid inbound slot has
        // flushed == false (the slot may be missing if a nack removed it)
        for calldata in &self.pending_flush {
            let key = calldata.clone().try_into().expect("valid key");
            if let Some(tracker) = self.inbound_batches.get(key) {
                assert!(
                    !tracker.flushed,
                    "pending_flush entry has flushed=true (should have been drained on flush)"
                );
            }
        }

        // Per-tracker invariants
        for (_key, tracker) in self.inbound_batches.iter() {
            assert!(tracker.pending_acks <= 2,);
            assert!(!tracker.flushed || tracker.pending_acks > 0,);
            assert!(tracker.flushed || tracker.pending_acks <= 1,);
        }
    }

    async fn handle_ack(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        msg: AckMsg<OtapPdata>,
    ) -> Result<(), Error> {
        let Ok(outbound_key) = msg.unwind.route.calldata.clone().try_into() else {
            otel_warn!(telemetry::INVALID_CALLDATA_EVENT);
            return Ok(());
        };

        let Some(inbounds) = self.outbound_batches.take(outbound_key) else {
            otel_warn!(telemetry::INVALID_CALLDATA_EVENT);
            return Ok(());
        };

        for inbound_call_data in inbounds {
            // safety: Outbound calldata is entirely in our control so should always be
            // convertable to/from a slot key
            let inbound_key = inbound_call_data.try_into().expect("valid key");

            // The inbound may have already been removed by a nack on a
            // different outbound e.g. if the passthrough was nacked first
            let Some(tracker) = self.inbound_batches.get_mut(inbound_key) else {
                continue;
            };

            // Check and update pending acks. In general we can have at most two
            // pending acks for any input. One for aggregated data and one for
            // passthrough data.
            match tracker.pending_acks {
                0 => match tracker.flushed {
                    // Flushed data with 0 outbounds should never happen by invariants
                    // upheld entirely within this processor. If we ever hit 0 on a
                    // tracker marked as flushed then we should be acking it and
                    // removing it from the list.
                    true => {
                        panic!("Flushed tracker with 0 outbounds");
                    }

                    // "Best" case scenario is that this is an erroneous ack from
                    // some other misbehaving component. We're going to ignore it, but
                    // this implies a lack of correctness in another component or the
                    // engine itself.
                    false => {
                        otel_warn!(telemetry::ERRONEOUS_ACK_EVENT);
                        continue;
                    }
                },

                1 => match tracker.flushed {
                    // Refcount is going to 0, we're safe to ack this and remove it
                    true => {
                        let _ = tracker;
                        // safety: We know this is in the map, we just dropped the ref to it
                        let mut tracker = self.inbound_batches.take(inbound_key).expect("exists");
                        let payload = OtapPayload::empty(SignalType::Metrics);
                        let pdata = OtapPdata::new(std::mem::take(&mut tracker.ctx), payload);
                        effect_handler.notify_ack(AckMsg::new(pdata)).await?;
                    }

                    // We got a quick ack back on the passthrough batch before we
                    // flushed the stuff being aggregated. We update the outbounds
                    // and continue on.
                    false => {
                        tracker.pending_acks -= 1;
                        continue;
                    }
                },

                2 => match tracker.flushed {
                    // Both outbounds were pending, one came back. Decrement
                    // the counter.
                    true => {
                        tracker.pending_acks -= 1;
                    }

                    // Two is the maximum number of pending outbounds which we
                    // can only have if the data has been flushed. This invariant
                    // is upheld entirey by this processor alone.
                    false => {
                        panic!("Unflushed tracker with > 1 outbound");
                    }
                },

                _ => {
                    unreachable!("We can never have more than two outbounds for a batch");
                }
            }
        }

        Ok(())
    }

    async fn handle_nack(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        msg: NackMsg<OtapPdata>,
    ) -> Result<(), Error> {
        let Ok(outbound_key) = msg.unwind.route.calldata.clone().try_into() else {
            // It's either this or we raise an engine error and crash.
            // This is pretty bad as who knows if we're holding outbounds that
            // can ever be cleared if this is malfunctioning. Best case
            // scenario is the nack was erroneous and just didn't mean anything.
            otel_warn!(telemetry::INVALID_CALLDATA_EVENT);
            return Ok(());
        };

        let Some(inbounds) = self.outbound_batches.take(outbound_key) else {
            otel_warn!(telemetry::INVALID_CALLDATA_EVENT);
            return Ok(());
        };

        let NackMsg { reason, .. } = msg;
        for inbound in inbounds {
            // safety: The calldata is entirely in this processor's control and
            // should always represent a valid key for the inbound.
            let inbound_key = inbound.try_into().expect("valid slot key");

            // If we get multiple nacks for a single inbound, this is possible
            // and not really a problem. The most likely case is both the passthrough
            // batch and the aggregated batch were independently nacked.
            let Some(mut tracker) = self.inbound_batches.take(inbound_key) else {
                continue;
            };

            let payload = OtapPayload::empty(SignalType::Metrics);
            let pdata = OtapPdata::new(std::mem::take(&mut tracker.ctx), payload);
            effect_handler
                .notify_nack(NackMsg::new(reason.clone(), pdata))
                .await?;
        }

        Ok(())
    }

    /// Record a tracker for a new inbound batch by allocating a slot for the tracker,
    /// updating the pending_flush batches, and scheduling a wakeup if needed.
    fn record_pending_and_handle_wakeup(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        tracker: InboundTracker,
    ) -> Result<CallData, Error> {
        // safety: See [`TemporalReaggregationProcessor::accept_pdata`]
        // We always expect one inbound slot and two outbound slots to
        // be available before accepting pdata.
        let inbound_slot = self.inbound_batches.reserve().expect("available");
        let inbound_key = inbound_slot.insert(tracker);
        let inbound_calldata: CallData = inbound_key.into();

        self.ensure_wakeup_scheduled(effect_handler)?;

        self.pending_flush.push(inbound_calldata.clone());
        Ok(inbound_calldata)
    }

    /// Ensures a flush wakeup is scheduled if one isn't already pending.
    fn ensure_wakeup_scheduled(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if self.wakeup_revision.is_none() {
            self.reset_wakeup(effect_handler)?;
        }
        Ok(())
    }

    /// Cancels the current wakeup, if any, and schedules a new one.
    fn reset_wakeup(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        self.cancel_current_wakeup(effect_handler);
        self.schedule_wakeup(effect_handler)
    }

    // Schedule a new wakeup, note that this does not cancel the old one if
    // one is pending.
    fn schedule_wakeup(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let wakeup_time = Instant::now() + self.collection_period;
        match effect_handler.set_wakeup(FLUSH_WAKEUP_SLOT, wakeup_time) {
            Ok(outcome) => {
                self.wakeup_revision = Some(outcome.revision());
                Ok(())
            }
            // Shutdown is latched; We ignore this because the Shutdown message
            // will arrive after drain completes and trigger a final flush.
            Err(WakeupError::ShuttingDown) => Ok(()),
            Err(e) => Err(Error::ProcessorError {
                processor: effect_handler.processor_id(),
                kind: ProcessorErrorKind::Other,
                error: format!("could not set wakeup: {e:?}"),
                source_detail: String::new(),
            }),
        }
    }

    // Cancel current wakeup, if any
    fn cancel_current_wakeup(&mut self, effect_handler: &local::EffectHandler<OtapPdata>) {
        if self.wakeup_revision.is_some() {
            self.wakeup_revision = None;
            _ = effect_handler.cancel_wakeup(FLUSH_WAKEUP_SLOT);
        }
    }

    /// Parse and process a metrics pdata payload.
    ///
    /// # Returns
    ///
    /// - `Ok(ProcessResult::FullPassthrough)` if nothing in the batch is aggregatable
    ///   and the original pdata should be sent downstream as-is.
    /// - `Ok(ProcessResult::Passthrough(records))` if some metrics were non-aggregatable
    ///   and need to be sent downstream immediately.
    /// - `Ok(ProcessResult::AllAggregated)` if all metrics were aggregatable.
    async fn process_metric_pdata(
        &mut self,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
        pdata: OtapPdata,
    ) -> Result<(), Error> {
        let result = match pdata.payload_ref() {
            OtapPayload::OtapArrowRecords(records) => match OtapMetricsView::try_from(records) {
                Ok(view) => self.process_view(effect_handler, &view).await,
                Err(e) => {
                    otel_warn!(telemetry::VIEW_CREATION_FAILED_EVENT, error = %e);
                    self.metrics.batches_rejected.inc();
                    let msg = format!("Failed to create view: {:#}", e);
                    effect_handler
                        .notify_nack(NackMsg::new_permanent(msg, pdata))
                        .await?;

                    return Ok(());
                }
            },
            OtapPayload::OtlpBytes(otlp) => {
                let view = RawMetricsData::new(otlp.as_bytes());
                self.process_view(effect_handler, &view).await
            }
        };

        match result {
            Ok(agg_result) => match agg_result {
                AggregationResult::NoAggregations => {
                    Ok(effect_handler.send_message_with_source_node(pdata).await?)
                }
                AggregationResult::SomeAggregations(records) => {
                    // Pass data through if there are no subscribers — but we
                    // still need a wakeup to flush the aggregated portion.
                    let (inbound_ctx, _) = pdata.into_parts();
                    if !inbound_ctx.has_subscribers() {
                        self.ensure_wakeup_scheduled(effect_handler)?;
                        let pt_pdata =
                            OtapPdata::new(inbound_ctx, OtapPayload::OtapArrowRecords(records));

                        effect_handler
                            .send_message_with_source_node(pt_pdata)
                            .await?;
                        return Ok(());
                    }

                    // One ref for the passthrough, we will bump the ack count again
                    // when we actually flush the batch.
                    let tracker = InboundTracker {
                        ctx: inbound_ctx,
                        pending_acks: 1,
                        flushed: false,
                    };

                    let inbound_calldata: CallData =
                        self.record_pending_and_handle_wakeup(effect_handler, tracker)?;

                    // safety: See [`TemporalReaggregationProcessor::accept_pdata`].
                    // We always expect one inbound and two outbound slots available
                    // before accepting pdata.
                    let outbound_slot = self.outbound_batches.reserve().expect("available");
                    let outbound_key = outbound_slot.insert(vec![inbound_calldata]);

                    // Subscribe to the outbound
                    let outbound_calldata: CallData = outbound_key.into();
                    let context = Context::with_capacity(1);
                    let mut pt_pdata =
                        OtapPdata::new(context, OtapPayload::OtapArrowRecords(records));

                    effect_handler.subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        outbound_calldata,
                        &mut pt_pdata,
                    );

                    Ok(effect_handler
                        .send_message_with_source_node(pt_pdata)
                        .await?)
                }
                AggregationResult::AllAggregated => {
                    // Nothing to passthrough and no subscribers so we don't
                    // care about ack/nack — but we still need a wakeup to
                    // flush the aggregated data.
                    let (inbound_ctx, _) = pdata.into_parts();
                    if !inbound_ctx.has_subscribers() {
                        self.ensure_wakeup_scheduled(effect_handler)?;
                        return Ok(());
                    }

                    // 0 outbounds because we haven't actually sent anything yet. Once
                    // we flush we can bump the ref count
                    let tracker = InboundTracker {
                        ctx: inbound_ctx,
                        pending_acks: 0,
                        flushed: false,
                    };

                    _ = self.record_pending_and_handle_wakeup(effect_handler, tracker)?;

                    Ok(())
                }
            },
            Err(proc_err) => match proc_err {
                // Engine errors are fatal, we propagate those up the stack
                ProcessingError::Engine { source } => Err(source),

                // This is our classic "bad data" case where even after flushing
                // the current batch, it failed on retry due to being oversized
                // in some way. We can't handle it, so it gets a nack.
                ProcessingError::AggregationRetryFailed { source } => {
                    self.metrics.batches_rejected.inc();
                    let msg = format!("Failed to aggregate batch: {:#}", source);
                    effect_handler
                        .notify_nack(NackMsg::new_permanent(msg, pdata))
                        .await?;

                    Ok(())
                }
            },
        }
    }

    /// Flush accumulated metrics up to the given checkpoint and reset state.
    ///
    /// This is used when a [`process_view`] call fails due to ID overflow —
    /// the data appended before the checkpoint is clean and should be sent
    /// downstream, while the partial data from the failed call is discarded
    /// by slicing the record batches.
    async fn flush(
        &mut self,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
        checkpoint: Option<Checkpoint>,
    ) -> Result<(), Error> {
        let records = self.builder.finish(checkpoint);
        self.clear_state();
        // Whenever we flush we cancel the current wakeup if any. Wakeups are
        // scheduled whenever we start aggregating a new batch
        self.cancel_current_wakeup(effect_handler);

        if records.is_empty() {
            return Ok(());
        }

        let pending_flush_calldata = std::mem::take(&mut self.pending_flush);

        // Nobody was subscribed to anything here
        if pending_flush_calldata.is_empty() {
            let context = Context::default();
            let pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(records));
            effect_handler.send_message_with_source_node(pdata).await?;
            return Ok(());
        }

        // safety: See [`TemporalReaggregationProcessor::accept_pdata`].
        // We always expect to have enough available slots before accepting pdata
        let outbound_slot = self.outbound_batches.reserve().expect("available");
        for inbound_calldata in pending_flush_calldata.iter().cloned() {
            // safety: We created all of this calldata
            let inbound_key: SlotKey = inbound_calldata.try_into().expect("Valid slot key");
            // It's possible that the passthrough batch was nacked prior to
            // flush for some inbound pdata and we've already removed the inbound.
            let Some(inbound_tracker) = self.inbound_batches.get_mut(inbound_key) else {
                continue;
            };

            inbound_tracker.pending_acks += 1;
            inbound_tracker.flushed = true;
        }

        let outbound_key = outbound_slot.insert(pending_flush_calldata);
        let outbound_calldata: CallData = outbound_key.into();
        let outbound_ctx = Context::with_capacity(1);
        let mut pdata = OtapPdata::new(outbound_ctx, OtapPayload::OtapArrowRecords(records));
        effect_handler.subscribe_to(
            Interests::ACKS | Interests::NACKS,
            outbound_calldata,
            &mut pdata,
        );

        effect_handler.send_message_with_source_node(pdata).await?;
        Ok(())
    }

    async fn process_view<V: MetricsView>(
        &mut self,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
        view: &V,
    ) -> Result<AggregationResult, ProcessingError> {
        if !has_aggregatable_metrics(view) {
            return Ok(AggregationResult::NoAggregations);
        }

        let checkpoint = self.builder.checkpoint();
        match self.aggregate_view(view) {
            Ok(res) => Ok(res),
            Err(e) => match e {
                // Both ID overflow and stream cardinality overflow are handled
                // the same way: seal the current outbound batch and feed the
                // data back into a fresh one. This prevents complex ack/nack
                // scenarios where a single input batch has representation in
                // multiple output batches.
                AggregationError::IdOverflow | AggregationError::StreamCardinalityExceeded => {
                    self.metrics.flushes_overflow.inc();
                    self.flush(effect_handler, Some(checkpoint)).await?;
                    Ok(self
                        .aggregate_view(view)
                        .inspect_err(|_| self.clear_state())?)
                }
            },
        }
    }

    /// Process a view, routing aggregatable metrics to the in-progress batch
    /// and non-aggregatable metrics to a passthrough batch that is returned for
    /// immediate downstream delivery.
    ///
    /// May fail due to id overflow or other capacity related restrictions.
    fn aggregate_view<V: MetricsView>(
        &mut self,
        view: &V,
    ) -> Result<AggregationResult, AggregationError> {
        let mut next_pt_id = OtapIdState::default();
        let mut pt_builder = MetricSignalBuilder::new();

        for resource_metrics in view.resources() {
            let resource = resource_metrics.resource();
            let resource_schema_url = resource_metrics.schema_url().unwrap_or(b"");

            let mut agg_resource: Option<(u16, ResourceId)> = None;
            let mut pt_resource_id: Option<u16> = None;

            for scope_metrics in resource_metrics.scopes() {
                let scope = scope_metrics.scope();
                let scope_schema_url = scope_metrics.schema_url();

                let mut agg_scope: Option<(u16, ScopeId<'_>)> = None;
                let mut pt_scope_id: Option<u16> = None;

                for metric in scope_metrics.metrics() {
                    let Some(data) = metric.data() else {
                        continue;
                    };

                    if is_data_aggregatable(&data) {
                        // Lazily process resource and scopes until we know if
                        // they have something aggregatable.
                        let (res_otap_id, resource_identity) = match agg_resource {
                            Some(x) => x,
                            None => {
                                let rid = resource_id_of(&mut self.hash_buf, resource.as_ref());
                                let id = self.process_resource(rid, resource.as_ref())?;
                                agg_resource = Some((id, rid));
                                (id, rid)
                            }
                        };

                        let (scp_otap_id, scope_identity) = match agg_scope.as_ref() {
                            Some(x) => x,
                            None => {
                                let sid = scope_id_of(
                                    &mut self.hash_buf,
                                    resource_identity,
                                    scope.as_ref(),
                                );
                                let id = self.process_scope(sid.clone(), scope.as_ref())?;
                                agg_scope = Some((id, sid));
                                agg_scope.as_ref().expect("agg_scope is some")
                            }
                        };

                        // Compute full metric identity for dedup
                        let Some(metric_id) = metric_id_of(scope_identity.clone(), &metric) else {
                            continue;
                        };

                        let otap_metric_id = self.process_metric(
                            metric_id.clone(),
                            &metric,
                            res_otap_id,
                            resource_schema_url,
                            resource.as_ref(),
                            *scp_otap_id,
                            scope.as_ref(),
                            scope_schema_url,
                        )?;

                        self.process_aggregatable_datapoints(&data, &metric_id, otap_metric_id)?;
                    } else {
                        let res_otap_id = match pt_resource_id {
                            Some(x) => x,
                            None => {
                                let id = next_id_16(&mut next_pt_id.resource)?;
                                if let Some(ref resource) = resource {
                                    pt_builder.append_resource(id, resource);
                                }
                                pt_resource_id = Some(id);
                                id
                            }
                        };

                        let scp_otap_id = match pt_scope_id {
                            Some(x) => x,
                            None => {
                                let id = next_id_16(&mut next_pt_id.scope)?;
                                if let Some(ref scope) = scope {
                                    pt_builder.append_scope(id, scope);
                                }
                                pt_scope_id = Some(id);
                                id
                            }
                        };

                        let otap_metric_id = next_id_16(&mut next_pt_id.metric)?;
                        let (data_type, aggregation_temporality, is_monotonic) =
                            identity::metric_type_info_of(&data);
                        pt_builder.append_metric(
                            otap_metric_id,
                            &metric,
                            data_type,
                            aggregation_temporality,
                            is_monotonic,
                            res_otap_id,
                            resource_schema_url,
                            resource.as_ref(),
                            scp_otap_id,
                            scope.as_ref(),
                            scope_schema_url,
                        );

                        self.append_passthrough_datapoints(
                            &mut pt_builder,
                            &mut next_pt_id,
                            &data,
                            otap_metric_id,
                        )?;
                    }
                }
            }
        }

        if next_pt_id.metric == 0 {
            return Ok(AggregationResult::AllAggregated);
        }

        Ok(AggregationResult::SomeAggregations(pt_builder.finish(None)))
    }

    fn process_resource<R: ResourceView>(
        &mut self,
        resource_id: ResourceId,
        view: Option<&R>,
    ) -> Result<u16, AggregationError> {
        match self.identities.resources.entry_ref(&resource_id) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.identities.next.resource;
                if id == u16::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.resource += 1;
                if let Some(view) = view {
                    self.builder.append_resource(id, view);
                }
                _ = v.insert_with_key(resource_id, id);
                Ok(id)
            }
        }
    }

    fn process_scope<S: InstrumentationScopeView>(
        &mut self,
        scope_id: ScopeId<'_>,
        view: Option<&S>,
    ) -> Result<u16, AggregationError> {
        let lookup = ScopeIdRef(&scope_id.clone());
        match self.identities.scopes.entry_ref(&lookup) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.identities.next.scope;
                if id == u16::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.scope += 1;
                if let Some(view) = view {
                    self.builder.append_scope(id, view);
                }
                _ = v.insert_with_key(scope_id.into_owned(), id);
                Ok(id)
            }
        }
    }

    fn process_metric<M: MetricView, R: ResourceView, S: InstrumentationScopeView>(
        &mut self,
        metric_id: MetricId<'_>,
        view: &M,
        resource_otap_id: u16,
        resource_schema_url: &[u8],
        resource_view: Option<&R>,
        scope_otap_id: u16,
        scope_view: Option<&S>,
        scope_schema_url: &[u8],
    ) -> Result<u16, AggregationError> {
        let lookup = MetricIdRef(&metric_id.clone());
        match self.identities.metrics.entry_ref(&lookup) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.identities.next.metric;
                if id == u16::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.metric += 1;
                self.builder.append_metric(
                    id,
                    view,
                    metric_id.data_type,
                    metric_id.aggregation_temporality,
                    metric_id.is_monotonic,
                    resource_otap_id,
                    resource_schema_url,
                    resource_view,
                    scope_otap_id,
                    scope_view,
                    scope_schema_url,
                );
                _ = v.insert_with_key(metric_id.into_owned(), id);
                Ok(id)
            }
        }
    }

    /// Append all data points from an aggregatable metric to the in-progress
    /// aggregation batch.
    fn process_aggregatable_datapoints<'a, D: DataView<'a>>(
        &mut self,
        data: &D,
        metric_id: &MetricId<'_>,
        otap_metric_id: u16,
    ) -> Result<(), AggregationError> {
        match data.value_type() {
            DataType::Gauge => {
                if let Some(gauge) = data.as_gauge() {
                    for dp in gauge.data_points() {
                        let sid =
                            stream_id_of(&mut self.hash_buf, metric_id.clone(), dp.attributes());
                        self.process_number_dp(&dp, sid, otap_metric_id)?;
                    }
                }
            }
            DataType::Sum => {
                if let Some(sum) = data.as_sum() {
                    for dp in sum.data_points() {
                        let sid =
                            stream_id_of(&mut self.hash_buf, metric_id.clone(), dp.attributes());
                        self.process_number_dp(&dp, sid, otap_metric_id)?;
                    }
                }
            }
            DataType::Histogram => {
                if let Some(hist) = data.as_histogram() {
                    for dp in hist.data_points() {
                        let sid =
                            stream_id_of(&mut self.hash_buf, metric_id.clone(), dp.attributes());
                        self.process_histogram_dp(&dp, sid, otap_metric_id)?;
                    }
                }
            }
            DataType::ExponentialHistogram => {
                if let Some(exp) = data.as_exponential_histogram() {
                    for dp in exp.data_points() {
                        let sid =
                            stream_id_of(&mut self.hash_buf, metric_id.clone(), dp.attributes());
                        self.process_exp_histogram_dp(&dp, sid, otap_metric_id)?;
                    }
                }
            }
            DataType::Summary => {
                if let Some(summary) = data.as_summary() {
                    for dp in summary.data_points() {
                        let sid =
                            stream_id_of(&mut self.hash_buf, metric_id.clone(), dp.attributes());
                        self.process_summary_dp(&dp, sid, otap_metric_id)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn process_number_dp<V: NumberDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) -> Result<(), AggregationError> {
        let time = dp.time_unix_nano();
        let streams_len = self.identities.streams.len();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.identities.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_number_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                if streams_len >= self.max_stream_cardinality as usize {
                    return Err(AggregationError::StreamCardinalityExceeded);
                }
                let dp_id = self.identities.next.ndp;
                if dp_id == u32::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.ndp += 1;
                let row_index = self.builder.append_number_dp(dp_id, otap_metric_id, dp);
                _ = v.insert_with_key(
                    stream_id.into_owned(),
                    StreamMeta {
                        dp_row_index: row_index,
                        time_unix_nano: time,
                    },
                );
            }
        }
        Ok(())
    }

    fn process_histogram_dp<V: HistogramDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) -> Result<(), AggregationError> {
        let time = dp.time_unix_nano();
        let streams_len = self.identities.streams.len();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.identities.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_histogram_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                if streams_len >= self.max_stream_cardinality as usize {
                    return Err(AggregationError::StreamCardinalityExceeded);
                }
                let dp_id = self.identities.next.hdp;
                if dp_id == u32::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.hdp += 1;
                let row_index = self.builder.append_histogram_dp(dp_id, otap_metric_id, dp);
                _ = v.insert_with_key(
                    stream_id.into_owned(),
                    StreamMeta {
                        dp_row_index: row_index,
                        time_unix_nano: time,
                    },
                );
            }
        }
        Ok(())
    }

    fn process_exp_histogram_dp<V: ExponentialHistogramDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) -> Result<(), AggregationError> {
        let time = dp.time_unix_nano();
        let streams_len = self.identities.streams.len();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.identities.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_exp_histogram_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                if streams_len >= self.max_stream_cardinality as usize {
                    return Err(AggregationError::StreamCardinalityExceeded);
                }
                let dp_id = self.identities.next.ehdp;
                if dp_id == u32::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.ehdp += 1;
                let row_index = self
                    .builder
                    .append_exp_histogram_dp(dp_id, otap_metric_id, dp);
                _ = v.insert_with_key(
                    stream_id.into_owned(),
                    StreamMeta {
                        dp_row_index: row_index,
                        time_unix_nano: time,
                    },
                );
            }
        }
        Ok(())
    }

    fn process_summary_dp<V: SummaryDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) -> Result<(), AggregationError> {
        let time = dp.time_unix_nano();
        let streams_len = self.identities.streams.len();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.identities.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_summary_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                if streams_len >= self.max_stream_cardinality as usize {
                    return Err(AggregationError::StreamCardinalityExceeded);
                }
                let dp_id = self.identities.next.sdp;
                if dp_id == u32::MAX {
                    return Err(AggregationError::IdOverflow);
                }
                self.identities.next.sdp += 1;
                let row_index = self.builder.append_summary_dp(dp_id, otap_metric_id, dp);
                _ = v.insert_with_key(
                    stream_id.into_owned(),
                    StreamMeta {
                        dp_row_index: row_index,
                        time_unix_nano: time,
                    },
                );
            }
        }
        Ok(())
    }

    /// Append all data points from a non-aggregatable metric to the passthrough
    /// builder, including exemplars.
    fn append_passthrough_datapoints<'a, D: DataView<'a>>(
        &mut self,
        pt_builder: &mut MetricSignalBuilder,
        ids: &mut OtapIdState,
        data: &D,
        otap_metric_id: u16,
    ) -> Result<(), AggregationError> {
        match data.value_type() {
            DataType::Gauge => {
                if let Some(gauge) = data.as_gauge() {
                    for dp in gauge.data_points() {
                        let dp_id = next_id_32(&mut ids.ndp)?;
                        let _ = pt_builder.append_number_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_number_dp_exemplars(dp_id, &dp, &mut ids.ndp_exemplar)?;
                    }
                }
            }
            DataType::Sum => {
                if let Some(sum) = data.as_sum() {
                    for dp in sum.data_points() {
                        let dp_id = next_id_32(&mut ids.ndp)?;
                        let _ = pt_builder.append_number_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_number_dp_exemplars(dp_id, &dp, &mut ids.ndp_exemplar)?;
                    }
                }
            }
            DataType::Histogram => {
                if let Some(hist) = data.as_histogram() {
                    for dp in hist.data_points() {
                        let dp_id = next_id_32(&mut ids.hdp)?;
                        let _ = pt_builder.append_histogram_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_histogram_dp_exemplars(
                            dp_id,
                            &dp,
                            &mut ids.hdp_exemplar,
                        )?;
                    }
                }
            }
            DataType::ExponentialHistogram => {
                if let Some(exp) = data.as_exponential_histogram() {
                    for dp in exp.data_points() {
                        let dp_id = next_id_32(&mut ids.ehdp)?;
                        let _ = pt_builder.append_exp_histogram_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_exp_histogram_dp_exemplars(
                            dp_id,
                            &dp,
                            &mut ids.ehdp_exemplar,
                        )?;
                    }
                }
            }
            DataType::Summary => {
                if let Some(summary) = data.as_summary() {
                    for dp in summary.data_points() {
                        let dp_id = next_id_32(&mut ids.sdp)?;
                        let _ = pt_builder.append_summary_dp(dp_id, otap_metric_id, &dp);
                    }
                }
            }
        }
        Ok(())
    }

    fn clear_state(&mut self) {
        self.identities.clear();
        self.builder.clear();
    }
}

/// Allocate the next u16 ID from a counter, returning an error on overflow.
fn next_id_16(counter: &mut u16) -> Result<u16, AggregationError> {
    let id = *counter;
    if id == u16::MAX {
        return Err(AggregationError::IdOverflow);
    }
    *counter += 1;
    Ok(id)
}

/// Allocate the next u32 ID from a counter, returning an error on overflow.
fn next_id_32(counter: &mut u32) -> Result<u32, AggregationError> {
    let id = *counter;
    if id == u32::MAX {
        return Err(AggregationError::IdOverflow);
    }
    *counter += 1;
    Ok(id)
}

/// Lightweight check on a [`DataView`] to determine if it represents an
/// aggregatable metric type. This avoids computing a full [`MetricId`] which
/// requires scope/resource context.
fn is_data_aggregatable<'a, D: DataView<'a>>(data: &D) -> bool {
    match data.value_type() {
        DataType::Gauge | DataType::Summary => true,
        DataType::Sum => data.as_sum().is_some_and(|s| {
            s.aggregation_temporality() == AggregationTemporality::Cumulative && s.is_monotonic()
        }),
        DataType::Histogram => data
            .as_histogram()
            .is_some_and(|h| h.aggregation_temporality() == AggregationTemporality::Cumulative),
        DataType::ExponentialHistogram => data
            .as_exponential_histogram()
            .is_some_and(|e| e.aggregation_temporality() == AggregationTemporality::Cumulative),
    }
}

/// Returns `true` if the view contains at least one aggregatable metric.
/// This is a preflight check to determine if we can skip processing entirely
/// and pass the whole batch through unchanged.
fn has_aggregatable_metrics<V: MetricsView>(view: &V) -> bool {
    for resource_metrics in view.resources() {
        for scope_metrics in resource_metrics.scopes() {
            for metric in scope_metrics.metrics() {
                if let Some(data) = metric.data() {
                    if is_data_aggregatable(&data) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;
    use std::future::Future;

    use otap_df_engine::testing::processor::TestContext;
    use otap_df_pdata::otap::OtapBatchStore;
    use otap_df_pdata::otlp::metrics::MetricType;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
    use otap_df_pdata::{metrics, record_batch};
    use serde_json::json;
    use smallvec::smallvec;

    #[test]
    fn test_default_config_parsing() {
        let config: Config = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.period, Duration::from_secs(60));
    }

    #[test]
    fn test_custom_config_parsing() {
        let config: Config = serde_json::from_value(json!({
            "period": "30s",
        }))
        .unwrap();
        assert_eq!(config.period, Duration::from_secs(30));
    }

    #[test]
    fn test_factory_creation() {
        test_config(json!({ "period": "5s" }), |result| {
            assert!(
                result.is_ok(),
                "factory should create processor successfully"
            );
        });
    }

    #[test]
    fn test_factory_invalid_config() {
        test_config(json!({ "period": 12345 }), |result| {
            assert!(result.is_err(), "invalid config should fail");
        });
    }

    #[test]
    fn test_factory_rejects_period_below_minimum() {
        test_config(json!({ "period": "0s" }), |result| {
            assert!(result.is_err(), "zero period should fail validation");
        });
        test_config(json!({ "period": "50ms" }), |result| {
            assert!(result.is_err(), "sub-100ms period should fail validation");
        });
        test_config(json!({ "period": "100ms" }), |result| {
            assert!(result.is_ok(), "100ms period should pass validation");
        });
    }

    #[test]
    fn test_passthrough_logs() {
        let logs = create_logs_payload();
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::empty(),
                payload: logs.clone(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::AssertEquivalent(logs)],
            },
        ];
        run_test(config, actions);
    }

    #[test]
    fn test_passthrough_traces() {
        let traces = create_traces_payload();
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::empty(),
                payload: traces.clone(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::AssertEquivalent(traces)],
            },
        ];
        run_test(config, actions);
    }

    #[test]
    fn test_gauge_correlation() {
        // Two batches with the same gauge stream. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("time_unix_nano", TimestampNs, [1000]),
                    ("double_value", Float64, [10.0]))
            ));
            #[rustfmt::skip]
            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("time_unix_nano", TimestampNs, [2000]),
                    ("double_value", Float64, [20.0]))
            ));
            let expected = batch2.clone();

            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .unwrap();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    #[test]
    fn test_cumulative_sum_correlation() {
        // Two batches with the same cumulative monotonic sum. The later
        // timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Sum as u8]),
                    ("name", Utf8, ["requests"]),
                    ("aggregation_temporality", Int32, [2]),
                    ("is_monotonic", Boolean, [true])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("time_unix_nano", TimestampNs, [1000]),
                    ("int_value", Int64, [100]))
            ));
            #[rustfmt::skip]
            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Sum as u8]),
                    ("name", Utf8, ["requests"]),
                    ("aggregation_temporality", Int32, [2]),
                    ("is_monotonic", Boolean, [true])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("time_unix_nano", TimestampNs, [2000]),
                    ("int_value", Int64, [200]))
            ));
            let expected = batch2.clone();

            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .unwrap();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    #[test]
    fn test_cumulative_histogram_correlation() {
        // Two batches with the same cumulative histogram. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency")
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    HistogramDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .count(10u64)
                                        .sum(100.0f64)
                                        .bucket_counts(vec![2, 3, 5])
                                        .explicit_bounds(vec![10.0, 50.0])
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency")
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    HistogramDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .count(20u64)
                                        .sum(200.0f64)
                                        .bucket_counts(vec![4, 6, 10])
                                        .explicit_bounds(vec![10.0, 50.0])
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_cumulative_exp_histogram_correlation() {
        // Two batches with the same cumulative exp histogram. The later
        // timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency.exp")
                            .data_exponential_histogram(ExponentialHistogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    ExponentialHistogramDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .count(5u64)
                                        .scale(2i32)
                                        .zero_count(1u64)
                                        .positive(Buckets::new(0, vec![1, 2]))
                                        .negative(Buckets::new(0, vec![1, 1]))
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency.exp")
                            .data_exponential_histogram(ExponentialHistogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    ExponentialHistogramDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .count(10u64)
                                        .scale(2i32)
                                        .zero_count(2u64)
                                        .positive(Buckets::new(0, vec![3, 4]))
                                        .negative(Buckets::new(0, vec![2, 1]))
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_summary_correlation() {
        // Two batches with the same summary stream. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("request.duration")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .count(10u64)
                                    .sum(500.0f64)
                                    .quantile_values(vec![
                                        ValueAtQuantile::new(0.5, 45.0),
                                        ValueAtQuantile::new(0.99, 95.0),
                                    ])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("request.duration")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(2000u64)
                                    .count(20u64)
                                    .sum(1000.0f64)
                                    .quantile_values(vec![
                                        ValueAtQuantile::new(0.5, 50.0),
                                        ValueAtQuantile::new(0.99, 99.0),
                                    ])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_different_resources_preserved() {
        // Two gauges with different resource attributes should be treated as
        // separate streams and both preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"])),
                (ResourceAttrs,
                    ("parent_id", UInt16, [0]),
                    ("key", Utf8, ["env"]),
                    ("type", UInt8, [1]),
                    ("str", Utf8, ["prod"])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("double_value", Float64, [10.0]))
            ));
            #[rustfmt::skip]
            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"])),
                (ResourceAttrs,
                    ("parent_id", UInt16, [0]),
                    ("key", Utf8, ["env"]),
                    ("type", UInt8, [1]),
                    ("str", Utf8, ["staging"])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("double_value", Float64, [20.0]))
            ));
            let expected1 = batch1.clone();
            let expected2 = batch2.clone();

            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .unwrap();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected1, expected2]);
        });
    }

    #[test]
    fn test_different_scope_attributes_preserved() {
        // Two gauges with the same resource but different scope attributes
        // should both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let input = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0, 1]),
                    ("resource.id", UInt16, [0, 0]),
                    ("scope.id", UInt16, [0, 1]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8, MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu", "cpu"])),
                (ScopeAttrs,
                    ("parent_id", UInt16, [0, 1]),
                    ("key", Utf8, ["lib", "lib"]),
                    ("type", UInt8, [1, 1]),
                    ("str", Utf8, ["opentelemetry", "prometheus"])),
                (NumberDataPoints,
                    ("id", UInt32, [0, 1]),
                    ("parent_id", UInt16, [0, 1]),
                    ("double_value", Float64, [10.0, 20.0]))
            ));
            let expected = input.clone();

            ctx.process(Message::PData(make_pdata(input)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    #[test]
    fn test_different_scope_name_preserved() {
        // Two gauges with the same resource but different scope names should
        // both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"]),
                    ("scope.name", Utf8, ["scope-a"])),
                (ResourceAttrs,
                    ("parent_id", UInt16, [0]),
                    ("key", Utf8, ["service"]),
                    ("type", UInt8, [1]),
                    ("str", Utf8, ["myapp"])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("double_value", Float64, [10.0]))
            ));
            #[rustfmt::skip]
            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"]),
                    ("scope.name", Utf8, ["scope-b"])),
                (ResourceAttrs,
                    ("parent_id", UInt16, [0]),
                    ("key", Utf8, ["service"]),
                    ("type", UInt8, [1]),
                    ("str", Utf8, ["myapp"])),
                (NumberDataPoints,
                    ("id", UInt32, [0]),
                    ("parent_id", UInt16, [0]),
                    ("double_value", Float64, [20.0]))
            ));
            let expected1 = batch1.clone();
            let expected2 = batch2.clone();

            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .unwrap();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected1, expected2]);
        });
    }

    #[test]
    fn test_different_metric_name_preserved() {
        // Two gauges with the same resource/scope but different metric names
        // should both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let input = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0, 1]),
                    ("resource.id", UInt16, [0, 0]),
                    ("scope.id", UInt16, [0, 0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8, MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu", "memory"])),
                (NumberDataPoints,
                    ("id", UInt32, [0, 1]),
                    ("parent_id", UInt16, [0, 1]),
                    ("double_value", Float64, [10.0, 20.0]))
            ));
            let expected = input.clone();

            ctx.process(Message::PData(make_pdata(input)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    #[test]
    fn test_different_metric_type_preserved() {
        // A gauge and a cumulative sum with the same name should be treated as
        // different metrics and both preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let input = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0, 1]),
                    ("resource.id", UInt16, [0, 0]),
                    ("scope.id", UInt16, [0, 0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8, MetricType::Sum as u8]),
                    ("name", Utf8, ["cpu", "cpu"]),
                    ("aggregation_temporality", Int32, [0, 2]),
                    ("is_monotonic", Boolean, [false, true])),
                (NumberDataPoints,
                    ("id", UInt32, [0, 1]),
                    ("parent_id", UInt16, [0, 1]),
                    ("double_value", Float64, [10.0, 20.0]))
            ));
            let expected = input.clone();

            ctx.process(Message::PData(make_pdata(input)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    #[test]
    fn test_different_dp_attributes_preserved() {
        // One gauge with two data points that have different DP attributes
        // should treat them as distinct streams and preserve both.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let input = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0]),
                    ("resource.id", UInt16, [0]),
                    ("scope.id", UInt16, [0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8]),
                    ("name", Utf8, ["cpu"])),
                (NumberDataPoints,
                    ("id", UInt32, [0, 1]),
                    ("parent_id", UInt16, [0, 0]),
                    ("double_value", Float64, [10.0, 20.0])),
                (NumberDpAttrs,
                    ("parent_id", UInt32, [0, 1]),
                    ("key", Utf8, ["host", "host"]),
                    ("type", UInt8, [1, 1]),
                    ("str", Utf8, ["host-a", "host-b"]))
            ));
            let expected = input.clone();

            ctx.process(Message::PData(make_pdata(input)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    #[test]
    fn test_mixed_metric_types_in_single_batch() {
        // A batch containing both a gauge and a cumulative sum should preserve
        // both in the output.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let input = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0, 1]),
                    ("resource.id", UInt16, [0, 0]),
                    ("scope.id", UInt16, [0, 0]),
                    ("metric_type", UInt8, [MetricType::Gauge as u8, MetricType::Sum as u8]),
                    ("name", Utf8, ["temperature", "requests"]),
                    ("aggregation_temporality", Int32, [0, 2]),
                    ("is_monotonic", Boolean, [false, true])),
                (NumberDataPoints,
                    ("id", UInt32, [0, 1]),
                    ("parent_id", UInt16, [0, 1]),
                    ("double_value", Float64, [22.5, 1000.0]))
            ));
            let expected = input.clone();

            ctx.process(Message::PData(make_pdata(input)))
                .await
                .unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &[expected]);
        });
    }

    /// Create a processor wrapped in TestRuntime, run a scenario, validate.
    fn run_processor_test<F, Fut>(config_json: serde_json::Value, scenario: F)
    where
        F: FnOnce(TestContext<OtapPdata>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let (rt, proc) = try_create_processor(config_json).expect("valid config");
        rt.set_processor(proc)
            .run_test(scenario)
            .validate(|_ctx| async {});
    }

    fn test_config<F>(config: serde_json::Value, assert_fn: F)
    where
        F: FnOnce(Result<ProcessorWrapper<OtapPdata>, ConfigError>),
    {
        let res = try_create_processor(config).map(|(_, proc)| proc);
        assert_fn(res);
    }

    #[test]
    fn test_metric_id_overflow_triggers_early_flush() {
        // Fill the accumulator with u16::MAX - 1 unique metrics (the maximum
        // that fits), then send a batch with 2 more unique metrics. The first
        // batch should be flushed early and the overflowing batch retried into
        // clean state.
        //
        // Set max_stream_cardinality to u16::MAX so that the stream cardinality
        // limit is never hit here and only the metric ID overflow path is exercised.
        let max_metrics = (u16::MAX - 1) as usize;
        run_processor_test(
            json!({ "max_stream_cardinality": u16::MAX }),
            move |mut ctx| async move {
                let full_batch = make_otlp_bytes_pdata(make_n_gauge_metrics(max_metrics));
                // Use offset to ensure distinct metric names from the first batch.
                let overflow_batch =
                    make_otlp_bytes_pdata(make_n_gauge_metrics_with_offset(2, max_metrics));

                ctx.process(Message::PData(full_batch)).await.unwrap();
                ctx.process(Message::PData(overflow_batch)).await.unwrap();
                let _ = ctx.fire_wakeup().await.unwrap();

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 2, "expected early flush + wakeup flush");
                assert_output_metric_count(&output[0], max_metrics);
                assert_output_metric_count(&output[1], 2);
            },
        );
    }

    #[test]
    fn test_stream_cardinality_overflow_triggers_early_flush() {
        // Configure the processor to allow at most 2 unique streams in a batch.
        // Send a batch with 2 streams (fills to the limit), then a second batch
        // with 1 new stream. The new stream should trigger a cardinality overflow:
        // the first batch is flushed early, then the overflowing data is retried
        // into a fresh batch and flushed on the next wakeup.
        run_processor_test(
            json!({ "max_stream_cardinality": 2 }),
            |mut ctx| async move {
                let batch1 = make_otlp_bytes_pdata(make_n_gauge_metrics(2));
                // Use offset to ensure distinct metric names from the first batch.
                let batch2 = make_otlp_bytes_pdata(make_n_gauge_metrics_with_offset(1, 2));

                ctx.process(Message::PData(batch1)).await.unwrap();
                ctx.process(Message::PData(batch2)).await.unwrap();
                let _ = ctx.fire_wakeup().await.unwrap();

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 2, "expected early flush + wakeup flush");
                assert_output_metric_count(&output[0], 2);
                assert_output_metric_count(&output[1], 1);
            },
        );
    }

    #[test]
    fn test_otlp_gauge_correlation() {
        // Two OTLP gauge batches with the same stream. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_gauge("cpu", 10.0)],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(2000u64)
                                    .value_double(20.0f64)
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_bytes_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_otlp_cumulative_sum_correlation() {
        // Two OTLP cumulative monotonic sum batches. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_sum("requests", true, 100, vec![])],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("requests")
                            .data_sum(Sum::new(
                                AggregationTemporality::Cumulative,
                                true,
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .value_int(200i64)
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_bytes_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_otlp_histogram_correlation() {
        // Two OTLP cumulative histogram batches. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_histogram("latency", true, vec![])],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency")
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    HistogramDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .count(20u64)
                                        .sum(200.0f64)
                                        .bucket_counts(vec![4, 6, 10])
                                        .explicit_bounds(vec![10.0, 50.0])
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_bytes_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_otlp_exp_histogram_correlation() {
        // Two OTLP cumulative exponential histogram batches. Later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_exp_histogram("latency.exp", true, vec![])],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency.exp")
                            .data_exponential_histogram(ExponentialHistogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    ExponentialHistogramDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .count(10u64)
                                        .scale(2i32)
                                        .zero_count(2u64)
                                        .positive(Buckets::new(0, vec![3, 4]))
                                        .negative(Buckets::new(0, vec![2, 1]))
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_bytes_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_otlp_summary_correlation() {
        // Two OTLP summary batches. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("request.duration")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .count(10u64)
                                    .sum(500.0f64)
                                    .quantile_values(vec![
                                        ValueAtQuantile::new(0.5, 45.0),
                                        ValueAtQuantile::new(0.99, 95.0),
                                    ])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]));
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("request.duration")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(2000u64)
                                    .count(20u64)
                                    .sum(1000.0f64)
                                    .quantile_values(vec![
                                        ValueAtQuantile::new(0.5, 50.0),
                                        ValueAtQuantile::new(0.99, 99.0),
                                    ])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_bytes_pdata(expected_data.clone());

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_otlp_different_resources_preserved() {
        // Two OTLP gauges with different resource attributes should be treated
        // as separate streams.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("env", AnyValue::new_string("prod"))])
                    .finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_gauge("cpu", 10.0)],
                )],
            )]));
            let batch2 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("env", AnyValue::new_string("staging"))])
                    .finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_gauge("cpu", 20.0)],
                )],
            )]));

            ctx.process(Message::PData(batch1)).await.unwrap();
            ctx.process(Message::PData(batch2)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            // Both streams should be present: verify by converting the output
            // to OTLP and checking we have 2 resource_metrics entries.
            let actual = match output[0].payload_ref() {
                OtapPayload::OtapArrowRecords(r) => r,
                _ => panic!("expected OtapArrowRecords payload"),
            };
            let otlp = otap_to_otlp(actual);
            let md = match otlp {
                otap_df_pdata::proto::OtlpProtoMessage::Metrics(md) => md,
                _ => panic!("expected metrics"),
            };
            assert_eq!(
                md.resource_metrics.len(),
                2,
                "expected two distinct resources in output"
            );
        });
    }

    #[test]
    fn test_otlp_different_dp_attributes_preserved() {
        // One OTLP gauge with two data points that have different DP
        // attributes should treat them as distinct streams.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(10.0f64)
                                    .attributes(vec![KeyValue::new(
                                        "host",
                                        AnyValue::new_string("host-a"),
                                    )])
                                    .finish(),
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(20.0f64)
                                    .attributes(vec![KeyValue::new(
                                        "host",
                                        AnyValue::new_string("host-b"),
                                    )])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]));

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(10.0f64)
                                    .attributes(vec![KeyValue::new(
                                        "host",
                                        AnyValue::new_string("host-a"),
                                    )])
                                    .finish(),
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(20.0f64)
                                    .attributes(vec![KeyValue::new(
                                        "host",
                                        AnyValue::new_string("host-b"),
                                    )])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]);

            ctx.process(Message::PData(batch)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_mixed_otap_otlp_gauge_correlation() {
        // One OTAP batch and one OTLP bytes batch for the same gauge stream.
        // They should be correlated and the later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            // First batch via OTAP encoding.
            let otap_batch = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_gauge("cpu", 10.0)],
                )],
            )]));
            // Second batch via OTLP bytes.
            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(2000u64)
                                    .value_double(20.0f64)
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]);
            let otlp_batch = make_otlp_bytes_pdata(expected_data.clone());

            ctx.process(Message::PData(otap_batch)).await.unwrap();
            ctx.process(Message::PData(otlp_batch)).await.unwrap();
            let _ = ctx.fire_wakeup().await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_full_passthrough_delta_sum() {
        // A batch containing only a delta sum (non-aggregatable) should be
        // passed through immediately without waiting for a wakeup.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_sum(
                        "delta.requests",
                        false,
                        50,
                        vec![
                            Exemplar::build()
                                .time_unix_nano(500u64)
                                .value_double(3.2)
                                .trace_id([1u8; 16])
                                .span_id([1u8; 8])
                                .filtered_attributes(vec![KeyValue::new(
                                    "exemplar.key",
                                    AnyValue::new_string("exemplar.val"),
                                )])
                                .finish(),
                        ],
                    )],
                )],
            )]);

            let pdata = make_otlp_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            // Should get output immediately (no wakeup needed)
            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "delta sum should pass through immediately");
            assert_output_otlp_equivalent(&output[0], input_data);
        });
    }

    #[test]
    fn test_full_passthrough_non_monotonic_cumulative_sum() {
        // A non-monotonic cumulative sum is not aggregatable and should pass
        // through immediately.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("temperature")
                            .data_sum(Sum::new(
                                AggregationTemporality::Cumulative,
                                false, // non-monotonic
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .value_double(23.5f64)
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);

            let pdata = make_otlp_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(
                output.len(),
                1,
                "non-monotonic cumulative sum should pass through"
            );
            assert_output_otlp_equivalent(&output[0], input_data);
        });
    }

    #[test]
    fn test_full_passthrough_delta_histogram() {
        // A delta histogram should pass through immediately.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_histogram(
                        "delta.latency",
                        false,
                        vec![
                            Exemplar::build()
                                .time_unix_nano(800u64)
                                .value_int(42i64)
                                .trace_id([2u8; 16])
                                .span_id([2u8; 8])
                                .finish(),
                        ],
                    )],
                )],
            )]);

            let pdata = make_otlp_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(
                output.len(),
                1,
                "delta histogram should pass through immediately"
            );
            assert_output_otlp_equivalent(&output[0], input_data);
        });
    }

    #[test]
    fn test_mixed_aggregatable_and_passthrough() {
        // A batch with both a cumulative monotonic sum (aggregatable) and a
        // delta sum (passthrough) in the same resource and scope. The delta
        // sum should be passed through immediately while the cumulative sum
        // should be buffered.
        run_processor_test(json!({}), |mut ctx| async move {
            let aggregatable = make_sum("requests.total", true, 100, vec![]);
            let passthrough = make_sum("requests.delta", false, 50, vec![]);

            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![aggregatable.clone(), passthrough.clone()],
                )],
            )]);

            let pdata = make_otlp_pdata(input_data);
            ctx.process(Message::PData(pdata)).await.unwrap();

            // The passthrough batch with only the delta sum should arrive
            // immediately.
            let output = ctx.drain_pdata().await;
            assert_eq!(
                output.len(),
                1,
                "passthrough batch should be emitted immediately"
            );

            let expected_passthrough = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![passthrough],
                )],
            )]);
            assert_output_otlp_equivalent(&output[0], expected_passthrough);

            // Now trigger a wakeup to flush the aggregated metric
            let _ = ctx.fire_wakeup().await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1, "aggregated metric should flush on wakeup");

            let expected_aggregated = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![aggregatable],
                )],
            )]);
            assert_output_otlp_equivalent(&flushed[0], expected_aggregated);
        });
    }

    #[test]
    fn test_passthrough_all_aggregated_no_immediate_output() {
        // When a batch contains only aggregatable metrics, nothing should be
        // emitted until a wakeup.
        run_processor_test(json!({}), |mut ctx| async move {
            let pdata = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_gauge("cpu.gauge", 42.0)],
                )],
            )]));

            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert!(
                output.is_empty(),
                "all-aggregatable batch should not emit anything immediately"
            );
        });
    }

    #[test]
    fn test_passthrough_delta_exp_histogram() {
        // A delta exponential histogram should pass through.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_exp_histogram(
                        "delta.exp.latency",
                        false,
                        vec![
                            Exemplar::build()
                                .time_unix_nano(900u64)
                                .value_double(1.5)
                                .trace_id([1u8; 16])
                                .span_id([2u8; 8])
                                .filtered_attributes(vec![
                                    KeyValue::new("attr.one", AnyValue::new_string("v1")),
                                    KeyValue::new("attr.two", AnyValue::new_string("v2")),
                                ])
                                .finish(),
                        ],
                    )],
                )],
            )]);

            let pdata = make_otlp_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(
                output.len(),
                1,
                "delta exp histogram should pass through immediately"
            );
            assert_output_otlp_equivalent(&output[0], input_data);
        });
    }

    #[test]
    fn test_passthrough_multiple_exemplars_per_data_point() {
        // Multiple exemplars on a single data point should all survive
        // the passthrough path, exercising the exemplar loop and ID
        // allocation in append_number_dp_exemplars.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("delta.multi_exemplar")
                            .data_sum(Sum::new(
                                AggregationTemporality::Delta,
                                true,
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .value_double(99.9f64)
                                        .exemplars(vec![
                                            Exemplar::build()
                                                .time_unix_nano(100u64)
                                                .value_double(1.1)
                                                .trace_id([1u8; 16])
                                                .span_id([2u8; 8])
                                                .filtered_attributes(vec![KeyValue::new(
                                                    "ex.key",
                                                    AnyValue::new_string("ex.val"),
                                                )])
                                                .finish(),
                                            Exemplar::build()
                                                .time_unix_nano(200u64)
                                                .value_int(7i64)
                                                .trace_id([3u8; 16])
                                                .span_id([4u8; 8])
                                                .finish(),
                                        ])
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);

            let pdata = make_otlp_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "delta sum should pass through immediately");
            assert_output_otlp_equivalent(&output[0], input_data);
        });
    }

    #[test]
    fn test_mixed_batch_all_passthrough_types_with_exemplars() {
        // A mixed batch forces the slow path through the builder so all three
        // exemplar append functions are exercised.
        run_processor_test(json!({}), |mut ctx| async move {
            let aggregatable_sum = make_sum("requests.total", true, 100, vec![]);

            let delta_sum = make_sum(
                "delta.requests",
                false,
                50,
                vec![
                    Exemplar::build()
                        .time_unix_nano(500u64)
                        .value_double(3.2)
                        .trace_id([1u8; 16])
                        .span_id([1u8; 8])
                        .filtered_attributes(vec![KeyValue::new(
                            "sum.exemplar.key",
                            AnyValue::new_string("sum.exemplar.val"),
                        )])
                        .finish(),
                ],
            );

            let delta_histogram = make_histogram(
                "delta.latency",
                false,
                vec![
                    Exemplar::build()
                        .time_unix_nano(800u64)
                        .value_int(42i64)
                        .trace_id([2u8; 16])
                        .span_id([2u8; 8])
                        .finish(),
                ],
            );

            let delta_exp_histogram = make_exp_histogram(
                "delta.exp.latency",
                false,
                vec![
                    Exemplar::build()
                        .time_unix_nano(900u64)
                        .value_double(1.5)
                        .trace_id([3u8; 16])
                        .span_id([3u8; 8])
                        .filtered_attributes(vec![
                            KeyValue::new("attr.one", AnyValue::new_string("v1")),
                            KeyValue::new("attr.two", AnyValue::new_string("v2")),
                        ])
                        .finish(),
                ],
            );

            let input = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        aggregatable_sum.clone(),
                        delta_sum.clone(),
                        delta_histogram.clone(),
                        delta_exp_histogram.clone(),
                    ],
                )],
            )]));
            ctx.process(Message::PData(input)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(
                output.len(),
                1,
                "passthrough batch should be emitted immediately"
            );

            let expected_passthrough = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![delta_sum, delta_histogram, delta_exp_histogram],
                )],
            )]);
            assert_output_otlp_equivalent(&output[0], expected_passthrough);

            let _ = ctx.fire_wakeup().await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1, "aggregated metric should flush on timer");

            let expected_aggregated = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![aggregatable_sum],
                )],
            )]);
            assert_output_otlp_equivalent(&flushed[0], expected_aggregated);
        });
    }

    #[test]
    fn test_mixed_resources_passthrough_and_aggregate() {
        // Two different resources: one with only aggregatable metrics, the
        // other with only passthrough metrics.
        run_processor_test(json!({}), |mut ctx| async move {
            let passthrough = make_sum("requests.delta", false, 10, vec![]);

            let input_data = MetricsData::new(vec![
                // Resource A: gauge (aggregatable)
                ResourceMetrics::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new("env", AnyValue::new_string("prod"))])
                        .finish(),
                    vec![ScopeMetrics::new(
                        InstrumentationScope::build().finish(),
                        vec![make_gauge("cpu", 75.0)],
                    )],
                ),
                // Resource B: delta sum (passthrough)
                ResourceMetrics::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new("env", AnyValue::new_string("staging"))])
                        .finish(),
                    vec![ScopeMetrics::new(
                        InstrumentationScope::build().finish(),
                        vec![passthrough.clone()],
                    )],
                ),
            ]);

            let pdata = make_otlp_pdata(input_data);
            ctx.process(Message::PData(pdata)).await.unwrap();

            // Passthrough for resource B should be emitted
            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "passthrough batch should be emitted");

            let expected_passthrough = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("env", AnyValue::new_string("staging"))])
                    .finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![passthrough],
                )],
            )]);
            assert_output_otlp_equivalent(&output[0], expected_passthrough);

            // Flush the aggregated gauge
            let _ = ctx.fire_wakeup().await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1, "aggregated gauge should flush on timer");
        });
    }

    #[test]
    fn test_none_resource_aggregation() {
        // A batch with resource=None should still be processed (not skipped).
        // The gauge data point should be aggregated and flushed on timer.
        // We use OTLP bytes encoding because the OTAP encoder normalizes
        // None resources, but the raw bytes view preserves the None.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![resource_metrics_without_resource(vec![
                ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_gauge("cpu", 42.0)],
                ),
            ])]);

            let pdata = make_otlp_bytes_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            // No immediate output (gauge is aggregated)
            let output = ctx.drain_pdata().await;
            assert!(
                output.is_empty(),
                "gauge should be buffered, not emitted immediately"
            );

            // Flush
            let _ = ctx.fire_wakeup().await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1, "buffered gauge should flush on timer");
            assert_output_otlp_equivalent(&flushed[0], input_data);
        });
    }

    #[test]
    fn test_none_scope_aggregation() {
        // A batch with scope=None should still be processed.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![scope_metrics_without_scope(vec![make_gauge("mem", 1024.0)])],
            )]);

            let pdata = make_otlp_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert!(output.is_empty(), "gauge should be buffered");

            let _ = ctx.fire_wakeup().await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1);
            assert_output_otlp_equivalent(&flushed[0], input_data);
        });
    }

    #[test]
    fn test_none_resource_passthrough() {
        // A delta sum under a None resource should pass through immediately.
        // This uses OTLP bytes encoding and the full passthrough path forwards
        // the original pdata unchanged (still as OTLP bytes, not OTAP records).
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![resource_metrics_without_resource(vec![
                ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![make_sum("delta.count", false, 5, vec![])],
                ),
            ])]);

            let pdata = make_otlp_bytes_pdata(input_data);
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "delta sum should pass through immediately");
            // Full passthrough forwards original OTLP bytes unchanged -
            // just verify something was emitted, the payload format is preserved.
            assert!(
                matches!(output[0].payload_ref(), OtapPayload::OtlpBytes(_)),
                "full passthrough should preserve OTLP bytes payload"
            );
        });
    }

    #[test]
    fn test_none_resource_and_scope() {
        // Both resource=None and scope=None should still work.
        run_processor_test(json!({}), |mut ctx| async move {
            let input_data = MetricsData::new(vec![resource_metrics_without_resource(vec![
                scope_metrics_without_scope(vec![make_gauge("temp", 98.6)]),
            ])]);

            let pdata = make_otlp_bytes_pdata(input_data.clone());
            ctx.process(Message::PData(pdata)).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert!(output.is_empty(), "gauge should be buffered");

            let _ = ctx.fire_wakeup().await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1);
            assert_output_otlp_equivalent(&flushed[0], input_data);
        });
    }

    #[test]
    fn test_nack_passthrough_then_flush() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![
                    PdataAction::AssertSubscribers(true),
                    PdataAction::Nack("test rejection"),
                ],
            },
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("test rejection")],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
        ];
        run_test(config, actions);
    }

    // Ack passthrough, then ack the aggregated flush. Upstream should get
    // exactly one ack (held until both outbounds are acked).
    #[test]
    fn test_ack_passthrough_then_ack_aggregated() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::AssertSubscribers(true), PdataAction::Ack],
            },
            // Passthrough acked but not flushed yet -> no upstream ack
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            // Both acked and flushed -> upstream ack
            Action::AssertUpstream(UpstreamExpectation::AckCount(1)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    // Ack passthrough, then nack the aggregated flush. Upstream should get
    // one nack (all-or-nothing semantics).
    #[test]
    fn test_ack_passthrough_then_nack_aggregated() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("downstream failure")],
            },
            // Nack wins -> upstream nack, no ack
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
        ];
        run_test(config, actions);
    }

    // Nack both passthrough and aggregated independently. Upstream should
    // receive exactly one nack (second nack for same inbound is a no-op).
    #[test]
    fn test_nack_both_passthrough_and_aggregated() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("first nack")],
            },
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("second nack")],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
        ];
        run_test(config, actions);
    }

    // AllAggregated path: all metrics are aggregatable, nothing passthrough.
    // Ack the flushed batch -> upstream ack.
    #[test]
    fn test_all_aggregated_ack_after_flush() {
        let config = json!({});
        let payload = make_otap_payload_from_metrics(MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().finish(),
                vec![make_gauge("cpu.gauge", 42.0)],
            )],
        )]));
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload,
            },
            Action::AssertNoPdata,
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(1)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    // AllAggregated path: nack the flushed batch -> upstream nack.
    #[test]
    fn test_all_aggregated_nack_after_flush() {
        let config = json!({});
        let payload = make_otap_payload_from_metrics(MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().finish(),
                vec![make_gauge("cpu.gauge", 42.0)],
            )],
        )]));
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload,
            },
            Action::AssertNoPdata,
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("aggregated nack")],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
        ];
        run_test(config, actions);
    }

    // No subscriber interest on the inbound pdata. Processor should not
    // track ack/nack state and no upstream ack/nack should appear.
    #[test]
    fn test_no_subscribers_no_tracking() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::empty(),
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::AssertSubscribers(false)],
            },
            Action::FireWakeup,
            // Flushed aggregated batch also has no subscribers
            Action::DrainPdata {
                actions: vec![PdataAction::AssertSubscribers(false)],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    // Nack passthrough, then ack the aggregated flush. The inbound was
    // already removed by the nack so handle_ack hits the stale_inbound
    // warning path. Upstream should get 1 nack (from the passthrough), 0 acks.
    #[test]
    fn test_nack_passthrough_then_ack_aggregated() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("test rejection")],
            },
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
        ];
        run_test(config, actions);
    }

    // Two inbounds with mixed metrics accumulate in the same flush window.
    // Both passthroughs acked, then the single aggregated flush acked.
    // Both inbounds should get upstream acks.
    #[test]
    fn test_multiple_inbounds_flush_acked() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(2)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    // Two inbounds with mixed metrics, both passthroughs acked, then the
    // aggregated flush nacked. Both inbounds should get upstream nacks.
    #[test]
    fn test_multiple_inbounds_flush_nacked() {
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_mixed_metrics_payload(),
            },
            Action::DrainPdata {
                actions: vec![PdataAction::Ack],
            },
            Action::FireWakeup,
            Action::DrainPdata {
                actions: vec![PdataAction::Nack("downstream failure")],
            },
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(2)),
        ];
        run_test(config, actions);
    }

    #[test]
    fn test_single_batch_over_limit_nacked() {
        let too_many = u16::MAX as usize + 1;
        let config = json!({});
        let actions = vec![
            Action::SendPdata {
                interests: Interests::ACKS | Interests::NACKS,
                payload: make_otlp_payload_from_metrics(make_n_gauge_metrics(too_many)),
            },
            Action::AssertNoPdata,
            Action::FireWakeup,
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(1)),
        ];
        run_test(config, actions);
    }

    /// Sending an ack whose calldata has the wrong number of elements
    /// (0 or 2 instead of exactly 1) should be silently ignored.
    #[test]
    fn test_ack_with_invalid_calldata_format() {
        let config = json!({});
        let actions = vec![
            Action::SendControl(NodeControlMsg::Ack(ack_with_calldata(CallData::default()))),
            Action::SendControl(NodeControlMsg::Ack(ack_with_calldata(smallvec![
                0u64.into(),
                0u64.into()
            ]))),
            // No upstream acks or nacks should have been generated
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    /// Sending a nack whose calldata has the wrong number of elements
    /// should be silently ignored.
    #[test]
    fn test_nack_with_invalid_calldata_format() {
        let config = json!({});
        let actions = vec![
            Action::SendControl(NodeControlMsg::Nack(nack_with_calldata(
                CallData::default(),
                "bad calldata",
            ))),
            Action::SendControl(NodeControlMsg::Nack(nack_with_calldata(
                smallvec![0u64.into(), 0u64.into()],
                "bad calldata",
            ))),
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    /// Sending an ack with a structurally valid calldata (length 1) but
    /// a key that does not correspond to any outbound slot should be
    /// silently ignored.
    #[test]
    fn test_ack_with_unrecognized_calldata() {
        let config = json!({});
        // A single-element calldata passes TryFrom<CallData> for Key,
        // but the fabricated value won't match any slot in outbound_batches.
        let fabricated: CallData = smallvec![1000000000u64.into()];
        let actions = vec![
            Action::SendControl(NodeControlMsg::Ack(ack_with_calldata(fabricated))),
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    #[test]
    fn test_nack_with_unrecognized_calldata() {
        let config = json!({});
        let fabricated: CallData = smallvec![1000000000u64.into()];
        let actions = vec![
            Action::SendControl(NodeControlMsg::Nack(nack_with_calldata(
                fabricated,
                "stale nack",
            ))),
            Action::AssertUpstream(UpstreamExpectation::AckCount(0)),
            Action::AssertUpstream(UpstreamExpectation::NackCount(0)),
        ];
        run_test(config, actions);
    }

    /// Deliver a wakeup with the correct slot but a mismatched revision. The
    /// processor compares `self.wakeup_revision` against the delivered revision
    /// and ignores stale/unknown revisions.
    #[test]
    fn test_wakeup_mismatched_revision_ignored() {
        run_test(
            json!({}),
            vec![
                Action::SendPdata {
                    interests: Interests::empty(),
                    payload: make_otap_payload_from_metrics(make_n_gauge_metrics(1)),
                },
                // Wrong revision — should be silently ignored.
                Action::SendControl(NodeControlMsg::Wakeup {
                    slot: FLUSH_WAKEUP_SLOT,
                    when: Instant::now(),
                    revision: 999,
                }),
                Action::AssertNoPdata,
                // The real wakeup still flushes.
                Action::FireWakeup,
                Action::DrainPdata { actions: vec![] },
            ],
        );
    }

    /// After a metric-id overflow triggers an early flush, the old wakeup
    /// revision is cancelled. Delivering a wakeup with the stale revision
    /// should be a no-op. The wakeup for the new batch (scheduled during
    /// the retry of the overflowing data) should still flush successfully.
    #[test]
    fn test_stale_wakeup_after_overflow_ignored() {
        let max_metrics = (u16::MAX - 1) as usize;
        run_test(
            json!({ "max_stream_cardinality": u16::MAX }),
            vec![
                Action::SendPdata {
                    interests: Interests::empty(),
                    payload: make_otlp_payload_from_metrics(make_n_gauge_metrics(max_metrics)),
                },
                // This triggers an overflow flush internally.
                Action::SendPdata {
                    interests: Interests::empty(),
                    payload: make_otlp_payload_from_metrics(make_n_gauge_metrics_with_offset(
                        2,
                        max_metrics,
                    )),
                },
                // Drain the early-flush output.
                Action::DrainPdata {
                    actions: vec![PdataAction::AssertCustom(Box::new(move |output| {
                        assert_output_metric_count(output, max_metrics);
                    }))],
                },
                // Deliver a stale wakeup with revision 0 - should be ignored
                // because the overflow flush cancelled it.
                Action::SendControl(NodeControlMsg::Wakeup {
                    slot: FLUSH_WAKEUP_SLOT,
                    when: Instant::now(),
                    revision: 0,
                }),
                Action::AssertNoPdata,
                // The real wakeup (scheduled when the overflow data was retried)
                // flushes the remaining data.
                Action::FireWakeup,
                Action::DrainPdata {
                    actions: vec![PdataAction::AssertCustom(Box::new(|output| {
                        assert_output_metric_count(output, 2);
                    }))],
                },
            ],
        );
    }

    /// When the processor is idle (no data), `fire_wakeup` should return
    /// `Ok(false)` because no wakeup was ever scheduled.
    #[test]
    fn test_no_wakeup_scheduled_when_idle() {
        run_processor_test(json!({}), |mut ctx| async move {
            let fired = ctx.fire_wakeup().await.unwrap();
            assert!(
                !fired,
                "fire_wakeup should return false when no data has been sent"
            );
            assert!(ctx.drain_pdata().await.is_empty());
        });
    }
}
