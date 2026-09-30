// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::encoder;
use super::otlp_to_geneva::{self, Config};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::{OtlpProtoBytes, PayloadData};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PreparedPublications {
    pub(super) publications: Vec<(String, Vec<u8>)>,
    pub(super) rejected_data_points: usize,
    pub(super) cardinality_overflows: usize,
}

pub(super) fn prepare_publications(
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
pub(super) enum PrepareError {
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
    use super::super::{AuthConfig, ExporterConfig};
    use super::*;
    use bytes::Bytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use prost::Message as _;

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
            auth: AuthConfig::Bearer,
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
            auth: AuthConfig::Bearer,
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
