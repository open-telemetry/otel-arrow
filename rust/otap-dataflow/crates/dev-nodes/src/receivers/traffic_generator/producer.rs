// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata_codec::{CodecGenerator, CodecRegistry, GenerationError, OtapPayload};
use prost::EncodeError;
use weaver_forge::v1::registry::ResolvedRegistry;

use super::config::{
    Config, DataSource, GenerationStrategy, ResourceAttributeSet, TrafficConfig,
    build_rotation_table,
};
use super::{semconv_signal, synthetic_signal};

/// A SignalProducer yields the signals that need to be exported in a given
/// second in order to meet the data volume production requirements with a
/// reasonably smooth curve.
pub struct TrafficProducer {
    strategy: ProductionStrategy,
    signal_count: u64,
    generator: Box<dyn SignalGenerator>,
    /// Optional upper bound on the total number of signals this producer may
    /// emit across its lifetime.
    max_signal_count: Option<u64>,
    /// Running total of signals confirmed exported by the receiver.
    produced_count: u64,
}

/// Describes the shape of traffic for one production run (one second of signals).
///
/// Each entry is a `(SignalType, count)` pair indicating how many signals of that
/// type to produce in a single batch. The entries are processed in order by the
/// [`TrafficRun`] iterator.
pub type TrafficShape = Vec<(SignalType, usize)>;

impl TrafficProducer {
    /// Build a `TrafficProducer` from the receiver configuration.
    ///
    /// This selects the appropriate `SignalGenerator` implementation based on
    /// `data_source` and builds either a `Fresh` or `Replay` production
    /// strategy from `generation_strategy`.
    pub fn from_config(
        config: &Config,
        codec_registry: &CodecRegistry,
    ) -> Result<Self, GenerateError> {
        config
            .validate()
            .map_err(|error| GenerateError::Configuration(error.to_string()))?;
        let traffic_config = config.get_traffic_config();

        // Build the concrete SignalGenerator based on the data source.
        let mut generator: Box<dyn SignalGenerator> = match config.data_source() {
            DataSource::SemanticConventions => {
                let registry = config
                    .get_registry()
                    .map_err(GenerateError::Configuration)?
                    .expect("SemanticConventions data source should return Some registry");
                Box::new(WeaverGenerator { registry })
            }
            DataSource::Synthetic => {
                let entries = config.resource_attributes().to_vec();
                let rotation = build_rotation_table(&entries);
                Box::new(SyntheticGenerator {
                    idx: 0,
                    entries,
                    rotation,
                    log_body_size_bytes: traffic_config.log_body_size_bytes(),
                    num_log_attributes: traffic_config.num_log_attributes(),
                    use_trace_context: traffic_config.use_trace_context(),
                    num_metric_attributes: traffic_config.num_metric_attributes(),
                    num_data_points_per_metric: traffic_config.num_data_points_per_metric(),
                })
            }
            DataSource::Codec => Box::new(CodecSignalGenerator {
                generators: config.codec_generators(codec_registry)?,
                timestamp_unix_nano: config.codec_timestamp_unix_nano(),
            }),
        };

        // Build the production strategy.
        let shape = create_shape(traffic_config);
        let mut strategy = match config.generation_strategy() {
            GenerationStrategy::Fresh => ProductionStrategy::Fresh {
                shape: shape.clone(),
            },
            GenerationStrategy::PreGenerated => {
                let payloads = create_fresh_payloads(&mut *generator, &shape)?;
                ProductionStrategy::Replay { payloads }
            }
        };

        let signal_count = strategy.signal_count();
        Ok(Self {
            strategy,
            signal_count,
            generator,
            max_signal_count: traffic_config.get_max_signal_count(),
            produced_count: 0,
        })
    }

    /// The number of batches in a run
    #[must_use]
    pub fn run_len(&self) -> usize {
        self.strategy.len()
    }

    /// The total number of signals in a run
    #[must_use]
    pub fn run_count(&self) -> u64 {
        self.signal_count
    }

    /// Record that the receiver successfully exported `count` signals.
    ///
    /// Called between runs (after the [`TrafficRun`] is dropped) so there is
    /// no borrow conflict with the generator.
    pub fn record_production(&mut self, count: u64) {
        self.produced_count += count;
    }

    /// Return a [`TrafficRun`] iterator that yields the payloads needed for
    /// one second of traffic, or `None` if the producer has reached
    /// `max_signal_count` and should stop.
    ///
    /// When a limit is set and the current strategy would exceed the remaining
    /// budget, the strategy is truncated in-place before the run is created.
    pub fn next_run(&mut self) -> Result<Option<TrafficRun<'_>>, GenerateError> {
        match self.max_signal_count {
            Some(max) if self.produced_count >= max => return Ok(None),
            Some(max) => {
                let remaining = max - self.produced_count;
                if self.signal_count > remaining {
                    self.cap_strategy(remaining)?;
                }
            }
            None => {}
        }

        Ok(Some(TrafficRun {
            generator: &mut *self.generator,
            strategy: &mut self.strategy,
            idx: 0,
        }))
    }

    /// Truncate the current strategy so it produces at most `max_signals`.
    ///
    /// For `Fresh`: truncates the shape vector, keeping whole entries that fit
    /// and appending a partial entry for the remainder.
    ///
    /// For `Replay`: keeps whole pre-generated payloads that fit, then
    /// generates one fresh partial payload for the remainder using the
    /// signal type from the original shape.
    fn cap_strategy(&mut self, max_signals: u64) -> Result<(), GenerateError> {
        let mut to_keep: usize = 0;
        let mut total = 0u64;

        for i in 0..self.strategy.len() {
            let curr_size = self.strategy.size_at(i) as u64;
            if total + curr_size > max_signals {
                break;
            }

            to_keep += 1;
            total += curr_size;
        }

        // If it's an exact fit we can just truncate
        if total == max_signals {
            self.strategy.truncate(to_keep);
            self.signal_count = self.strategy.signal_count();
            return Ok(());
        }

        // Otherwise we compute the leftover amount and append that on
        assert!(self.strategy.len() >= 1);
        let leftover_signal = self.strategy.signal_at(to_keep);
        let leftover_count = (max_signals - total) as usize;
        self.strategy.truncate(to_keep);
        match self.strategy {
            ProductionStrategy::Fresh { ref mut shape } => {
                shape.push((leftover_signal, leftover_count));
            }
            ProductionStrategy::Replay { ref mut payloads } => {
                let new_batch = match leftover_signal {
                    SignalType::Traces => self.generator.generate_traces(leftover_count),
                    SignalType::Metrics => self.generator.generate_metrics(leftover_count),
                    SignalType::Logs => self.generator.generate_logs(leftover_count),
                }?;

                payloads.push(new_batch);
            }
        }

        self.signal_count = self.strategy.signal_count();

        Ok(())
    }
}

