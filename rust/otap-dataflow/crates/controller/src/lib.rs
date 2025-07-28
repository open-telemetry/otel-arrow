// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Engine Controller

use otap_df_config::pipeline_group::PipelineGroupConfig;

/// Error messages.
pub mod error;

/// Controller for managing pipeline groups and their pipelines.
pub struct Controller {
}

impl Controller {
    /// Starts the controller with the given pipeline group configuration.
    pub fn start(_pipeline_group: PipelineGroupConfig) -> Result<(), error::Error> {
        // Use pipeline group quota to determine resource allocation such as CPU cores
        
        // Start one engine per pipeline per core

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};
    use otap_df_otlp::otlp_exporter::OTLP_EXPORTER_URN;
    use otap_df_otlp::OTLP_PIPELINE_FACTORY;
    use otap_df_otlp::otlp_receiver::OTLP_RECEIVER_URN;

    #[test]
    #[ignore]
    fn test_start_controller() -> Result<(), Box<dyn std::error::Error>> {
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

        // Uncomment the following lines once the controller is fully implemented
        // let mut pipeline_group_cfg = PipelineGroupConfig::new();
        // pipeline_group_cfg.add_pipeline("pipeline".into(), pipeline_cfg.clone())?;
        // 
        // let controller = Controller::new();
        // controller.start_pipeline_group(pipeline_group_cfg)?;
        
        println!(
            "Pipeline configuration:\n{}",
            serde_json::to_string_pretty(&pipeline_cfg)?
        );

        // Use the pipeline factory to create a runtime pipeline from the passed configuration
        let runtime_pipeline = OTLP_PIPELINE_FACTORY.build(pipeline_cfg)?;

        // Start one instance of the engine
        runtime_pipeline.start()?;
        Ok(())
    }
}