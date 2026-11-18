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

// Cap publications at 20 MB to leave headroom below the 20 MiB limit.
pub(super) const MAX_PUBLICATION_BYTES: usize = 20_000_000;

#[derive(Debug, PartialEq)]
pub(super) struct PublicationPreparation {
    pub(super) publications: Vec<encoder::Packet>,
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
        // Keep the existing best-effort policy for malformed nested input (#4029).
        _ = RawMetricsData::try_new(view.bytes()).map_err(PrepareError::Decode)?;
        otlp_to_geneva::map_otlp_bytes(view.bytes(), config, receive_time)?
    } else {
        return Err(PrepareError::UnsupportedPayload);
    };
    let rejected_data_points = mapped.rejected_data_points;
    let cardinality_overflows = mapped.cardinality_overflows.len();
    let publications = encoder::partition_bounded(mapped.packet, MAX_PUBLICATION_BYTES)?;
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

/// Fixtures shared by the preparation and export loop tests.
#[cfg(test)]
pub(super) mod test_support {
    use super::super::otlp_to_geneva::default_max_mapping_bytes;
    use super::{Config, OtapPdata};
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;
    use otel_arrow_dfe_pdata_codec::OtapPayload;

    pub(crate) fn mapping_config() -> Config {
        Config {
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
            max_mapping_bytes: default_max_mapping_bytes(),
        }
    }

    pub(crate) fn gauge_point(attributes: Vec<KeyValue>, flags: u32) -> NumberDataPoint {
        NumberDataPoint {
            attributes,
            time_unix_nano: 1_700_000_000_000_000_000,
            flags,
            value: Some(number_data_point::Value::AsDouble(12.5)),
            ..Default::default()
        }
    }

