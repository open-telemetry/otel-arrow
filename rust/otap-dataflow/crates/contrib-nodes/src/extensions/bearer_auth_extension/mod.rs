// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bearer token authentication extension with periodic token refresh.
//!
//! This extension provides client-side (exporter) bearer token authentication.
//! The token is refreshed periodically in the extension's background loop,
//! and the `ClientAuthenticatorHandle` automatically sees updated values.
//!
//! # Configuration
//!
//! ```yaml
//! extensions:
//!   my_auth:
//!     type: "urn:otap:extension:auth/bearer"
//!     config:
//!       refresh_interval_secs: 300
//! ```
//!
//! # Usage
//!
//! Exporters can attach credentials to outgoing requests:
//!
//! ```rust,ignore
//! let auth = extension_registry
//!     .get::<ClientAuthenticatorHandle>("my_auth")?;
//! for (key, value) in auth.get_request_metadata()? {
//!     request.headers_mut().insert(key, value);
//! }
//! ```

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::Error;
use otap_df_engine::extension::ExtensionWrapper;
use otap_df_engine::extensions::{
    AuthError, ClientAuthenticator, ClientAuthenticatorHandle, ExtensionHandles,
};
use otap_df_engine::local::extension as local;
use otap_df_engine::node::NodeId;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A shared, mutable token that the extension updates and the authenticator reads.
type SharedToken = Arc<Mutex<String>>;

/// Default interval between token refreshes.
const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(300);

/// URN identifying the bearer auth extension in pipeline configuration.
pub const BEARER_AUTH_EXTENSION_URN: &str = "urn:otap:extension:auth/bearer";

/// Configuration for the bearer token auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Token refresh interval in seconds. Defaults to 300 (5 minutes).
    #[serde(default = "default_refresh_interval_secs")]
    pub refresh_interval_secs: u64,
}

fn default_refresh_interval_secs() -> u64 {
    TOKEN_REFRESH_INTERVAL.as_secs()
}

/// Register the bearer auth extension with the OTAP extension factory.
///
/// Uses the `distributed_slice` macro for automatic discovery by the dataflow engine.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static BEARER_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: BEARER_AUTH_EXTENSION_URN,
    create: |_pipeline_ctx: PipelineContext,
             node_id: NodeId,
             node_config: Arc<NodeUserConfig>,
             extension_config: &ExtensionConfig| {
        let cfg: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Shared token — starts empty, populated by the first refresh in the start loop.
        let shared_token: SharedToken = Arc::new(Mutex::new(String::new()));

        let mut handles = ExtensionHandles::new();
        handles.register(ClientAuthenticatorHandle::new(BearerClientAuth {
            token: Arc::clone(&shared_token),
        }));

        Ok(ExtensionWrapper::local(
            BearerAuthExtension {
                token: shared_token,
                refresh_interval: Duration::from_secs(cfg.refresh_interval_secs),
            },
            handles,
            node_id,
            node_config,
            extension_config,
        ))
    },
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

/// The extension instance that runs as a background task.
///
/// Periodically refreshes the shared bearer token. The new token is a
/// mock value generated from the current timestamp — in a real extension
/// this would call an identity provider or secret store.
struct BearerAuthExtension {
    token: SharedToken,
    refresh_interval: Duration,
}

impl BearerAuthExtension {
    /// Generates a mock token that looks like a GUID.
    ///
    /// Uses the current system time to produce a deterministic-looking but
    /// unique value. A production implementation would fetch a real token
    /// from an identity provider.
    fn generate_token() -> String {
        use std::time::SystemTime;

        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // Format as a GUID-like string from the timestamp bits.
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (nanos >> 96) as u32,
            (nanos >> 80) as u16,
            (nanos >> 64) as u16,
            (nanos >> 48) as u16,
            nanos as u64 & 0xffff_ffff_ffff,
        )
    }

    fn refresh_token(&self) {
        let new_token = Self::generate_token();
        *self.token.lock().expect("token lock poisoned") = new_token;
    }
}

