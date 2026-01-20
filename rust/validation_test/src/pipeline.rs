// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

use otap_df_config::PipelineGroupId;
use otap_df_config::PipelineId;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{
    PipelineCtrlMsgReceiver, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel,
};
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use otap_df_otap::pdata::OtapPdata;
use otap_df_state::{DeployedPipelineKey, store::ObservedStateStore};
use otap_df_telemetry::MetricsSystem;
use otap_df_telemetry::reporter::MetricsReporter;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};
use tokio::time::{Duration, timeout};

const DEFAULT_CORE_ID: usize = 0;
const DEFAULT_THREAD_ID: usize = 0;

const DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE: usize = 100;

const LOAD_GEN_TEMPLATE_PATH: &str = "templates/load_gen/config.yaml.tera";
const DATA_COLLECT_TEMPLATE_PATH: &str = "templates/data_collect/config.yaml.tera";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformType {
    Otlp,
    Otap,
}

#[derive(Debug, Deserialize)]
struct PipelineValidtionConfig {
    pub tests: Vec<PipelineValidation>,
}

#[derive(Debug, Deserialize)]
struct PipelineValidation {
    name: String,
    pipeline_config_path: PathBuf,
    variables: TemplateVariables,
}

#[derive(Debug, Deserialize, Serialize)]
struct TemplateVariables {
    loadgen_exporter_type: PlatformType,
    backend_receiver_type: PlatformType,
    transformitive: bool,
}

/// struct to simulate a pipeline running, reads a config and starts a pipeline to send and receive data
pub struct PipelineSimulator {
    pipeline_context: PipelineContext,
    metrics_system: MetricsSystem,
    pipeline_key: DeployedPipelineKey,
    pipeline_config: PipelineConfig,
}

impl PipelineSimulator {
    // if pipeline alters the data via a processor that performs some transofmration we should expect the equivalent assert to fail
    // otherwise the assert should succeed

    pub fn new_from_file(pipeline_config: PathBuf) -> Result<Self, String> {
        let core_id = DEFAULT_CORE_ID;
        let thread_id = DEFAULT_THREAD_ID;
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
            pipeline_config,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            pipeline_context,
            metrics_system,
            pipeline_key,
            pipeline_config,
        })
    }

    pub fn new_from_yaml(yaml: &str) -> Result<Self, String> {
        let core_id = DEFAULT_CORE_ID;
        let thread_id = DEFAULT_THREAD_ID;
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
        let pipeline_config =
            PipelineConfig::from_yaml(pipeline_group_id.clone(), pipeline_id.clone(), yaml)
                .map_err(|e| e.to_string())?;

        Ok(Self {
            pipeline_context,
            metrics_system,
            pipeline_key,
            pipeline_config,
        })
    }

    pub fn run(
        &self,
        ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
        ctrl_msg_rx: PipelineCtrlMsgReceiver<OtapPdata>,
    ) {
        let pipeline_key = self.pipeline_key.clone();
        let pipeline_context = self.pipeline_context.clone();
        let obs_state_store = ObservedStateStore::new(self.pipeline_config.pipeline_settings());
        let obs_evt_reporter = obs_state_store.reporter();
        let pipeline_runtime = OTAP_PIPELINE_FACTORY
            .build(pipeline_context.clone(), self.pipeline_config.clone())
            .expect("failed to create runtime");
        let metrics_reporter = self.metrics_system.reporter();
        pipeline_runtime
            .run_forever(
                pipeline_key,
                pipeline_context,
                obs_evt_reporter,
                metrics_reporter,
                ctrl_msg_tx,
                ctrl_msg_rx,
            )
            .expect("failed to start pipeline");
    }
}

