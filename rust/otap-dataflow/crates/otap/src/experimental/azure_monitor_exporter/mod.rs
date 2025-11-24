// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Monitor Exporter for OTAP.
//!
//! Sends OpenTelemetry logs to Azure Monitor using the Data Collection Rules (DCR) API.

use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::node::NodeId;
use serde_json;
use std::sync::Arc;

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;

mod client;
mod config;
mod exporter;
mod transformer;

pub use client::LogsIngestionClient;
pub use config::Config;
pub use exporter::AzureMonitorExporter;
pub use transformer::Transformer;

/// URN identifying the Azure Monitor Exporter in configuration pipelines.
pub const AZURE_MONITOR_EXPORTER_URN: &str = "urn:otel:azuremonitor:exporter";

/// Register Azure Monitor Exporter with the OTAP exporter factory.
///
/// Uses the `distributed_slice` macro for automatic discovery by the dataflow engine.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static AZURE_MONITOR_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: AZURE_MONITOR_EXPORTER_URN,
    create: |_: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        // Deserialize user config JSON into typed Config
        let cfg: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        Ok(ExporterWrapper::local(
            AzureMonitorExporter::new(cfg)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urn_constant() {
        assert_eq!(AZURE_MONITOR_EXPORTER_URN, "urn:otel:azuremonitor:exporter");
    }
}
