// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.
//! Note: This receiver will be replaced in the future with a more sophisticated implementation.
//!

use crate::receivers::fake_data_generator::config::{
    Config, DataSource, GenerationStrategy, ResourceAttributeSet, build_rotation_table,
};
use async_trait::async_trait;
use linkme::distributed_slice;
use metrics::FakeSignalReceiverMetrics;
use otap_df_channel::error::RecvError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::CallData;
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{
    Interests, ProducerEffectHandlerExtension, ReceiverFactory, control::NodeControlMsg,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_debug, otel_info, otel_warn};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, Instant, sleep};
use weaver_forge::registry::ResolvedRegistry;

pub mod attributes;
/// allows the user to configure their fake signal receiver
pub mod config;
/// provides the fake signal with fake data
pub mod fake_data;
/// fake signal metrics implementation
pub mod metrics;
/// generates signals based on OTel semantic conventions registry
pub mod semconv_signal;
/// Static hardcoded signal generators for lightweight load testing
pub mod static_signal;

/// The URN for the fake data generator receiver
pub const OTAP_FAKE_DATA_GENERATOR_URN: &str = "urn:otel:receiver:traffic_generator";

/// A Receiver that generates fake OTAP data for testing purposes.
pub struct FakeGeneratorReceiver {
    /// Configuration for the fake data generator
    config: Config,
    /// Metrics for the fake data generator
    metrics: MetricSet<FakeSignalReceiverMetrics>,
}

/// Declares the fake data generator as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_FAKE_DATA_GENERATOR: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTAP_FAKE_DATA_GENERATOR_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            FakeGeneratorReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl FakeGeneratorReceiver {
    /// creates a new FakeSignalReceiver
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let metrics = pipeline_ctx.register_metrics::<FakeSignalReceiverMetrics>();
        Self { config, metrics }
    }

    /// Creates a new fake data generator from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        Ok(FakeGeneratorReceiver::new(
            pipeline_ctx,
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?,
        ))
    }
}

/// Abstraction over signal generation to support different data sources
enum SignalGenerator {
    /// Uses semantic conventions registry via weaver
    SemanticConventions(ResolvedRegistry),
    /// Uses static hardcoded signals with weighted per-batch resource attribute
    /// rotation and optional payload customization.
    Static {
        entries: Vec<ResourceAttributeSet>,
        rotation: Vec<usize>,
        /// Target log body size in bytes (None = default body)
        log_body_size_bytes: Option<usize>,
        /// Number of log attributes (None = default attributes)
        num_log_attributes: Option<usize>,
        /// Whether to populate trace_id/span_id on log records
        use_trace_context: bool,
    },
}

impl SignalGenerator {
    /// Return the resource attributes for the current batch, or `None` when no
    /// custom attributes are configured.
    ///
    /// The slot is selected as `rotation[batch_index % rotation.len()]`, which
    /// gives each entry a share of batches proportional to its weight.
    fn attrs_for_batch(&self, batch_index: u64) -> Option<&HashMap<String, String>> {
        match self {
            SignalGenerator::Static {
                entries, rotation, ..
            } if !rotation.is_empty() => {
                let slot = rotation[batch_index as usize % rotation.len()];
                Some(&entries[slot].attrs)
            }
            _ => None,
        }
    }

    /// Generate OTLP traces
    fn generate_traces(&self, count: usize, batch_index: u64) -> OtlpProtoMessage {
        match self {
            SignalGenerator::SemanticConventions(registry) => {
                OtlpProtoMessage::Traces(semconv_signal::semconv_otlp_traces(count, registry))
            }
            SignalGenerator::Static { .. } => OtlpProtoMessage::Traces(
                static_signal::static_otlp_traces(count, self.attrs_for_batch(batch_index)),
            ),
        }
    }

    /// Generate OTLP metrics
    fn generate_metrics(&self, count: usize, batch_index: u64) -> OtlpProtoMessage {
        match self {
            SignalGenerator::SemanticConventions(registry) => {
                OtlpProtoMessage::Metrics(semconv_signal::semconv_otlp_metrics(count, registry))
            }
            SignalGenerator::Static { .. } => OtlpProtoMessage::Metrics(
                static_signal::static_otlp_metrics(count, self.attrs_for_batch(batch_index)),
            ),
        }
    }

    /// Generate OTLP logs
    fn generate_logs(&self, count: usize, batch_index: u64) -> OtlpProtoMessage {
        match self {
            SignalGenerator::SemanticConventions(registry) => {
                OtlpProtoMessage::Logs(semconv_signal::semconv_otlp_logs(count, registry))
            }
            SignalGenerator::Static {
                log_body_size_bytes,
                num_log_attributes,
                use_trace_context,
                ..
            } => OtlpProtoMessage::Logs(static_signal::static_otlp_logs_with_config(
                count,
                *log_body_size_bytes,
                *num_log_attributes,
                *use_trace_context,
                self.attrs_for_batch(batch_index),
            )),
        }
    }
}

