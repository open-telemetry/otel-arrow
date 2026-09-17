import re

path = 'rust/otap-dataflow/crates/core-nodes/src/exporters/topic_exporter/mod.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

pattern = r'/// Telemetry metrics for the topic exporter\.\n#\[metric_set\(name = "exporter\.topic"\)\]\n#\[derive\(Debug, Default, Clone\)\]\npub struct TopicExporterMetrics \{.*?\n\}\n'
old_struct_match = re.search(pattern, text, re.DOTALL)
if old_struct_match:
    text = text.replace(old_struct_match.group(0), "")

new_metrics = """
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetRegistrar, MetricSetSnapshot};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set};

// -- Drop reason attributes ---------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum DropReason {
    QueueFull,
    OutcomeCapacity,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct DropAttributes {
    pub reason: DropReason,
}

#[metric_set(
    name = "exporter.topic.dropped",
    measurement_attributes = DropAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TopicExporterDroppedMetrics {
    #[metric(unit = "{item}")]
    pub messages: Counter<u64>,
}

// -- End-to-end response attributes -------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ResponseType {
    Ack,
    Nack,
    ShutdownNack,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ResponseAttributes {
    pub response_type: ResponseType,
}

#[metric_set(
    name = "exporter.topic.end_to_end_responses",
    measurement_attributes = ResponseAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TopicExporterResponseMetrics {
    #[metric(unit = "{item}")]
    pub responses: Counter<u64>,
}

// -- Other (non-dimensionable) metrics ----------------------------------------

#[metric_set(name = "exporter.topic.other")]
#[derive(Debug, Default, Clone)]
pub struct TopicExporterOtherMetrics {
    #[metric(unit = "{item}")]
    pub published_messages: Counter<u64>,
    #[metric(unit = "{item}")]
    pub tracked_in_flight: Gauge<u64>,
    #[metric(unit = "{item}")]
    pub outcome_timeouts: Counter<u64>,
}

// -- Top-level wrapper ---------------------------------------------------------

pub struct TopicExporterMetrics {
    pub dropped: MeasurementMetricSet<TopicExporterDroppedMetrics>,
    pub responses: MeasurementMetricSet<TopicExporterResponseMetrics>,
    pub other: MetricSet<TopicExporterOtherMetrics>,
}

impl TopicExporterMetrics {
    pub fn register(pipeline_ctx: &PipelineContext, topic_name: &str) -> Self {
        Self {
            dropped: TopicExporterDroppedMetrics::register_with_attributes(pipeline_ctx, vec![otel_arrow_dfe_telemetry::instrument::KeyValue::new("topic", topic_name.to_owned())]),
            responses: TopicExporterResponseMetrics::register_with_attributes(pipeline_ctx, vec![otel_arrow_dfe_telemetry::instrument::KeyValue::new("topic", topic_name.to_owned())]),
            other: pipeline_ctx.register_metric_set_with_attributes::<TopicExporterOtherMetrics>(vec![otel_arrow_dfe_telemetry::instrument::KeyValue::new("topic", topic_name.to_owned())]),
        }
    }

    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.dropped.terminal_snapshots();
        snapshots.extend(self.responses.terminal_snapshots());
        snapshots.extend(self.other.terminal_snapshots());
        snapshots
    }

    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        reporter.report_measurement(&mut self.dropped)?;
        reporter.report_measurement(&mut self.responses)?;
        reporter.report(&mut self.other)?;
        Ok(())
    }
}
"""
text = text.replace('pub const TOPIC_EXPORTER_URN: &str = "urn:otel:exporter:topic";', 
                    'pub const TOPIC_EXPORTER_URN: &str = "urn:otel:exporter:topic";\n' + new_metrics)

text = re.sub(r'metrics: MetricSet<TopicExporterMetrics>', 'metrics: TopicExporterMetrics', text)
text = re.sub(r'metrics: &mut MetricSet<TopicExporterMetrics>', 'metrics: &mut TopicExporterMetrics', text)
text = re.sub(r'_ = metrics_reporter\.report\(&mut metrics\);', '_ = metrics.report(&mut metrics_reporter);', text)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
