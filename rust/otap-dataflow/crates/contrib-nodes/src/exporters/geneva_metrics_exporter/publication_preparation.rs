// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::encoder;
use super::otlp_to_geneva::{self, ACCOUNT_ATTRIBUTE, Config, PREVIOUS_ACCOUNT_ATTRIBUTE};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest as OtlpMetricsRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue as OtlpKeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric as otlp_metric;
use otel_arrow_dfe_pdata::{OtlpProtoBytes, PayloadData};
use prost::Message as _;
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
    let request =
        OtlpMetricsRequest::decode(bytes.as_ref()).map_err(otlp_to_geneva::MappingError::Decode)?;
    let mapped = otlp_to_geneva::map_request(&request, config, receive_time)?;
    let rejected_data_points = mapped.rejected_data_points;
    let cardinality_overflows = mapped.cardinality_overflows.len();
    let publication_count = mapped.publications.len();
    // Recheck routing only for partial mappings because rejected points do not create publications.
    if publication_count > 1
        || (rejected_data_points > 0
            && selects_multiple_monitoring_accounts(&request, &config.monitoring_account))
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

fn selects_multiple_monitoring_accounts<'a>(
    request: &'a OtlpMetricsRequest,
    fallback_account: &'a str,
) -> bool {
    let mut selected_account = None;
    for resource_metrics in &request.resource_metrics {
        let resource_attributes = resource_metrics
            .resource
            .as_ref()
            .map_or(&[][..], |resource| resource.attributes.as_slice());
        let resource_account = resource_monitoring_account(resource_attributes, fallback_account);

        for scope_metrics in &resource_metrics.scope_metrics {
            for metric in &scope_metrics.metrics {
                let Some(data) = &metric.data else {
                    continue;
                };
                let multiple = match data {
                    otlp_metric::Data::Gauge(gauge) => gauge.data_points.iter().any(|point| {
                        records_different_account(
                            &mut selected_account,
                            &point.attributes,
                            resource_account,
                        )
                    }),
                    otlp_metric::Data::Sum(sum) => sum.data_points.iter().any(|point| {
                        records_different_account(
                            &mut selected_account,
                            &point.attributes,
                            resource_account,
                        )
                    }),
                    otlp_metric::Data::Histogram(histogram) => {
                        histogram.data_points.iter().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                &point.attributes,
                                resource_account,
                            )
                        })
                    }
                    otlp_metric::Data::ExponentialHistogram(histogram) => {
                        histogram.data_points.iter().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                &point.attributes,
                                resource_account,
                            )
                        })
                    }
                    otlp_metric::Data::Summary(summary) => {
                        summary.data_points.iter().any(|point| {
                            records_different_account(
                                &mut selected_account,
                                &point.attributes,
                                resource_account,
                            )
                        })
                    }
                };
                if multiple {
                    return true;
                }
            }
        }
    }
    false
}

fn resource_monitoring_account<'a>(
    attributes: &'a [OtlpKeyValue],
    fallback_account: &'a str,
) -> &'a str {
    attributes
        .iter()
        .rev()
        .find(|attribute| {
            attribute.key == ACCOUNT_ATTRIBUTE || attribute.key == PREVIOUS_ACCOUNT_ATTRIBUTE
        })
        .map(routing_value)
        .filter(|account| !account.is_empty())
        .unwrap_or(fallback_account)
}

fn records_different_account<'a>(
    selected_account: &mut Option<&'a str>,
    attributes: &'a [OtlpKeyValue],
    resource_account: &'a str,
) -> bool {
    let account = attributes
        .iter()
        .rev()
        .find(|attribute| attribute.key == ACCOUNT_ATTRIBUTE)
        .map_or(resource_account, routing_value);
    match selected_account {
        Some(selected_account) => *selected_account != account,
        None => {
            *selected_account = Some(account);
            false
        }
    }
}

fn routing_value(attribute: &OtlpKeyValue) -> &str {
    attribute
        .value
        .as_ref()
        .and_then(|value| match value.value.as_ref() {
            Some(any_value::Value::StringValue(value)) => Some(value.as_str()),
            _ => None,
        })
        .unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
pub(super) enum PrepareError {
    #[error("Geneva metrics exporter received an unexpected payload")]
    UnsupportedPayload,
    #[error("system clock cannot be represented as an OTLP timestamp")]
    SystemTime,
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
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };

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
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
            AnyValue, KeyValue, any_value,
        };

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

        let error = prepare_publication(&data, &(&config).into())
            .expect_err("multiple accounts should be rejected");

        assert!(matches!(error, PrepareError::MultipleMonitoringAccounts));
    }
}
