// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::config::{AuthConfig, Config as ExporterConfig};
use super::metric_export_loop::GenevaMetricsExporter;
use super::publishing::MetricsPublisher;
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
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
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

fn invalid_config(error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

fn parse_config(value: &serde_json::Value) -> Result<ExporterConfig, ConfigError> {
    let config: ExporterConfig = serde_json::from_value(value.clone()).map_err(invalid_config)?;
    config.validate().map_err(invalid_config)?;
    Ok(config)
}

fn validate_config(value: &serde_json::Value) -> Result<(), ConfigError> {
    parse_config(value).map(drop)
}

fn create_publisher(config: &ExporterConfig) -> Result<MetricsPublisher, ConfigError> {
    match &config.auth {
        AuthConfig::Bearer => {
            MetricsPublisher::new(&config.endpoint, &config.monitoring_account, config.timeout)
        }
        AuthConfig::TokenExchange => MetricsPublisher::new_token_exchange(
            &config.endpoint,
            &config.monitoring_account,
            config.timeout,
        ),
    }
    .map_err(invalid_config)
}

fn create_exporter(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &EngineExporterConfig,
    capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
) -> Result<ExporterWrapper<OtapPdata>, ConfigError> {
    let config = parse_config(&node_config.config)?;
    let token_provider = capabilities
        .require_local::<BearerTokenProvider>()
        .map_err(invalid_config)?;
    let publisher = create_publisher(&config)?;
    let mapping_config = (&config).into();

    Ok(ExporterWrapper::local(
        GenevaMetricsExporter::new(
            mapping_config,
            publisher,
            token_provider,
            ExporterMetrics::register(&pipeline),
        ),
        node,
        node_config,
        exporter_config,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{AUTHORIZATION, HeaderValue};
    use serde_json::json;
    use wiremock::matchers::{body_bytes, header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config_with_auth(auth: Option<serde_json::Value>) -> serde_json::Value {
        let mut config = json!({
            "endpoint": "https://example.test/metrics",
            "monitoring_account": "example-account",
            "metric_namespace": "example-namespace",
        });
        if let Some(auth) = auth {
            config["auth"] = auth;
        }
        config
    }

    fn publisher_config(auth: &str, endpoint: &str) -> ExporterConfig {
        let mut config = config_with_auth(Some(json!({ "type": auth })));
        config["endpoint"] = json!(endpoint);
        // Test publisher selection over loopback HTTP; parse_config still enforces HTTPS.
        serde_json::from_value(config).expect("valid publisher configuration")
    }

    /// Scenario: Factory publisher selection receives direct bearer auth and an account-templated ingestion URL.
    /// Guarantees: The packet is posted directly with the source credential and configured route, without a token exchange.
    #[tokio::test]
    async fn selects_direct_bearer_publication() {
        let server = MockServer::start().await;
        let packet = vec![6, 0, 1, 2, 3];
        Mock::given(method("POST"))
            .and(path("/metrics"))
            .and(query_param("account", "example-account"))
            .and(header("authorization", "Bearer source-token"))
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let config = publisher_config(
            "bearer",
            &format!("{}/metrics?account={{monitoring_account}}", server.uri()),
        );
        let publisher = create_publisher(&config).expect("bearer publisher should build");

        publisher
            .publish(
                packet,
                Some((
                    AUTHORIZATION,
                    HeaderValue::from_static("Bearer source-token"),
                )),
            )
            .await
            .expect("direct publication should succeed");

        server.verify().await;
    }

    /// Scenario: Factory publisher selection receives token_exchange auth and a service origin.
    /// Guarantees: The source credential is used only for exchange, and the packet reaches the returned account route with the publication credential.
    #[tokio::test]
    async fn selects_token_exchange_publication() {
        let exchange = MockServer::start().await;
        let publication = MockServer::start().await;
        let packet = vec![6, 0, 1, 2, 3];
        Mock::given(method("GET"))
            .and(path(
                "/user-api/v2/authorization/gig/monitoringAccount/example-account",
            ))
            .and(header("authorization", "Bearer source-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "gigEndpoint": publication.uri(),
                "gigAuthToken": "publication-token",
            })))
            .expect(1)
            .mount(&exchange)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&exchange)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ingestion/ingest"))
            .and(query_param("accountId", "example-account"))
            .and(header("authorization", "Bearer publication-token"))
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&publication)
            .await;
        let config = publisher_config("token_exchange", &exchange.uri());
        let publisher = create_publisher(&config).expect("token exchange publisher should build");

        publisher
            .publish(
                packet,
                Some((
                    AUTHORIZATION,
                    HeaderValue::from_static("Bearer source-token"),
                )),
            )
            .await
            .expect("exchanged publication should succeed");

        exchange.verify().await;
        publication.verify().await;
    }

    /// Scenario: The exporter receives its minimal valid configuration.
    /// Guarantees: Configuration validation succeeds before runtime startup.
    #[test]
    fn validates_minimal_config() {
        let config = config_with_auth(Some(json!({ "type": "bearer" })));

        assert!((GENEVA_METRICS_EXPORTER.validate_config)(&config).is_ok());
    }

    /// Scenario: Token exchange authentication is selected with a Geneva service origin.
    /// Guarantees: Component validation accepts the token_exchange configuration value.
    #[test]
    fn validates_token_exchange_config() {
        let mut config = config_with_auth(Some(json!({ "type": "token_exchange" })));
        config["endpoint"] = json!("https://example.test");
        assert!((GENEVA_METRICS_EXPORTER.validate_config)(&config).is_ok());
    }

    /// Scenario: The documented sample pipeline is parsed and its metrics exporter is validated.
    /// Guarantees: The sample has a valid pipeline graph and capability binding, and its exporter configuration matches the registered component.
    #[test]
    fn validates_sample_pipeline() {
        let spec = otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(include_str!(
            "otlp-geneva-metrics.yaml"
        ))
        .expect("sample pipeline should be structurally valid");
        let pipeline = &spec.groups["default"].pipelines["main"];
        let exporter = &pipeline.nodes()["geneva-metrics"];
        assert_eq!(exporter.r#type.as_ref(), GENEVA_METRICS_EXPORTER_URN);
        (GENEVA_METRICS_EXPORTER.validate_config)(&exporter.config)
            .expect("sample exporter configuration should be valid");
    }

    /// Scenario: Configuration omits authentication or selects an unsupported mode.
    /// Guarantees: The exporter cannot start without explicitly selecting a supported authentication mode.
    #[test]
    fn rejects_missing_or_unsupported_authentication_config() {
        for (auth, expected) in [
            (None, "missing field `auth`"),
            (Some(json!({ "type": "none" })), "unknown variant `none`"),
            (
                Some(json!({ "type": "managed_identity" })),
                "unknown variant `managed_identity`",
            ),
        ] {
            let error = (GENEVA_METRICS_EXPORTER.validate_config)(&config_with_auth(auth))
                .expect_err("missing or unsupported authentication should be rejected");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }
}
