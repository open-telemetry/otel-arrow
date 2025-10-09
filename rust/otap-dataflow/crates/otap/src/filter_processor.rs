// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP Debug processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Use OTLP Views instead of the OTLP Request structs


use crate::{
    OTAP_PROCESSOR_FACTORIES,
    pdata::OtapPdata
};
use async_trait::async_trait;
use linkme::distributed_slice;

use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use serde_json::Value;
use std::sync::Arc;

/// The URN for the filter processor
pub const FILTER_PROCESSOR_URN: &str = "urn:otel:filter:processor";

/// processor that outputs all data received to stdout
pub struct FilterProcessor {
    config: Config,
}



#[async_trait(?Send)]
impl local::Processor<OtapPdata> for FilterProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {

        match msg {
            Message::Control(control) => {
                match control {
                    NodeControlMsg::TimerTick {} => {
                        debug_output.output_message("Timer tick received\n").await?;
                    }
                    NodeControlMsg::Config { .. } => {
                        debug_output
                            .output_message("Config message received\n")
                            .await?;
                    }
                    NodeControlMsg::Shutdown { .. } => {
                        debug_output
                            .output_message("Shutdown message received\n")
                            .await?;
                    }
                    _ => {}
                }
                Ok(())
            }
            Message::PData(pdata) => {

            
            Ok(())
            }
        }
    }
}
