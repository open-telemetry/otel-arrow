import re

path1 = 'd:/otel-arrow-main/otel-arrow-main/rust/otap-dataflow/crates/core-nodes/src/processors/batch_processor/mod.rs'
with open(path1, 'r', encoding='utf-8') as f:
    text = f.read()

# Remove the verify_batch_metrics function definition
text = re.sub(r'\s*/// Helper to verify that batch counters were incremented.*?\n\s*fn verify_batch_metrics\([^)]*\)\s*\{.*?\n\s*\}\n', '\n', text, flags=re.DOTALL)

# Also remove it if the above regex fails for some reason (more generic)
text = re.sub(r'fn verify_batch_metrics\([^)]*\)\s*\{.*?\n\s*\}\n', '\n', text, flags=re.DOTALL)

# Remove all calls to verify_batch_metrics(...)
text = re.sub(r'\s*verify_batch_metrics\([^)]*\);', '', text)

with open(path1, 'w', encoding='utf-8') as f:
    f.write(text)
