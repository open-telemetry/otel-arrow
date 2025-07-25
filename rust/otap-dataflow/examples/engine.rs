//! Create and run a pipeline

use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};
use otap_df_otlp::OTLP_PIPELINE_FACTORY;
use otap_df_otlp::otlp_exporter::OTLP_EXPORTER_URN;
use otap_df_otlp::otlp_receiver::OTLP_RECEIVER_URN;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //console_subscriber::init();

    // Configure the pipeline with an OTLP receiver and debug exporter
    let config = PipelineConfigBuilder::new()
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
        .round_robin("otlp_receiver", "out_port", ["exporter"])
        .build(PipelineType::Otlp, "namespace", "pipeline")?;

    println!(
        "Pipeline configuration:\n{}",
        serde_json::to_string_pretty(&config)?
    );

    // Build and start the runtime pipeline
    let runtime_pipeline = OTLP_PIPELINE_FACTORY.build(config)?;

    println!("Press Ctrl+C to stop");

    runtime_pipeline.start()?;

    // Keep the pipeline running
    std::thread::park();

    Ok(())
}
