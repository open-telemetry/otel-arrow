// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::encoder;
use super::otlp_to_geneva::{self, ACCOUNT_ATTRIBUTE, Config, PREVIOUS_ACCOUNT_ATTRIBUTE};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otel_arrow_dfe_pdata::{OtlpProtoBytes, PayloadData};
use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView};
use otel_arrow_dfe_pdata_views::views::metrics::{
    DataType, DataView, ExponentialHistogramDataPointView, ExponentialHistogramView, GaugeView,
    HistogramDataPointView, HistogramView, MetricView, MetricsView, NumberDataPointView,
    ResourceMetricsView, ScopeMetricsView, SumView, SummaryDataPointView, SummaryView,
};
use otel_arrow_dfe_pdata_views::views::resource::ResourceView;
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
    let publication_count = mapped.publications.len();
    // Recheck routing only for partial mappings because rejected points do not create publications.
    if publication_count > 1
        || (rejected_data_points > 0
            && selects_multiple_monitoring_accounts(&metrics, &config.monitoring_account))
    {
        return Err(PrepareError::MultipleMonitoringAccounts);
    }
    let publication = if let Some(publication) = mapped.publications.into_iter().next() {
        let bytes = encoder::encode(&publication.packet)?;
        Some((publication.monitoring_account, bytes))
    } else {
        None
    };
    Ok(PublicationPreparation {
        publication,
        rejected_data_points,
        cardinality_overflows,
    })
}

fn selects_multiple_monitoring_accounts<M>(metrics: &M, fallback_account: &str) -> bool
where
    M: MetricsView,
{
    let mut selected_account = None;
    for resource_metrics in metrics.resources() {
        let resource_account = resource_metrics.resource().map_or_else(
            || fallback_account.as_bytes().to_vec(),
            |resource| resource_monitoring_account(resource.attributes(), fallback_account),
        );

        for scope_metrics in resource_metrics.scopes() {
            for metric in scope_metrics.metrics() {
                let Some(data) = metric.data() else {
                    continue;
                };
                let multiple = match data.value_type() {
                    DataType::Gauge => data.as_gauge().is_some_and(|gauge| {
                        gauge.data_points().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                point.attributes(),
                                &resource_account,
                            )
                        })
                    }),
                    DataType::Sum => data.as_sum().is_some_and(|sum| {
                        sum.data_points().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                point.attributes(),
                                &resource_account,
                            )
                        })
                    }),
                    DataType::Histogram => data.as_histogram().is_some_and(|histogram| {
                        histogram.data_points().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                point.attributes(),
                                &resource_account,
                            )
                        })
                    }),
                    DataType::ExponentialHistogram => {
                        data.as_exponential_histogram().is_some_and(|histogram| {
                            histogram.data_points().any(|point| {
                                records_different_account(
                                    &mut selected_account,
                                    point.attributes(),
                                    &resource_account,
                                )
                            })
                        })
                    }
                    DataType::Summary => data.as_summary().is_some_and(|summary| {
                        summary.data_points().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                point.attributes(),
                                &resource_account,
                            )
                        })
                    }),
                };
                if multiple {
                    return true;
                }
            }
        }
    }
    false
}

fn resource_monitoring_account<A>(
    attributes: impl IntoIterator<Item = A>,
    fallback_account: &str,
) -> Vec<u8>
where
    A: AttributeView,
{
    let attribute = attributes
        .into_iter()
        .filter(|attribute| {
            let key = attribute.key();
            key == ACCOUNT_ATTRIBUTE.as_bytes() || key == PREVIOUS_ACCOUNT_ATTRIBUTE.as_bytes()
        })
        .last();
    let value = attribute.as_ref().and_then(|attribute| attribute.value());
    value
        .as_ref()
        .and_then(|value| value.as_string())
        .filter(|account| !account.is_empty())
        .unwrap_or(fallback_account.as_bytes())
        .to_vec()
}