impl PipelineValidation {
    pub fn validate(&self) {
        // start pipeline

        // Create the pipeline control channel outside the thread so we can send shutdown
        let (data_collect_pipeline_ctrl_msg_tx, data_collect_pipeline_ctrl_msg_rx) =
            pipeline_ctrl_msg_channel::<OtapPdata>(DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE);
        // Clone the sender before moving into the thread so we can use it for shutdown
        let data_collect_sender = data_collect_pipeline_ctrl_msg_tx.clone();

        let data_collect_pipeline_config = self.pipeline_config_path.clone();
        let data_collect_pipeline = PipelineSimulator::new_from_yaml(
            self.render_template(DATA_COLLECT_TEMPLATE_PATH).as_str(),
        )
        .expect("failed to create simulator");
        let data_collect_metrics_reporter = data_collect_pipeline.get_metric_reporter();
        let _data_collect_thread = std::thread::spawn(move || {
            // create pipeline simulator and run it
            data_collect_pipeline.run(
                data_collect_pipeline_ctrl_msg_tx,
                data_collect_pipeline_ctrl_msg_rx,
            );
        });


        

        // Create the pipeline control channel outside the thread so we can send shutdown
        let (suv_pipeline_ctrl_msg_tx, suv_pipeline_ctrl_msg_rx) =
            pipeline_ctrl_msg_channel::<OtapPdata>(DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE);
        // Clone the sender before moving into the thread so we can use it for shutdown
        let suv_sender = suv_pipeline_ctrl_msg_tx.clone();

        let suv_pipeline = PipelineSimulator::new_from_file(self.pipeline_config_path.clone())
            .expect("failed to create simulator");
        let _suv_thread = std::thread::spawn(move || {
            // create pipeline simulator and run it

            suv_pipeline.run(suv_pipeline_ctrl_msg_tx, suv_pipeline_ctrl_msg_rx);
        });






        // Create the pipeline control channel outside the thread so we can send shutdown
        let (load_gen_pipeline_ctrl_msg_tx, load_gen_pipeline_ctrl_msg_rx) =
            pipeline_ctrl_msg_channel::<OtapPdata>(DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE);
        // Clone the sender before moving into the thread so we can use it for shutdown
        let load_gen_sender = load_gen_pipeline_ctrl_msg_tx.clone();
        let load_gen_pipeline =
            PipelineSimulator::new_from_yaml(self.render_template(LOAD_GEN_TEMPLATE_PATH).as_str())
                .expect("failed to create simulator");
        let _load_gen_thread = std::thread::spawn(move || {
            load_gen_pipeline.run(load_gen_pipeline_ctrl_msg_tx, load_gen_pipeline_ctrl_msg_rx);
        });

        // monitor ctrl messages from the simulators

        // monitor metrics from the load gen pipeline until we notice max signal reached then we can start to kill the pipelines

        // get result form
    }

    pub fn render_template(&self, template_path: &str) -> String {
        let template = fs::read_to_string(template_path).expect("failed to read template");
        // one_off avoids building a template registry; autoescape=false for YAML
        Tera::one_off(
            &template,
            &Context::from_serialize(&self.variables).expect("failed to pass context"),
            false,
        )
        .expect("failed to render template")
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
    use std::fs::File;
    use std::path::Path;

    const PIPELINE_CONFIG_YAML: &str = "pipeline_validation_configs.yaml";

    // validate the encoding and decoding
    #[test]
    fn validate_pipeline() {
        let file = File::open(PIPELINE_CONFIG_YAML).expect("failed to get config file");
        let config: PipelineValidtionConfig =
            serde_yaml::from_reader(file).expect("Could not deserialize config");
        for config in config.tests {
            let rendered_template = config.render_template(LOAD_GEN_TEMPLATE_PATH);
            println!("{}", rendered_template);
        }
    }

    // async fn validate_pipelines() {
    //     read from the validate_pipelines.yaml file
    //     iterate for each pipeline
    //     create traffic-gen with otlp/otap exporter -> this last
    //     pipeline created second
    //     create collector with otlp/otap receiver -> this first

    // }
}