/// A traffic run is a fixed size iterator of payloads that need to be delivered
/// in the current second. Consumers of this are expected to process every item
/// in the iterator within a second and grab a new iterator for the next second.
pub struct TrafficRun<'a> {
    generator: &'a mut dyn SignalGenerator,
    strategy: &'a mut ProductionStrategy,
    idx: usize,
}

impl TrafficRun<'_> {
    /// The number of signals remaining in the current run.
    #[must_use]
    pub fn remaining_signal_count(&mut self) -> u64 {
        (self.idx..self.strategy.len())
            .map(|idx| self.strategy.size_at(idx) as u64)
            .sum()
    }
}

impl<'a> Iterator for TrafficRun<'a> {
    type Item = GenerateResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            return None;
        }

        let next = match &self.strategy {
            ProductionStrategy::Fresh { shape, .. } => {
                let (signal_type, count) = shape[self.idx];
                match signal_type {
                    SignalType::Traces => self.generator.generate_traces(count),
                    SignalType::Metrics => self.generator.generate_metrics(count),
                    SignalType::Logs => self.generator.generate_logs(count),
                }
            }
            ProductionStrategy::Replay { payloads, .. } => Ok(payloads[self.idx].clone()),
        };

        self.idx += 1;
        Some(next)
    }
}

impl<'a> ExactSizeIterator for TrafficRun<'a> {
    fn len(&self) -> usize {
        self.strategy.len() - self.idx
    }
}

fn create_shape(cfg: &TrafficConfig) -> TrafficShape {
    let total_weight = cfg.log_weight + cfg.trace_weight + cfg.metric_weight;
    assert!(total_weight > 0);

    let log_percent = signal_percent(cfg.log_weight, total_weight);
    let metric_percent = signal_percent(cfg.metric_weight, total_weight);
    let trace_percent = signal_percent(cfg.trace_weight, total_weight);

    // When signals_per_second is None (uncapped / saturation mode), use a high
    // target so the shape contains enough batches to keep the open-loop sender
    // busy for a full 1-second run window. We use 1024 x max_batch_size which
    // is large enough for any realistic throughput while remaining bounded for
    // PreGenerated mode memory usage.
    //
    // When max_signal_count is also set, cap the uncapped target to that value
    // since the producer will never emit more than max_signal_count total.
    let uncapped_target = match cfg.get_max_signal_count() {
        Some(max) => (max as usize).min(cfg.max_batch_size.saturating_mul(1024)),
        None => cfg.max_batch_size.saturating_mul(1024),
    };
    let signals_per_second = cfg.signals_per_second.unwrap_or(uncapped_target) as u32;
    let logs_per_second = signal_per_second(signals_per_second, log_percent);
    let metrics_per_second = signal_per_second(signals_per_second, metric_percent);
    let traces_per_second = signal_per_second(signals_per_second, trace_percent);
    let total_per_second = logs_per_second + metrics_per_second + traces_per_second;

    // At this point total_per_second is within 3 of signals_per_second as we
    // could have clamped down at most 3 times and up at most 3 times.
    let mut per_second = [
        (SignalType::Logs, logs_per_second),
        (SignalType::Metrics, metrics_per_second),
        (SignalType::Traces, traces_per_second),
    ];
    per_second.sort_unstable_by_key(|x| x.1);
    per_second.reverse();

    // Add the extras to each signal, prioritizing the signals with the
    // highest share.
    let deficit = signals_per_second as i64 - total_per_second as i64;
    if deficit >= 1 {
        per_second[0].1 += 1
    }

    if deficit >= 2 {
        per_second[1].1 += 1
    }

    if deficit >= 3 {
        per_second[2].1 += 1
    }

    // Figure out approximately how to interweave the three signals so that we get
    // roughly even distribution of each's total payload within a given second
    let ratio1 = per_second[0].1 / per_second[2].1.max(1);
    let ratio2 = per_second[1].1 / per_second[2].1.max(1);
    let ratio3 = per_second[2].1 / per_second[2].1.max(1);

    let shape1 = get_traffic_shape(cfg.max_batch_size, per_second[0]);
    let shape2 = get_traffic_shape(cfg.max_batch_size, per_second[1]);
    let shape3 = get_traffic_shape(cfg.max_batch_size, per_second[2]);

    let mut shape1_i = shape1.iter();
    let mut shape2_i = shape2.iter();
    let mut shape3_i = shape3.iter();

    let mut result = Vec::new();
    loop {
        let before = result.len();

        result.extend(shape1_i.by_ref().take(ratio1));
        result.extend(shape2_i.by_ref().take(ratio2));
        result.extend(shape3_i.by_ref().take(ratio3));

        let after = result.len();
        if before == after {
            break;
        }
    }

    assert!(shape1_i.next().is_none());
    assert!(shape2_i.next().is_none());
    assert!(shape3_i.next().is_none());

    result
}

fn get_traffic_shape(max_batch_size: usize, per_second: (SignalType, usize)) -> TrafficShape {
    let full_buckets = per_second.1.div_euclid(max_batch_size);
    let mut result = Vec::new();
    for _ in 0..full_buckets {
        result.push((per_second.0, max_batch_size));
    }

    let remainder = per_second.1 % max_batch_size;
    if remainder > 0 {
        result.push((per_second.0, per_second.1 % max_batch_size));
    }

    result
}

fn signal_per_second(signals_per_second: u32, signal_percent: f32) -> usize {
    let mut signals = (signal_percent * signals_per_second as f32).floor() as usize;
    if signal_percent > 0.0 && signals == 0 {
        signals = 1
    }

    signals
}

fn signal_percent(signal_weight: u32, total_weight: u32) -> f32 {
    signal_weight as f32 / total_weight as f32
}