#[async_trait(?Send)]
impl local::Extension for BearerAuthExtension {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel,
        effect_handler: local::EffectHandler,
    ) -> Result<(), Error> {
        // Fetch the first token immediately.
        self.refresh_token();
        effect_handler
            .info("[bearer-auth] initial token acquired")
            .await;

        let mut refresh_timer = tokio::time::interval(self.refresh_interval);
        // The first tick completes immediately — skip it since we just refreshed.
        let _ = refresh_timer.tick().await;

        loop {
            tokio::select! {
                msg = ctrl_chan.recv() => {
                    match msg {
                        Ok(msg) if msg.is_shutdown() => break,
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
                _ = refresh_timer.tick() => {
                    self.refresh_token();
                    effect_handler.info("[bearer-auth] token refreshed").await;
                }
            }
        }
        Ok(())
    }
}

/// Client-side authenticator that reads the current token from the shared
/// `Arc<Mutex<String>>` and produces an `Authorization: Bearer <token>` header.
struct BearerClientAuth {
    token: SharedToken,
}

impl ClientAuthenticator for BearerClientAuth {
    fn get_request_metadata(
        &self,
    ) -> Result<Vec<(http::HeaderName, http::HeaderValue)>, AuthError> {
        let current_token = self.token.lock().expect("token lock poisoned");
        if current_token.is_empty() {
            return Err(AuthError {
                message: "token not yet available".into(),
            });
        }
        Ok(vec![(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_str(&format!("Bearer {}", *current_token)).map_err(|e| {
                AuthError {
                    message: e.to_string(),
                }
            })?,
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared_token(s: &str) -> SharedToken {
        Arc::new(Mutex::new(s.to_owned()))
    }

    #[test]
    fn config_deserializes_defaults() {
        let json = serde_json::json!({});
        let cfg: Config = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.refresh_interval_secs, 300);
    }

    #[test]
    fn config_with_custom_interval() {
        let json = serde_json::json!({ "refresh_interval_secs": 60 });
        let cfg: Config = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.refresh_interval_secs, 60);
    }

    #[test]
    fn config_rejects_unknown_fields() {
        let json = serde_json::json!({ "unknown": true });
        assert!(serde_json::from_value::<Config>(json).is_err());
    }

    #[test]
    fn client_auth_produces_bearer_header() {
        let auth = ClientAuthenticatorHandle::new(BearerClientAuth {
            token: shared_token("outgoing-token"),
        });
        let metadata = auth.get_request_metadata().unwrap();
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0].0, http::header::AUTHORIZATION);
        assert_eq!(metadata[0].1, "Bearer outgoing-token");
    }

    #[test]
    fn client_auth_errors_when_token_empty() {
        let auth = ClientAuthenticatorHandle::new(BearerClientAuth {
            token: shared_token(""),
        });
        let err = auth.get_request_metadata().unwrap_err();
        assert!(err.message.contains("not yet available"));
    }

    #[test]
    fn token_refresh_updates_client_handle() {
        let token = shared_token("initial");
        let client = ClientAuthenticatorHandle::new(BearerClientAuth {
            token: Arc::clone(&token),
        });

        // Simulate a token refresh.
        *token.lock().unwrap() = "refreshed".to_owned();

        let metadata = client.get_request_metadata().unwrap();
        assert_eq!(metadata[0].1, "Bearer refreshed");
    }

    #[test]
    fn generate_token_produces_guid_format() {
        let token = BearerAuthExtension::generate_token();
        let parts: Vec<&str> = token.split('-').collect();
        assert_eq!(parts.len(), 5, "token should have 5 dash-separated parts");
    }

    #[test]
    fn handle_registers_in_extension_handles() {
        let token = shared_token("tok");
        let mut handles = ExtensionHandles::new();
        handles.register(ClientAuthenticatorHandle::new(BearerClientAuth { token }));
    }
}
