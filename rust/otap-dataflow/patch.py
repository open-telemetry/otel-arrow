import sys

content = open('crates/engine/src/context.rs').read()
test_code = '''
    #[test]
    fn register_node_channel_entity_includes_custom_attributes() {
        use std::borrow::Cow;
        use std::collections::HashMap;
        let registry = TelemetryRegistryHandle::new();
        let ctx = ControllerContext::new_with_identity(
            registry.clone(),
            "proc-123",
            "machine-abc",
            "container-xyz",
        );
        let mut custom_attrs = HashMap::new();
        custom_attrs.insert(
            "custom.identity.foo".to_string(),
            otap_df_config::pipeline::telemetry::TelemetryAttribute::new(
                otap_df_config::pipeline::telemetry::AttributeValue::String("bar".to_string())
            ),
        );
        let pipeline_ctx = PipelineContext::with_node_context(
            &ctx,
            ConfigNodeId::new("test-node"),
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
        let has_custom_attr = registry.visit_entity(entity_key, |attrs| {
            let mut found = false;
            for (key, _val) in attrs.iter_attributes() {
                if key == "custom.identity.foo" {
                    found = true;
                }
            }
            found
        }).unwrap_or(false);
        assert!(has_custom_attr, "The custom identity attribute should be present in the channel entity");
    }
}
'''

content = content.rstrip()
if content.endswith('}'):
    content = content[:-1] + test_code
    open('crates/engine/src/context.rs', 'w').write(content)
    print('Patched successfully')
else:
    print('Failed to patch')