fn create_fresh_payloads(
    generator: &mut dyn SignalGenerator,
    shape: &TrafficShape,
) -> Result<Vec<OtapPayload>, GenerateError> {
    shape
        .iter()
        .map(|(signal_type, count)| match signal_type {
            SignalType::Traces => generator.generate_traces(*count),
            SignalType::Metrics => generator.generate_metrics(*count),
            SignalType::Logs => generator.generate_logs(*count),
        })
        .collect()
}

enum ProductionStrategy {
    Fresh { shape: TrafficShape },
    Replay { payloads: Vec<OtapPayload> },
}

impl ProductionStrategy {
    fn len(&self) -> usize {
        match self {
            ProductionStrategy::Fresh { shape, .. } => shape.len(),
            ProductionStrategy::Replay { payloads, .. } => payloads.len(),
        }
    }

    fn signal_count(&mut self) -> u64 {
        match self {
            ProductionStrategy::Fresh { shape, .. } => shape.iter().map(|x| x.1 as u64).sum(),
            ProductionStrategy::Replay { payloads, .. } => {
                payloads.iter_mut().map(|x| x.num_items() as u64).sum()
            }
        }
    }

    fn size_at(&mut self, idx: usize) -> usize {
        match self {
            ProductionStrategy::Fresh { shape, .. } => shape[idx].1,
            ProductionStrategy::Replay { payloads, .. } => payloads[idx].num_items(),
        }
    }

    fn truncate(&mut self, len: usize) {
        match self {
            ProductionStrategy::Fresh { shape, .. } => shape.truncate(len),
            ProductionStrategy::Replay { payloads, .. } => payloads.truncate(len),
        }
    }

    fn signal_at(&mut self, idx: usize) -> SignalType {
        match self {
            ProductionStrategy::Fresh { shape, .. } => shape[idx].0,
            ProductionStrategy::Replay { payloads, .. } => payloads[idx].signal_type(),
        }
    }
}

/// Result type for signal generation operations.
pub type GenerateResult = Result<OtapPayload, GenerateError>;

/// Errors that can occur during signal generation.
#[derive(thiserror::Error, Debug)]
pub enum GenerateError {
    /// Codec-owned generation failed.
    #[error(transparent)]
    CodecGeneration(#[from] GenerationError),
    /// Failed to encode the generated signal into the OTAP payload format.
    #[error("Failed to encode: {source:?}")]
    Encoding {
        /// The underlying encoding error.
        #[from]
        source: EncodeError,
    },
    /// A configuration error prevented producer construction.
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Trait for generating OTAP payloads containing telemetry signals.
///
/// Implementations produce synthetic logs, metrics, or traces and encode them
/// into [`OtapPayload`] values. Codec-backed implementations can reject
/// requests or output that violates their configured bounds.
pub trait SignalGenerator {
    /// Generate a log payload containing `count` log records.
    fn generate_logs(&mut self, count: usize) -> GenerateResult;
    /// Generate a metrics payload containing `count` data points.
    fn generate_metrics(&mut self, count: usize) -> GenerateResult;
    /// Generate a traces payload containing `count` spans.
    fn generate_traces(&mut self, count: usize) -> GenerateResult;
}

struct CodecSignalGenerator {
    generators: Vec<(SignalType, CodecGenerator)>,
    timestamp_unix_nano: Option<i64>,
}

impl CodecSignalGenerator {
    fn generate(&mut self, signal: SignalType, count: usize) -> GenerateResult {
        self.generate_with_clock(signal, count, || {
            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    GenerationError::Output(format!(
                        "system clock precedes the Unix epoch: {error}"
                    ))
                })?;
            i64::try_from(elapsed.as_nanos()).map_err(|error| {
                GenerationError::Output(format!(
                    "system timestamp exceeds i64 nanoseconds: {error}"
                ))
            })
        })
    }

    fn generate_with_clock(
        &mut self,
        signal: SignalType,
        count: usize,
        now: impl FnOnce() -> Result<i64, GenerationError>,
    ) -> GenerateResult {
        let generator = self
            .generators
            .iter_mut()
            .find(|(kind, _)| *kind == signal)
            .ok_or_else(|| {
                GenerateError::Configuration(format!(
                    "no codec generator configured for {signal:?}"
                ))
            })?;
        let timestamp = match self.timestamp_unix_nano {
            Some(timestamp) => timestamp,
            None => now()?,
        };
        Ok(generator.1.generate(count, timestamp)?)
    }
}

impl SignalGenerator for CodecSignalGenerator {
    fn generate_logs(&mut self, count: usize) -> GenerateResult {
        self.generate(SignalType::Logs, count)
    }

    fn generate_metrics(&mut self, count: usize) -> GenerateResult {
        self.generate(SignalType::Metrics, count)
    }

    fn generate_traces(&mut self, count: usize) -> GenerateResult {
        self.generate(SignalType::Traces, count)
    }
}

/// Signal generator backed by the Weaver semantic conventions registry.
///
/// Produces signals whose attributes are derived from the resolved registry,
/// giving realistic cardinality and naming.
pub struct WeaverGenerator {
    registry: ResolvedRegistry,
}

impl SignalGenerator for WeaverGenerator {
    fn generate_logs(&mut self, count: usize) -> GenerateResult {
        let payload = semconv_signal::semconv_otlp_logs(count, &self.registry);
        let payload = OtlpProtoMessage::Logs(payload);
        Ok(payload.try_into()?)
    }

    fn generate_metrics(&mut self, count: usize) -> GenerateResult {
        let payload = semconv_signal::semconv_otlp_metrics(count, &self.registry);
        let payload = OtlpProtoMessage::Metrics(payload);
        Ok(payload.try_into()?)
    }

    fn generate_traces(&mut self, count: usize) -> GenerateResult {
        let payload = semconv_signal::semconv_otlp_traces(count, &self.registry);
        let payload = OtlpProtoMessage::Traces(payload);
        Ok(payload.try_into()?)
    }
}