fn records_different_account<A>(
    selected_account: &mut Option<Vec<u8>>,
    attributes: impl IntoIterator<Item = A>,
    resource_account: &[u8],
) -> bool
where
    A: AttributeView,
{
    let attribute = attributes
        .into_iter()
        .filter(|attribute| attribute.key() == ACCOUNT_ATTRIBUTE.as_bytes())
        .last();
    let value = attribute.as_ref().map(|attribute| attribute.value());
    // A present override with a missing or non-string value selects the empty account.
    let account = match &value {
        Some(value) => value
            .as_ref()
            .and_then(|value| value.as_string())
            .unwrap_or_default(),
        None => resource_account,
    };
    match selected_account {
        Some(selected_account) => selected_account.as_slice() != account,
        None => {
            *selected_account = Some(account.to_vec());
            false
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum PrepareError {
    #[error("Geneva metrics exporter received an unexpected payload")]
    UnsupportedPayload,
    #[error("system clock cannot be represented as an OTLP timestamp")]
    SystemTime,
    #[error("invalid OTLP metrics request: {0}")]
    Decode(#[source] otel_arrow_dfe_pdata::error::Error),
    #[error(
        "Geneva metrics exporter supports one monitoring account per OTLP request, but the request selected more than one"
    )]
    MultipleMonitoringAccounts,
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
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
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

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
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

    /// Scenario: An invalid data point selects a different account from a valid point.
    /// Guarantees: Preparation rejects the request even though mapping omits the invalid point.
    #[test]
    fn rejects_multiple_monitoring_accounts() {
        let account_attribute = |account: &str| KeyValue {
            key: "_microsoft_metrics_account".to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(account.to_string())),
            }),
        };
        let point = |account: &str, flags: u32| NumberDataPoint {
            attributes: vec![account_attribute(account)],
            start_time_unix_nano: 0,
            time_unix_nano: 1_700_000_000_000_000_000,
            exemplars: Vec::new(),
            flags,
            value: Some(number_data_point::Value::AsDouble(12.5)),
        };
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
                            data_points: vec![point("account-a", 0), point("account-b", 1)],
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

        let error = prepare_publication(&data, &(&config).into())
            .expect_err("multiple accounts should be rejected");

        assert!(matches!(error, PrepareError::MultipleMonitoringAccounts));
    }

    /// Scenario: A rejected point inherits a resource account set by the previous attribute name, and a valid point uses another resource account.
    /// Guarantees: Preparation rejects the request because rejected points still take part in single-account routing.
    #[test]
    fn rejects_resource_accounts_selected_by_rejected_points() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![
                gauge_resource(
                    vec![string_attribute(ACCOUNT_ATTRIBUTE, "account-a")],
                    vec![gauge_point(Vec::new(), 0)],
                ),
                gauge_resource(
                    vec![string_attribute(PREVIOUS_ACCOUNT_ATTRIBUTE, "account-b")],
                    vec![gauge_point(Vec::new(), 1)],
                ),
            ],
        };

        let error = prepare_publication(&metrics_pdata(request), &mapping_config())
            .expect_err("multiple resource accounts should be rejected");

        assert!(matches!(error, PrepareError::MultipleMonitoringAccounts));
    }

    /// Scenario: An empty resource account falls back to the configured account, and a rejected point names that account explicitly.
    /// Guarantees: Preparation publishes the valid point because every point, including the rejected one, selects the same account.
    #[test]
    fn accepts_rejected_points_that_select_the_published_account() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![gauge_resource(
                vec![string_attribute(ACCOUNT_ATTRIBUTE, "")],
                vec![
                    gauge_point(Vec::new(), 0),
                    gauge_point(
                        vec![string_attribute(ACCOUNT_ATTRIBUTE, "example-account")],
                        1,
                    ),
                ],
            )],
        };

        let prepared = prepare_publication(&metrics_pdata(request), &mapping_config())
            .expect("a single selected account should prepare");

        let (monitoring_account, _) = prepared
            .publication
            .expect("the valid point should be published");
        assert_eq!(monitoring_account, "example-account");
        assert_eq!(prepared.rejected_data_points, 1);
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
