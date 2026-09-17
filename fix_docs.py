path = 'rust/otap-dataflow/crates/core-nodes/src/exporters/topic_exporter/mod.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

replacements = [
    ("pub enum DropReason", "/// Reason a message was dropped.\npub enum DropReason"),
    ("    QueueFull", "    /// The queue was full.\n    QueueFull"),
    ("    OutcomeCapacity", "    /// Outcome tracking capacity was exhausted.\n    OutcomeCapacity"),
    ("pub struct DropAttributes", "/// Attributes for dropped messages.\npub struct DropAttributes"),
    ("    pub reason: DropReason", "    /// The reason the message was dropped.\n    pub reason: DropReason"),
    ("pub struct TopicExporterDroppedMetrics", "/// Metrics for dropped messages.\npub struct TopicExporterDroppedMetrics"),
    ("    pub messages: Counter<u64>", "    /// Number of dropped messages.\n    pub messages: Counter<u64>"),
    
    ("pub enum ResponseType", "/// Type of end-to-end response bridged back to upstream.\npub enum ResponseType"),
    ("    Ack", "    /// Positive acknowledgement.\n    Ack"),
    ("    Nack", "    /// Negative acknowledgement.\n    Nack"),
    ("    ShutdownNack", "    /// Negative acknowledgement during shutdown.\n    ShutdownNack"),
    ("pub struct ResponseAttributes", "/// Attributes for end-to-end responses.\npub struct ResponseAttributes"),
    ("    pub response_type: ResponseType", "    /// The response type.\n    pub response_type: ResponseType"),
    ("pub struct TopicExporterResponseMetrics", "/// Metrics for end-to-end responses.\npub struct TopicExporterResponseMetrics"),
    ("    pub responses: Counter<u64>", "    /// Number of responses.\n    pub responses: Counter<u64>"),
    
    ("pub struct TopicExporterOtherMetrics", "/// Other metrics for topic exporter.\npub struct TopicExporterOtherMetrics"),
    ("    pub published_messages: Counter<u64>", "    /// Number of messages published.\n    pub published_messages: Counter<u64>"),
    ("    pub tracked_in_flight: Gauge<u64>", "    /// Current number of tracked publishes in flight.\n    pub tracked_in_flight: Gauge<u64>"),
    ("    pub outcome_timeouts: Counter<u64>", "    /// Number of publishes that timed out.\n    pub outcome_timeouts: Counter<u64>"),

    ("pub struct TopicExporterMetrics", "/// Container for topic exporter metrics.\npub struct TopicExporterMetrics"),
    ("    pub dropped: MeasurementMetricSet<TopicExporterDroppedMetrics>", "    /// Dropped metrics.\n    pub dropped: MeasurementMetricSet<TopicExporterDroppedMetrics>"),
    ("    pub responses: MeasurementMetricSet<TopicExporterResponseMetrics>", "    /// Response metrics.\n    pub responses: MeasurementMetricSet<TopicExporterResponseMetrics>"),
    ("    pub other: MetricSet<TopicExporterOtherMetrics>", "    /// Other scalar metrics.\n    pub other: MetricSet<TopicExporterOtherMetrics>"),

    ("    pub fn register(", "    /// Registers all metrics with the pipeline context.\n    pub fn register("),
    ("    pub fn terminal_snapshots(", "    /// Generates final snapshots of the metrics.\n    pub fn terminal_snapshots("),
    ("    pub fn report(", "    /// Reports modified metrics to the provided reporter.\n    pub fn report("),

    ("use otel_arrow_dfe_telemetry::metrics::{", "use otel_arrow_dfe_telemetry::metrics::{"),
    ("MeasurementMetricSet, MetricSetRegistrar, MetricSetSnapshot", "MeasurementMetricSet, MetricSetSnapshot"),
]

for old, new in replacements:
    text = text.replace(old, new)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