/// Pre-generated batch cache for high-throughput load testing.
/// A single batch is generated once at startup and cloned at runtime.
/// Clone is O(1) since OtapPdata contains Bytes which is ref-counted.
struct BatchCache {
    /// Pre-generated metrics batch
    metrics: Option<OtapPdata>,
    /// Pre-generated traces batch
    traces: Option<OtapPdata>,
    /// Pre-generated logs batch
    logs: Option<OtapPdata>,
    /// Number of records in the metrics batch
    metrics_batch_size: usize,
    /// Number of records in the traces batch
    traces_batch_size: usize,
    /// Number of records in the logs batch
    logs_batch_size: usize,
}

impl BatchCache {
    /// Create a new batch cache by pre-generating a single batch for each signal type.
    /// The batch contains up to `batch_size` records, which will be sent multiple times
    /// per iteration to match the total signal count.
    ///
    /// **Note:** Cached batches always use the first resource-attribute set (slot 0).
    /// Resource attribute rotation is not supported with `pre_generated` strategy;
    /// use `fresh` or `templates` for per-batch attribute rotation.
    fn new(
        generator: &SignalGenerator,
        batch_size: usize,
        metric_count: usize,
        trace_count: usize,
        log_count: usize,
    ) -> Result<Self, Error> {
        // Pre-generate single metrics batch
        let metrics_batch_size = metric_count.min(batch_size);
        let metrics = if metric_count > 0 {
            Some(
                generator
                    .generate_metrics(metrics_batch_size, 0)
                    .try_into()?,
            )
        } else {
            None
        };

        // Pre-generate single traces batch
        let traces_batch_size = trace_count.min(batch_size);
        let traces = if trace_count > 0 {
            Some(generator.generate_traces(traces_batch_size, 0).try_into()?)
        } else {
            None
        };

        // Pre-generate single logs batch
        let logs_batch_size = log_count.min(batch_size);
        let logs = if log_count > 0 {
            let pdata: OtapPdata = generator.generate_logs(logs_batch_size, 0).try_into()?;
            let (_, payload) = pdata.clone().into_parts();
            let size = payload.num_bytes().unwrap_or(0);
            otel_info!(
                "fake_data_generator.batch_cache.logsize",
                log_record_count = logs_batch_size,
                batch_size_bytes = size,
                message = "Pre-generated log batch ready"
            );
            Some(pdata)
        } else {
            None
        };

        Ok(Self {
            metrics,
            traces,
            logs,
            metrics_batch_size,
            traces_batch_size,
            logs_batch_size,
        })
    }
}

/// Handle a control message received on the control channel.
///
/// Returns `Ok(Some(terminal_state))` when the receiver should exit,
/// `Ok(None)` when it should continue the event loop, or `Err` on a
/// channel error.
async fn handle_control_msg(
    ctrl_msg: Result<NodeControlMsg<OtapPdata>, RecvError>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut MetricSet<FakeSignalReceiverMetrics>,
) -> Result<Option<TerminalState>, Error> {
    match ctrl_msg {
        Ok(NodeControlMsg::CollectTelemetry {
            mut metrics_reporter,
        }) => {
            _ = metrics_reporter.report(metrics);
            Ok(None)
        }
        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
            otel_info!("fake_data_generator.drain_ingress");
            effect_handler.notify_receiver_drained().await?;
            Ok(Some(TerminalState::new(deadline, [metrics.snapshot()])))
        }
        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
            otel_info!("fake_data_generator.shutdown");
            Ok(Some(TerminalState::new(deadline, [metrics.snapshot()])))
        }
        Err(e) => Err(Error::ChannelRecvError(e)),
        _ => Ok(None),
    }
}

