//! Create and run a multi-core pipeline

use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};
use otap_df_config::pipeline_group::Quota;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use otap_df_otap::otlp_exporter::OTLP_EXPORTER_URN;
use otap_df_otap::otlp_receiver::OTLP_RECEIVER_URN;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple OTLP pipeline configuration
    let pipeline_cfg = PipelineConfigBuilder::new()
        .add_receiver(
            "otlp_receiver",
            OTLP_RECEIVER_URN,
            Some(json!({
                "listening_addr": "127.0.0.1:4317"
            })),
        )
        .add_exporter(
            "otlp_exporter",
            OTLP_EXPORTER_URN,
            Some(json!({
                "grpc_endpoint": "http://127.0.0.1:1235"
            })),
        )
        .round_robin("otlp_receiver", "out_port", ["otlp_exporter"])
        .build(PipelineType::Otlp, "namespace", "pipeline")?;

    println!(
        "Pipeline configuration:\n{}",
        serde_json::to_string_pretty(&pipeline_cfg)?
    );

    // Create controller and start pipeline with multi-core support
    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let quota = Quota { num_cores: 72 }; // Use 2 cores for testing

    // Start the pipeline (this would run indefinitely in a real scenario)
    controller.start_pipeline(pipeline_cfg, quota)?;
    Ok(())
}
