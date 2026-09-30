// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter for mapping OTLP metrics to Geneva Metrics protocol version 6 packets.
//!
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::{SignalFormat, SignalType};
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::ExporterFactory;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::config::ExporterConfig as EngineExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider as LocalBearerTokenProvider;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::{OtlpProtoBytes, PayloadData};
use otel_arrow_dfe_telemetry::otel_warn;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod client;
mod config;
pub mod encoder;
pub mod otlp_to_geneva;

use client::{CertificateIdentity, MetricsPublisher};
pub use config::{AuthConfig, Config as ExporterConfig};
pub use otlp_to_geneva::{Config, ScopeAttributes};

/// URN identifying the Geneva metrics exporter.
pub const GENEVA_METRICS_EXPORTER_URN: &str = "urn:otel:exporter:geneva_metrics";

const UNSUPPORTED_SIGNAL_MESSAGE: &str = "Geneva metrics exporter accepts metrics only";
const UNSUPPORTED_FORMAT_MESSAGE: &str = "Geneva metrics exporter accepts OTLP only";

/// Exporter that converts OTLP metrics to Geneva protocol v6 packets.
pub struct GenevaMetricsExporter {
    mapping_config: Config,
    publisher: MetricsPublisher,
    token_provider: Option<Box<dyn LocalBearerTokenProvider>>,
}

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

