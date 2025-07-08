// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!

use crate::fake_signal_receiver::config::MetricType;
use crate::fake_signal_receiver::fake_data::*;
use crate::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
    metrics::v1::{
        Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
        HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        Sum, Summary, SummaryDataPoint, exemplar::Value as ExemplarValue,
        exponential_histogram_data_point::Buckets, metric::Data,
        number_data_point::Value as NumberValue, summary_data_point::ValueAtQuantile,
    },
    profiles::v1development::{Profile, ProfilesData, ResourceProfiles, ScopeProfiles},
    resource::v1::Resource,
    trace::v1::{
        ResourceSpans, ScopeSpans, Span, TracesData,
        span::{Event, Link},
    },
};

/// Genererates MetricsData based on the provided count for resource, scope, metric, datapoint count
#[must_use]
pub fn fake_otlp_metrics(
    resource_metrics_count: usize,
    scope_metrics_count: usize,
    metric_count: usize,
    datapoint_count: usize,
    datapoint_type: MetricType,
) -> MetricsData {
    let mut resource_metrics: Vec<ResourceMetrics> = vec![];

    for _ in 0..resource_metrics_count {
        let mut scope_metrics: Vec<ScopeMetrics> = vec![];
        for _ in 0..scope_metrics_count {
            // create some fake metrics based on the metric_count value provided
            let mut metrics: Vec<Metric> = vec![];
            for _ in 0..metric_count {
                let metric_data = match datapoint_type {
                    MetricType::Gauge => fake_gauge(datapoint_count),
                    MetricType::Sum => fake_sum(datapoint_count),
                    MetricType::Histogram => fake_histogram(datapoint_count),
                    MetricType::ExponentialHistogram => fake_exp_histogram(datapoint_count),
                    MetricType::Summary => fake_summary(datapoint_count),
                };

                metrics.push(Metric {
                    name: "system.cpu.time".to_string(),
                    description: "time cpu has ran".to_string(),
                    unit: "s".to_string(),
                    metadata: vec![],
                    data: Some(metric_data),
                });
            }
            scope_metrics.push(ScopeMetrics {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: get_scope_name(),
                    version: get_scope_version(),
                    attributes: get_attributes(),
                    dropped_attributes_count: 0,
                }),
                metrics: metrics.clone(),
            });
        }

        resource_metrics.push(ResourceMetrics {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: get_attributes(),
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_metrics: scope_metrics.clone(),
        });
    }

    MetricsData {
        resource_metrics: resource_metrics.clone(),
    }
}

/// Genererates TracesData based on the provided count for resource, scope, span, event, link count
#[must_use]
pub fn fake_otlp_traces(
    resource_spans_count: usize,
    scope_spans_count: usize,
    span_count: usize,
    event_count: usize,
    link_count: usize,
) -> TracesData {
    let mut resource_spans: Vec<ResourceSpans> = vec![];

    for _ in 0..resource_spans_count {
        let mut scope_spans: Vec<ScopeSpans> = vec![];
        for _ in 0..scope_spans_count {
            let mut spans: Vec<Span> = vec![];
            for _ in 0..span_count {
                let mut links: Vec<Link> = vec![];
                for _ in 0..link_count {
                    links.push(Link {
                        trace_id: get_trace_id(),
                        span_id: get_span_id(),
                        attributes: get_attributes(),
                        trace_state: get_trace_state(),
                        dropped_attributes_count: 0,
                        flags: 4,
                    });
                }
                let mut events: Vec<Event> = vec![];
                for _ in 0..event_count {
                    events.push(Event {
                        time_unix_nano: get_time_unix_nano(),
                        name: get_event_name(),
                        attributes: get_attributes(),
                        dropped_attributes_count: 0,
                    });
                }
                spans.push(Span {
                    end_time_unix_nano: get_end_time_unix_nano(),
                    start_time_unix_nano: get_start_time_unix_nano(),
                    name: get_span_name(),
                    kind: 4,
                    trace_state: get_trace_state(),
                    status: get_status(),
                    links: links.clone(),
                    events: events.clone(),
                    attributes: get_attributes(),
                    trace_id: get_trace_id(),
                    span_id: get_span_id(),
                    parent_span_id: vec![],
                    dropped_attributes_count: 0,
                    flags: 4,
                    dropped_events_count: 0,
                    dropped_links_count: 0,
                });
            }
            scope_spans.push(ScopeSpans {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: get_scope_name(),
                    version: get_scope_version(),
                    attributes: get_attributes(),
                    dropped_attributes_count: 0,
                }),
                spans: spans.clone(),
            });
        }

        resource_spans.push(ResourceSpans {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: get_attributes(),
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_spans: scope_spans.clone(),
        });
    }

    TracesData {
        resource_spans: resource_spans.clone(),
    }
}