/// Implement the Receiver trait for the FakeGeneratorReceiver
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for FakeGeneratorReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        //start event loop
        let traffic_config = self.config.get_traffic_config();
        let data_source = self.config.data_source().clone();
        let generation_strategy = self.config.generation_strategy().clone();

        // Create the appropriate signal generator based on data source
        let signal_generator = match data_source {
            DataSource::SemanticConventions => {
                let registry = self
                    .config
                    .get_registry()
                    .map_err(|err| Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Configuration,
                        error: err,
                        source_detail: String::new(),
                    })?
                    .expect("SemanticConventions data source should return Some registry");
                SignalGenerator::SemanticConventions(registry)
            }
            DataSource::Static => {
                let entries = self.config.resource_attributes().to_vec();
                let rotation = build_rotation_table(&entries);
                SignalGenerator::Static {
                    entries,
                    rotation,
                    log_body_size_bytes: traffic_config.log_body_size_bytes(),
                    num_log_attributes: traffic_config.num_log_attributes(),
                    use_trace_context: traffic_config.use_trace_context(),
                }
            }
        };

        let (metric_count, trace_count, log_count) = traffic_config.calculate_signal_count();
        let max_signal_count = traffic_config.get_max_signal_count();
        let signals_per_second = traffic_config.get_signal_rate();
        let max_batch_size = traffic_config.get_max_batch_size();
        let enable_ack_nack = self.config.enable_ack_nack();
        let rate_limit_status = match signals_per_second {
            Some(rate) => format!("{} signals/sec", rate),
            None => "uncapped".to_string(),
        };
        otel_info!(
            "fake_data_generator.start",
            signals_per_second = rate_limit_status,
            max_batch_size = max_batch_size,
            metrics_per_iteration = metric_count,
            traces_per_iteration = trace_count,
            logs_per_iteration = log_count,
            data_source = format!("{:?}", self.config.data_source()),
            generation_strategy = format!("{:?}", generation_strategy),
            message = "Fake data generator receiver started"
        );

        // Create batch cache if using PreGenerated strategy
        let batch_cache = match generation_strategy {
            GenerationStrategy::PreGenerated => {
                if !self.config.resource_attributes().is_empty() {
                    otel_warn!(
                        "fake_data_generator.config_warning",
                        message = "resource_attributes rotation is not supported with pre_generated strategy; \
                                   only the first attribute set will be used. Use 'fresh' or 'templates' for rotation."
                    );
                }
                otel_info!(
                    "fake_data_generator.pre_generate",
                    message = "Pre-generating batch for high-throughput mode"
                );
                Some(
                    BatchCache::new(
                        &signal_generator,
                        max_batch_size,
                        metric_count,
                        trace_count,
                        log_count,
                    )
                    .map_err(|e| Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Configuration,
                        error: format!("Failed to pre-generate batch: {}", e),
                        source_detail: String::new(),
                    })?,
                )
            }
            GenerationStrategy::Fresh | GenerationStrategy::Templates => None,
        };

        let mut signal_count: u64 = 0;
        let mut batch_rotation_index: u64 = 0;
        let one_second_duration = Duration::from_secs(1);

        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        loop {
            let wait_till = Instant::now() + one_second_duration;
            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks
                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
                    if let Some(terminal) = handle_control_msg(ctrl_msg, &effect_handler, &mut self.metrics).await? {
                        return Ok(terminal);
                    }
                }
                // generate and send signal based on provided configuration
                signal_status = send_signals(
                    effect_handler.clone(),
                    max_signal_count,
                    &mut signal_count,
                    &mut batch_rotation_index,
                    max_batch_size,
                    metric_count,
                    trace_count,
                    log_count,
                    &signal_generator,
                    &batch_cache,
                    enable_ack_nack,
                ), if max_signal_count.is_none_or(|max| max > signal_count) => {
                    // if signals per second is set then we should rate limit
                    match signal_status {
                        Ok(_) => {
                            self.metrics.logs_produced.add(log_count as u64);
                            self.metrics.metrics_produced.add(metric_count as u64);
                            self.metrics.spans_produced.add(trace_count as u64);
                            if signals_per_second.is_some() {
                                // check if need to sleep
                                let remaining_time = wait_till - Instant::now();
                                if remaining_time.as_secs_f64() > 0.0 {
                                    otel_debug!(
                                        "fake_data_generator.rate_limit.sleep",
                                        sleep_duration_ms = remaining_time.as_millis() as u64,
                                        "Sleeping to maintain configured signal rate"
                                    );
                                    // Keep the original sleep deadline if non-terminal control
                                    // messages arrive. Only DrainIngress/Shutdown should interrupt
                                    // the rate-limit wait early.
                                    let sleep_until = sleep(remaining_time);
                                    tokio::pin!(sleep_until);

                                    loop {
                                        tokio::select! {
                                            biased;
                                            ctrl_msg = ctrl_msg_recv.recv() => {
                                                if let Some(terminal) = handle_control_msg(ctrl_msg, &effect_handler, &mut self.metrics).await? {
                                                    return Ok(terminal);
                                                }
                                            }
                                            _ = &mut sleep_until => break,
                                        }
                                    }
                                }
                                // ToDo: Handle negative time, not able to keep up with specified rate limit
                            } else {
                                otel_debug!(
                                    "fake_data_generator.rate_limit.uncapped",
                                    "Rate limiting disabled, continuing immediately"
                                );
                            }
                        }
                        Err(e) => {
                            let source_detail = format_error_sources(&e);
                            return Err(Error::ReceiverError {
                                receiver: effect_handler.receiver_id(),
                                kind: ReceiverErrorKind::Other,
                                error: e.to_string(),
                                source_detail,
                            });
                        }
                    }
                }


            }
        }
    }
}

