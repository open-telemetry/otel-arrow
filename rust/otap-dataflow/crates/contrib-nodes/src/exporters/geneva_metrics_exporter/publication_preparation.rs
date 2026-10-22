// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::encoder;
use super::otlp_to_geneva::{self, Config};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::views::otap::OtapMetricsView;
use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otel_arrow_dfe_pdata_codec::PdataEncoding;
use std::time::{SystemTime, UNIX_EPOCH};

// ME reserves headroom below GIG's 20 MiB limit using this same bound.
pub(super) const MAX_PUBLICATION_BYTES: usize = 20_000_000;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PublicationPreparation {
    pub(super) publications: Vec<encoder::EncodedPacket>,
    pub(super) rejected_data_points: usize,
    pub(super) cardinality_overflows: usize,
}

pub(super) fn prepare_publication(
    data: &OtapPdata,
    config: &Config,
) -> Result<PublicationPreparation, PrepareError> {
    let receive_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PrepareError::SystemTime)?
        .as_nanos();
    let receive_time = u64::try_from(receive_time).map_err(|_| PrepareError::SystemTime)?;
    prepare_publication_at(data, config, receive_time)
}

fn prepare_publication_at(
    data: &OtapPdata,
    config: &Config,
    receive_time: u64,
) -> Result<PublicationPreparation, PrepareError> {
    if data.signal_type() != SignalType::Metrics {
        return Err(PrepareError::UnsupportedPayload);
    }
    let payload = data.payload_ref();
    let mapped = if let Some(records) = payload.otap_ref() {
        let mut records = records.clone();
        records
            .decode_transport_optimized_ids()
            .map_err(PrepareError::Decode)?;
        let metrics = OtapMetricsView::try_from(&records).map_err(PrepareError::Decode)?;
        otlp_to_geneva::map_metrics(&metrics, config, receive_time)?
    } else if let Some(view) = payload
        .encoded_view()
        .filter(|view| view.encoding() == &PdataEncoding::OTLP)
    {
        // Only top-level framing is validated; other content is read best-effort (see #4029).
        let metrics = RawMetricsData::try_new(view.bytes()).map_err(PrepareError::Decode)?;
        otlp_to_geneva::map_metrics(&metrics, config, receive_time)?
    } else {
        return Err(PrepareError::UnsupportedPayload);
    };
    let rejected_data_points = mapped.rejected_data_points;
    let cardinality_overflows = mapped.cardinality_overflows.len();
    let publications = encoder::encode_bounded(&mapped.packet, MAX_PUBLICATION_BYTES)?;
    Ok(PublicationPreparation {
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
    #[error("invalid metrics request: {0}")]
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
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;
    use otel_arrow_dfe_pdata_codec::OtapPayload;

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

    /// Scenario: Preparation receives OTLP bytes for a non-metrics signal.
    /// Guarantees: Unsupported payloads are rejected before their content is interpreted as OTLP metrics.
    #[test]
    fn rejects_unsupported_payloads() {
        for payload in [
            OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
            OtlpProtoBytes::ExportTracesRequest(Bytes::new()).into(),
        ] {
            let data = OtapPdata::new_default(payload);

            assert!(matches!(
                prepare_publication(&data, &mapping_config()),
                Err(PrepareError::UnsupportedPayload)
            ));
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

        assert_eq!(prepared.publications.len(), 1);
        assert!(!prepared.publications[0].bytes.is_empty());
        assert_eq!(prepared.publications[0].item_count, 1);
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

        assert!(prepared.publications.is_empty());
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

        assert_eq!(prepared.publications.len(), 1);
        assert!(!prepared.publications[0].bytes.is_empty());
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

            assert!(prepared.publications.is_empty());
            assert_eq!(prepared.rejected_data_points, 0);
        }
    }

    /// Scenario: Preparation receives native OTAP gauge metrics from a pipeline source.
    /// Guarantees: Native metrics produce a publication with the same mapped point count as OTLP.
    #[test]
    fn prepares_native_otap_metrics() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![gauge_resource(Vec::new(), vec![gauge_point(Vec::new(), 0)])],
        };
        let records = otlp_to_otap(&OtlpProtoMessage::Metrics(request.clone().into()));
        let native = OtapPdata::new_default(OtapPayload::from_otap(records));
        let encoded = metrics_pdata(request);

        for data in [native, encoded] {
            let prepared = prepare_publication(&data, &mapping_config())
                .expect("native and encoded metrics should prepare");
            assert_eq!(prepared.publications.len(), 1);
            assert_eq!(prepared.publications[0].item_count, 1);
            assert_eq!(prepared.rejected_data_points, 0);
        }
    }

    /// Scenario: Native OTAP contains three gauges with transport-encoded IDs and distinct point attributes.
    /// Guarantees: Normalization preserves metric/value/attribute associations, matches OTLP publication bytes, and leaves the original payload unchanged.
    #[test]
    fn normalizes_transport_encoded_metrics_before_mapping() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, any_value};

        let mut resource = gauge_resource(Vec::new(), Vec::new());
        resource.scope_metrics[0].metrics = (0..3)
            .map(|index| Metric {
                name: format!("gauge{index}"),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![NumberDataPoint {
                        attributes: vec![KeyValue {
                            key: "point".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(format!("value{index}"))),
                            }),
                        }],
                        value: Some(number_data_point::Value::AsInt(index + 10)),
                        ..gauge_point(Vec::new(), 0)
                    }],
                })),
                ..Default::default()
            })
            .collect();
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![resource],
        };
        let records = otlp_to_otap(&OtlpProtoMessage::Metrics(request.clone().into()));
        let mut optimized = records.clone();
        optimized
            .encode_transport_optimized()
            .expect("transport optimization should succeed");
        assert_ne!(records.root_record_batch(), optimized.root_record_batch());
        let original_root = optimized.root_record_batch().cloned();
        let native = OtapPdata::new_default(OtapPayload::from_otap(optimized));
        let encoded = metrics_pdata(request);
        let receive_time = 1_700_000_000_000_000_000;
        let expected = prepare_publication_at(&encoded, &mapping_config(), receive_time)
            .expect("OTLP metrics should prepare");
        let actual = prepare_publication_at(&native, &mapping_config(), receive_time)
            .expect("transport-encoded native metrics should prepare");
        assert_eq!(actual, expected);
        assert_eq!(actual.publications[0].item_count, 3);
        assert_eq!(
            native
                .payload_ref()
                .otap_ref()
                .expect("native payload")
                .root_record_batch(),
            original_root.as_ref(),
        );
    }

    /// Scenario: A valid OTLP request contains over 20 MiB of distinct dimension values.
    /// Guarantees: Preparation preserves every point in independently checksummed packets below the GIG publication limit.
    #[test]
    fn splits_large_otlp_request_below_gig_limit() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, any_value};

        let points = (0..21_000)
            .map(|index| {
                gauge_point(
                    vec![KeyValue {
                        key: "dimension".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(format!(
                                "{index:08}{}",
                                "x".repeat(1_016)
                            ))),
                        }),
                    }],
                    0,
                )
            })
            .collect();
        let data = metrics_pdata(ExportMetricsServiceRequest {
            resource_metrics: vec![gauge_resource(Vec::new(), points)],
        });
        let prepared = prepare_publication(&data, &mapping_config())
            .expect("large request should be split, not rejected");
        assert!(prepared.publications.len() > 1);
        assert_eq!(prepared.rejected_data_points, 0);
        assert_eq!(
            prepared
                .publications
                .iter()
                .map(|packet| packet.item_count)
                .sum::<u64>(),
            21_000,
        );
        for packet in prepared.publications {
            assert!(packet.bytes.len() <= MAX_PUBLICATION_BYTES);
            let crc = u32::from_le_bytes(packet.bytes[2..6].try_into().expect("CRC field"));
            assert_eq!(crc, crc32fast::hash(&packet.bytes[6..]));
        }
    }
}
