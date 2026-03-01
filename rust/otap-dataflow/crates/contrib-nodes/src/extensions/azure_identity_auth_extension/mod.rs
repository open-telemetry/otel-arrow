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
//!
//! # Usage
//!
//! Configure the extension in the pipeline configuration:
//!
//! ```yaml
//! extensions:
//!   azure_auth:
//!     type: "urn:microsoft:extension:azure_identity_auth"
//!     config:
//!       method: managed_identity
//!       scope: "https://monitor.azure.com/.default"
//! ```
//!
//! Consumers retrieve the extension by name from the registry:
//!
//! ```ignore
//! let provider: Box<dyn BearerTokenProvider> = extension_registry
//!     .get::<dyn BearerTokenProvider>("azure_auth")?;
//! let mut token_rx = provider.subscribe_token_refresh();
//! ```

use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::extension::ExtensionWrapper;
use otap_df_engine::node::NodeId;
use std::sync::Arc;

use otap_df_otap::OTAP_EXTENSION_FACTORIES;
use otap_df_otap::pdata::OtapPdata;

mod config;
mod error;
mod extension;

pub use config::{AuthMethod, Config};
pub use error::Error;
pub use extension::AzureIdentityAuthExtension;

/// URN identifying the Azure Identity Auth Extension in configuration pipelines.
pub const AZURE_IDENTITY_AUTH_EXTENSION_URN: &str = "urn:microsoft:extension:azure_identity_auth";

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
            node,
            node_config,
            extension_config,
        ))
    },
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_urn() {
        assert_eq!(
            AZURE_IDENTITY_AUTH_EXTENSION_URN,
            "urn:microsoft:extension:azure_identity_auth"
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