fn validate_auth_binding(
    auth: &AuthConfig,
    token_provider_bound: bool,
) -> Result<(), &'static str> {
    match (auth, token_provider_bound) {
        (AuthConfig::None, true) => Err("a bearer_token_provider is bound but auth.type is none"),
        (AuthConfig::Bearer, false) => {
            Err("auth.type bearer requires a bearer_token_provider capability")
        }
        (AuthConfig::Certificate { .. }, true) => {
            Err("certificate authentication cannot be combined with a bearer_token_provider")
        }
        _ => Ok(()),
    }
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
        .optional_local::<BearerTokenProvider>()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    validate_auth_binding(&config.auth, token_provider.is_some()).map_err(|error| {
        ConfigError::InvalidUserConfig {
            error: error.to_string(),
        }
    })?;
    let certificate_password = match &config.auth {
        AuthConfig::Certificate { password_env, .. } => {
            Some(std::env::var(password_env).map_err(|error| {
                ConfigError::InvalidUserConfig {
                    error: format!(
                        "certificate password environment variable {password_env} is unavailable: {error}"
                    ),
                }
            })?)
        }
        AuthConfig::None | AuthConfig::Bearer => None,
    };
    let certificate = match (&config.auth, certificate_password.as_deref()) {
        (AuthConfig::Certificate { path, .. }, Some(password)) => {
            Some(CertificateIdentity { path, password })
        }
        _ => None,
    };
    let publisher =
        MetricsPublisher::new(&config.endpoint, config.timeout, certificate).map_err(|error| {
            ConfigError::InvalidUserConfig {
                error: error.to_string(),
            }
        })?;
    let mapping_config = (&config).into();

    Ok(ExporterWrapper::local(
        GenevaMetricsExporter {
            mapping_config,
            publisher,
            token_provider,
        },
        node,
        node_config,
        exporter_config,
    ))
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaMetricsExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    let signal_type = data.signal_type();
                    let signal_format = data.signal_format();
                    if signal_type != SignalType::Metrics {
                        let nack = NackMsg::new_permanent(UNSUPPORTED_SIGNAL_MESSAGE, data);
                        effect_handler.notify_nack(nack).await?;
                    } else if signal_format != SignalFormat::OtlpBytes {
                        let nack = NackMsg::new_permanent(UNSUPPORTED_FORMAT_MESSAGE, data);
                        effect_handler.notify_nack(nack).await?;
                    } else {
                        match prepare_publications(&data, &self.mapping_config) {
                            Ok(prepared) => {
                                if prepared.rejected_data_points > 0
                                    || prepared.cardinality_overflows > 0
                                {
                                    otel_warn!(
                                        "geneva_metrics_exporter.mapping.partial",
                                        rejected_data_points = prepared.rejected_data_points,
                                        cardinality_overflows = prepared.cardinality_overflows,
                                    );
                                }
                                if prepared.publications.is_empty() {
                                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                                    continue;
                                }

                                let bearer_token = if let Some(provider) = &self.token_provider {
                                    match provider.get_token().await {
                                        Ok(token) => Some(token),
                                        Err(error) => {
                                            let nack = NackMsg::new(
                                                format!(
                                                    "failed to acquire Geneva metrics bearer token: {error}"
                                                ),
                                                data,
                                            );
                                            effect_handler.notify_nack(nack).await?;
                                            continue;
                                        }
                                    }
                                } else {
                                    None
                                };
                                let mut failure = None;
                                for (monitoring_account, packet) in prepared.publications {
                                    if let Err(error) = self
                                        .publisher
                                        .publish(
                                            &monitoring_account,
                                            packet,
                                            bearer_token.as_ref().map(|token| token.expose_token()),
                                        )
                                        .await
                                    {
                                        failure = Some((monitoring_account, error));
                                        break;
                                    }
                                }

                                if let Some((monitoring_account, error)) = failure {
                                    let reason = format!(
                                        "failed to publish Geneva metrics for account {monitoring_account}: {error}"
                                    );
                                    let nack = if error.is_retryable() {
                                        NackMsg::new(reason, data)
                                    } else {
                                        NackMsg::new_permanent(reason, data)
                                    };
                                    effect_handler.notify_nack(nack).await?;
                                } else {
                                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                                }
                            }
                            Err(error) => {
                                let nack = NackMsg::new_permanent(error.to_string(), data);
                                effect_handler.notify_nack(nack).await?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(TerminalState::default())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PreparedPublications {
    publications: Vec<(String, Vec<u8>)>,
    rejected_data_points: usize,
    cardinality_overflows: usize,
}

fn prepare_publications(
    data: &OtapPdata,
    config: &Config,
) -> Result<PreparedPublications, PrepareError> {
    let PayloadData::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(bytes)) =
        data.payload_ref().data()
    else {
        return Err(PrepareError::UnsupportedPayload);
    };
    let receive_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PrepareError::SystemTime)?
        .as_nanos();
    let receive_time = u64::try_from(receive_time).map_err(|_| PrepareError::SystemTime)?;
    let mapped = otlp_to_geneva::decode_and_map(bytes, config, receive_time)?;
    let rejected_data_points = mapped.rejected_data_points;
    let cardinality_overflows = mapped.cardinality_overflows.len();
    let publications = mapped
        .publications
        .into_iter()
        .map(|publication| {
            let bytes = encoder::encode(&publication.packet)?;
            Ok((publication.monitoring_account, bytes))
        })
        .collect::<Result<Vec<_>, encoder::EncodeError>>()?;
    Ok(PreparedPublications {
        publications,
        rejected_data_points,
        cardinality_overflows,
    })
}

#[derive(Debug, thiserror::Error)]
enum PrepareError {
    #[error("Geneva metrics exporter received an unexpected payload")]
    UnsupportedPayload,
    #[error("system clock cannot be represented as an OTLP timestamp")]
    SystemTime,
    #[error(transparent)]
    Mapping(#[from] otlp_to_geneva::MappingError),
    #[error(transparent)]
    Encoding(#[from] encoder::EncodeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use prost::Message as _;
    use serde_json::json;
    use std::path::PathBuf;

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
            "metric_namespace": "example-namespace"
        });

        assert!((GENEVA_METRICS_EXPORTER.validate_config)(&config).is_ok());
    }

    /// Scenario: Authentication modes are paired with absent or present bearer-token capabilities.
    /// Guarantees: Only bindings compatible with the selected authentication mode are accepted.
    #[test]
    fn validates_auth_capability_bindings() {
        let certificate = || AuthConfig::Certificate {
            path: PathBuf::from("client.p12"),
            password_env: "CERT_PASSWORD".to_string(),
        };

        for (auth, provider_bound) in [
            (AuthConfig::None, false),
            (AuthConfig::Bearer, true),
            (certificate(), false),
        ] {
            assert_eq!(validate_auth_binding(&auth, provider_bound), Ok(()));
        }

        for (auth, provider_bound, expected) in [
            (
                AuthConfig::None,
                true,
                "a bearer_token_provider is bound but auth.type is none",
            ),
            (
                AuthConfig::Bearer,
                false,
                "auth.type bearer requires a bearer_token_provider capability",
            ),
            (
                certificate(),
                true,
                "certificate authentication cannot be combined with a bearer_token_provider",
            ),
        ] {
            assert_eq!(validate_auth_binding(&auth, provider_bound), Err(expected));
        }
    }

    /// Scenario: The exporter receives a serialized OTLP gauge before HTTP publication is available.
    /// Guarantees: Runtime preparation decodes, maps, and serializes the request into one account packet.
    #[test]
    fn prepares_otlp_metric_publication() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![Metric {
                        name: "temperature".to_string(),
                        description: String::new(),
                        unit: String::new(),
                        metadata: Vec::new(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: Vec::new(),
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_700_000_000_000_000_000,
                                exemplars: Vec::new(),
                                flags: 0,
                                value: Some(number_data_point::Value::AsDouble(12.5)),
                            }],
                        })),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut bytes = Vec::new();
        request
            .encode(&mut bytes)
            .expect("request should serialize");
        let data =
            OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)).into());
        let config = ExporterConfig {
            endpoint: "https://example.test/metrics".to_string(),
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            timeout: std::time::Duration::from_secs(30),
            auth: AuthConfig::None,
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        };

        let mapping_config = (&config).into();
        let prepared =
            prepare_publications(&data, &mapping_config).expect("publication should prepare");

        assert_eq!(prepared.publications.len(), 1);
        assert_eq!(prepared.publications[0].0, "example-account");
        assert!(!prepared.publications[0].1.is_empty());
        assert_eq!(prepared.rejected_data_points, 0);
        assert_eq!(prepared.cardinality_overflows, 0);
    }

    /// Scenario: An OTLP request contains a data point marked with no recorded value.
    /// Guarantees: Runtime preparation reports the rejected point and produces no empty publication.
    #[test]
    fn reports_rejected_data_points_during_preparation() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![Metric {
                        name: "temperature".to_string(),
                        description: String::new(),
                        unit: String::new(),
                        metadata: Vec::new(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: Vec::new(),
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_700_000_000_000_000_000,
                                exemplars: Vec::new(),
                                flags: 1,
                                value: Some(number_data_point::Value::AsDouble(12.5)),
                            }],
                        })),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut bytes = Vec::new();
        request
            .encode(&mut bytes)
            .expect("request should serialize");
        let data =
            OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)).into());
        let config = ExporterConfig {
            endpoint: "https://example.test/metrics".to_string(),
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            timeout: std::time::Duration::from_secs(30),
            auth: AuthConfig::None,
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        };

        let prepared = prepare_publications(&data, &(&config).into())
            .expect("invalid points should not reject the request");

        assert!(prepared.publications.is_empty());
        assert_eq!(prepared.rejected_data_points, 1);
        assert_eq!(prepared.cardinality_overflows, 0);
    }
}
