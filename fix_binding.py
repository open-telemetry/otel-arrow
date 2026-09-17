path = 'rust/otap-dataflow/crates/core-nodes/src/exporters/topic_exporter/mod.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('TopicExporterMetrics::register(&pipeline, topic_binding.name())', 'TopicExporterMetrics::register(&pipeline, topic_binding.name().as_str())')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
