// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! Defines the necessary service traits that could be used in a test gRPC server to confirm client activity
//!
//! Uses a tokio channel to confirm that the gRPC server has received data from a client
//!

use crate::grpc::OTLPData;
use crate::proto::opentelemetry::collector::{
    logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse, logs_service_server::LogsService,
    },
    metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
        metrics_service_server::MetricsService,
    },
    profiles::v1development::{
        ExportProfilesServiceRequest, ExportProfilesServiceResponse,
        profiles_service_server::ProfilesService,
    },
    trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse, trace_service_server::TraceService,
    },
};
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};

#[cfg(test)]
use crate::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    metrics::v1::{
        Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
        HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary,
        SummaryDataPoint, exemplar::Value as ExemplarValue,
        exponential_histogram_data_point::Buckets, metric::Data,
        number_data_point::Value as NumberValue, summary_data_point::ValueAtQuantile,
    },
    profiles::v1development::{Profile, ResourceProfiles, ScopeProfiles},
    resource::v1::Resource,
    trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status as SpanStatus,
        span::{Event, Link},
    },
};

/// struct that implements the Log Service trait
pub struct LogsServiceMock {
    sender: Sender<OTLPData>,
}

impl LogsServiceMock {
    /// creates a new mock logs service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Metrics Service trait
pub struct MetricsServiceMock {
    sender: Sender<OTLPData>,
}

impl MetricsServiceMock {
    /// creates a new mock metrics service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Trace Service trait
pub struct TraceServiceMock {
    sender: Sender<OTLPData>,
}

impl TraceServiceMock {
    /// creates a new mock trace service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Profiles Service trait
pub struct ProfilesServiceMock {
    sender: Sender<OTLPData>,
}

impl ProfilesServiceMock {
    /// creates a new mock profiles service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl LogsService for LogsServiceMock {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Logs(request.into_inner()))
            .await
            .expect("Logs failed to be sent through channel");
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceMock {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Metrics(request.into_inner()))
            .await
            .expect("Metrics failed to be sent through channel");
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceMock {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Traces(request.into_inner()))
            .await
            .expect("Traces failed to be sent through channel");
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl ProfilesService for ProfilesServiceMock {
    async fn export(
        &self,
        request: Request<ExportProfilesServiceRequest>,
    ) -> Result<Response<ExportProfilesServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Profiles(request.into_inner()))
            .await
            .expect("Profiles failed to be sent through channel");
        Ok(Response::new(ExportProfilesServiceResponse {
            partial_success: None,
        }))
    }
}

#[cfg(test)]
pub(crate) fn create_otlp_metric(
    resource_metrics_count: usize,
    scope_metrics_count: usize,
    metric_count: usize,
    datapoint_count: usize,
) -> ExportMetricsServiceRequest {
    let mut resource_metrics: Vec<ResourceMetrics> = vec![];

    for _ in 0..resource_metrics_count {
        let mut scope_metrics: Vec<ScopeMetrics> = vec![];
        for _ in 0..scope_metrics_count {
            let mut metrics: Vec<Metric> = vec![];
            for metric_index in 0..metric_count {
                let metric_data = if (metric_index + 1) % 5 == 0 {
                    // summary datapoint
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
                } else if (metric_index + 1) % 4 == 0 {
                    // sum datapoint
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
                } else if (metric_index + 1) % 3 == 0 {
                    // histogram datapoint
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
                } else if (metric_index + 1) % 2 == 0 {
                    // exponential histogram datapoint
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
                } else {
                    // gauge datapoint
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
                    name: "library".to_string(),
                    version: "v1".to_string(),
                    attributes: vec![KeyValue {
                        key: "instrumentation_scope_k1".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k1 value".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 5,
                }),
                metrics: metrics.clone(),
            });
        }

        resource_metrics.push(ResourceMetrics {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "ip".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("192.168.0.2".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_metrics: scope_metrics.clone(),
        });
    }

    ExportMetricsServiceRequest {
        resource_metrics: resource_metrics.clone(),
    }
}

#[cfg(test)]
pub(crate) fn create_otlp_trace(
    resource_spans_count: usize,
    scope_spans_count: usize,
    span_count: usize,
    event_count: usize,
    link_count: usize,
) -> ExportTraceServiceRequest {
    let mut resource_spans: Vec<ResourceSpans> = vec![];

    for _ in 0..resource_spans_count {
        let mut scope_spans: Vec<ScopeSpans> = vec![];
        for _ in 0..scope_spans_count {
            let mut spans: Vec<Span> = vec![];
            for _ in 0..span_count {
                let mut links: Vec<Link> = vec![];
                for _ in 0..link_count {
                    links.push(Link {
                        trace_id: Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                        span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                        attributes: vec![KeyValue {
                            key: "hostname".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("host2.org".to_string())),
                            }),
                        }],
                        trace_state: "ended".to_string(),
                        dropped_attributes_count: 0,
                        flags: 4,
                    });
                }
                let mut events: Vec<Event> = vec![];
                for _ in 0..event_count {
                    events.push(Event {
                        time_unix_nano: 1647648000000000108,
                        name: "message-receive".to_string(),
                        attributes: vec![KeyValue {
                            key: "hostname".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("host5.retailer.com".to_string())),
                            }),
                        }],
                        dropped_attributes_count: 0,
                    });
                }
                spans.push(Span {
                    end_time_unix_nano: 1647648000000000104,
                    start_time_unix_nano: 1647648000000000106,
                    name: "user-account".to_string(),
                    kind: 4,
                    trace_state: "ended".to_string(),
                    status: Some(SpanStatus {
                        code: 2,
                        message: "Error".to_string(),
                    }),
                    links: links.clone(),
                    events: events.clone(),
                    attributes: vec![KeyValue {
                        key: "hostname".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("host4.gov".to_string())),
                        }),
                    }],
                    trace_id: Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                    span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                    parent_span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                    dropped_attributes_count: 0,
                    flags: 4,
                    dropped_events_count: 0,
                    dropped_links_count: 0,
                });
            }
            scope_spans.push(ScopeSpans {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: "library".to_string(),
                    version: "v1".to_string(),
                    attributes: vec![KeyValue {
                        key: "hostname".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("host5.retailer.com".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 5,
                }),
                spans: spans.clone(),
            });
        }

        resource_spans.push(ResourceSpans {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "ip".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("192.168.0.1".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_spans: scope_spans.clone(),
        });
    }

    ExportTraceServiceRequest {
        resource_spans: resource_spans.clone(),
    }
}

