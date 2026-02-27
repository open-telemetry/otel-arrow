// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod config;
pub(crate) mod processor;

use self::config::RecordsetKqlProcessorConfig;
use self::processor::RecordsetKqlProcessor;
use otap_df_otap::pdata::OtapPdata;

use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::{
    config::ProcessorConfig, context::PipelineContext, node::NodeId, processor::ProcessorWrapper,
};
use std::sync::Arc;

// TODO metrics

/// Factory function to create a KQL processor
pub fn create_recordset_kql_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: RecordsetKqlProcessorConfig = serde_json::from_value(node_config.config.clone())
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse KQL configuration: {e}"),
        })?;

    let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config)?;

    Ok(ProcessorWrapper::local(
        processor,
        node,
        node_config,
        processor_config,
    ))
}
