// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! # Microsoft JSON receiver
//!
//! An HTTP receiver that ingests Microsoft Common Schema (CS) formatted JSON events
//! via the OneCollector and Application Insights / Azure Monitor protocols.
//!
//! Three ingestion modes are supported:
//!
//! - **POST `/OneCollector/1.0`** -- the body carries `application/x-json-stream`
//!   (newline-delimited JSON) or `application/json`. Each JSON line becomes an
//!   OTel log record.
//! - **GET `/OneCollector/1.0[/t.js]?<encoded-values>`** -- the query string
//!   carries the encoded event data (browser beacon / `<script>` tag fallback).
//! - **POST `/v2/track`** -- the body carries `application/json` as a JSON array
//!   of event objects (Application Insights / Azure Monitor protocol).
//!
//! Because browsers may trigger CORS preflight requests, the receiver handles
//! `OPTIONS` on all paths with protocol-specific headers.
//!
//! ## Implementation status
//!
//! The ingestion modes above describe the intended receiver behavior. This module
//! is a skeleton: construction deserializes and validates receiver configuration,
//! then returns [`Error::NotImplemented`]. It does not open a listener, load endpoint
//! files, handle requests, or register metrics.
//!
//! ## Module layout
//!
//! - [`config`] -- receiver-level [`Config`](config::Config) and
//!   [`MicrosoftJsonReceiverMetrics`](config::MicrosoftJsonReceiverMetrics),
//!   endpoint configuration, and field-mapping deserialization types.
//! - `field_mapping` (planned) -- YAML-driven field-mapping engine: compiled
//!   representation, event overrides, and validation.
//! - `arrow_builder` (planned) -- config-driven Arrow record-batch builder that
//!   converts JSON events into OTel log records.
//! - `handler` (planned) -- per-connection HTTP request handler, CORS helpers, and
//!   response builders.
//! - `ikey_filter` -- instrumentation-key rule compilation and evaluation.
//! - `user_agent` (planned) -- user-agent parsing and enrichment.
//! - `error` -- receiver configuration and unimplemented-runtime errors.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = MICROSOFT_JSON_RECEIVER_URN,
    target = "microsoft.receiver.json",
);

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::validation::deserialize_typed_config;
use otel_arrow_dfe_engine::ReceiverFactory;
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::error::{Error as EngineError, ReceiverErrorKind};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use std::sync::Arc;

pub mod config;
mod error;
mod ikey_filter;

pub use config::Config;
pub use error::Error;

/// URN identifying the Microsoft JSON receiver in pipeline configuration.
pub const MICROSOFT_JSON_RECEIVER_URN: &str = "urn:microsoft:receiver:json";

// ---------------------------------------------------------------------------
// Receiver struct + factory
// ---------------------------------------------------------------------------

/// Microsoft JSON receiver framework.
#[derive(Debug)]
pub struct MicrosoftJsonReceiver {
    config: Config,
}

/// Registers the Microsoft JSON receiver with the OTAP receiver factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(
    category = Receiver,
    description = "Browser-focused HTTP receiver for unauthenticated Microsoft Common Schema JSON telemetry",
    attributes(protocol = "HTTP", auth = "NONE"),
)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static MICROSOFT_JSON_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: MICROSOFT_JSON_RECEIVER_URN,
    create:
        |pipeline_ctx: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         receiver_config: &ReceiverConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            let receiver = MicrosoftJsonReceiver::from_config(pipeline_ctx, &node_config.config)?;
            Ok(ReceiverWrapper::local(
                receiver,
                node,
                node_config,
                receiver_config,
            ))
        },
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |raw| {
        let config = deserialize_config(raw)?;
        config.validate().map_err(invalid_config)
    },
    context_declarations: None,
};

impl MicrosoftJsonReceiver {
    /// Creates a new `MicrosoftJsonReceiver` from a configuration JSON value.
    ///
    /// Returns an error if the configuration cannot be deserialized or validated.
    /// Valid configuration returns the unimplemented-runtime error until the
    /// receiver implementation is added.
    fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let config = deserialize_config(config)?;
        Self::new(pipeline_ctx, config).map_err(invalid_config)
    }

    /// Validates receiver configuration before returning [`Error::NotImplemented`].
    pub fn new(_pipeline_ctx: PipelineContext, config: Config) -> Result<Self, Error> {
        config.validate()?;
        Err(Error::NotImplemented)
    }
}

