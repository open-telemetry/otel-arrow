path = 'rust/otap-dataflow/crates/core-nodes/src/exporters/topic_exporter/mod.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

bad_register = """    pub fn register(pipeline_ctx: &PipelineContext, topic_name: &str) -> Self {
        Self {
            dropped: TopicExporterDroppedMetrics::register_with_attributes(pipeline_ctx, vec![otel_arrow_dfe_telemetry::instrument::KeyValue::new("topic", topic_name.to_owned())]),
            responses: TopicExporterResponseMetrics::register_with_attributes(pipeline_ctx, vec![otel_arrow_dfe_telemetry::instrument::KeyValue::new("topic", topic_name.to_owned())]),
            other: pipeline_ctx.register_metric_set_with_attributes::<TopicExporterOtherMetrics>(vec![otel_arrow_dfe_telemetry::instrument::KeyValue::new("topic", topic_name.to_owned())]),
        }
    }"""

good_register = """    pub fn register(pipeline_ctx: &PipelineContext, topic_name: &str) -> Self {
        Self {
            dropped: pipeline_ctx.register_measurement_metrics_with_topic::<TopicExporterDroppedMetrics>(topic_name.to_owned().into()),
            responses: pipeline_ctx.register_measurement_metrics_with_topic::<TopicExporterResponseMetrics>(topic_name.to_owned().into()),
            other: pipeline_ctx.register_metrics_with_topic::<TopicExporterOtherMetrics>(topic_name.to_owned().into()),
        }
    }"""

text = text.replace(bad_register, good_register)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
