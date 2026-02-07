// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth Extension for OTAP.
//!
//! Provides Azure authentication services to the pipeline using Azure Identity.
//! This extension manages token acquisition and refresh, making credentials
//! available to other components (e.g., exporters) that need Azure authentication.
//!
//! # Features
//!
//! - Managed Identity authentication (system or user-assigned)
//! - Developer tools authentication (Azure CLI, Azure Developer CLI)
//! - Automatic token refresh with exponential backoff retry
//! - Shared credential access across pipeline components

use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::extension::ExtensionWrapper;
use otap_df_engine::extensions::BearerTokenProvider;
use otap_df_engine::node::NodeId;
use otap_df_engine::{ExtensionFactory, extension_traits};
use serde_json;
use std::sync::Arc;

use crate::OTAP_EXTENSION_FACTORIES;
use crate::pdata::OtapPdata;

mod config;
mod error;
mod extension;

pub use config::{AuthMethod, Config};
pub use error::Error;
pub use extension::AzureIdentityAuthExtension;

/// URN identifying the Azure Identity Auth Extension in configuration pipelines.
pub const AZURE_IDENTITY_AUTH_EXTENSION_URN: &str = "urn:otel:azureidentityauth:extension";

/// Register Azure Identity Auth Extension with the OTAP extension factory.
///
/// Uses the `distributed_slice` macro for automatic discovery by the dataflow engine.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static AZURE_IDENTITY_AUTH_EXTENSION: ExtensionFactory<OtapPdata> = ExtensionFactory {
    name: AZURE_IDENTITY_AUTH_EXTENSION_URN,
    create: |_: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             extension_config: &ExtensionConfig| {
        // Deserialize user config JSON into typed Config
        let cfg: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Validate the configuration
        cfg.validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?;

        // Create the extension
        let extension = AzureIdentityAuthExtension::new(cfg).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        Ok(ExtensionWrapper::local(
            extension,
            extension_traits!(AzureIdentityAuthExtension => BearerTokenProvider),
            node,
            node_config,
            extension_config,
        ))
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_urn() {
        assert_eq!(
            AZURE_IDENTITY_AUTH_EXTENSION_URN,
            "urn:otel:azureidentityauth:extension"
        );
    }

    #[test]
    fn test_factory_name_matches_urn() {
        assert_eq!(
            AZURE_IDENTITY_AUTH_EXTENSION.name,
            AZURE_IDENTITY_AUTH_EXTENSION_URN
        );
    }
}
