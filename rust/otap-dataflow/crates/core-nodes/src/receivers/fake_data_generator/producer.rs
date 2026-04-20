use std::collections::HashMap;

use otap_df_config::SignalType;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::proto::OtlpProtoMessage;
use prost::EncodeError;
use weaver_forge::registry::ResolvedRegistry;

use super::config::{
    Config, DataSource, GenerationStrategy, ResourceAttributeSet, TrafficConfig,
    build_rotation_table,
};
use super::{semconv_signal, static_signal};

/// A SignalProducer yields the signals that need to be exported in a given
/// second in order to meet the data volume production requirements with a
/// reasonably smooth curve.
pub struct TrafficProducer {
    strategy: ProductionStrategy,
    generator: Box<dyn SignalGenerator>,
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
    pub fn from_config(config: &Config) -> Result<Self, GenerateError> {
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
            DataSource::Static => {
                let entries = config.resource_attributes().to_vec();
                let rotation = build_rotation_table(&entries);
                Box::new(StaticGenerator {
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
        };

        // Build the production strategy.
        let shape = create_shape(traffic_config);
        let strategy = match config.generation_strategy() {
            GenerationStrategy::Fresh => ProductionStrategy::Fresh(shape),
            GenerationStrategy::PreGenerated => {
                let payloads = create_fresh_payloads(&mut *generator, &shape)?;
                ProductionStrategy::Replay(payloads)
            }
        };

        Ok(Self {
            strategy,
            generator,
        })
    }

    /// The number of batches in a run
    #[must_use]
    pub fn run_len(&self) -> usize {
        self.strategy.len()
    }

    /// The total number of signals in a run
    #[must_use]
    pub fn run_count(&self) -> usize {
        self.strategy.signal_count()
    }

    /// Return a [`TrafficRun`] iterator that yields the payloads needed for
    /// one second of traffic.
    pub fn next_run(&mut self) -> TrafficRun<'_> {
        TrafficRun {
            generator: &mut *self.generator,
            strategy: &self.strategy,
            idx: 0,
        }
    }
}

/// A traffic run is a fixed size iterator of payloads that need to be delivered
/// in the current second. Consumers of this are expected to process every item
/// in the iterator within a second and grab a new iterator for the next second.
pub struct TrafficRun<'a> {
    generator: &'a mut dyn SignalGenerator,
    strategy: &'a ProductionStrategy,
    idx: usize,
}

impl<'a> Iterator for TrafficRun<'a> {
    type Item = GenerateResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            return None;
        }

        let next = match self.strategy {
            ProductionStrategy::Fresh(v) => {
                let shape = v[self.idx];
                match shape.0 {
                    SignalType::Traces => self.generator.generate_traces(shape.1),
                    SignalType::Metrics => self.generator.generate_metrics(shape.1),
                    SignalType::Logs => self.generator.generate_logs(shape.1),
                }
            }
            ProductionStrategy::Replay(v) => Ok(v[self.idx].clone()),
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

    let signals_per_second = cfg.signals_per_second.unwrap_or(cfg.max_batch_size) as u32;
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
    Fresh(Vec<(SignalType, usize)>),
    Replay(Vec<OtapPayload>),
}

impl ProductionStrategy {
    fn len(&self) -> usize {
        match self {
            ProductionStrategy::Fresh(v) => v.len(),
            ProductionStrategy::Replay(v) => v.len(),
        }
    }

    fn signal_count(&self) -> usize {
        match self {
            ProductionStrategy::Fresh(v) => v.iter().map(|x| x.1).sum(),
            ProductionStrategy::Replay(v) => v.iter().map(|x| x.num_items()).sum(),
        }
    }
}

/// Result type for signal generation operations.
pub type GenerateResult = Result<OtapPayload, GenerateError>;