// ---------------------------------------------------------------------------
// Receiver trait implementation
// ---------------------------------------------------------------------------

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for MicrosoftJsonReceiver {
    async fn start(
        self: Box<Self>,
        _ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let _framework_config = &self.config;
        Err(EngineError::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Configuration,
            error: Error::NotImplemented.to_string(),
            source_detail: String::new(),
        })
    }
}

fn deserialize_config(raw: &serde_json::Value) -> Result<Config, ConfigError> {
    deserialize_typed_config(raw)
}

fn invalid_config(error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: factory validation receives the documented minimal receiver configuration.
    /// Guarantees: valid configuration is accepted without attempting to start the unimplemented runtime.
    #[test]
    fn factory_validation_accepts_valid_config() {
        (MICROSOFT_JSON_RECEIVER.validate_config)(&json!({
            "listening_addr": "127.0.0.1:4319",
            "endpoints": {
                "/OneCollector/1.0": {
                    "config_file": "onecollector-1ds-endpoint.yaml"
                }
            }
        }))
        .expect("valid receiver config");
    }

    /// Scenario: factory validation receives a configuration with no listener address.
    /// Guarantees: the direct-listener skeleton rejects the configuration before receiver construction.
    #[test]
    fn factory_validation_requires_listener_address() {
        let error = (MICROSOFT_JSON_RECEIVER.validate_config)(&json!({}))
            .expect_err("listening_addr is required");
        assert!(error.to_string().contains("listening_addr"), "{error}");
    }

    /// Scenario: receiver construction receives semantically valid configuration.
    /// Guarantees: phase one terminates explicitly with NotImplemented instead of appearing operational.
    #[test]
    fn constructor_returns_not_implemented() {
        let config: Config = serde_json::from_value(json!({
            "listening_addr": "127.0.0.1:4319"
        }))
        .expect("valid config");
        let (pipeline_ctx, _) = otel_arrow_dfe_engine::testing::test_pipeline_ctx();

        let error = MicrosoftJsonReceiver::new(pipeline_ctx, config)
            .expect_err("runtime is intentionally absent");
        assert!(matches!(error, Error::NotImplemented));
    }

    /// Scenario: typed receiver construction receives a zero concurrency limit.
    /// Guarantees: configuration errors take precedence over the unimplemented-runtime error.
    #[test]
    fn constructor_validates_before_not_implemented() {
        let config: Config = serde_json::from_value(json!({
            "listening_addr": "127.0.0.1:4319",
            "max_concurrent_requests": 0
        }))
        .expect("config deserializes");
        let (pipeline_ctx, _) = otel_arrow_dfe_engine::testing::test_pipeline_ctx();

        let error = MicrosoftJsonReceiver::new(pipeline_ctx, config)
            .expect_err("invalid concurrency must fail validation");
        assert!(
            matches!(error, Error::Config(ref message) if message.contains("max_concurrent_requests")),
            "{error}"
        );
    }

    /// Scenario: the factory entry point receives malformed, invalid, and valid configuration.
    /// Guarantees: parsing and validation run before valid input reaches the explicit NotImplemented error.
    #[test]
    fn from_config_validates_before_not_implemented() {
        for (raw, expected) in [
            (json!({}), "listening_addr"),
            (
                json!({
                    "listening_addr": "127.0.0.1:4319",
                    "max_request_body_size": 0
                }),
                "max_request_body_size",
            ),
            (
                json!({"listening_addr": "127.0.0.1:4319"}),
                &Error::NotImplemented.to_string(),
            ),
        ] {
            let (pipeline_ctx, _) = otel_arrow_dfe_engine::testing::test_pipeline_ctx();
            let error = MicrosoftJsonReceiver::from_config(pipeline_ctx, &raw)
                .expect_err("the receiver is a skeleton");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }
}
