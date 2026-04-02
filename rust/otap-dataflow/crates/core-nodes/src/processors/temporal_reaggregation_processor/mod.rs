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

use self::builder::{Checkpoint, MetricSignalBuilder, ResourceMeta, ScopeMeta, StreamMeta};
use self::config::Config;
use self::identity::{
    AttributeHash, HashBuffer, MetricId, MetricIdRef, ResourceId, ScopeId, ScopeIdRef, StreamId,
    StreamIdRef, metric_id_of, resource_id_of, scope_id_of, stream_id_of,
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
    resources: HashMap<ResourceId, ResourceMeta>,
    scopes: HashMap<ScopeId<'static>, ScopeMeta>,
    metrics: HashMap<MetricId<'static>, u16>,
    streams: HashMap<StreamId<'static>, StreamMeta>,

    next_id_resource: u16,
    next_id_scope: u16,
    next_id_metric: u16,
    next_id_ndp: u32,
    next_id_hdp: u32,
    next_id_ehdp: u32,
    next_id_sdp: u32,
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

                        // If we successfully process then we're done
                        let Err(e) = self.process_metrics_pdata(&pdata) else {
                            return Ok(());
                        };

                        match e {
                            // Id overflows are fine. In this case we seal the current outbound
                            // batch and then feed the data back into a next one. We do this to
                            // prevent complex ack/nack scenarios where an input batch has
                            // representation in multiple output batches.
                            ProcessPdataError::IdOverflow => {
                                self.metrics.flushes_overflow.inc();
                                self.flush_at(effect_handler, &checkpoint).await?;
                                match self.process_metrics_pdata(&pdata) {
                                    Ok(_) => {
                                        return Ok(());
                                    }
                                    Err(_) => {
                                        // TODO: Nack this data
                                        self.metrics.batches_rejected.inc();
                                        self.clear_state();
                                    }
                                }
                            }

                            // If we can't even create the view then we just have
                            // to throw the batch away, we'll never be able to
                            // operate on it.
                            ProcessPdataError::ViewCreationFailed { .. } => {
                                self.metrics.batches_rejected.inc();
                                // TODO: Nack this data
                            }
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
                    self.flush(effect_handler).await?;
                    self.metrics.flushes_timer.add(1);
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. } => {
                    self.flush(effect_handler).await?;
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

    /// Parse and process a metrics pdata payload. Returns `None` if the
    /// payload could not be parsed as a metrics view.
    fn process_metrics_pdata(&mut self, pdata: &OtapPdata) -> Result<(), ProcessPdataError> {
        match pdata.payload_ref() {
            OtapPayload::OtapArrowRecords(records) => OtapMetricsView::try_from(records)
                .map_err(|e| ProcessPdataError::ViewCreationFailed { inner: Some(e) })
                .and_then(|v| self.process_view(&v)),
            OtapPayload::OtlpBytes(otlp) => {
                let data = RawMetricsData::new(otlp.as_bytes());
                self.process_view(&data)
            }
        }
    }

    /// Flush accumulated metrics and send downstream.
    async fn flush(
        &mut self,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if !self.state.streams.is_empty() {
            // TODO: Track some kind of failure metric here
            if let Ok(records) = self.builder.finish(None) {
                let pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(records));
                effect_handler.send_message_with_source_node(pdata).await?;
            }
            self.clear_state();
        }
        Ok(())
    }

    /// Flush accumulated metrics up to the given checkpoint and reset state.
    ///
    /// This is used when a [`process_view`] call fails due to ID overflow —
    /// the data appended before the checkpoint is clean and should be sent
    /// downstream, while the partial data from the failed call is discarded
    /// by slicing the record batches.
    async fn flush_at(
        &mut self,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
        checkpoint: &Checkpoint,
    ) -> Result<(), Error> {
        if let Ok(records) = self.builder.finish(Some(checkpoint)) {
            if !records.is_empty() {
                // TODO: Ack/nack and context propagation
                let pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(records));
                effect_handler.send_message_with_source_node(pdata).await?;
            }
        }
        self.clear_state();
        Ok(())
    }

    /// Process a view, attempting to aggregate all of the records. May fail due
    /// to id overflow or other capacity related restrictions.
    fn process_view<V: MetricsView>(&mut self, view: &V) -> Result<(), ProcessPdataError> {
        for resource_metrics in view.resources() {
            let Some(resource) = resource_metrics.resource() else {
                continue;
            };

            let attrs = AttributeHash::compute(&mut self.hash_buf, resource.attributes());
            let resource_id = resource_id_of(attrs);
            self.process_resource(
                resource_id,
                &resource,
                resource_metrics.schema_url().unwrap_or(b""),
            )?;

            for scope_metrics in resource_metrics.scopes() {
                let Some(scope) = scope_metrics.scope() else {
                    continue;
                };

                let attrs = AttributeHash::compute(&mut self.hash_buf, scope.attributes());
                let scope_id = scope_id_of(resource_id, &scope, attrs);
                let scope_schema_url = scope_metrics.schema_url();
                self.process_scope(scope_id.clone(), &scope)?;

                for metric in scope_metrics.metrics() {
                    let Some(metric_id) = metric_id_of(scope_id.clone(), &metric) else {
                        continue;
                    };

                    if !is_aggregatable(&metric_id) {
                        // TODO: Build passthrough batch.
                        continue;
                    }

                    let otap_metric_id = self.process_metric(
                        metric_id.clone(),
                        &metric,
                        resource_id,
                        &scope_id,
                        scope_schema_url,
                    )?;

                    let Some(data) = metric.data() else {
                        continue;
                    };

                    match data.value_type() {
                        DataType::Gauge => {
                            if let Some(gauge) = data.as_gauge() {
                                for dp in gauge.data_points() {
                                    let attrs =
                                        AttributeHash::compute(&mut self.hash_buf, dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.process_number_dp(&dp, stream_id, otap_metric_id)?;
                                }
                            }
                        }
                        DataType::Sum => {
                            if let Some(sum) = data.as_sum() {
                                for dp in sum.data_points() {
                                    let attrs =
                                        AttributeHash::compute(&mut self.hash_buf, dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.process_number_dp(&dp, stream_id, otap_metric_id)?;
                                }
                            }
                        }
                        DataType::Histogram => {
                            if let Some(hist) = data.as_histogram() {
                                for dp in hist.data_points() {
                                    let attrs =
                                        AttributeHash::compute(&mut self.hash_buf, dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.process_histogram_dp(&dp, stream_id, otap_metric_id)?;
                                }
                            }
                        }
                        DataType::ExponentialHistogram => {
                            if let Some(exp) = data.as_exponential_histogram() {
                                for dp in exp.data_points() {
                                    let attrs =
                                        AttributeHash::compute(&mut self.hash_buf, dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.process_exp_histogram_dp(&dp, stream_id, otap_metric_id)?;
                                }
                            }
                        }
                        DataType::Summary => {
                            if let Some(summary) = data.as_summary() {
                                for dp in summary.data_points() {
                                    let attrs =
                                        AttributeHash::compute(&mut self.hash_buf, dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.process_summary_dp(&dp, stream_id, otap_metric_id)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn process_resource<R: ResourceView>(
        &mut self,
        resource_id: ResourceId,
        view: &R,
        schema_url: &[u8],
    ) -> Result<(), ProcessPdataError> {
        if let Vacant(v) = self.state.resources.entry_ref(&resource_id) {
            let id = self.state.next_id_resource;
            if id == u16::MAX {
                return Err(ProcessPdataError::IdOverflow);
            }
            self.state.next_id_resource += 1;

            self.builder.append_resource(id, view);

            _ = v.insert_with_key(
                resource_id,
                ResourceMeta {
                    id,
                    schema_url: schema_url.to_vec(),
                    dropped_attributes_count: view.dropped_attributes_count(),
                },
            );
        }
        Ok(())
    }

    fn process_scope<S: InstrumentationScopeView>(
        &mut self,
        scope_id: ScopeId<'_>,
        view: &S,
    ) -> Result<(), ProcessPdataError> {
        // ScopeIdRef clones cheaply
        let lookup_scope = ScopeIdRef(&scope_id.clone());
        if let Vacant(v) = self.state.scopes.entry_ref(&lookup_scope) {
            let id = self.state.next_id_scope;
            if id == u16::MAX {
                return Err(ProcessPdataError::IdOverflow);
            }
            self.state.next_id_scope += 1;

            self.builder.append_scope(id, view);

            _ = v.insert_with_key(
                scope_id.into_owned(),
                ScopeMeta {
                    id,
                    name: view.name().unwrap_or(b"").to_vec(),
                    version: view.version().unwrap_or(b"").to_vec(),
                    dropped_attributes_count: view.dropped_attributes_count(),
                },
            );
        }
        Ok(())
    }

    fn process_metric<M: MetricView>(
        &mut self,
        metric_id: MetricId<'_>,
        view: &M,
        resource_id: ResourceId,
        scope_id: &ScopeId<'_>,
        scope_schema_url: &[u8],
    ) -> Result<u16, ProcessPdataError> {
        // MetricIdRef clones cheaply
        let lookup_metric = MetricIdRef(&metric_id.clone());
        match self.state.metrics.entry_ref(&lookup_metric) {
            Occupied(o) => Ok(*o.get()),
            Vacant(v) => {
                let id = self.state.next_id_metric;
                if id == u16::MAX {
                    return Err(ProcessPdataError::IdOverflow);
                }
                self.state.next_id_metric += 1;

                let resource_meta = &self.state.resources[&resource_id];
                let scope_meta = self
                    .state
                    .scopes
                    .get(&ScopeIdRef(scope_id))
                    .expect("scope must exist");

                self.builder.append_metric(
                    id,
                    &metric_id,
                    view,
                    resource_meta,
                    scope_meta,
                    scope_schema_url,
                );

                _ = v.insert_with_key(metric_id.into_owned(), id);
                Ok(id)
            }
        }
    }

    fn process_number_dp<V: NumberDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) -> Result<(), ProcessPdataError> {
        let time = dp.time_unix_nano();
        // StreamIdRef clones cheaply
        let lookup_stream = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup_stream) {
            Occupied(mut o) => {
                let state = o.get_mut();
                if time > state.time_unix_nano {
                    let row_index = state.dp_row_index;
                    state.time_unix_nano = time;
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
        // StreamIdRef clones cheaply
        let lookup_stream = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup_stream) {
            Occupied(mut o) => {
                let state = o.get_mut();
                if time > state.time_unix_nano {
                    let row_index = state.dp_row_index;
                    state.time_unix_nano = time;
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
        // StreamIdRef clones cheaply
        let lookup_stream = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup_stream) {
            Occupied(mut o) => {
                let state = o.get_mut();
                if time > state.time_unix_nano {
                    let row_index = state.dp_row_index;
                    state.time_unix_nano = time;
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
        // StreamIdRef clones cheaply
        let lookup_stream = StreamIdRef(&stream_id.clone());
        match self.state.streams.entry_ref(&lookup_stream) {
            Occupied(mut o) => {
                let state = o.get_mut();
                if time > state.time_unix_nano {
                    let row_index = state.dp_row_index;
                    state.time_unix_nano = time;
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

    fn clear_state(&mut self) {
        self.state.clear();
        self.builder.clear();
    }
}

/// Returns `true` if this metric type should be aggregated (buffered and
/// flushed on timer), `false` if it should be passed through unchanged.
fn is_aggregatable(metric_id: &MetricId<'_>) -> bool {
    let Some(data_type) = DataType::from_u8(metric_id.data_type) else {
        return false;
    };

    let cumulative = metric_id.aggregation_temporality == AggregationTemporality::Cumulative as u8;
    let monotonic = metric_id.is_monotonic;

    match data_type {
        DataType::Gauge | DataType::Summary => true,
        DataType::Sum if cumulative && monotonic => true,
        DataType::Histogram | DataType::ExponentialHistogram if cumulative => true,
        _ => false,
    }
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
        AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
        Histogram, HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum, Summary, SummaryDataPoint,
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
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(10.0f64)
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
                    vec![
                        Metric::build()
                            .name("requests")
                            .data_sum(Sum::new(
                                AggregationTemporality::Cumulative,
                                true,
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .value_int(100i64)
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
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(10.0f64)
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]));
            let batch2 = make_otlp_bytes_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("env", AnyValue::new_string("staging"))])
                    .finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(20.0f64)
                                    .finish(),
                            ]))
                            .finish(),
                    ],
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
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .value_double(10.0f64)
                                    .finish(),
                            ]))
                            .finish(),
                    ],
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
}