/// Errors that can occur during signal generation.
#[derive(thiserror::Error, Debug)]
pub enum GenerateError {
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
/// into [`OtapPayload`] values. In practice encoding is infallible because
/// the only failure mode is insufficient buffer capacity, and `BytesMut` can
/// grow up to `isize::MAX`.
pub trait SignalGenerator {
    /// Generate a log payload containing `count` log records.
    fn generate_logs(&mut self, count: usize) -> GenerateResult;
    /// Generate a metrics payload containing `count` data points.
    fn generate_metrics(&mut self, count: usize) -> GenerateResult;
    /// Generate a traces payload containing `count` spans.
    fn generate_traces(&mut self, count: usize) -> GenerateResult;
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

/// Signal generator that produces signals from hardcoded static templates.
///
/// Resource attributes are rotated across batches according to a weighted
/// round-robin table built from the configured [`ResourceAttributeSet`] entries.
pub struct StaticGenerator {
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

impl StaticGenerator {
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
                // self.idx += 1;
                Some(&self.entries[slot].attrs)
            }
        }
    }
}

impl SignalGenerator for StaticGenerator {
    fn generate_logs(&mut self, count: usize) -> GenerateResult {
        self.idx += 1;

        let attrs = self.attrs_for_batch();
        let payload = static_signal::static_otlp_logs_with_config(
            count,
            self.log_body_size_bytes,
            self.num_log_attributes,
            self.use_trace_context,
            attrs,
        );
        let payload = OtlpProtoMessage::Logs(payload);

        Ok(payload.try_into()?)
    }

    fn generate_metrics(&mut self, count: usize) -> GenerateResult {
        self.idx += 1;

        let attrs = self.attrs_for_batch();
        let payload = static_signal::static_otlp_metrics_with_config(
            count,
            self.num_metric_attributes,
            self.num_data_points_per_metric,
            attrs,
        );
        let payload = OtlpProtoMessage::Metrics(payload);

        Ok(payload.try_into()?)
    }

    fn generate_traces(&mut self, count: usize) -> GenerateResult {
        self.idx += 1;

        let attrs = self.attrs_for_batch();
        let payload = static_signal::static_otlp_traces(count, attrs);
        let payload = OtlpProtoMessage::Traces(payload);

        Ok(payload.try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::TrafficConfig;
    use super::*;

    /// Helper to build a minimal `StaticGenerator` with no resource attributes.
    fn static_generator() -> StaticGenerator {
        StaticGenerator {
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
            trace_total >= 48 && trace_total <= 52,
            "traces should get ~50 signals, got {trace_total}"
        );
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

        let mut producer = TrafficProducer {
            strategy: ProductionStrategy::Fresh(shape),
            generator: Box::new(static_generator()),
        };

        let run = producer.next_run();

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
        let mut generator = static_generator();
        let payloads =
            create_fresh_payloads(&mut generator, &shape).expect("pre-generation should succeed");
        let expected_count = payloads.len();

        let mut producer = TrafficProducer {
            strategy: ProductionStrategy::Replay(payloads.clone()),
            generator: Box::new(static_generator()),
        };

        let results: Vec<GenerateResult> = producer.next_run().collect();
        assert_eq!(results.len(), expected_count);

        // Each replayed payload should match the original pre-generated payload.
        for (i, (result, original)) in results.iter().zip(payloads.iter()).enumerate() {
            let payload = result.as_ref().expect("replay should always succeed");
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
        use otap_df_pdata::OtlpProtoBytes;
        use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
        use prost::Message;
        use std::num::NonZeroU32;

        let make_entry = |tenant: &str| ResourceAttributeSet {
            attrs: HashMap::from([("tenant.id".to_string(), tenant.to_string())]),
            weight: NonZeroU32::new(1).unwrap(),
        };
        let entries = vec![make_entry("prod"), make_entry("staging")];
        let rotation = build_rotation_table(&entries);

        let mut generator = StaticGenerator {
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
            let otlp_bytes: OtlpProtoBytes = payload.try_into().expect("convert to otlp bytes");
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
                    use otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value;
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
        use otap_df_pdata::OtlpProtoBytes;
        use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
        use prost::Message;

        // static_generator() has empty entries and rotation — no custom resource attrs.
        let mut generator = static_generator();

        let batch_1 = generator
            .generate_logs(1)
            .expect("batch 1 with empty attrs");
        let batch_2 = generator
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
            let otlp_bytes: OtlpProtoBytes = batch.try_into().expect("convert to otlp bytes");
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
}
