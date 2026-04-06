// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Temporal reaggregation processor for OTAP metrics.
//!
//! This processor decreases telemetry volume by reaggregating metrics collected
//! at a higher frequency into a lower one.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hashbrown::HashMap;
use hashbrown::hash_map::EntryRef::{Occupied, Vacant};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
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

mod builder;
mod config;
mod identity;
mod metrics;

use self::builder::{Checkpoint, MetricSignalBuilder, StreamMeta};
use self::config::Config;
use self::identity::{
    HashBuffer, MetricId, MetricIdRef, ResourceId, ScopeId, ScopeIdRef, StreamId, StreamIdRef,
    metric_id_of, resource_id_of, scope_id_of, stream_id_of,
};
use self::metrics::TemporalReaggregationMetrics;

/// Errors that can occur during view processing.
#[derive(thiserror::Error, Debug)]
enum ProcessPdataError {
    /// An ID counter would exceed its maximum value.
    #[error("Id overflow")]
    IdOverflow,

    /// Failed to create the view for the incoming pdata
    #[error("Failed to create view: {inner:?}")]
    ViewCreationFailed {
        #[source]
        inner: Option<otap_df_pdata::error::Error>,
    },
}

/// Result of processing a single metrics pdata payload.
#[allow(clippy::large_enum_variant)]
enum ProcessResult {
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

/// State for the current in-progress batch. This is all the stuff that has to
/// be cleared between batches
struct BatchState {
    resources: HashMap<ResourceId, u16>,
    scopes: HashMap<ScopeId<'static>, u16>,
    metrics: HashMap<MetricId<'static>, u16>,
    streams: HashMap<StreamId<'static>, StreamMeta>,

    next_id_resource: u16,
    next_id_scope: u16,
    next_id_metric: u16,
    next_id_ndp: u32,
    next_id_hdp: u32,
    next_id_ehdp: u32,
    next_id_sdp: u32,
    next_id_ndp_exemplar: u32,
    next_id_hdp_exemplar: u32,
    next_id_ehdp_exemplar: u32,
}

impl BatchState {
    fn new() -> Self {
        Self {
            resources: HashMap::new(),
            scopes: HashMap::new(),
            metrics: HashMap::new(),
            streams: HashMap::new(),

            next_id_resource: 0,
            next_id_scope: 0,
            next_id_metric: 0,
            next_id_ndp: 0,
            next_id_hdp: 0,
            next_id_ehdp: 0,
            next_id_sdp: 0,
            next_id_ndp_exemplar: 0,
            next_id_hdp_exemplar: 0,
            next_id_ehdp_exemplar: 0,
        }
    }

