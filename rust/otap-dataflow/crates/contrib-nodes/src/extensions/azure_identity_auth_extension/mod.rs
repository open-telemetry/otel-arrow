// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth Extension for OTAP.
//!
//! Provides Azure authentication services to the pipeline using Azure Identity.
//! This extension manages token acquisition and refresh, making credentials
//! available to other components (e.g., exporters) via the
//! [`BearerTokenProviderHandle`](otap_df_engine::extensions::BearerTokenProviderHandle).
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
//! Consumers retrieve the handle from the extension registry:
//!
//! ```ignore
//! let handle = extension_registry
//!     .get::<BearerTokenProviderHandle>("azure_auth")?;
//! let token = handle.get_token().await?;
//! ```

use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::extension::ExtensionWrapper;
use otap_df_engine::extensions::{BearerTokenProviderHandle, ExtensionHandles};
use otap_df_engine::node::NodeId;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;
use std::sync::Arc;

pub mod config;
pub mod error;
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
pub static AZURE_IDENTITY_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: AZURE_IDENTITY_AUTH_EXTENSION_URN,
    create: |_pipeline_ctx: PipelineContext,
             node_id: NodeId,
             node_config: Arc<NodeUserConfig>,
             extension_config: &ExtensionConfig| {
        let cfg: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        cfg.validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?;

        let (extension, provider) = AzureIdentityAuthExtension::new(&cfg).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        let mut handles = ExtensionHandles::new();
        handles.register(BearerTokenProviderHandle::new(provider));

        Ok(ExtensionWrapper::local(
            extension,
            handles,
            node_id,
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
    fn extension_urn() {
        assert_eq!(
            AZURE_IDENTITY_AUTH_EXTENSION_URN,
            "urn:microsoft:extension:azure_identity_auth"
        );
    }

    #[test]
    fn factory_name_matches_urn() {
        assert_eq!(
            AZURE_IDENTITY_AUTH_EXTENSION.name,
            AZURE_IDENTITY_AUTH_EXTENSION_URN
        );
    }
}