/// Send signals using either pre-generated cache or fresh generation
async fn send_signals(
    effect_handler: local::EffectHandler<OtapPdata>,
    max_signal_count: Option<u64>,
    signal_count: &mut u64,
    batch_rotation_index: &mut u64,
    max_batch_size: usize,
    metric_count: usize,
    trace_count: usize,
    log_count: usize,
    generator: &SignalGenerator,
    batch_cache: &Option<BatchCache>,
    enable_ack_nack: bool,
) -> Result<(), Error> {
    match batch_cache {
        Some(cache) => {
            send_cached_signals(
                effect_handler,
                max_signal_count,
                signal_count,
                metric_count,
                trace_count,
                log_count,
                cache,
                enable_ack_nack,
            )
            .await
        }
        None => {
            generate_signal_fresh(
                effect_handler,
                max_signal_count,
                signal_count,
                batch_rotation_index,
                max_batch_size,
                metric_count,
                trace_count,
                log_count,
                generator,
                enable_ack_nack,
            )
            .await
        }
    }
}

/// Send one generated pdata message, optionally subscribing to Ack/Nack interests.
async fn send_generated_pdata(
    effect_handler: &local::EffectHandler<OtapPdata>,
    mut pdata: OtapPdata,
    enable_ack_nack: bool,
) -> Result<(), Error> {
    if enable_ack_nack {
        effect_handler.subscribe_to(
            Interests::ACKS | Interests::NACKS,
            CallData::default(),
            &mut pdata,
        );
    }
    effect_handler.send_message_with_source_node(pdata).await?;
    Ok(())
}

/// Send signals from pre-generated cache (PreGenerated strategy).
/// Sends the cached batch multiple times to match the total signal count.
async fn send_cached_signals(
    effect_handler: local::EffectHandler<OtapPdata>,
    max_signal_count: Option<u64>,
    signal_count: &mut u64,
    metric_count: usize,
    trace_count: usize,
    log_count: usize,
    cache: &BatchCache,
    enable_ack_nack: bool,
) -> Result<(), Error> {
    let total_per_iteration = (metric_count + trace_count + log_count) as u64;

    // Check if we've reached max signal count
    if let Some(max_count) = max_signal_count {
        if *signal_count >= max_count {
            return Ok(());
        }
    }

    // Send cached metrics (multiple times if needed)
    if metric_count > 0 && cache.metrics_batch_size > 0 {
        if let Some(batch) = &cache.metrics {
            let send_count = metric_count / cache.metrics_batch_size;
            for _ in 0..send_count {
                send_generated_pdata(&effect_handler, batch.clone(), enable_ack_nack).await?;
            }
        }
    }

    // Send cached traces (multiple times if needed)
    if trace_count > 0 && cache.traces_batch_size > 0 {
        if let Some(batch) = &cache.traces {
            let send_count = trace_count / cache.traces_batch_size;
            for _ in 0..send_count {
                send_generated_pdata(&effect_handler, batch.clone(), enable_ack_nack).await?;
            }
        }
    }

    // Send cached logs (multiple times if needed)
    if log_count > 0 && cache.logs_batch_size > 0 {
        if let Some(batch) = &cache.logs {
            let send_count = log_count / cache.logs_batch_size;
            for _ in 0..send_count {
                send_generated_pdata(&effect_handler, batch.clone(), enable_ack_nack).await?;
            }
        }
    }

    *signal_count += total_per_iteration;
    Ok(())
}