/// Genererates LogsData based on the provided count for resource logs, scope logs and log records
#[must_use]
pub fn fake_otlp_logs(
    resource_logs_count: usize,
    scope_logs_count: usize,
    log_records_count: usize,
) -> LogsData {
    let mut resource_logs: Vec<ResourceLogs> = vec![];

    for _ in 0..resource_logs_count {
        let mut scope_logs: Vec<ScopeLogs> = vec![];
        for _ in 0..scope_logs_count {
            let mut log_records: Vec<LogRecord> = vec![];
            for _ in 0..log_records_count {
                log_records.push(LogRecord {
                    time_unix_nano: get_time_unix_nano(),
                    observed_time_unix_nano: get_observed_time_unix_nano(),
                    severity_text: get_severity_text(),
                    severity_number: 2,
                    event_name: get_event_name(),
                    attributes: get_attributes(),
                    trace_id: get_trace_id(),
                    span_id: get_span_id(),
                    body: get_body_text(),
                    flags: 8,
                    dropped_attributes_count: 0,
                });
            }
            scope_logs.push(ScopeLogs {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: "library".to_string(),
                    version: "v1".to_string(),
                    attributes: get_attributes(),
                    dropped_attributes_count: 0,
                }),
                log_records: log_records.clone(),
            });
        }
        resource_logs.push(ResourceLogs {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: get_attributes(),
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_logs: scope_logs.clone(),
        });
    }

    LogsData {
        resource_logs: resource_logs.clone(),
    }
}

/// Genererates LogsData based on the provided count for resource, scope, profile count
#[must_use]
pub fn fake_otlp_profiles(
    resource_profiles_count: usize,
    scope_profiles_count: usize,
    profile_count: usize,
) -> ProfilesData {
    let mut resource_profiles: Vec<ResourceProfiles> = vec![];
    for _ in 0..resource_profiles_count {
        let mut scope_profiles: Vec<ScopeProfiles> = vec![];
        for _ in 0..scope_profiles_count {
            let mut profiles: Vec<Profile> = vec![];
            for _ in 0..profile_count {
                profiles.push(Profile {
                    sample_type: vec![],
                    sample: vec![],
                    location_indices: vec![],
                    time_nanos: 0,
                    duration_nanos: 0,
                    period_type: None,
                    period: 0,
                    comment_strindices: vec![],
                    default_sample_type_index: 0,
                    profile_id: vec![],
                    dropped_attributes_count: 0,
                    original_payload: vec![],
                    original_payload_format: "".to_string(),
                    attribute_indices: vec![],
                });
            }

            scope_profiles.push(ScopeProfiles {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: get_scope_name(),
                    version: get_scope_version(),
                    attributes: get_attributes(),
                    dropped_attributes_count: 0,
                }),
                profiles: profiles.clone(),
            });
        }

        resource_profiles.push(ResourceProfiles {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "hostname".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("host7.com".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_profiles: scope_profiles.clone(),
        });
    }

    ProfilesData {
        resource_profiles: resource_profiles.clone(),
        attribute_table: vec![],
        attribute_units: vec![],
        function_table: vec![],
        link_table: vec![],
        location_table: vec![],
        mapping_table: vec![],
        string_table: vec![],
    }
}

// Helper functions for generating datapoints for metrics

/// generate gauge datapoints
#[must_use]
fn fake_gauge(datapoint_count: usize) -> Data {
    let mut datapoints = vec![];
    for datapoint in 0..datapoint_count {
        datapoints.push(NumberDataPoint {
            start_time_unix_nano: 1650499200000000100,
            time_unix_nano: 1663718400000001400,
            attributes: vec![],
            value: Some(NumberValue::AsInt(datapoint as i64)),
            flags: 0,
            exemplars: vec![],
        });
    }
    Data::Gauge(Gauge {
        data_points: datapoints.clone(),
    })
}

