// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

// ToDo: Support transformative processors in a pipeline,
// we should be able to know when the assert equivalent will fail

use otap_df_config::PipelineGroupId;
use otap_df_config::PipelineId;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_engine::PipelineFactory;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
use otap_df_otap::otlp_grpc::OTLPData;
use otap_df_pdata::otap::{OtapArrowRecords, from_record_messages};
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
use otap_df_pdata::{Consumer, Producer};
use otap_df_state::{DeployedPipelineKey, store::ObservedStateStore};
use otap_df_telemetry::MetricsSystem;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, timeout};
use tonic::transport::server::{Router, Server};
use tonic::{Request, Response, Status};
use serde::Deserialize;


const DEFAULT_CORE_ID: usize = 0;
const DEFAULT_THREAD_ID: usize = 0;

const DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE: usize = 100;

#[derive(Debug, Deserialize)]
pub enum PlatformType {
    OTLP,
    OTAP
}

#[derive(Debug, Deserialize)]
struct PipelineValidtionConfig {
    pub test: Vec<PipelineValidation>
}

#[derive(Debug, Deserialize)]
struct PipelineValidation {
    name: String,
    pipeline_config: PathBuf,
    loadgen_exporter_type: PlatformType,
    backend_receiver_type: PlatformType,
    transformitive: bool
}


/// struct to simulate a pipeline running, reads a config and starts a pipeline to send and receive data
pub struct PipelineSimulator {
    pipeline_context: PipelineContext,
    pipeline_id: PipelineId,
    pipeline_group_id: PipelineGroupId,
    metrics_system: MetricsSystem,
    pipeline_key: DeployedPipelineKey,
    pipeline_config: PipelineConfig
    pipeline_runtime:
}


impl PipelineSimulator {
    // if pipeline alters the data via a processor that performs some transofmration we should expect the equivalent assert to fail
    // otherwise the assert should succeed

    pub fn new(pipeline_config: PathBuf, pipeline_factory: &'static PipelineFactory<PData>) -> Result<Self, String> {
        let core_id = DEFAULT_CORE_ID
        let thread_id = DEFAULT_THREAD_ID
        let metrics_system = MetricsSystem::default();
        let controller_context = ControllerContext::new(metrics_system.registry());
        let pipeline_id = PipelineId::default();
        let pipeline_group_id = PipelineGroupId::default();
        let pipeline_context = controller_context.pipeline_context_with(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            core_id,
            thread_id,
        );
        let pipeline_key = DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.clone(),
            pipeline_id: pipeline_id.clone(),
            core_id,
        };
        let pipeline_config = PipelineConfig::from_file(
                pipeline_group_id.clone(),
                pipeline_id.clone(),
                config_file_path,
            )
        let pipeline_runtime = pipeline_factory
                .build(pipeline_context.clone(), pipeline_config.clone()).map_err(|e| e.to_string())?;
        Ok(Self {
            pipeline_context,
            pipeline_id,
            pipeline_group_id,
            metrics_system,
            pipeline_key,
            pipeline_config,
            pipeline_runtime,
        })
    }

    pub fn run(&self, ctrl_msg_tx: PipelineCtrlMsgSender<PData>, ctrl_msg_rx: PipelineCtrlMsgReceiver<PData>) {
        let obs_state_store = ObservedStateStore::new(self.pipeline_config.pipeline_settings());
        let obs_evt_reporter = obs_state_store.reporter();
        self.pipeline_runtime
            .run_forever(
                self.pipeline_key,
                self.pipeline_context,
                obs_evt_reporter,
                metrics_reporter,
                pipeline_ctrl_msg_tx,
                pipeline_ctrl_msg_rx,
            ).expect("failed to start pipeline");
    }
}

impl PipelineValidation {
    pub fn validate(&self) {
        // start pipeline

        // use the variables to configure the templates (load gen and data collect)

                // Create the pipeline control channel outside the thread so we can send shutdown
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) =
            pipeline_ctrl_msg_channel::<PData>(DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE);
        // Clone the sender before moving into the thread so we can use it for shutdown
        let shutdown_sender = pipeline_ctrl_msg_tx.clone();
        let suv_pipeline_config = self.pipeline_config.clone()
        let _suv_thread = std::thread::spawn(move || {
            // create pipeline simulator and run it
            let suv_pipeline = PipelineSimulator(suv_pipeline_config, &OTAP_PIPELINE_FACTORY);
            suv_pipeline.run();
        })

        let _load_gen_thread = std::thread::spawn(move || {
            let load_gen_pipeline = PipelineSimulator(LOAD_GEN_CONFIG, &OTAP_PIPELINE_FACTORY);
            load_gen_pipeline.run();
        })

        // monitor ctrl messages from the simulators
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_otap::OTAP_PIPELINE_FACTORY;
    use otap_df_otap::fake_data_generator::fake_signal::{
        fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
    };
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use std::fs;
    use std::path::Path;

    const LOG_SIGNAL_COUNT: usize = 100;
    const METRIC_SIGNAL_COUNT: usize = 100;
    const TRACE_SIGNAL_COUNT: usize = 100;
    const ITERATIONS: usize = 10;
    const PIPELINE_CONFIG_YAML: &str = "pipeline_validation_configs.yaml";

    const MESSAGE_COUNT: usize = 10;


    // validate the encoding and decoding
    #[test]
    fn validate_pipeline() {
        let file = File::open(PIPELINE_CONFIG_YAML).expect("failed to get config file")
        let config: PipelineConfig = serde_yaml::from_reader(file).expect("Could not deserialize config");
        for config in config.tests {
            let config 
        }

    }

    async fn validate_pipelines() {
        read from the validate_pipelines.yaml file
        iterate for each pipeline
        create traffic-gen with otlp/otap exporter -> this last
        pipeline created second
        create collector with otlp/otap receiver -> this first

    }
}
