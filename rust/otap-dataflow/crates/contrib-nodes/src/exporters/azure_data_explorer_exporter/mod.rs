// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Data Explorer (ADX) Exporter for OTAP.
//!
//! Sends OpenTelemetry logs, metrics, and traces to Azure Data Explorer
//! through the streaming ingestion REST API. Its row layouts are based on the
//! Go `azuredataexplorerexporter` from otel-collector-contrib.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = AZURE_DATA_EXPLORER_EXPORTER_URN,
    target = "microsoft.exporter.azure_data_explorer",
);

use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ExporterFactory;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::node::NodeId;
use std::sync::Arc;

use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;

mod client;
/// Configuration types for the Azure Data Explorer Exporter.
pub mod config;
mod encoding;
mod error;
mod exporter;
/// Metrics for the Azure Data Explorer Exporter.
pub mod metrics;
mod transformer;

pub use client::AzureDataExplorerClient;
pub use config::Config;
pub use error::Error;
pub use exporter::AzureDataExplorerExporter;
pub use metrics::{
    AzureDataExplorerExporterBatchMetrics, AzureDataExplorerExporterHttpMetrics,
    AzureDataExplorerExporterMetricsRc,
};
pub use transformer::Transformer;

/// URN identifying the Azure Data Explorer Exporter in configuration pipelines.
pub const AZURE_DATA_EXPLORER_EXPORTER_URN: &str = "urn:microsoft:exporter:azure_data_explorer";

/// Register Azure Data Explorer Exporter with the OTAP exporter factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static AZURE_DATA_EXPLORER_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: AZURE_DATA_EXPLORER_EXPORTER_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
        let cfg: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        let token_provider = capabilities
            .require_local::<BearerTokenProvider>()
            .map_err(|error| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: format!(
                    "exporter node `{node}` requires a bound `bearer_token_provider` capability: {error}"
                ),
            })?;

        Ok(ExporterWrapper::local(
            AzureDataExplorerExporter::new(pipeline_ctx, cfg, token_provider).map_err(|e| {
                otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<Config>,
    context_declarations: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the Azure Data Explorer exporter is registered by its public URN.
    /// Guarantees: pipeline configuration can select the exporter using the documented identifier.
    #[test]
    fn test_urn_constant() {
        assert_eq!(
            AZURE_DATA_EXPLORER_EXPORTER_URN,
            "urn:microsoft:exporter:azure_data_explorer"
        );
    }

    /// Scenario: an exporter node has no bearer-token-provider capability binding.
    /// Guarantees: capability resolution fails with the required capability name instead of starting unauthenticated.
    #[test]
    fn missing_bearer_token_provider_is_rejected() {
        let capabilities = otel_arrow_dfe_engine::capability::registry::Capabilities::empty();
        let error = match capabilities.require_local::<BearerTokenProvider>() {
            Ok(_) => panic!("authentication capability must be required"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("bearer_token_provider"));
    }
}
