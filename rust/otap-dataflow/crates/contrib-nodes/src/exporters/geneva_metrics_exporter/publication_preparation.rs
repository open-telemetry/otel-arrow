// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::encoder;
use super::otlp_to_geneva::{self, Config};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otel_arrow_dfe_pdata::{OtlpProtoBytes, PayloadData};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PublicationPreparation {
    pub(super) publication: Option<(String, Vec<u8>)>,
    pub(super) rejected_data_points: usize,
    pub(super) cardinality_overflows: usize,
}

pub(super) fn prepare_publication(
    data: &OtapPdata,
    config: &Config,
) -> Result<PublicationPreparation, PrepareError> {
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
    // Only top-level framing is validated; other content is read best-effort (see #4029).
    let metrics = RawMetricsData::try_new(bytes.as_ref()).map_err(PrepareError::Decode)?;
    let mapped = otlp_to_geneva::map_metrics(&metrics, config, receive_time)?;
    let rejected_data_points = mapped.rejected_data_points;
    let cardinality_overflows = mapped.cardinality_overflows.len();
    let publication = if mapped.packet.metrics.is_empty() {
        None
    } else {
        let bytes = encoder::encode(&mapped.packet)?;
        Some((config.monitoring_account.clone(), bytes))
    };
    Ok(PublicationPreparation {
        publication,
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
    #[error("invalid OTLP metrics request: {0}")]
    Decode(#[source] otel_arrow_dfe_pdata::error::Error),
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
    use otel_arrow_dfe_pdata::OtapPayload;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;

    fn metrics_pdata(request: ExportMetricsServiceRequest) -> OtapPdata {
        let payload = OtapPayload::try_from(OtlpProtoMessage::Metrics(request.into()))
            .expect("request should serialize");
        OtapPdata::new_default(payload)
    }

    fn mapping_config() -> Config {
        Config {
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        }
    }

    fn gauge_point(attributes: Vec<KeyValue>, flags: u32) -> NumberDataPoint {
        NumberDataPoint {
            attributes,
            time_unix_nano: 1_700_000_000_000_000_000,
            flags,
            value: Some(number_data_point::Value::AsDouble(12.5)),
            ..Default::default()
        }
    }

    fn gauge_resource(
        resource_attributes: Vec<KeyValue>,
        points: Vec<NumberDataPoint>,
    ) -> ResourceMetrics {
        ResourceMetrics {
            resource: Some(Resource {
                attributes: resource_attributes,
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                metrics: vec![Metric {
                    name: "temperature".to_string(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: points,
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
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
        let data = metrics_pdata(request);
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
            prepare_publication(&data, &mapping_config).expect("publication should prepare");

        let (monitoring_account, packet) = prepared
            .publication
            .expect("one publication should be prepared");
        assert_eq!(monitoring_account, "example-account");
        assert!(!packet.is_empty());
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
        let data = metrics_pdata(request);
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

        let prepared = prepare_publication(&data, &(&config).into())
            .expect("invalid points should not reject the request");

        assert!(prepared.publication.is_none());
        assert_eq!(prepared.rejected_data_points, 1);
        assert_eq!(prepared.cardinality_overflows, 0);
    }

    /// Scenario: OTLP metrics bytes have a truncated length, a length past the end, or an unsupported wire type at the top level.
    /// Guarantees: Preparation rejects malformed framing as a decode error instead of acknowledging it without publication.
    #[test]
    fn rejects_malformed_otlp_framing() {
        for malformed in [&[0x0a][..], &[0x0a, 0x05, 0x01][..], &[0x0b][..]] {
            let data = OtapPdata::new_default(
                OtlpProtoBytes::ExportMetricsRequest(Bytes::copy_from_slice(malformed)).into(),
            );

            let error = prepare_publication(&data, &mapping_config())
                .expect_err("malformed framing should be rejected");

            assert!(matches!(error, PrepareError::Decode(_)));
        }
    }

    /// Scenario: A well-formed resource is followed by a resource whose nested scope length runs past the end of its message.
    /// Guarantees: Until strict nested validation exists (#4029), preparation publishes the well-formed metrics and skips the malformed content.
    #[test]
    fn skips_malformed_nested_otlp_content() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![gauge_resource(Vec::new(), vec![gauge_point(Vec::new(), 0)])],
        };
        let mut bytes = Vec::new();
        OtlpProtoMessage::Metrics(request.into())
            .encode(&mut bytes)
            .expect("request should serialize");
        // ResourceMetrics whose scope_metrics field claims five bytes but contains one.
        bytes.extend_from_slice(&[0x0a, 0x03, 0x12, 0x05, 0x01]);
        let data =
            OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)).into());

        let prepared = prepare_publication(&data, &mapping_config())
            .expect("well-formed top-level framing should be accepted");

        let (monitoring_account, packet) = prepared
            .publication
            .expect("well-formed metrics should be published");
        assert_eq!(monitoring_account, "example-account");
        assert!(!packet.is_empty());
        assert_eq!(prepared.rejected_data_points, 0);
    }

    /// Scenario: The resource_metrics field, or the resource field nested inside it, uses a varint instead of a length-delimited wire type.
    /// Guarantees: Until strict validation exists (#4029), preparation ignores the mistyped fields instead of rejecting the request, so nothing is published.
    #[test]
    fn ignores_otlp_fields_with_unexpected_wire_types() {
        for mistyped in [&[0x08, 0x01][..], &[0x0a, 0x02, 0x08, 0x01][..]] {
            let data = OtapPdata::new_default(
                OtlpProtoBytes::ExportMetricsRequest(Bytes::copy_from_slice(mistyped)).into(),
            );

            let prepared = prepare_publication(&data, &mapping_config())
                .expect("well-formed top-level framing should be accepted");

            assert!(prepared.publication.is_none());
            assert_eq!(prepared.rejected_data_points, 0);
        }
    }
}