/// Signal generator that produces signals from hardcoded synthetic templates.
///
/// Resource attributes are rotated across batches according to a weighted
/// round-robin table built from the configured [`ResourceAttributeSet`] entries.
pub struct SyntheticGenerator {
    idx: usize,
    entries: Vec<ResourceAttributeSet>,
    rotation: Vec<usize>,
    /// Target log body size in bytes (None = default body)
    log_body_size_bytes: Option<usize>,
    /// Number of log attributes (None = default attributes)
    num_log_attributes: Option<usize>,
    /// Whether to populate trace_id/span_id on log records
    use_trace_context: bool,
    /// Number of metric attributes per data point (None = default attributes)
    num_metric_attributes: Option<usize>,
    /// Number of data points per metric (None = default)
    num_data_points_per_metric: Option<usize>,
}

impl SyntheticGenerator {
    /// Return the resource attributes for the current batch, or `None` when no
    /// custom attributes are configured.
    ///
    /// The slot is selected as `rotation[batch_index % rotation.len()]`, which
    /// gives each entry a share of batches proportional to its weight.
    fn attrs_for_batch(&self) -> Option<&HashMap<String, String>> {
        match self.rotation.is_empty() {
            true => None,
            false => {
                let slot = self.rotation[self.idx % self.rotation.len()];
                Some(&self.entries[slot].attrs)
            }
        }
    }
}

impl SignalGenerator for SyntheticGenerator {
    fn generate_logs(&mut self, count: usize) -> GenerateResult {
        let attrs = self.attrs_for_batch();
        let payload = synthetic_signal::static_otlp_logs_with_config(
            count,
            self.log_body_size_bytes,
            self.num_log_attributes,
            self.use_trace_context,
            attrs,
        );
        let payload = OtlpProtoMessage::Logs(payload);

        self.idx += 1;
        Ok(payload.try_into()?)
    }

    fn generate_metrics(&mut self, count: usize) -> GenerateResult {
        let attrs = self.attrs_for_batch();
        let payload = synthetic_signal::static_otlp_metrics_with_config(
            count,
            self.num_metric_attributes,
            self.num_data_points_per_metric,
            attrs,
        );
        let payload = OtlpProtoMessage::Metrics(payload);

        self.idx += 1;
        Ok(payload.try_into()?)
    }