    fn clear(&mut self) {
        self.resources.clear();
        self.scopes.clear();
        self.metrics.clear();
        self.streams.clear();
        self.next_id_resource = 0;
        self.next_id_scope = 0;
        self.next_id_metric = 0;
        self.next_id_ndp = 0;
        self.next_id_hdp = 0;
        self.next_id_ehdp = 0;
        self.next_id_sdp = 0;
        self.next_id_ndp_exemplar = 0;
        self.next_id_hdp_exemplar = 0;
        self.next_id_ehdp_exemplar = 0;
    }
}

/// The temporal reaggregation processor.
///
/// Accumulates metrics data over a collection interval, deduplicates
/// resources/scopes/metrics by identity, and tracks the latest data point per
/// stream. On each timer tick it flushes the accumulated state as an
/// [`OtapArrowRecords`] batch.
pub struct TemporalReaggregationProcessor {
    metrics: MetricSet<TemporalReaggregationMetrics>,
    collection_period: Duration,
    /// Whether the periodic flush timer has been started.
    timer_started: bool,
    // Reusable byte buffer for computing attribute hashes.
    hash_buf: HashBuffer,
    state: BatchState,
    builder: MetricSignalBuilder,
    /// Reusable state for the passthrough batch. Cleared before each use in
    /// [`process_view`] to track resources/scopes/metrics that appear in the
    /// passthrough output.
    passthrough_state: BatchState,
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for TemporalReaggregationProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::PData(pdata) => {
                self.ensure_timer_started(effect_handler).await?;

                match pdata.signal_type() {
                    SignalType::Metrics => {
                        let checkpoint = self.builder.checkpoint();

                        match self.process_metric_pdata(&pdata) {
                            Ok(result) => {
                                self.send_process_result(result, pdata, effect_handler)
                                    .await?;
                                return Ok(());
                            }
                            Err(e) => match e {
                                // Id overflows are fine. In this case we seal
                                // the current outbound batch and then feed the
                                // data back into the next one. We do this to
                                // prevent complex ack/nack scenarios where an
                                // input batch has representation in multiple
                                // output batches.
                                ProcessPdataError::IdOverflow => {
                                    self.metrics.flushes_overflow.inc();
                                    self.flush(effect_handler, Some(checkpoint)).await?;
                                    match self.process_metric_pdata(&pdata) {
                                        Ok(result) => {
                                            self.send_process_result(result, pdata, effect_handler)
                                                .await?;
                                            return Ok(());
                                        }
                                        Err(_) => {
                                            // TODO: Nack this data
                                            self.metrics.batches_rejected.inc();
                                            self.clear_state();
                                        }
                                    }
                                }

                                // If we can't even create the view then we just
                                // have to throw the batch away, we'll never be
                                // able to operate on it.
                                ProcessPdataError::ViewCreationFailed { .. } => {
                                    self.metrics.batches_rejected.inc();
                                    // TODO: Nack this data
                                }
                            },
                        }
                    }
                    // Non-metrics signals pass through unchanged.
                    SignalType::Logs | SignalType::Traces => {
                        effect_handler.send_message_with_source_node(pdata).await?;
                    }
                }
                Ok(())
            }
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::TimerTick {} => {
                    self.flush(effect_handler, None).await?;
                    self.metrics.flushes_timer.add(1);
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. } => {
                    self.flush(effect_handler, None).await?;
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    _ = metrics_reporter.report(&mut self.metrics);
                    Ok(())
                }
                _ => Ok(()),
            },
        }
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
            timer_started: false,
            hash_buf: HashBuffer::new(),
            state: BatchState::new(),
            builder: MetricSignalBuilder::new(),
            passthrough_state: BatchState::new(),
        })
    }

    /// Starts the periodic flush timer if it has not already been started.
    async fn ensure_timer_started(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if !self.timer_started {
            let _handle = effect_handler
                .start_periodic_timer(self.collection_period)
                .await?;
            self.timer_started = true;
        }
        Ok(())
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
    fn process_metric_pdata(
        &mut self,
        pdata: &OtapPdata,
    ) -> Result<ProcessResult, ProcessPdataError> {
        match pdata.payload_ref() {
            OtapPayload::OtapArrowRecords(records) => {
                let view = OtapMetricsView::try_from(records)
                    .map_err(|e| ProcessPdataError::ViewCreationFailed { inner: Some(e) })?;

                if !has_aggregatable_metrics(&view) {
                    return Ok(ProcessResult::NoAggregations);
                }

                match self.process_view(&view)? {
                    Some(records) => Ok(ProcessResult::SomeAggregations(records)),
                    None => Ok(ProcessResult::AllAggregated),
                }
            }
            OtapPayload::OtlpBytes(otlp) => {
                let data = RawMetricsData::new(otlp.as_bytes());

                if !has_aggregatable_metrics(&data) {
                    return Ok(ProcessResult::NoAggregations);
                }

                match self.process_view(&data)? {
                    Some(records) => Ok(ProcessResult::SomeAggregations(records)),
                    None => Ok(ProcessResult::AllAggregated),
                }
            }
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
        if let Ok(records) = self.builder.finish(checkpoint) {
            if !records.is_empty() {
                // TODO: Ack/nack and context propagation
                let pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(records));
                effect_handler.send_message_with_source_node(pdata).await?;
            }
        }
        self.clear_state();
        Ok(())
    }

    /// Send the result of [`process_metrics_pdata`] downstream.
    ///
    /// For [`ProcessResult::FullPassthrough`], the original pdata is forwarded.
    /// For [`ProcessResult::Passthrough`], a newly built batch containing only
    /// the non-aggregatable metrics is sent. For [`ProcessResult::AllAggregated`]
    /// nothing is sent (data is buffered for the next flush).
    async fn send_process_result(
        &mut self,
        result: ProcessResult,
        original_pdata: OtapPdata,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match result {
            ProcessResult::NoAggregations => {
                effect_handler
                    .send_message_with_source_node(original_pdata)
                    .await?;
            }
            ProcessResult::SomeAggregations(records) => {
                let pt_pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(records));
                effect_handler
                    .send_message_with_source_node(pt_pdata)
                    .await?;
            }
            ProcessResult::AllAggregated => {}
        }
        Ok(())
    }

    /// Process a view, routing aggregatable metrics to the in-progress batch
    /// and non-aggregatable metrics to a passthrough batch that is returned for
    /// immediate downstream delivery.
    ///
    /// May fail due to id overflow or other capacity related restrictions.
    fn process_view<V: MetricsView>(
        &mut self,
        view: &V,
    ) -> Result<Option<OtapArrowRecords>, ProcessPdataError> {
        self.passthrough_state.clear();
        let mut pt_builder = MetricSignalBuilder::new();

        for resource_metrics in view.resources() {
            let resource = resource_metrics.resource();
            let resource_schema_url = resource_metrics.schema_url().unwrap_or(b"");

            // Lazily computed on first aggregatable metric under this resource.
            // ResourceId is Copy so this tuple is cheap to cache.
            let mut agg_resource: Option<(u16, ResourceId)> = None;
            // Passthrough: just the assigned u16, no identity needed.
            let mut pt_resource_id: Option<u16> = None;

            for scope_metrics in resource_metrics.scopes() {
                let scope = scope_metrics.scope();
                let scope_schema_url = scope_metrics.schema_url();

                // Lazily computed. ScopeId borrows from `scope` which lives
                // for this loop body.
                let mut agg_scope: Option<(u16, ScopeId<'_>)> = None;
                let mut pt_scope_id: Option<u16> = None;

                for metric in scope_metrics.metrics() {
                    let Some(data) = metric.data() else {
                        continue;
                    };

                    if is_data_aggregatable(&data) {
                        // Lazily process resource
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
                                let id = next_id_16(&mut self.passthrough_state.next_id_resource)?;
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
                                let id = next_id_16(&mut self.passthrough_state.next_id_scope)?;
                                if let Some(ref scope) = scope {
                                    pt_builder.append_scope(id, scope);
                                }
                                pt_scope_id = Some(id);
                                id
                            }
                        };

                        let otap_metric_id =
                            next_id_16(&mut self.passthrough_state.next_id_metric)?;
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

                        self.append_passthrough_datapoints(&mut pt_builder, &data, otap_metric_id)?;
                    }
                }
            }
        }

        if self.passthrough_state.next_id_metric > 0 {
            pt_builder
                .finish(None)
                .map(Some)
                .map_err(|_| ProcessPdataError::IdOverflow)
        } else {
            Ok(None)
        }
    }

    fn process_resource<R: ResourceView>(
        &mut self,
        resource_id: ResourceId,
        view: Option<&R>,
    ) -> Result<u16, ProcessPdataError> {
        match self.state.resources.entry_ref(&resource_id) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.state.next_id_resource;
                if id == u16::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_resource += 1;
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
    ) -> Result<u16, ProcessPdataError> {
        let lookup = ScopeIdRef(&scope_id.clone());
        match self.state.scopes.entry_ref(&lookup) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.state.next_id_scope;
                if id == u16::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_scope += 1;
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
    ) -> Result<u16, ProcessPdataError> {
        let lookup = MetricIdRef(&metric_id.clone());
        match self.state.metrics.entry_ref(&lookup) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.state.next_id_metric;
                if id == u16::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_metric += 1;
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
    ) -> Result<(), ProcessPdataError> {
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
    ) -> Result<(), ProcessPdataError> {
        let time = dp.time_unix_nano();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_number_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                let dp_id = self.state.next_id_ndp;
                if dp_id == u32::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_ndp += 1;
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
    ) -> Result<(), ProcessPdataError> {
        let time = dp.time_unix_nano();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_histogram_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                let dp_id = self.state.next_id_hdp;
                if dp_id == u32::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_hdp += 1;
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
    ) -> Result<(), ProcessPdataError> {
        let time = dp.time_unix_nano();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_exp_histogram_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                let dp_id = self.state.next_id_ehdp;
                if dp_id == u32::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_ehdp += 1;
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
    ) -> Result<(), ProcessPdataError> {
        let time = dp.time_unix_nano();
        let lookup = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup) {
            Occupied(mut o) => {
                let s = o.get_mut();
                if time > s.time_unix_nano {
                    let row_index = s.dp_row_index;
                    s.time_unix_nano = time;
                    self.builder.replace_summary_dp(row_index, dp);
                }
            }
            Vacant(v) => {
                let dp_id = self.state.next_id_sdp;
                if dp_id == u32::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_sdp += 1;
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
        data: &D,
        otap_metric_id: u16,
    ) -> Result<(), ProcessPdataError> {
        match data.value_type() {
            DataType::Gauge => {
                if let Some(gauge) = data.as_gauge() {
                    for dp in gauge.data_points() {
                        let dp_id = next_id_32(&mut self.passthrough_state.next_id_ndp)?;
                        let _ = pt_builder.append_number_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_number_dp_exemplars(
                            dp_id,
                            &dp,
                            &mut self.passthrough_state.next_id_ndp_exemplar,
                        )?;
                    }
                }
            }
            DataType::Sum => {
                if let Some(sum) = data.as_sum() {
                    for dp in sum.data_points() {
                        let dp_id = next_id_32(&mut self.passthrough_state.next_id_ndp)?;
                        let _ = pt_builder.append_number_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_number_dp_exemplars(
                            dp_id,
                            &dp,
                            &mut self.passthrough_state.next_id_ndp_exemplar,
                        )?;
                    }
                }
            }
            DataType::Histogram => {
                if let Some(hist) = data.as_histogram() {
                    for dp in hist.data_points() {
                        let dp_id = next_id_32(&mut self.passthrough_state.next_id_hdp)?;
                        let _ = pt_builder.append_histogram_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_histogram_dp_exemplars(
                            dp_id,
                            &dp,
                            &mut self.passthrough_state.next_id_hdp_exemplar,
                        )?;
                    }
                }
            }
            DataType::ExponentialHistogram => {
                if let Some(exp) = data.as_exponential_histogram() {
                    for dp in exp.data_points() {
                        let dp_id = next_id_32(&mut self.passthrough_state.next_id_ehdp)?;
                        let _ = pt_builder.append_exp_histogram_dp(dp_id, otap_metric_id, &dp);
                        pt_builder.append_exp_histogram_dp_exemplars(
                            dp_id,
                            &dp,
                            &mut self.passthrough_state.next_id_ehdp_exemplar,
                        )?;
                    }
                }
            }
            DataType::Summary => {
                if let Some(summary) = data.as_summary() {
                    for dp in summary.data_points() {
                        let dp_id = next_id_32(&mut self.passthrough_state.next_id_sdp)?;
                        let _ = pt_builder.append_summary_dp(dp_id, otap_metric_id, &dp);
                    }
                }
            }
        }
        Ok(())
    }

    fn clear_state(&mut self) {
        self.state.clear();
        self.builder.clear();
    }
}