    /// A request with one `temperature` gauge holding `points`.
    pub(crate) fn gauge_request(points: Vec<NumberDataPoint>) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
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
            }],
        }
    }

    pub(crate) fn single_point_request() -> ExportMetricsServiceRequest {
        gauge_request(vec![gauge_point(Vec::new(), 0)])
    }

    pub(crate) fn fragmented_request(fragments: impl IntoIterator<Item = metric::Data>) -> Vec<u8> {
        use prost::Message;

        let mut bytes = Vec::new();
        for (index, data) in fragments.into_iter().enumerate() {
            Metric {
                name: if index == 0 {
                    "temperature".into()
                } else {
                    String::new()
                },
                data: Some(data),
                ..Default::default()
            }
            .encode(&mut bytes)
            .expect("metric fragment should encode");
        }
        wrap_metric(bytes)
    }

    pub(crate) fn wrap_metric(mut bytes: Vec<u8>) -> Vec<u8> {
        use prost::encoding::{WireType, encode_key, encode_varint};

        for field in [2, 2, 1] {
            let mut parent = Vec::new();
            encode_key(field, WireType::LengthDelimited, &mut parent);
            encode_varint(bytes.len() as u64, &mut parent);
            parent.extend_from_slice(&bytes);
            bytes = parent;
        }
        bytes
    }

    /// Over 20 MiB of distinct dimension values, so preparation must split it.
    pub(crate) fn oversized_request() -> ExportMetricsServiceRequest {
        gauge_request(
            (0..21_000)
                .map(|index| {
                    let value = format!("{index:08}{}", "x".repeat(1_016));
                    let attribute = KeyValue {
                        key: "dimension".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(value)),
                        }),
                    };
                    gauge_point(vec![attribute], 0)
                })
                .collect(),
        )
    }

    pub(crate) fn otlp_pdata(request: ExportMetricsServiceRequest) -> OtapPdata {
        let payload = OtapPayload::try_from(OtlpProtoMessage::Metrics(request.into()))
            .expect("request should serialize");
        OtapPdata::new_default(payload)
    }

    pub(crate) fn otap_pdata(request: ExportMetricsServiceRequest) -> OtapPdata {
        let records = otlp_to_otap(&OtlpProtoMessage::Metrics(request.into()));
        OtapPdata::new_default(OtapPayload::from_otap(records))
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{
        fragmented_request, gauge_point, gauge_request, mapping_config, otap_pdata, otlp_pdata,
        oversized_request, single_point_request,
    };
    use super::*;
    use bytes::Bytes;
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;
    use otel_arrow_dfe_pdata_codec::OtapPayload;

    fn raw_otlp(bytes: &[u8]) -> OtapPdata {
        OtapPdata::new_default(
            OtlpProtoBytes::ExportMetricsRequest(Bytes::copy_from_slice(bytes)).into(),
        )
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

    /// Scenario: An OTLP request contains a data point marked with no recorded value.
    /// Guarantees: Runtime preparation reports the rejected point and produces no empty publication.
    #[test]
    fn reports_rejected_data_points_during_preparation() {
        let data = otlp_pdata(gauge_request(vec![gauge_point(Vec::new(), 1)]));

        let prepared = prepare_publication(&data, &mapping_config())
            .expect("invalid points should not reject the request");

        assert!(prepared.publications.is_empty());
        assert_eq!(prepared.rejected_data_points, 1);
        assert_eq!(prepared.cardinality_overflows, 0);
    }

    /// Scenario: One metric's gauge is split across two valid protobuf messages with distinct points.
    /// Guarantees: Preparation publishes both points without rejection, exactly matching a canonical single-gauge request.
    #[test]
    fn merges_repeated_gauge_messages_before_publication() {
        let first = gauge_point(Vec::new(), 0);
        let second = NumberDataPoint {
            value: Some(number_data_point::Value::AsInt(20)),
            ..first.clone()
        };
        let request = gauge_request(vec![first.clone(), second.clone()]);
        let bytes = fragmented_request([
            metric::Data::Gauge(Gauge {
                data_points: vec![first],
            }),
            metric::Data::Gauge(Gauge {
                data_points: vec![second],
            }),
        ]);
        let receive_time = 1_700_000_000_000_000_000;
        let expected =
            prepare_publication_at(&otlp_pdata(request), &mapping_config(), receive_time)
                .expect("canonical request should prepare");
        let actual = prepare_publication_at(&raw_otlp(&bytes), &mapping_config(), receive_time)
            .expect("fragmented gauge should prepare");

        assert_eq!(actual, expected);
        assert_eq!(actual.publications[0].metrics.len(), 2);
        assert_eq!(actual.rejected_data_points, 0);
    }

    /// Scenario: OTLP metrics bytes have a truncated length, a length past the end, or an unsupported wire type at the top level.
    /// Guarantees: Preparation rejects malformed framing as a decode error instead of acknowledging it without publication.
    #[test]
    fn rejects_malformed_otlp_framing() {
        for malformed in [&[0x0a][..], &[0x0a, 0x05, 0x01][..], &[0x0b][..]] {
            let error = prepare_publication(&raw_otlp(malformed), &mapping_config())
                .expect_err("malformed framing should be rejected");

            assert!(matches!(error, PrepareError::Decode(_)));
        }
    }

    /// Scenario: Ordinary and split-gauge resources precede a resource whose nested scope length runs past its message.
    /// Guarantees: Normalization preserves all three valid points and the existing best-effort handling of the malformed resource (#4029).
    #[test]
    fn skips_malformed_nested_otlp_content() {
        let mut bytes = Vec::new();
        OtlpProtoMessage::Metrics(single_point_request().into())
            .encode(&mut bytes)
            .expect("request should serialize");
        let gauge = metric::Data::Gauge(Gauge {
            data_points: vec![gauge_point(Vec::new(), 0)],
        });
        bytes.extend_from_slice(&fragmented_request([gauge.clone(), gauge]));
        // ResourceMetrics whose scope_metrics field claims five bytes but contains one.
        bytes.extend_from_slice(&[0x0a, 0x03, 0x12, 0x05, 0x01]);

        let prepared = prepare_publication(&raw_otlp(&bytes), &mapping_config())
            .expect("well-formed top-level framing should be accepted");

        assert_eq!(prepared.publications.len(), 1);
        assert_eq!(prepared.publications[0].metrics.len(), 3);
        assert!(
            !encoder::encode(&prepared.publications[0])
                .expect("packet should encode")
                .is_empty()
        );
        assert_eq!(prepared.rejected_data_points, 0);
    }

    /// Scenario: A well-formed gauge is followed by a metric whose name is the invalid UTF-8 byte 0xff.
    /// Guarantees: Like protobuf decoding, invalid UTF-8 rejects the whole request instead of publishing part of it.
    #[test]
    fn rejects_invalid_utf8_strings() {
        let mut bytes = Vec::new();
        OtlpProtoMessage::Metrics(single_point_request().into())
            .encode(&mut bytes)
            .expect("request should serialize");
        // ResourceMetrics > ScopeMetrics > Metric whose name is the single byte 0xff.
        bytes.extend_from_slice(&[0x0a, 0x07, 0x12, 0x05, 0x12, 0x03, 0x0a, 0x01, 0xff]);

        let error = prepare_publication(&raw_otlp(&bytes), &mapping_config())
            .expect_err("invalid UTF-8 should reject the request");

        assert!(matches!(
            error,
            PrepareError::Mapping(otlp_to_geneva::MappingError::InvalidUtf8(_))
        ));
    }

    /// Scenario: The resource_metrics field, or the resource field nested inside it, uses a varint instead of a length-delimited wire type.
    /// Guarantees: Until strict validation exists (#4029), preparation ignores the mistyped fields instead of rejecting the request, so nothing is published.
    #[test]
    fn ignores_otlp_fields_with_unexpected_wire_types() {
        for mistyped in [&[0x08, 0x01][..], &[0x0a, 0x02, 0x08, 0x01][..]] {
            let prepared = prepare_publication(&raw_otlp(mistyped), &mapping_config())
                .expect("well-formed top-level framing should be accepted");

            assert!(prepared.publications.is_empty());
            assert_eq!(prepared.rejected_data_points, 0);
        }
    }

    /// Scenario: Native OTAP contains three gauges with transport-encoded IDs and distinct point attributes.
    /// Guarantees: Normalization preserves metric/value/attribute associations, matches OTLP publication bytes, and leaves the original payload unchanged.
    #[test]
    fn normalizes_transport_encoded_metrics_before_mapping() {
        let mut request = gauge_request(Vec::new());
        request.resource_metrics[0].scope_metrics[0].metrics = (0..3)
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
        let records = otlp_to_otap(&OtlpProtoMessage::Metrics(request.clone().into()));
        let mut optimized = records.clone();
        optimized
            .encode_transport_optimized()
            .expect("transport optimization should succeed");
        assert_ne!(records.root_record_batch(), optimized.root_record_batch());
        let original_root = optimized.root_record_batch().cloned();
        let native = OtapPdata::new_default(OtapPayload::from_otap(optimized));
        let encoded = otlp_pdata(request);
        let receive_time = 1_700_000_000_000_000_000;
        let expected = prepare_publication_at(&encoded, &mapping_config(), receive_time)
            .expect("OTLP metrics should prepare");
        let actual = prepare_publication_at(&native, &mapping_config(), receive_time)
            .expect("transport-encoded native metrics should prepare");
        assert_eq!(actual, expected);
        assert_eq!(actual.publications[0].metrics.len(), 3);
        assert_eq!(
            native
                .payload_ref()
                .otap_ref()
                .expect("native payload")
                .root_record_batch(),
            original_root.as_ref(),
        );
    }

    /// Scenario: OTLP and native OTAP batches contain 70,000 attribute-free gauge points using the default mapping budget.
    /// Guarantees: Preparation preserves every point without rejection instead of refusing a compact batch under default configuration.
    #[test]
    fn prepares_large_attribute_free_gauge_with_default_budget() {
        let request = gauge_request(vec![gauge_point(Vec::new(), 0); 70_000]);
        for data in [otlp_pdata(request.clone()), otap_pdata(request)] {
            let prepared = prepare_publication(&data, &mapping_config())
                .expect("compact gauge batch should fit the default budget");

            assert_eq!(prepared.rejected_data_points, 0);
            assert_eq!(prepared.cardinality_overflows, 0);
            assert_eq!(
                prepared
                    .publications
                    .iter()
                    .map(|packet| packet.metrics.len())
                    .sum::<usize>(),
                70_000,
            );
        }
    }

    /// Scenario: A valid OTLP request contains over 20 MiB of distinct dimension values.
    /// Guarantees: Every point is preserved in independently checksummed packets of at most 20 MB.
    #[test]
    fn splits_large_otlp_request_below_publication_limit() {
        let prepared = prepare_publication(&otlp_pdata(oversized_request()), &mapping_config())
            .expect("large request should be split, not rejected");
        assert!(prepared.publications.len() > 1);
        assert_eq!(prepared.rejected_data_points, 0);
        assert_eq!(
            prepared
                .publications
                .iter()
                .map(|packet| packet.metrics.len())
                .sum::<usize>(),
            21_000,
        );
        for packet in prepared.publications {
            let bytes = encoder::encode(&packet).expect("packet should encode");
            assert!(bytes.len() <= MAX_PUBLICATION_BYTES);
            let crc = u32::from_le_bytes(bytes[2..6].try_into().expect("CRC field"));
            assert_eq!(crc, crc32fast::hash(&bytes[6..]));
        }
    }
}