/// generate histogram datapoints
#[must_use]
fn fake_histogram(datapoint_count: usize) -> Data {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(HistogramDataPoint {
            attributes: vec![KeyValue {
                key: "freq".to_string(),
                value: Some(AnyValue {
                    value: Some(Value::StringValue("3GHz".to_string())),
                }),
            }],
            start_time_unix_nano: 1650499200000000000,
            time_unix_nano: 1663718400000001400,
            explicit_bounds: vec![94.17542094619048, 65.66722851519177],
            bucket_counts: vec![0],
            sum: Some(56.0),
            count: 0,
            flags: 0,
            min: Some(12.0),
            max: Some(100.1),
            exemplars: vec![Exemplar {
                time_unix_nano: 1663718400000001400,
                span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                trace_id: Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                value: Some(ExemplarValue::AsDouble(22.2)),
                filtered_attributes: vec![KeyValue {
                    key: "cpu".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::IntValue(0)),
                    }),
                }],
            }],
        });
    }

    Data::Histogram(Histogram {
        data_points: datapoints.clone(),
        aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
    })
}

/// generate exponential histogram datapoints
#[must_use]
fn fake_exp_histogram(datapoint_count: usize) -> Data {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(ExponentialHistogramDataPoint {
            attributes: vec![KeyValue {
                key: "freq".to_string(),
                value: Some(AnyValue {
                    value: Some(Value::StringValue("3GHz".to_string())),
                }),
            }],
            start_time_unix_nano: 1650499200000000000,
            time_unix_nano: 1663718400000001400,
            sum: Some(56.0),
            count: 0,
            flags: 0,
            min: Some(12.0),
            max: Some(100.1),
            exemplars: vec![Exemplar {
                time_unix_nano: 1663718400000001400,
                span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                trace_id: Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                value: Some(ExemplarValue::AsDouble(22.2)),
                filtered_attributes: vec![KeyValue {
                    key: "cpu".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::IntValue(0)),
                    }),
                }],
            }],
            scale: 1,
            positive: Some(Buckets {
                offset: 0,
                bucket_counts: vec![0],
            }),
            negative: Some(Buckets {
                offset: 0,
                bucket_counts: vec![0],
            }),
            zero_threshold: 0.0,
            zero_count: 0,
        });
    }

    Data::ExponentialHistogram(ExponentialHistogram {
        data_points: datapoints.clone(),
        aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
    })
}

/// generate summary datapoints
#[must_use]
fn fake_summary(datapoint_count: usize) -> Data {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(SummaryDataPoint {
            start_time_unix_nano: 1650499200000000100,
            time_unix_nano: 1663718400000001400,
            attributes: vec![KeyValue {
                key: "cpu_cores".to_string(),
                value: Some(AnyValue {
                    value: Some(Value::StringValue("4".to_string())),
                }),
            }],
            sum: 56.0,
            count: 0,
            flags: 0,
            quantile_values: vec![ValueAtQuantile {
                quantile: 0.0,
                value: 0.0,
            }],
        });
    }
    Data::Summary(Summary {
        data_points: datapoints.clone(),
    })
}

/// generate sum datapoints
#[must_use]
fn fake_sum(datapoint_count: usize) -> Data {
    let mut datapoints = vec![];
    for datapoint in 0..datapoint_count {
        datapoints.push(NumberDataPoint {
            start_time_unix_nano: 1650499200000000000,
            time_unix_nano: 1663718400000001400,
            attributes: vec![KeyValue {
                key: "cpu_logical_processors".to_string(),
                value: Some(AnyValue {
                    value: Some(Value::StringValue("8".to_string())),
                }),
            }],
            value: Some(NumberValue::AsInt(datapoint as i64)),
            flags: 0,
            exemplars: vec![Exemplar {
                time_unix_nano: 1663718400000001400,
                span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                trace_id: Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                value: Some(ExemplarValue::AsDouble(22.2)),
                filtered_attributes: vec![KeyValue {
                    key: "************".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::BoolValue(true)),
                    }),
                }],
            }],
        });
    }

    Data::Sum(Sum {
        data_points: datapoints.clone(),
        aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
        is_monotonic: true,
    })
}