#[cfg(test)]
pub(crate) fn create_otlp_log(
    resource_logs_count: usize,
    scope_logs_count: usize,
    log_records_count: usize,
) -> ExportLogsServiceRequest {
    let mut resource_logs: Vec<ResourceLogs> = vec![];

    for _ in 0..resource_logs_count {
        let mut scope_logs: Vec<ScopeLogs> = vec![];
        for _ in 0..scope_logs_count {
            let mut log_records: Vec<LogRecord> = vec![];
            for _ in 0..log_records_count {
                log_records.push(LogRecord {
                    time_unix_nano: 2_000_000_000,
                    observed_time_unix_nano: 1663718400000001300,
                    severity_text: "INFO".to_string(),
                    severity_number: 2,
                    event_name: "event1".to_string(),
                    attributes: vec![KeyValue {
                        key: "hostname".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("host3.thedomain.edu".to_string())),
                        }),
                    }],
                    trace_id: Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                    span_id: Vec::from("7271ee06d7e5925f".as_bytes()),
                    body: Some(AnyValue {
                        value: Some(Value::StringValue(
                            "Sint impedit non ut eligendi nisi neque harum maxime adipisci."
                                .to_string(),
                        )),
                    }),
                    flags: 8,
                    dropped_attributes_count: 0,
                });
            }
            scope_logs.push(ScopeLogs {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: "library".to_string(),
                    version: "v1".to_string(),
                    attributes: vec![KeyValue {
                        key: "hostname".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("host5.retailer.com".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 5,
                }),
                log_records: log_records.clone(),
            });
        }

        resource_logs.push(ResourceLogs {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "version".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("2.0".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_logs: scope_logs.clone(),
        });
    }

    ExportLogsServiceRequest {
        resource_logs: resource_logs.clone(),
    }
}

#[cfg(test)]
pub(crate) fn create_otlp_profile(
    resource_profiles_count: usize,
    scope_profiles_count: usize,
    profile_count: usize,
) -> ExportProfilesServiceRequest {
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
                    name: "library".to_string(),
                    version: "v1".to_string(),
                    attributes: vec![KeyValue {
                        key: "hostname".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("host5.retailer.com".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 5,
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

    ExportProfilesServiceRequest {
        resource_profiles: resource_profiles.clone(),
    }
}
