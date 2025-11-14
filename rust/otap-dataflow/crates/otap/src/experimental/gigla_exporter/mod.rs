// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! GigLA (Geneva Infrastructure General-purpose Logging Analytics) exporter for OTAP.

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

mod config;
mod exporter;
mod gigla_client;
mod transformer;

pub use config::Config;
pub use exporter::GigLaExporter;
pub use gigla_client::GigLaClient;
pub use transformer::Transformer;

/// URN identifying the GigLA exporter in configuration pipelines.
pub const GIGLA_EXPORTER_URN: &str = "urn:otel:gigla:exporter";

/// Register GigLA exporter with the OTAP exporter factory.
///
/// Uses the `distributed_slice` macro for automatic discovery by the dataflow engine.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static GIGLA_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: GIGLA_EXPORTER_URN,
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
            GigLaExporter::new(cfg)?,
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
        assert_eq!(GIGLA_EXPORTER_URN, "urn:otel:gigla:exporter");
    }
}
