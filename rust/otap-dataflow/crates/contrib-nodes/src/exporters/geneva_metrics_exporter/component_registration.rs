// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::client::MetricsPublisher;
use super::config::Config as ExporterConfig;
use super::metric_export_loop::GenevaMetricsExporter;
use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ExporterFactory;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::config::ExporterConfig as EngineExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use std::sync::Arc;

/// URN identifying the Geneva metrics exporter.
pub const GENEVA_METRICS_EXPORTER_URN: &str = "urn:otel:exporter:geneva_metrics";

/// Registers the Geneva metrics exporter with the dataflow engine.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static GENEVA_METRICS_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: GENEVA_METRICS_EXPORTER_URN,
    create: create_exporter,
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config,
};

fn validate_config(value: &serde_json::Value) -> Result<(), ConfigError> {
    let config: ExporterConfig =
        serde_json::from_value(value.clone()).map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    config
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })
}

fn create_exporter(
    _pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &EngineExporterConfig,
    capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
) -> Result<ExporterWrapper<OtapPdata>, ConfigError> {
    let config: ExporterConfig =
        serde_json::from_value(node_config.config.clone()).map_err(|error| {
            ConfigError::InvalidUserConfig {
                error: error.to_string(),
            }
        })?;
    config
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;

    let token_provider = capabilities
        .require_local::<BearerTokenProvider>()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    let publisher = MetricsPublisher::new(&config.endpoint, config.timeout).map_err(|error| {
        ConfigError::InvalidUserConfig {
            error: error.to_string(),
        }
    })?;
    let mapping_config = (&config).into();

    Ok(ExporterWrapper::local(
        GenevaMetricsExporter::new(mapping_config, publisher, token_provider),
        node,
        node_config,
        exporter_config,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: The exporter is registered in the component inventory.
    /// Guarantees: Configuration can refer to the stable Geneva metrics exporter URN.
    #[test]
    fn exposes_expected_urn() {
        assert_eq!(
            GENEVA_METRICS_EXPORTER.name,
            "urn:otel:exporter:geneva_metrics"
        );
    }

    /// Scenario: The exporter receives its minimal valid configuration.
    /// Guarantees: Configuration validation succeeds before runtime startup.
    #[test]
    fn validates_minimal_config() {
        let config = json!({
            "endpoint": "https://example.test/metrics",
            "monitoring_account": "example-account",
            "metric_namespace": "example-namespace",
            "auth": {
                "type": "bearer"
            }
        });

        assert!((GENEVA_METRICS_EXPORTER.validate_config)(&config).is_ok());
    }

    /// Scenario: Exporter configuration omits the required managed identity marker.
    /// Guarantees: The exporter cannot start without explicitly selecting bearer authentication.
    #[test]
    fn rejects_missing_bearer_authentication_config() {
        let config = json!({
            "endpoint": "https://example.test/metrics",
            "monitoring_account": "example-account",
            "metric_namespace": "example-namespace"
        });

        let error = (GENEVA_METRICS_EXPORTER.validate_config)(&config)
            .expect_err("missing authentication should be rejected");
        assert!(error.to_string().contains("missing field `auth`"));
    }

    /// Scenario: Configuration selects an unauthenticated publication mode.
    /// Guarantees: Only the managed identity bearer marker is accepted.
    #[test]
    fn rejects_unauthenticated_config() {
        let config = json!({
            "endpoint": "https://example.test/metrics",
            "monitoring_account": "example-account",
            "metric_namespace": "example-namespace",
            "auth": {
                "type": "none"
            }
        });

        let error = (GENEVA_METRICS_EXPORTER.validate_config)(&config)
            .expect_err("unauthenticated publication should be rejected");
        assert!(error.to_string().contains("unknown variant `none`"));
    }
}
