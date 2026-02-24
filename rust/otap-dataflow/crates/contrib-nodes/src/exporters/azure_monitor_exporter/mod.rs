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

use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;

mod auth;
mod client;
mod config;
mod error;
mod exporter;
mod gzip_batcher;
mod heartbeat;
mod in_flight_exports;
mod state;
mod stats;
mod transformer;

pub use client::LogsIngestionClient;
pub use config::Config;
pub use error::Error;
pub use exporter::AzureMonitorExporter;
pub use heartbeat::Heartbeat;
pub use stats::AzureMonitorExporterStats;
pub use transformer::Transformer;

/// URN identifying the Azure Monitor Exporter in configuration pipelines.
pub const AZURE_MONITOR_EXPORTER_URN: &str = "urn:microsoft_azure:monitor:exporter";

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
            AzureMonitorExporter::new(cfg).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urn_constant() {
        assert_eq!(
            AZURE_MONITOR_EXPORTER_URN,
            "urn:microsoft_azure:monitor:exporter"
        );
    }
}