/// generate and send signals (Fresh strategy - original behavior)
async fn generate_signal_fresh(
    effect_handler: local::EffectHandler<OtapPdata>,
    max_signal_count: Option<u64>,
    signal_count: &mut u64,
    batch_rotation_index: &mut u64,
    max_batch_size: usize,
    metric_count: usize,
    trace_count: usize,
    log_count: usize,
    generator: &SignalGenerator,
    enable_ack_nack: bool,
) -> Result<(), Error> {
    // nothing to send
    if max_batch_size == 0 {
        return Ok(());
    }

    let metric_count_split = metric_count / max_batch_size;
    let metric_count_remainder = metric_count % max_batch_size;
    let trace_count_split = trace_count / max_batch_size;
    let trace_count_remainder = trace_count % max_batch_size;
    let log_count_split = log_count / max_batch_size;
    let log_count_remainder = log_count % max_batch_size;

    if let Some(max_count) = max_signal_count {
        // don't generate signals if we reached max signal
        let mut current_count = *signal_count;
        if current_count >= max_count {
            return Ok(());
        }
        // update the counts here to allow us to reach the max_signal_count

        for _ in 0..metric_count_split {
            if max_count >= current_count + max_batch_size as u64 {
                send_generated_pdata(
                    &effect_handler,
                    generator
                        .generate_metrics(max_batch_size, *batch_rotation_index)
                        .try_into()?,
                    enable_ack_nack,
                )
                .await?;
                *batch_rotation_index += 1;
                current_count += max_batch_size as u64;
            } else {
                // generate last remaining signals
                let remaining_count: usize =
                    (max_count - current_count)
                        .try_into()
                        .map_err(|_| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Other,
                            error: "failed to convert u64 to usize".to_string(),
                            source_detail: String::new(),
                        })?;
                send_generated_pdata(
                    &effect_handler,
                    generator
                        .generate_metrics(remaining_count, *batch_rotation_index)
                        .try_into()?,
                    enable_ack_nack,
                )
                .await?;
                *batch_rotation_index += 1;

                // no more signals we have reached the max
                *signal_count = max_count;
                return Ok(());
            }
        }
        if metric_count_remainder > 0 && max_count >= current_count + metric_count_remainder as u64
        {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_metrics(metric_count_remainder, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            current_count += metric_count_remainder as u64;
        }

        // generate and send traces
        for _ in 0..trace_count_split {
            if max_count >= current_count + max_batch_size as u64 {
                send_generated_pdata(
                    &effect_handler,
                    generator
                        .generate_traces(max_batch_size, *batch_rotation_index)
                        .try_into()?,
                    enable_ack_nack,
                )
                .await?;
                *batch_rotation_index += 1;
                current_count += max_batch_size as u64;
            } else {
                let remaining_count: usize =
                    (max_count - current_count)
                        .try_into()
                        .map_err(|_| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Other,
                            error: "failed to convert u64 to usize".to_string(),
                            source_detail: String::new(),
                        })?;
                send_generated_pdata(
                    &effect_handler,
                    generator
                        .generate_traces(remaining_count, *batch_rotation_index)
                        .try_into()?,
                    enable_ack_nack,
                )
                .await?;
                *batch_rotation_index += 1;
                // no more signals we have reached the max
                *signal_count = max_count;
                return Ok(());
            }
        }
        if trace_count_remainder > 0 && max_count >= current_count + trace_count_remainder as u64 {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_traces(trace_count_remainder, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            current_count += trace_count_remainder as u64;
        }

        // generate and send logs
        for _ in 0..log_count_split {
            if max_count >= current_count + max_batch_size as u64 {
                send_generated_pdata(
                    &effect_handler,
                    generator
                        .generate_logs(max_batch_size, *batch_rotation_index)
                        .try_into()?,
                    enable_ack_nack,
                )
                .await?;
                *batch_rotation_index += 1;
                current_count += max_batch_size as u64;
            } else {
                let remaining_count: usize =
                    (max_count - current_count)
                        .try_into()
                        .map_err(|_| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Other,
                            error: "failed to convert u64 to usize".to_string(),
                            source_detail: String::new(),
                        })?;
                send_generated_pdata(
                    &effect_handler,
                    generator
                        .generate_logs(remaining_count, *batch_rotation_index)
                        .try_into()?,
                    enable_ack_nack,
                )
                .await?;
                *batch_rotation_index += 1;
                // no more signals we have reached the max
                *signal_count = max_count;
                return Ok(());
            }
        }
        if log_count_remainder > 0 && max_count >= current_count + log_count_remainder as u64 {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_logs(log_count_remainder, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            current_count += log_count_remainder as u64;
        }

        *signal_count = current_count;
    } else {
        // generate and send metric
        for _ in 0..metric_count_split {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_metrics(max_batch_size, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            *signal_count += max_batch_size as u64;
        }
        if metric_count_remainder > 0 {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_metrics(metric_count_remainder, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            *signal_count += metric_count_remainder as u64;
        }

        // generate and send traces
        for _ in 0..trace_count_split {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_traces(max_batch_size, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            *signal_count += max_batch_size as u64;
        }
        if trace_count_remainder > 0 {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_traces(trace_count_remainder, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            *signal_count += trace_count_remainder as u64;
        }

        // generate and send logs
        for _ in 0..log_count_split {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_logs(max_batch_size, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            *signal_count += max_batch_size as u64;
        }
        if log_count_remainder > 0 {
            send_generated_pdata(
                &effect_handler,
                generator
                    .generate_logs(log_count_remainder, *batch_rotation_index)
                    .try_into()?,
                enable_ack_nack,
            )
            .await?;
            *batch_rotation_index += 1;
            *signal_count += log_count_remainder as u64;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::receivers::fake_data_generator::config::{Config, TrafficConfig};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::MetricsData;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data;
    use otap_df_pdata::proto::opentelemetry::trace::v1::TracesData;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::time::{Duration, sleep};

    use std::collections::HashSet;
    use weaver_common::vdir::VirtualDirectoryPath;
    use weaver_forge::registry::ResolvedRegistry;

    const RESOURCE_COUNT: usize = 1;
    const SCOPE_COUNT: usize = 1;
    const MESSAGE_COUNT: usize = 1;
    const RUN_TILL_SHUTDOWN: u64 = 999;
    const MESSAGE_PER_SECOND: usize = 3;
    const MAX_SIGNALS: u64 = 3;
    const MAX_BATCH: usize = 30;

    /// Convert OtapPdata signal to OtlpProtoMessage for testing purposes.
    fn pdata_to_otlp_message(value: OtapPdata) -> OtlpProtoMessage {
        let otlp_bytes: OtlpProtoBytes = value
            .payload()
            .try_into()
            .expect("can convert signal to otlp bytes");
        match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                OtlpProtoMessage::Logs(LogsData::decode(bytes.as_ref()).expect("can decode bytes"))
            }
            OtlpProtoBytes::ExportMetricsRequest(bytes) => OtlpProtoMessage::Metrics(
                MetricsData::decode(bytes.as_ref()).expect("can decode bytes"),
            ),
            OtlpProtoBytes::ExportTracesRequest(bytes) => OtlpProtoMessage::Traces(
                TracesData::decode(bytes.as_ref()).expect("can decode bytes"),
            ),
        }
    }

    /// Test closure that simulates a typical receiver scenario.
    fn scenario() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // no scenario to run here as scenario is already defined in the configuration
                // wait for the scenario to finish running
                sleep(Duration::from_millis(RUN_TILL_SHUTDOWN)).await;
                // send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(std::time::Instant::now(), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure(
        resolved_registry: ResolvedRegistry,
    ) -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler
                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            // loop and check count
                            let resource_count = metric.resource_metrics.len();
                            assert!(resource_count == RESOURCE_COUNT);
                            for resource in metric.resource_metrics.iter() {
                                let scope_count = resource.scope_metrics.len();
                                assert!(scope_count == SCOPE_COUNT);
                                for scope in resource.scope_metrics.iter() {
                                    let metric_count = scope.metrics.len();
                                    assert!(metric_count == MESSAGE_COUNT);
                                    for metric in scope.metrics.iter() {
                                        let metric_definition = resolved_registry
                                            .groups
                                            .iter()
                                            .find(|group| {
                                                group.metric_name == Some(metric.name.clone())
                                            })
                                            .expect("metric not found in registry");
                                        assert_eq!(metric.description, metric_definition.brief);
                                        assert_eq!(
                                            Some(metric.unit.clone()),
                                            metric_definition.unit
                                        );

                                        let keys_metric_definition: HashSet<&str> =
                                            metric_definition
                                                .attributes
                                                .iter()
                                                .map(|attribute| attribute.name.as_str())
                                                .collect();

                                        match metric.data.as_ref().expect("metric has no data") {
                                            Data::Sum(sum) => {
                                                for datapoints in sum.data_points.iter() {
                                                    let keys: HashSet<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();

                                                    assert_eq!(keys, keys_metric_definition);
                                                }
                                            }
                                            Data::Gauge(gauge) => {
                                                for datapoints in gauge.data_points.iter() {
                                                    let keys: HashSet<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();

                                                    assert_eq!(keys, keys_metric_definition);
                                                }
                                            }
                                            Data::Histogram(histogram) => {
                                                for datapoints in histogram.data_points.iter() {
                                                    let keys: HashSet<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();

                                                    assert_eq!(keys, keys_metric_definition);
                                                }
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            let resource_count = span.resource_spans.len();
                            assert!(resource_count == RESOURCE_COUNT);
                            for resource in span.resource_spans.iter() {
                                let scope_count = resource.scope_spans.len();
                                assert!(scope_count == SCOPE_COUNT);
                                for scope in resource.scope_spans.iter() {
                                    let span_count = scope.spans.len();
                                    assert!(span_count == MESSAGE_COUNT);
                                    for span in scope.spans.iter() {
                                        let span_definition = resolved_registry
                                            .groups
                                            .iter()
                                            .find(|group| group.id == span.name)
                                            .expect("span not found in registry");
                                        let keys: HashSet<&str> = span
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        let keys_span_definition: HashSet<&str> = span_definition
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.name.as_str())
                                            .collect();

                                        assert_eq!(keys, keys_span_definition);

                                        let events: HashSet<&str> = span
                                            .events
                                            .iter()
                                            .map(|event| event.name.as_str())
                                            .collect();

                                        let events_span_definition: HashSet<&str> = span_definition
                                            .events
                                            .iter()
                                            .map(|event_name| event_name.as_str())
                                            .collect();
                                        assert_eq!(events, events_span_definition);
                                    }
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            let resource_count = log.resource_logs.len();
                            assert!(resource_count == RESOURCE_COUNT);
                            for resource in log.resource_logs.iter() {
                                let scope_count = resource.scope_logs.len();
                                assert!(scope_count == SCOPE_COUNT);
                                for scope in resource.scope_logs.iter() {
                                    let log_record_count = scope.log_records.len();
                                    assert!(log_record_count == MESSAGE_COUNT);
                                    for log_record in scope.log_records.iter() {
                                        let log_record_definition = resolved_registry
                                            .groups
                                            .iter()
                                            .find(|group| {
                                                group.name == Some(log_record.event_name.clone())
                                            })
                                            .expect("metric not found in registry");
                                        let keys: HashSet<&str> = log_record
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        let keys_log_record_definition: HashSet<&str> =
                                            log_record_definition
                                                .attributes
                                                .iter()
                                                .map(|attribute| attribute.name.as_str())
                                                .collect();

                                        assert_eq!(keys, keys_log_record_definition);
                                    }
                                }
                            }
                        }
                    }
                }
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, MAX_BATCH, 1, 1, 1);
        let config = Config::new(traffic_config, registry_path);
        let registry = config
            .get_registry()
            .expect("failed to get registry")
            .expect("registry should be Some for SemanticConventions data source");

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure(registry));
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure_message_rate()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            // loop and check count
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                    assert!(scope.metrics.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                    assert!(scope.spans.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                    assert!(scope.log_records.len() <= MAX_BATCH);
                                }
                            }
                        }
                    }
                }

                // Allow 1 to 2x (observed)
                assert!(received_messages >= MESSAGE_PER_SECOND);
                assert!(received_messages <= 2 * MESSAGE_PER_SECOND);
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver_message_rate_only() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, MAX_BATCH, 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver"),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_message_rate());
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure_max_signal()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            // loop and check count
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                    assert!(scope.metrics.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                    assert!(scope.spans.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                    assert!(scope.log_records.len() <= MAX_BATCH);
                                }
                            }
                        }
                    }
                }

                assert!(received_messages as u64 == MAX_SIGNALS);
            })
        }
    }
    #[test]
    fn test_fake_signal_receiver_max_signal_count_only() {
        let test_runtime = TestRuntime::new();
        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(None, Some(MAX_SIGNALS), MAX_BATCH, 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver"),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_max_signal());
    }

    /// Validation closure for PreGenerated strategy test
    fn validation_procedure_pregenerated()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                }
                            }
                        }
                    }
                }

                // Should have received at least some messages
                assert!(
                    received_messages > 0,
                    "Should receive messages from pre-generated cache"
                );
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver_static_pregenerated() {
        let test_runtime = TestRuntime::new();

        // Use Static data source with PreGenerated strategy
        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, MAX_BATCH, 1, 1, 1);
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::PreGenerated);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_pregenerated"),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_pregenerated());
    }

    /// Regression test: verifies that the receiver handles DrainIngress
    /// promptly instead of stalling until the drain deadline expires.
    /// Without proper DrainIngress handling the receiver would sleep
    /// through the entire rate-limit interval, causing DrainDeadlineReached.
    #[test]
    fn test_drain_ingress_exits_promptly() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        // signals_per_second=1 means the receiver sleeps ~1s between sends.
        // DrainIngress must interrupt that sleep and exit promptly.
        let traffic_config = TrafficConfig::new(Some(1), None, 1, 0, 0, 1);
        let config =
            Config::new(traffic_config, registry_path).with_data_source(DataSource::Static);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_drain"),
            node_config,
            test_runtime.config(),
        );

        let drain_scenario =
            move |ctx: TestContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    // Let the receiver start and enter its rate-limit sleep.
                    sleep(Duration::from_millis(200)).await;
                    let deadline = std::time::Instant::now() + Duration::from_secs(5);
                    ctx.send_control_msg(NodeControlMsg::DrainIngress {
                        deadline,
                        reason: "test drain".to_owned(),
                    })
                    .await
                    .expect("Failed to send DrainIngress");
                })
            };

        let drain_validation =
            |_ctx: NotSendValidateContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async {})
            };

        test_runtime
            .set_receiver(receiver)
            .run_test(drain_scenario)
            .run_validation(drain_validation);
    }

    /// Regression test: verifies that a non-terminal control message
    /// (CollectTelemetry) arriving during the rate-limit sleep does NOT
    /// break the sleep early – the receiver should still respect the
    /// original wait_till deadline.
    #[test]
    fn test_non_terminal_ctrl_msg_does_not_break_rate_limit_sleep() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        // signals_per_second=1 with a single log per iteration means the
        // receiver will sleep ~1s between sends. If the non-terminal control
        // message breaks the sleep, we'd see more than 2 batches in 1.5s.
        let traffic_config = TrafficConfig::new(Some(1), None, 1, 0, 0, 1);
        let config =
            Config::new(traffic_config, registry_path).with_data_source(DataSource::Static);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_ctrl_sleep"),
            node_config,
            test_runtime.config(),
        );

        let ctrl_scenario =
            move |ctx: TestContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    // Let the receiver start and enter its rate-limit sleep.
                    sleep(Duration::from_millis(200)).await;
                    // Fire a CollectTelemetry message mid-sleep. This is a
                    // non-terminal control message and must NOT break the
                    // rate-limit sleep.
                    let (_rx, metrics_reporter) =
                        otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
                    ctx.send_control_msg(NodeControlMsg::CollectTelemetry { metrics_reporter })
                        .await
                        .expect("Failed to send CollectTelemetry");

                    // Wait long enough for the first sleep to expire plus a
                    // small margin, but NOT long enough for a third iteration.
                    sleep(Duration::from_millis(1300)).await;

                    ctx.send_shutdown(std::time::Instant::now(), "Test")
                        .await
                        .expect("Failed to send Shutdown");
                })
            };

        let ctrl_validation =
            |mut ctx: NotSendValidateContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    let mut received_batches: u64 = 0;

                    while let Ok(_received_signal) = ctx.recv().await {
                        received_batches += 1;
                    }

                    // With 1 signal/sec and ~1.5s total runtime we expect at
                    // most 2 batches. If the non-terminal control message
                    // broke the sleep, we would see 3+.
                    assert!(
                        received_batches <= 2,
                        "Non-terminal control message broke the rate-limit sleep: \
                         expected at most 2 batches, got {received_batches}"
                    );
                })
            };

        test_runtime
            .set_receiver(receiver)
            .run_test(ctrl_scenario)
            .run_validation(ctrl_validation);
    }

    #[test]
    fn test_resource_attribute_rotation_across_batches() {
        use crate::receivers::fake_data_generator::config::ResourceAttributeSet;
        use std::collections::HashMap;
        use std::num::NonZeroU32;

        let make_entry = |tenant: &str| ResourceAttributeSet {
            attrs: HashMap::from([("tenant.id".to_string(), tenant.to_string())]),
            weight: NonZeroU32::new(1).unwrap(),
        };
        let entries = vec![make_entry("prod"), make_entry("staging")];
        let rotation = build_rotation_table(&entries);
        let generator = SignalGenerator::Static {
            entries,
            rotation,
            log_body_size_bytes: None,
            num_log_attributes: None,
            use_trace_context: false,
        };

        // attrs_for_batch should rotate through the two sets
        assert_eq!(generator.attrs_for_batch(0).unwrap()["tenant.id"], "prod");
        assert_eq!(
            generator.attrs_for_batch(1).unwrap()["tenant.id"],
            "staging"
        );
        assert_eq!(generator.attrs_for_batch(2).unwrap()["tenant.id"], "prod");
        assert_eq!(
            generator.attrs_for_batch(3).unwrap()["tenant.id"],
            "staging"
        );

        // Verify generated signals carry the rotated attributes
        let logs_batch_0 = match generator.generate_logs(1, 0) {
            OtlpProtoMessage::Logs(logs) => logs,
            _ => panic!("expected logs"),
        };
        let logs_batch_1 = match generator.generate_logs(1, 1) {
            OtlpProtoMessage::Logs(logs) => logs,
            _ => panic!("expected logs"),
        };

        let get_tenant_id = |logs: &LogsData| -> String {
            logs.resource_logs[0]
                .resource
                .as_ref()
                .unwrap()
                .attributes
                .iter()
                .find(|kv| kv.key == "tenant.id")
                .expect("tenant.id attribute missing")
                .value
                .as_ref()
                .unwrap()
                .to_string()
        };

        assert_ne!(
            get_tenant_id(&logs_batch_0),
            get_tenant_id(&logs_batch_1),
            "consecutive batches should use different resource attribute sets"
        );
    }

    #[test]
    fn test_resource_attribute_rotation_empty_attrs() {
        let generator = SignalGenerator::Static {
            entries: vec![],
            rotation: vec![],
            log_body_size_bytes: None,
            num_log_attributes: None,
            use_trace_context: false,
        };
        assert!(generator.attrs_for_batch(0).is_none());
        assert!(generator.attrs_for_batch(1).is_none());
    }
}