    fn generate_traces(&mut self, count: usize) -> GenerateResult {
        let attrs = self.attrs_for_batch();
        let payload = synthetic_signal::static_otlp_traces(count, attrs);
        let payload = OtlpProtoMessage::Traces(payload);

        self.idx += 1;
        Ok(payload.try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::TrafficConfig;
    use super::*;
    use otel_arrow_dfe_config::codecs::CodecsConfig;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, any_value::Value as AnyValueValue,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_pdata_codec::{
        CodecGeneratorRegistration, CodecMetadata, CodecRegistration, CodecServiceBuilder,
        GenerationProfile, PdataEncoding, PdataGenerator, RegistryError, ResolvedCodec,
        builtins::OtlpBestEffortDecoder, register_pdata_codec,
    };
    use prost::Message;
    use std::cell::Cell;
    use std::sync::Arc;

    static FIXTURE_METADATA: CodecMetadata =
        CodecMetadata::new(PdataEncoding::new("trafficgen-test"), &[SignalType::Logs]);

    register_pdata_codec!(
        TRAFFICGEN_TEST_CODEC,
        CodecRegistration::new(&FIXTURE_METADATA)
            .with_decoder(|_| Box::new(OtlpBestEffortDecoder))
            .with_item_counter(|_, bytes| {
                LogsData::decode(bytes).ok().map(|logs| {
                    logs.resource_logs
                        .iter()
                        .flat_map(|resource| &resource.scope_logs)
                        .map(|scope| scope.log_records.len())
                        .sum()
                })
            })
            .with_generator(CodecGeneratorRegistration::new(
                &[GenerationProfile {
                    name: "default",
                    signals: &[SignalType::Logs],
                }],
                |codec, config| {
                    Ok(Box::new(FixtureGenerator {
                        codec,
                        sequence: config.seed,
                    }))
                },
            )),
    );

    // The binding regression and its generator factory run synchronously on the same test thread.
    thread_local! {
        static BOUND_GENERATORS_CREATED: Cell<usize> = const { Cell::new(0) };
    }

    const fn binding_fixture_codec(implementation: &'static str) -> CodecRegistration {
        CodecRegistration::new(&FIXTURE_METADATA)
            .with_implementation(implementation)
            .with_decoder(|_| Box::new(OtlpBestEffortDecoder))
            .with_item_counter(|_, bytes| {
                LogsData::decode(bytes).ok().map(|logs| {
                    logs.resource_logs
                        .iter()
                        .flat_map(|resource| &resource.scope_logs)
                        .map(|scope| scope.log_records.len())
                        .sum()
                })
            })
    }

    const fn bound_fixture_codec() -> CodecRegistration {
        binding_fixture_codec("bound").with_generator(CodecGeneratorRegistration::new(
            &[GenerationProfile {
                name: "bound-only",
                signals: &[SignalType::Logs],
            }],
            |codec, _| {
                BOUND_GENERATORS_CREATED.with(|created| created.set(created.get() + 1));
                Ok(Box::new(FixtureGenerator { codec, sequence: 0 }))
            },
        ))
    }

    struct FixtureGenerator {
        codec: ResolvedCodec,
        sequence: u64,
    }

    impl PdataGenerator for FixtureGenerator {
        fn generate(
            &mut self,
            count: usize,
            timestamp_unix_nano: i64,
        ) -> Result<OtapPayload, GenerationError> {
            let log_records = (0..count)
                .map(|_| {
                    let record = LogRecord {
                        time_unix_nano: timestamp_unix_nano as u64,
                        observed_time_unix_nano: timestamp_unix_nano as u64,
                        body: Some(AnyValue::new_string(self.sequence.to_string())),
                        ..Default::default()
                    };
                    self.sequence = self.sequence.wrapping_add(1);
                    record
                })
                .collect();
            let logs = LogsData {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            Ok(self
                .codec
                .admit(SignalType::Logs, logs.encode_to_vec().into())?
                .into())
        }
    }

    fn codec_config_value() -> serde_json::Value {
        serde_json::json!({
            "data_source": "codec",
            "codec": {
                "encoding": "trafficgen-test",
                "profile": "default",
                "timestamp_unix_nano": 42
            },
            "traffic_config": {
                "signals_per_second": 2,
                "max_batch_size": 2,
                "log_weight": 1
            }
        })
    }

    fn decode_codec_logs(
        payload: OtapPayload,
        service: &otel_arrow_dfe_pdata_codec::CodecService,
    ) -> LogsData {
        let otlp: OtlpProtoBytes = payload
            .try_into_otap(service)
            .unwrap()
            .try_into_with_default()
            .unwrap();
        let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp else {
            panic!("expected logs");
        };
        LogsData::decode(bytes).unwrap()
    }

    /// Scenario: An explicit codec binding overrides a distribution default or resolves an ambiguous encoding.
    /// Guarantees: Config validation creates no generators; both strategies construct the bound implementation once and retain it for generation.
    #[test]
    fn codec_generation_honors_pipeline_bindings_without_reinitializing() {
        static WITH_DEFAULT: [CodecRegistration; 2] = [
            binding_fixture_codec("default").with_default_for_encoding(),
            bound_fixture_codec(),
        ];
        static WITHOUT_DEFAULT: [CodecRegistration; 2] =
            [binding_fixture_codec("default"), bound_fixture_codec()];
        let bindings: CodecsConfig = serde_json::from_value(serde_json::json!({
            "bindings": {"trafficgen-test": {"implementation": "bound"}}
        }))
        .unwrap();
        for (registrations, has_default) in [(&WITH_DEFAULT, true), (&WITHOUT_DEFAULT, false)] {
            let registry = CodecRegistry::validate(registrations).unwrap();
            if has_default {
                assert_eq!(
                    registry
                        .resolve(FIXTURE_METADATA.encoding())
                        .unwrap()
                        .implementation(),
                    "default"
                );
            } else {
                assert!(matches!(
                    registry.resolve(FIXTURE_METADATA.encoding()),
                    Err(RegistryError::AmbiguousImplementation { .. })
                ));
            }
            let service = CodecServiceBuilder::from_registry(Arc::new(registry))
                .with_configuration(CodecsConfig::resolve([&bindings]))
                .build()
                .unwrap();
            for strategy in ["fresh", "pre_generated"] {
                BOUND_GENERATORS_CREATED.with(|created| created.set(0));
                let mut value = codec_config_value();
                value["codec"]["profile"] = "bound-only".into();
                value["generation_strategy"] = strategy.into();
                super::super::validate_config(&value).unwrap();
                BOUND_GENERATORS_CREATED.with(|created| assert_eq!(created.get(), 0));
                let config: Config = serde_json::from_value(value).unwrap();
                let mut producer =
                    TrafficProducer::from_config(&config, service.registry()).unwrap();
                BOUND_GENERATORS_CREATED.with(|created| assert_eq!(created.get(), 1));
                for _ in 0..2 {
                    let payload = producer
                        .next_run()
                        .unwrap()
                        .unwrap()
                        .next()
                        .unwrap()
                        .unwrap();
                    assert_eq!(payload.format().codec().unwrap().implementation(), "bound");
                    let logs = decode_codec_logs(payload, &service);
                    assert_eq!(logs.resource_logs[0].scope_logs[0].log_records.len(), 2);
                    producer.record_production(2);
                }
                BOUND_GENERATORS_CREATED.with(|created| assert_eq!(created.get(), 1));
            }
        }
    }

    /// Scenario: Codec-backed trafficgen uses a controlled per-batch clock or a fixed override.
    /// Guarantees: Default timestamps refresh per batch, fixed overrides bypass the clock, and clock errors propagate.
    #[test]
    fn codec_generation_selects_current_or_fixed_timestamps() {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        for fixed in [None, Some(7_i64)] {
            let mut value = codec_config_value();
            value["codec"]["timestamp_unix_nano"] = serde_json::json!(fixed);
            let config: Config = serde_json::from_value(value).unwrap();
            let mut generator = CodecSignalGenerator {
                generators: config.codec_generators(service.registry()).unwrap(),
                timestamp_unix_nano: config.codec_timestamp_unix_nano(),
            };
            for now in [42, 43] {
                let payload = generator
                    .generate_with_clock(SignalType::Logs, 2, || {
                        assert!(fixed.is_none(), "fixed timestamps must not read the clock");
                        Ok(now)
                    })
                    .unwrap();
                let logs = decode_codec_logs(payload, &service);
                for record in &logs.resource_logs[0].scope_logs[0].log_records {
                    assert_eq!(record.observed_time_unix_nano, fixed.unwrap_or(now) as u64);
                }
            }
            if fixed.is_none() {
                assert!(matches!(
                    generator.generate_with_clock(SignalType::Logs, 2, || {
                        Err(GenerationError::Output("clock failed".into()))
                    }),
                    Err(error) if error.to_string().contains("clock failed")
                ));
            }
        }
    }

    /// Scenario: Codec-backed trafficgen runs both generation strategies using a test-only codec.
    /// Guarantees: Payloads retain the selected encoding, decode to consecutive records of the requested count, and replay only in pre-generated mode.
    #[test]
    fn codec_generation_integrates_with_trafficgen_strategies() {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        for strategy in ["fresh", "pre_generated"] {
            let mut value = codec_config_value();
            value["generation_strategy"] = strategy.into();
            let config: Config = serde_json::from_value(value).unwrap();
            let mut producer = TrafficProducer::from_config(&config, service.registry()).unwrap();
            let mut first = producer
                .next_run()
                .unwrap()
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(first.num_items(), 2);
            assert_eq!(first.encoding().unwrap(), FIXTURE_METADATA.encoding());
            let first_bytes = first.encoded_bytes().unwrap().clone();
            let logs = decode_codec_logs(first, &service);
            let records = &logs.resource_logs[0].scope_logs[0].log_records;
            assert_eq!(records.len(), 2);
            let Some(AnyValue {
                value: Some(AnyValueValue::StringValue(first)),
            }) = &records[0].body
            else {
                panic!("expected a generated sequence");
            };
            let sequence = first.parse::<u64>().unwrap();
            assert_eq!(
                records[1].body,
                Some(AnyValue::new_string(sequence.wrapping_add(1).to_string()))
            );
            producer.record_production(2);
            let second = producer
                .next_run()
                .unwrap()
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(
                first_bytes == *second.encoded_bytes().unwrap(),
                strategy == "pre_generated"
            );
        }
    }

    /// Scenario: Codec-backed trafficgen explicitly configures a fixed timestamp in either strategy.
    /// Guarantees: The configured override reaches decoded records unchanged across successive runs.
    #[test]
    fn codec_generation_preserves_fixed_timestamps() {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        let timestamp = 1_704_067_200_123_456_000_u64;
        for strategy in ["fresh", "pre_generated"] {
            let mut value = codec_config_value();
            value["codec"]["timestamp_unix_nano"] = timestamp.into();
            value["generation_strategy"] = strategy.into();
            let config: Config = serde_json::from_value(value).unwrap();
            let mut producer = TrafficProducer::from_config(&config, service.registry()).unwrap();
            for _ in 0..2 {
                let payload = producer
                    .next_run()
                    .unwrap()
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap();
                let logs = decode_codec_logs(payload, &service);
                let records = &logs.resource_logs[0].scope_logs[0].log_records;
                assert_eq!(records.len(), 2);
                for record in records {
                    assert_eq!(record.observed_time_unix_nano, timestamp);
                    assert_eq!(record.time_unix_nano, timestamp);
                }
                producer.record_production(2);
            }
        }
    }

    /// Scenario: A codec-backed producer reaches its signal limit partway through a batch.
    /// Guarantees: Both strategies emit a valid partial final batch and stop at exactly the configured total.
    #[test]
    fn codec_generation_honors_partial_final_batch() {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        for strategy in ["fresh", "pre_generated"] {
            let mut value = codec_config_value();
            value["generation_strategy"] = strategy.into();
            value["traffic_config"]["max_signal_count"] = 3.into();
            let config: Config = serde_json::from_value(value).unwrap();
            let mut producer = TrafficProducer::from_config(&config, service.registry()).unwrap();
            let mut counts = Vec::new();
            while let Some(run) = producer.next_run().unwrap() {
                let count = run
                    .map(|payload| {
                        payload
                            .unwrap()
                            .try_into_otap(&service)
                            .unwrap()
                            .num_items() as u64
                    })
                    .sum();
                counts.push(count);
                producer.record_production(count);
            }
            assert_eq!(counts, [2, 1]);
        }
    }

    /// Scenario: Trafficgen is configured with missing codec settings, invalid limits, or synthetic-only options.
    /// Guarantees: Structural errors are rejected at config load time without requiring a selected codec registry.
    #[test]
    fn codec_generation_validates_trafficgen_configuration() {
        let changes = [
            ("codec", serde_json::Value::Null, "requires `codec`"),
            (
                "codec",
                serde_json::json!({
                    "encoding": "trafficgen-test",
                    "profile": "default",
                    "timestamp_unix_nano": -1
                }),
                "timestamp_unix_nano must not be negative",
            ),
            (
                "codec",
                serde_json::json!({
                    "encoding": "trafficgen-test",
                    "profile": "default",
                    "max_batch_bytes": 0
                }),
                "max_batch_bytes must be positive",
            ),
            ("data_source", "synthetic".into(), "require data_source"),
            (
                "resource_attributes",
                serde_json::json!({"tenant.id": "test"}),
                "does not support synthetic",
            ),
            (
                "traffic_config",
                serde_json::json!({"log_weight": 1, "log_body_size_bytes": 100}),
                "does not support synthetic",
            ),
        ];
        for (field, replacement, expected) in changes {
            let mut value = codec_config_value();
            value[field] = replacement;
            let error = super::super::validate_config(&value)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{error}");
        }
    }

    /// Scenario: Structurally valid codec settings request unsupported encodings, profiles, or signals.
    /// Guarantees: Implementation-specific errors are deferred until producer setup and propagate before any traffic is emitted.
    #[test]
    fn codec_generation_validates_selected_implementation() {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        let changes = [
            (
                "codec",
                serde_json::json!({"encoding": "missing", "profile": "default"}),
                "no pdata codec",
            ),
            (
                "codec",
                serde_json::json!({"encoding": "otlp-bytes", "profile": "default"}),
                "no synthetic generator",
            ),
            (
                "codec",
                serde_json::json!({"encoding": "trafficgen-test", "profile": "missing"}),
                "no profile",
            ),
            (
                "traffic_config",
                serde_json::json!({"log_weight": 0, "metric_weight": 1, "max_batch_size": 2}),
                "cannot decode Metrics",
            ),
        ];
        for (field, replacement, expected) in changes {
            let mut value = codec_config_value();
            value[field] = replacement;
            super::super::validate_config(&value).unwrap();
            let config: Config = serde_json::from_value(value).unwrap();
            let error = match TrafficProducer::from_config(&config, service.registry()) {
                Ok(_) => panic!("invalid configuration was accepted: {field}"),
                Err(error) => error.to_string(),
            };
            assert!(error.contains(expected), "{error}");
        }
    }

    fn into_otlp(payload: OtapPayload) -> OtlpProtoBytes {
        TryIntoWithOptions::try_into_with_default(payload).expect("OTLP conversion")
    }

    /// Helper to build a minimal `SyntheticGenerator` with no resource attributes.
    fn synthetic_generator() -> SyntheticGenerator {
        SyntheticGenerator {
            idx: 0,
            entries: Vec::new(),
            rotation: Vec::new(),
            log_body_size_bytes: None,
            num_log_attributes: None,
            use_trace_context: false,
            num_metric_attributes: None,
            num_data_points_per_metric: None,
        }
    }

    /// Helper to build a `TrafficProducer` with a Fresh strategy from a config.
    fn fresh_producer(cfg: &TrafficConfig, max_signal_count: Option<u64>) -> TrafficProducer {
        let shape = create_shape(cfg);
        let mut strategy = ProductionStrategy::Fresh { shape };
        let signal_count = strategy.signal_count();
        TrafficProducer {
            strategy,
            signal_count,
            generator: Box::new(synthetic_generator()),
            max_signal_count,
            produced_count: 0,
        }
    }

    #[test]
    fn test_create_shape_distributes_signals() {
        let cfg = TrafficConfig::new(
            Some(100), // signals_per_second
            None,      // max_signal_count
            50,        // max_batch_size
            1,         // metric_weight
            2,         // trace_weight
            1,         // log_weight
        );

        let shape = create_shape(&cfg);

        // Every entry must respect max_batch_size.
        for (_, count) in &shape {
            assert!(*count <= 50, "batch size {count} exceeds max_batch_size 50");
        }

        // Total signals across all entries should equal signals_per_second.
        let total: usize = shape.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 100, "total signals should equal signals_per_second");

        // All three signal types must be represented.
        let has_logs = shape.iter().any(|(s, _)| *s == SignalType::Logs);
        let has_metrics = shape.iter().any(|(s, _)| *s == SignalType::Metrics);
        let has_traces = shape.iter().any(|(s, _)| *s == SignalType::Traces);
        assert!(has_logs, "shape should contain Logs");
        assert!(has_metrics, "shape should contain Metrics");
        assert!(has_traces, "shape should contain Traces");

        // Traces have weight 2 (50%), metrics and logs each have weight 1 (25%).
        // Verify traces get ~50 signals and the others get ~25 each.
        let trace_total: usize = shape
            .iter()
            .filter(|(s, _)| *s == SignalType::Traces)
            .map(|(_, c)| c)
            .sum();
        assert!(
            (48..=52).contains(&trace_total),
            "traces should get ~50 signals, got {trace_total}"
        );
    }

    #[test]
    fn test_create_shape_uncapped_produces_large_shape() {
        let cfg = TrafficConfig::new(
            None, // signals_per_second = None means uncapped
            None, // max_signal_count
            512,  // max_batch_size
            0,    // metric_weight
            0,    // trace_weight
            100,  // log_weight
        );

        let shape = create_shape(&cfg);

        // With uncapped mode, the shape should contain many batches (1024 * max_batch_size / max_batch_size = 1024).
        assert_eq!(shape.len(), 1024, "uncapped shape should have 1024 batches");

        let total: usize = shape.iter().map(|(_, c)| c).sum();
        assert_eq!(
            total,
            512 * 1024,
            "uncapped shape total signals should be max_batch_size * 1024"
        );

        // Every entry should be max_batch_size.
        for (_, count) in &shape {
            assert_eq!(*count, 512, "each batch should be max_batch_size");
        }
    }

    #[test]
    fn test_traffic_producer_fresh_yields_correct_count() {
        let cfg = TrafficConfig::new(
            Some(30), // signals_per_second
            None,     // max_signal_count
            10,       // max_batch_size
            1,        // metric_weight
            1,        // trace_weight
            1,        // log_weight
        );

        let shape = create_shape(&cfg);
        let expected_batches = shape.len();
        let mut strategy = ProductionStrategy::Fresh { shape };
        let signal_count = strategy.signal_count();
        let mut producer = TrafficProducer {
            strategy,
            signal_count,
            generator: Box::new(synthetic_generator()),
            max_signal_count: None,
            produced_count: 0,
        };

        let run = producer
            .next_run()
            .expect("generation should succeed")
            .expect("should get a run");

        // ExactSizeIterator should report the right length up front.
        assert_eq!(
            run.len(),
            expected_batches,
            "ExactSizeIterator::len should match shape length"
        );

        let results: Vec<_> = run.collect();
        assert_eq!(
            results.len(),
            expected_batches,
            "iterator should yield exactly shape.len() items"
        );

        // Every result should be Ok.
        for (i, r) in results.iter().enumerate() {
            assert!(r.is_ok(), "payload {i} should be Ok, got {:?}", r);
        }
    }

    #[test]
    fn test_traffic_producer_replay_clones_payloads() {
        let cfg = TrafficConfig::new(
            Some(15), // signals_per_second
            None,     // max_signal_count
            10,       // max_batch_size
            1,        // metric_weight
            1,        // trace_weight
            1,        // log_weight
        );

        let shape = create_shape(&cfg);
        let mut generator = synthetic_generator();
        let mut payloads =
            create_fresh_payloads(&mut generator, &shape).expect("pre-generation should succeed");
        let expected_count = payloads.len();

        let mut strategy = ProductionStrategy::Replay {
            payloads: payloads.clone(),
        };
        let signal_count = strategy.signal_count();
        let mut producer = TrafficProducer {
            strategy,
            signal_count,
            generator: Box::new(synthetic_generator()),
            max_signal_count: None,
            produced_count: 0,
        };

        let mut results: Vec<GenerateResult> = producer
            .next_run()
            .expect("generation should succeed")
            .expect("should get a run")
            .collect();
        assert_eq!(results.len(), expected_count);

        // Each replayed payload should match the original pre-generated payload.
        for (i, (result, original)) in results.iter_mut().zip(payloads.iter_mut()).enumerate() {
            let payload = result.as_mut().expect("replay should always succeed");
            assert_eq!(
                payload.num_bytes(),
                original.num_bytes(),
                "payload {i} size should match the pre-generated original"
            );
        }
    }

    #[test]
    fn test_resource_attribute_rotation_across_batches() {
        use super::super::config::{ResourceAttributeSet, build_rotation_table};
        use otel_arrow_dfe_pdata::OtlpProtoBytes;
        use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
        use prost::Message;
        use std::num::NonZeroU32;

        let make_entry = |tenant: &str| ResourceAttributeSet {
            attrs: HashMap::from([("tenant.id".to_string(), tenant.to_string())]),
            weight: NonZeroU32::new(1).unwrap(),
        };
        let entries = vec![make_entry("prod"), make_entry("staging")];
        let rotation = build_rotation_table(&entries);

        let mut generator = SyntheticGenerator {
            idx: 0,
            entries,
            rotation,
            log_body_size_bytes: None,
            num_log_attributes: None,
            use_trace_context: false,
            num_metric_attributes: None,
            num_data_points_per_metric: None,
        };

        // Helper: extract the tenant.id attribute value from a log payload.
        let extract_tenant_id = |payload: OtapPayload| -> Option<String> {
            let otlp_bytes = into_otlp(payload);
            let bytes = match otlp_bytes {
                OtlpProtoBytes::ExportLogsRequest(b) => b,
                _ => panic!("expected logs"),
            };
            let logs = LogsData::decode(bytes.as_ref()).expect("decode logs");
            logs.resource_logs
                .first()?
                .resource
                .as_ref()?
                .attributes
                .iter()
                .find(|kv| kv.key == "tenant.id")
                .and_then(|kv| kv.value.as_ref())
                .and_then(|v| {
                    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value;
                    match v.value.as_ref()? {
                        Value::StringValue(s) => Some(s.clone()),
                        _ => None,
                    }
                })
        };

        let tenant_1 = extract_tenant_id(generator.generate_logs(1).expect("batch 1"))
            .expect("batch 1 should have tenant.id");
        let tenant_2 = extract_tenant_id(generator.generate_logs(1).expect("batch 2"))
            .expect("batch 2 should have tenant.id");
        let tenant_3 = extract_tenant_id(generator.generate_logs(1).expect("batch 3"))
            .expect("batch 3 should have tenant.id");
        let tenant_4 = extract_tenant_id(generator.generate_logs(1).expect("batch 4"))
            .expect("batch 4 should have tenant.id");

        // With a two-entry rotation [0, 1], consecutive batches alternate attribute sets.
        assert_ne!(
            tenant_1, tenant_2,
            "consecutive batches should use different attribute sets"
        );
        assert_eq!(
            tenant_1, tenant_3,
            "batches 1 and 3 should share the same attribute set"
        );
        assert_eq!(
            tenant_2, tenant_4,
            "batches 2 and 4 should share the same attribute set"
        );

        // Both configured values must appear.
        let tenants = [&tenant_1, &tenant_2];
        assert!(
            tenants.contains(&&"prod".to_string()),
            "prod should appear in the rotation"
        );
        assert!(
            tenants.contains(&&"staging".to_string()),
            "staging should appear in the rotation"
        );
    }

    #[test]
    fn test_resource_attribute_rotation_empty_attrs() {
        use otel_arrow_dfe_pdata::OtlpProtoBytes;
        use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
        use prost::Message;

        // synthetic_generator() has empty entries and rotation -- no custom resource attrs.
        let mut generator = synthetic_generator();

        let mut batch_1 = generator
            .generate_logs(1)
            .expect("batch 1 with empty attrs");
        let mut batch_2 = generator
            .generate_logs(1)
            .expect("batch 2 with empty attrs");

        // Both payloads should be valid and non-empty.
        assert!(
            batch_1.num_bytes().unwrap_or(0) > 0,
            "batch 1 should be non-empty"
        );
        assert!(
            batch_2.num_bytes().unwrap_or(0) > 0,
            "batch 2 should be non-empty"
        );

        // Neither payload should carry a tenant.id attribute.
        for (i, batch) in [batch_1, batch_2].into_iter().enumerate() {
            let otlp_bytes = into_otlp(batch);
            let bytes = match otlp_bytes {
                OtlpProtoBytes::ExportLogsRequest(b) => b,
                _ => panic!("expected logs"),
            };
            let logs = LogsData::decode(bytes.as_ref()).expect("decode logs");
            let has_tenant = logs
                .resource_logs
                .first()
                .and_then(|rl| rl.resource.as_ref())
                .map(|r| r.attributes.iter().any(|kv| kv.key == "tenant.id"))
                .unwrap_or(false);
            assert!(
                !has_tenant,
                "batch {i} should not have tenant.id when no attrs configured"
            );
        }
    }

    #[test]
    fn test_producer_no_limit_always_returns_run() {
        let cfg = TrafficConfig::new(Some(10), None, 10, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, None);

        // Without a limit, next_run should always return Some.
        assert!(producer.next_run().unwrap().is_some());
        producer.record_production(100);
        assert!(producer.next_run().unwrap().is_some());
        producer.record_production(1_000_000);
        assert!(producer.next_run().unwrap().is_some());
    }

    #[test]
    fn test_producer_limit_reached_returns_none() {
        let cfg = TrafficConfig::new(Some(10), None, 10, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, Some(10));

        // First run should be available (produced 0 of 10).
        assert!(producer.next_run().unwrap().is_some());

        // After producing exactly the limit, next_run returns None.
        producer.record_production(10);
        assert!(producer.next_run().unwrap().is_none());

        // Over the limit also returns None.
        producer.record_production(5);
        assert!(producer.next_run().unwrap().is_none());
    }

    #[test]
    fn test_producer_truncates_run_to_fit_limit() {
        // Shape: 10 metrics per second, max_batch=5 -> shape is 2 batches of 5
        let cfg = TrafficConfig::new(Some(10), None, 5, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, Some(7));

        assert_eq!(producer.run_count(), 10, "full run should have 10 signals");

        // First run should be truncated to 7 signals.
        let run = producer
            .next_run()
            .expect("generation should succeed")
            .expect("should get a run");
        let mut payloads: Vec<_> = run.collect();
        let total: usize = payloads
            .iter_mut()
            .map(|r| r.as_mut().unwrap().num_items())
            .sum();
        assert_eq!(total, 7, "truncated run should have exactly 7 signals");

        // After producing 7, no more runs.
        producer.record_production(7);
        assert!(producer.next_run().unwrap().is_none());
    }

    #[test]
    fn test_producer_zero_limit_returns_none_immediately() {
        let cfg = TrafficConfig::new(Some(10), None, 10, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, Some(0));

        assert!(
            producer.next_run().unwrap().is_none(),
            "zero limit should immediately return None"
        );
    }

    #[test]
    fn test_cap_strategy_exact_fit() {
        // 2 batches of 5 metrics = 10 total. Cap to 10 should be a no-op.
        let cfg = TrafficConfig::new(Some(10), None, 5, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, Some(10));

        let run = producer
            .next_run()
            .expect("generation should succeed")
            .expect("should get a run");
        let total: u64 = run.map(|r| r.unwrap().num_items() as u64).sum();
        assert_eq!(total, 10);
    }

    #[test]
    fn test_cap_strategy_partial_batch() {
        // 2 batches of 5 metrics = 10 total. Cap to 7: keep first batch (5)
        // + partial second batch (2).
        let cfg = TrafficConfig::new(Some(10), None, 5, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, Some(7));

        let run = producer
            .next_run()
            .expect("generation should succeed")
            .expect("should get a run");
        let items: Vec<usize> = run.map(|r| r.unwrap().num_items()).collect();
        assert_eq!(items, vec![5, 2]);
    }

    #[test]
    fn test_cap_strategy_zero_budget() {
        // Cap to 0 signals should return None (checked in next_run before cap).
        let cfg = TrafficConfig::new(Some(10), None, 5, 1, 0, 0);
        let mut producer = fresh_producer(&cfg, Some(0));

        assert!(producer.next_run().unwrap().is_none());
    }
}
