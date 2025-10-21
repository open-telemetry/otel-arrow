#![allow(missing_docs)]
//! Integration smoke test for fake-batch-perf-issue-1310.

// Integration smoke test: build runtime pipeline from repo config and ensure
// a batch processor node is present. Ignored by default.

#[test]
#[ignore]
fn smoke_has_batch_processor_node() {
    // Locate the repo-hosted config relative to this crate's Cargo.toml
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../configs/fake-batch-perf-issue-1310.yaml",
    );

    let pipeline = otap_df_config::pipeline::PipelineConfig::from_file(
        otap_df_config::PipelineGroupId::from("smoke".to_string()),
        otap_df_config::PipelineId::from("fake-batch-issue-1310".to_string()),
        path,
    )
    .expect("failed to load repo config fake-batch-perf-issue-1310.yaml");

    // Build runtime (no run)
    let metrics = otap_df_telemetry::registry::MetricsRegistryHandle::new();
    let controller_ctx = otap_df_engine::context::ControllerContext::new(metrics);
    let pipeline_ctx = controller_ctx.pipeline_context_with(
        otap_df_config::PipelineGroupId::from("smoke".to_string()),
        otap_df_config::PipelineId::from("fake-batch-issue-1310".to_string()),
        0,
        0,
    );

    let runtime = otap_df_otap::OTAP_PIPELINE_FACTORY
        .build(pipeline_ctx, pipeline)
        .expect("factory should build the runtime pipeline");

    assert!(runtime.node_count() > 0, "pipeline should contain nodes");

    // Verify presence of batch processor
    let mut found = false;
    for i in 0..runtime.node_count() {
        if let Some(node) = runtime.get_node(i) {
            let uc = node.user_config();
            if uc.kind == otap_df_config::node::NodeKind::Processor
                && uc.plugin_urn.as_ref() == "urn:otap:processor:batch"
            {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "expected a batch processor node (urn:otap:processor:batch)"
    );
}