/// Allocate the next u16 ID from a counter, returning an error on overflow.
fn next_id_16(counter: &mut u16) -> Result<u16, ProcessPdataError> {
    let id = *counter;
    if id == u16::MAX {
        return Err(ProcessPdataError::IdOverflow);
    }
    *counter += 1;
    Ok(id)
}

/// Allocate the next u32 ID from a counter, returning an error on overflow.
fn next_id_32(counter: &mut u32) -> Result<u32, ProcessPdataError> {
    let id = *counter;
    if id == u32::MAX {
        return Err(ProcessPdataError::IdOverflow);
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
mod tests {
    use super::*;
    use std::future::Future;

    use otap_df_config::error::Error as ConfigError;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::{TestContext, TestRuntime};
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_pdata::otap::{OtapArrowRecords, OtapBatchStore};
    use otap_df_pdata::otlp::metrics::MetricType;
    use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint,
        Gauge, Histogram, HistogramDataPoint, Metric, MetricsData, NumberDataPoint,
        ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_message_to_bytes, otlp_to_otap};
    use otap_df_pdata::{metrics, record_batch};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;

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
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let pdata = create_test_pdata();

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process message");

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "expected exactly one forwarded message");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_passthrough_traces() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let pdata = create_traces_pdata();

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process message");

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "expected exactly one forwarded message");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_timer_tick_with_no_data() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("timer tick should succeed");

                let output = ctx.drain_pdata().await;
                assert!(
                    output.is_empty(),
                    "timer tick with no data should emit nothing"
                );
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
        run_processor_test(json!({}), |mut ctx| async move {
            let max_metrics = (u16::MAX - 1) as usize;

            // First batch: fill to the limit.
            let full_batch = make_otlp_bytes_pdata(make_n_gauge_metrics(max_metrics));
            ctx.process(Message::PData(full_batch)).await.unwrap();

            // Second batch: 2 new unique metrics — triggers overflow.
            let overflow_batch =
                make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                    Resource::build().finish(),
                    vec![ScopeMetrics::new(
                        InstrumentationScope::build().finish(),
                        vec![
                            Metric::build()
                                .name("overflow_a")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .value_double(1.0f64)
                                        .finish(),
                                ]))
                                .finish(),
                            Metric::build()
                                .name("overflow_b")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .value_double(2.0f64)
                                        .finish(),
                                ]))
                                .finish(),
                        ],
                    )],
                )]));
            ctx.process(Message::PData(overflow_batch)).await.unwrap();

            // Flush via timer tick.
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

            let output = ctx.drain_pdata().await;
            // Should have 2 outputs: the early flush (max_metrics gauges) and
            // the timer flush (the 2 overflow metrics retried into clean state).
            assert_eq!(output.len(), 2, "expected early flush + timer flush");

            // First output: the early flush should contain max_metrics data points.
            let first = match output[0].payload_ref() {
                OtapPayload::OtapArrowRecords(r) => r,
                _ => panic!("expected OtapArrowRecords"),
            };
            let first_otlp = otap_to_otlp(first);
            let first_md = match first_otlp {
                otap_df_pdata::proto::OtlpProtoMessage::Metrics(md) => md,
                _ => panic!("expected metrics"),
            };
            let first_metric_count: usize = first_md
                .resource_metrics
                .iter()
                .flat_map(|rm| &rm.scope_metrics)
                .map(|sm| sm.metrics.len())
                .sum();
            assert_eq!(first_metric_count, max_metrics);

            // Second output: the retried batch should contain the 2 overflow metrics.
            let second = match output[1].payload_ref() {
                OtapPayload::OtapArrowRecords(r) => r,
                _ => panic!("expected OtapArrowRecords"),
            };
            let second_otlp = otap_to_otlp(second);
            let second_md = match second_otlp {
                otap_df_pdata::proto::OtlpProtoMessage::Metrics(md) => md,
                _ => panic!("expected metrics"),
            };
            let second_metric_count: usize = second_md
                .resource_metrics
                .iter()
                .flat_map(|rm| &rm.scope_metrics)
                .map(|sm| sm.metrics.len())
                .sum();
            assert_eq!(second_metric_count, 2);
        });
    }

    #[test]
    fn test_single_batch_over_limit() {
        // A single batch with more than u16::MAX unique metrics overflows even
        // It should be dropped and batches_rejected should be bumped.
        run_processor_test(json!({}), |mut ctx| async move {
            let too_many = u16::MAX as usize + 1;

            let data = make_n_gauge_metrics(too_many);
            let poison_batch = make_otlp_bytes_pdata(data);
            ctx.process(Message::PData(poison_batch)).await.unwrap();

            // Flush via timer tick.
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

            // No output should be produced — the batch was poison and dropped.
            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 0, "poison batch should produce no output");
        });
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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_full_passthrough_delta_sum() {
        // A batch containing only a delta sum (non-aggregatable) should be
        // passed through immediately without waiting for a timer tick.
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

            // Should get output immediately (no timer tick needed)
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

            // Now trigger a timer tick to flush the aggregated metric
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1, "aggregated metric should flush on timer");

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
        // emitted until a timer tick.
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

            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();
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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1, "aggregated gauge should flush on timer");
        });
    }

    /// Helper to build a `ResourceMetrics` with `resource: None`.
    fn resource_metrics_without_resource(scope_metrics: Vec<ScopeMetrics>) -> ResourceMetrics {
        ResourceMetrics {
            resource: None,
            scope_metrics,
            schema_url: String::new(),
        }
    }

    /// Helper to build a `ScopeMetrics` with `scope: None`.
    fn scope_metrics_without_scope(metrics: Vec<Metric>) -> ScopeMetrics {
        ScopeMetrics {
            scope: None,
            metrics,
            schema_url: String::new(),
        }
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
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();
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

            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();
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

            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();
            let flushed = ctx.drain_pdata().await;
            assert_eq!(flushed.len(), 1);
            assert_output_otlp_equivalent(&flushed[0], input_data);
        });
    }

    fn try_create_processor(
        config: serde_json::Value,
    ) -> Result<(TestRuntime<OtapPdata>, ProcessorWrapper<OtapPdata>), ConfigError> {
        let pipeline_ctx = create_test_pipeline_context();
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let node = test_node("temporal-reaggregation-config-test");

        let mut node_config =
            NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
        node_config.config = config;

        (TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY.create)(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        )
        .map(|proc| (rt, proc))
    }

    /// Wrap [`OtapArrowRecords`] in an [`OtapPdata`].
    fn make_pdata(records: OtapArrowRecords) -> OtapPdata {
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(records))
    }

    /// Convert OTLP [`MetricsData`] into an [`OtapPdata`] via OTAP encoding.
    fn make_otlp_pdata(metrics_data: MetricsData) -> OtapPdata {
        let otap_records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
            metrics_data,
        ));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    fn create_traces_pdata() -> OtapPdata {
        let mut datagen = otap_df_pdata::testing::fixtures::DataGenerator::new(3);
        let traces_data = datagen.generate_traces();
        let otap_records =
            otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Traces(traces_data));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    fn create_test_pipeline_context() -> PipelineContext {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
    }

    /// Assert that the processor output is semantically equivalent to the
    /// expected set of [`OtapArrowRecords`] batches combined.
    fn assert_output_equivalent(output: &OtapPdata, expected: &[OtapArrowRecords]) {
        let actual = match output.payload_ref() {
            OtapPayload::OtapArrowRecords(r) => r,
            _ => panic!("expected OtapArrowRecords payload"),
        };
        let expected_msgs: Vec<_> = expected.iter().map(otap_to_otlp).collect();
        assert_equivalent(&[otap_to_otlp(actual)], &expected_msgs);
    }

    /// Assert that the processor output is semantically equivalent to the
    /// expected OTLP [`MetricsData`].
    fn assert_output_otlp_equivalent(output: &OtapPdata, expected: MetricsData) {
        let actual = match output.payload_ref() {
            OtapPayload::OtapArrowRecords(r) => r,
            _ => panic!("expected OtapArrowRecords payload"),
        };
        assert_equivalent(
            &[otap_to_otlp(actual)],
            &[otap_df_pdata::proto::OtlpProtoMessage::Metrics(expected)],
        );
    }

    /// Encode OTLP [`MetricsData`] into serialized protobuf bytes and wrap
    /// as an [`OtapPdata`] with an [`OtlpBytes`] payload.
    fn make_otlp_bytes_pdata(metrics_data: MetricsData) -> OtapPdata {
        let msg = otap_df_pdata::proto::OtlpProtoMessage::Metrics(metrics_data);
        let otlp_bytes = otlp_message_to_bytes(&msg);
        OtapPdata::new_default(OtapPayload::OtlpBytes(otlp_bytes))
    }

    /// Build an OTLP [`MetricsData`] with `n` unique gauge metrics, each with
    /// a single data point. All metrics share one resource and one scope.
    fn make_gauge(name: &str, value: f64) -> Metric {
        Metric::build()
            .name(name)
            .data_gauge(Gauge::new(vec![
                NumberDataPoint::build()
                    .time_unix_nano(1000u64)
                    .value_double(value)
                    .finish(),
            ]))
            .finish()
    }

    fn make_sum(name: &str, aggregatable: bool, value: i64, exemplars: Vec<Exemplar>) -> Metric {
        let temporality = if aggregatable {
            AggregationTemporality::Cumulative
        } else {
            AggregationTemporality::Delta
        };
        Metric::build()
            .name(name)
            .data_sum(Sum::new(
                temporality,
                true,
                vec![
                    NumberDataPoint::build()
                        .time_unix_nano(1000u64)
                        .value_int(value)
                        .exemplars(exemplars)
                        .finish(),
                ],
            ))
            .finish()
    }

    fn make_histogram(name: &str, aggregatable: bool, exemplars: Vec<Exemplar>) -> Metric {
        let temporality = if aggregatable {
            AggregationTemporality::Cumulative
        } else {
            AggregationTemporality::Delta
        };
        Metric::build()
            .name(name)
            .data_histogram(Histogram::new(
                temporality,
                vec![
                    HistogramDataPoint::build()
                        .time_unix_nano(1000u64)
                        .count(10u64)
                        .sum(100.0f64)
                        .bucket_counts(vec![2, 3, 5])
                        .explicit_bounds(vec![10.0, 50.0])
                        .exemplars(exemplars)
                        .finish(),
                ],
            ))
            .finish()
    }

    fn make_exp_histogram(name: &str, aggregatable: bool, exemplars: Vec<Exemplar>) -> Metric {
        let temporality = if aggregatable {
            AggregationTemporality::Cumulative
        } else {
            AggregationTemporality::Delta
        };
        Metric::build()
            .name(name)
            .data_exponential_histogram(ExponentialHistogram::new(
                temporality,
                vec![
                    ExponentialHistogramDataPoint::build()
                        .time_unix_nano(1000u64)
                        .count(5u64)
                        .scale(2i32)
                        .zero_count(1u64)
                        .positive(Buckets::new(0, vec![1, 2]))
                        .negative(Buckets::new(0, vec![1, 1]))
                        .exemplars(exemplars)
                        .finish(),
                ],
            ))
            .finish()
    }

    fn make_n_gauge_metrics(n: usize) -> MetricsData {
        let metrics: Vec<_> = (0..n)
            .map(|i| {
                Metric::build()
                    .name(format!("metric_{i}"))
                    .data_gauge(Gauge::new(vec![
                        NumberDataPoint::build()
                            .time_unix_nano(1000u64)
                            .value_double(i as f64)
                            .finish(),
                    ]))
                    .finish()
            })
            .collect();
        MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().finish(),
                metrics,
            )],
        )])
    }
}
