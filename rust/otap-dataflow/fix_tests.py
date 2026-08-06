import sys

content = open('crates/engine/src/context.rs').read()

# Remove the incorrectly placed test block.
# We will find `    #[test]` and remove everything from there to the end, except the last `}`.
start_idx = content.find('    #[test]\n    fn register_node_channel_entity_includes_custom_attributes')
if start_idx == -1:
    print("Could not find test to remove.")
    sys.exit(1)

# The content before the test
before_test = content[:start_idx]

# We need to make sure we keep the closing brace for `impl ExtensionContext`.
# Since the file ended with `    }\n}\n`, we should just close the impl block.
before_test = before_test.rstrip() + '\n}\n\n'

new_test_module = '''#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use otap_df_config::pipeline::telemetry::AttributeValue;
    use crate::attributes::TelemetryAttribute;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_config::node::NodeUrn;

    #[test]
    fn register_node_channel_entity_includes_custom_attributes() {
        let registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry.clone());

        let pipeline_params = PipelineContextParams {
            pipeline_group_id: Cow::Borrowed("group1"),
            pipeline_id: Cow::Borrowed("pipe1"),
            core_id: 0,
            num_cores: 1,
            thread_id: 0,
        };
        let base_pipeline_ctx = PipelineContext::new(controller_ctx, pipeline_params);

        let mut custom_attrs = HashMap::new();
        let _ = custom_attrs.insert(
            "custom.identity.foo".to_string(),
            TelemetryAttribute::new(AttributeValue::String("bar".to_string())),
        );

        let pipeline_ctx = base_pipeline_ctx.with_node_context(
            Cow::Borrowed("test-node"),
            NodeUrn::parse("urn:otel:receiver:test").unwrap(),
            NodeKind::Receiver,
            custom_attrs,
        );

        let entity_key = pipeline_ctx.register_node_channel_entity(
            Cow::Borrowed("channel-1"),
            Cow::Borrowed("out"),
            "test_kind",
            "test_mode",
            "test_type",
            "test_impl",
        );

        let has_custom_attr = registry
            .visit_entity(entity_key, |attrs| {
                let mut found = false;
                for (key, val) in attrs.iter_attributes() {
                    if key == "custom" {
                        if format!("{:?}", val).contains("custom.identity.foo") {
                            found = true;
                        }
                    }
                }
                found
            })
            .unwrap_or(false);

        assert!(
            has_custom_attr,
            "The custom identity attribute should be present in the channel entity"
        );
    }
}
'''

open('crates/engine/src/context.rs', 'w').write(before_test + new_test_module)
print("Successfully fixed context.rs")
