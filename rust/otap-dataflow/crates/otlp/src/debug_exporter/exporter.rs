// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP Debug exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuratin changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::LOCAL_EXPORTERS;
use crate::debug_exporter::config::{Config, Verbosity};
use crate::debug_exporter::marshaler::{
    DetailedOTLPMarshaler, NormalOTLPMarshaler, PDataMarshaler,
};
use crate::grpc::OTLPData;
use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    metrics::v1::metric::Data,
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_engine::error::Error;
use otap_df_engine::local::{LocalExporterFactory, exporter as local};
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
/// Exporter that outputs all data received to stdout
struct DebugExporter {
    config: Config,
    output: Option<String>,
}

/// Declares the Debug exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(LOCAL_EXPORTERS)]
pub static DEBUG_EXPORTER: LocalExporterFactory<OTLPData> = LocalExporterFactory {
    name: "urn:otel:debug:exporter",
    create: |config: &Value| Box::new(DebugExporter::from_config(config)),
};

impl DebugExporter {
    /// Creates a new Debug exporter
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, output: Option<String>) -> Self {
        DebugExporter { config, output }
    }

    // Creates a new DebugExporter from a configuration object
    #[must_use]
    pub fn from_config(_config: &Value) -> Self {
        // ToDo: implement config parsing
        DebugExporter {
            config: Config::default(),
            output: None,
        }
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OTLPData> for DebugExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPData>,
        effect_handler: local::EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        let mut metric_count: u64 = 0;
        let mut profile_count: u64 = 0;
        let mut span_count: u64 = 0;
        let mut log_count: u64 = 0;
        let marshaler: Box<dyn PDataMarshaler> = if self.config.verbosity() == Verbosity::Normal {
            Box::new(NormalOTLPMarshaler)
        } else {
            Box::new(DetailedOTLPMarshaler)
        };
        let mut writer = get_writer(self.output);
        // Loop until a Shutdown event is received.
        loop {
            match msg_chan.recv().await? {
                // handle control messages
                Message::Control(ControlMsg::TimerTick { .. }) => {
                    _ = writeln!(writer, "Timer tick received");

                    // output count of messages received
                    _ = writeln!(writer, "Count of metrics objects received {}", metric_count);
                    _ = writeln!(writer, "Count of spans objects received {}", span_count);
                    _ = writeln!(
                        writer,
                        "Count of profiles objects received {}",
                        profile_count
                    );
                    _ = writeln!(writer, "Count of logs objects received {}", log_count);

                    metric_count = 0;
                    span_count = 0;
                    log_count = 0;
                    profile_count = 0;
                }
                Message::Control(ControlMsg::Config { .. }) => {
                    _ = writeln!(writer, "Config message received");
                }
                // shutdown the exporter
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    // ToDo: add proper deadline function
                    _ = writeln!(writer, "Shutdown message received");
                    break;
                }
                //send data
                Message::PData(message) => {
                    match message {
                        // match on OTLPData type and use the respective client to send message
                        // ToDo: Add Ack/Nack handling, send a signal that data has been exported
                        // check what message
                        OTLPData::Metrics(req) => {
                            push_metric(&self.config.verbosity(), req, &marshaler, &mut writer);
                            metric_count += 1;
                        }
                        OTLPData::Logs(req) => {
                            push_log(&self.config.verbosity(), req, &marshaler, &mut writer);
                            log_count += 1;
                        }
                        OTLPData::Traces(req) => {
                            push_trace(&self.config.verbosity(), req, &marshaler, &mut writer);
                            span_count += 1;
                        }
                        OTLPData::Profiles(req) => {
                            //   push_profile(&self.verbosity, req, &marshaler, &writer);
                            //   profile_count += 1;
                        }
                    }
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// determine if output goes to console or to a file
fn get_writer(output_file: Option<String>) -> Box<dyn Write> {
    match output_file {
        Some(file_name) => Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_name)
                .expect("could not open output file"),
        ),
        None => Box::new(std::io::stdout()),
    }
}

fn push_metric(
    verbosity: &Verbosity,
    metric_request: ExportMetricsServiceRequest,
    marshaler: &Box<dyn PDataMarshaler>,
    writer: &mut impl Write,
) {
    // collect number of resource metrics
    // collect number of metrics
    // collect number of datapoints
    let resouce_metrics = metric_request.resource_metrics.len();
    let mut data_points = 0;
    let mut metrics = 0;
    for resource_metrics in &metric_request.resource_metrics {
        for scope_metrics in &resource_metrics.scope_metrics {
            metrics += scope_metrics.metrics.len();
            for metric in &scope_metrics.metrics {
                if let Some(data) = &metric.data {
                    match data {
                        Data::Gauge(gauge) => {
                            data_points += gauge.data_points.len();
                        }
                        Data::Sum(sum) => {
                            data_points += sum.data_points.len();
                        }
                        Data::Histogram(histogram) => {
                            data_points += histogram.data_points.len();
                        }
                        Data::ExponentialHistogram(exponential_histogram) => {
                            data_points += exponential_histogram.data_points.len();
                        }
                        Data::Summary(summary) => {
                            data_points += summary.data_points.len();
                        }
                    }
                }
            }
        }
    }

    _ = writeln!(writer, "Received {} resource metrics", resouce_metrics);
    _ = writeln!(writer, "Received {} metrics", metrics);
    _ = writeln!(writer, "Received {} data points", data_points);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_metrics(metric_request);
    _ = writeln!(writer, "{}", report);
    return;
}

fn push_trace(
    verbosity: &Verbosity,
    trace_request: ExportTraceServiceRequest,
    marshaler: &Box<dyn PDataMarshaler>,
    writer: &mut impl Write,
) {
    // collect number of resource spans
    // collect number of spans
    let resource_spans = trace_request.resource_spans.len();
    let mut spans = 0;
    let mut events = 0;
    let mut links = 0;
    for resource_span in &trace_request.resource_spans {
        for scope_span in &resource_span.scope_spans {
            spans += scope_span.spans.len();
            for span in &scope_span.spans {
                events += span.events.len();
                links += span.links.len();
            }
        }
    }

    _ = writeln!(writer, "Received {} resource spans", resource_spans);
    _ = writeln!(writer, "Received {} spans", spans);
    _ = writeln!(writer, "Received {} events", events);
    _ = writeln!(writer, "Received {} links", links);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_traces(trace_request);
    _ = writeln!(writer, "{}", report);

    return;
}

fn push_log(
    verbosity: &Verbosity,
    log_request: ExportLogsServiceRequest,
    marshaler: &Box<dyn PDataMarshaler>,
    writer: &mut impl Write,
) {
    let resource_logs = log_request.resource_logs.len();
    let mut log_records = 0;
    let mut events = 0;
    for resource_log in &log_request.resource_logs {
        for scope_log in &resource_log.scope_logs {
            log_records += scope_log.log_records.len();
            for log_record in &scope_log.log_records {
                if !log_record.event_name.is_empty() {
                    events += 1;
                }
            }
        }
    }
    _ = writeln!(writer, "Received {} resource logs", resource_logs);
    _ = writeln!(writer, "Received {} log records", log_records);
    _ = writeln!(writer, "Received {} events", events);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_logs(log_request);
    _ = writeln!(writer, "{}", report);

    return;
}

fn push_profile(
    verbosity: &Verbosity,
    profile_request: ExportProfilesServiceRequest,
    marshaler: &dyn PDataMarshaler,
    writer: &mut impl Write,
) {
    // collect number of resource profiles
    // collect number of sample records
    let resource_profiles = profile_request.resource_profiles.len();
    let mut samples = 0;
    for resource_profile in &profile_request.resource_profiles {
        for scope_profile in &resource_profile.scope_profiles {
            for profile in &scope_profile.profiles {
                samples += profile.sample.len();
            }
        }
    }

    _ = writeln!(writer, "Received {} resource profiles, ", resource_profiles);
    _ = writeln!(writer, "Received {} samples", samples);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_profiles(profile_request);
    _ = writeln!(writer, "{}", report);

    return;
}

#[cfg(test)]
mod tests {

    use crate::debug_exporter::config::{Config, Verbosity};
    use crate::debug_exporter::exporter::DebugExporter;
    use crate::grpc::OTLPData;
    use crate::proto::opentelemetry::{
        collector::{
            logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
            profiles::v1development::ExportProfilesServiceRequest,
            trace::v1::ExportTraceServiceRequest,
        },
        common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        metrics::v1::{
            Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
            HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
            Summary, SummaryDataPoint, exemplar::Value as ExemplarValue,
            exponential_histogram_data_point::Buckets, metric::Data,
            number_data_point::Value as NumberValue, summary_data_point::ValueAtQuantile,
        },
        profiles::v1development::{Profile, ResourceProfiles, ScopeProfiles},
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span, Status,
            span::{Event, Link},
        },
    };

    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use tokio::time::{Duration, sleep};

    use std::fs::{File, remove_file};
    use std::io::{BufReader, prelude::*};

    fn create_otlp_metric(
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
                    let metric_data = if metric_index % 2 == 0 {
                        // summary datapoint
                        let mut datapoints = vec![];
                        for datapoint in 0..datapoint_count {
                            datapoints.push(SummaryDataPoint {
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_000_000_000,
                                attributes: vec![KeyValue {
                                    key: "datapoint_k1".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue("k1 value".to_string())),
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
                    } else if metric_index % 3 == 0 {
                        // sum datapoint
                        let mut datapoints = vec![];
                        for datapoint in 0..datapoint_count {
                            datapoints.push(NumberDataPoint {
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_000_000_000,
                                attributes: vec![KeyValue {
                                    key: "datapoint_k1".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue("k1 value".to_string())),
                                    }),
                                }],
                                value: Some(NumberValue::AsInt(datapoint as i64)),
                                flags: 0,
                                exemplars: vec![Exemplar {
                                    time_unix_nano: 1_000_000_000,
                                    span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                                    trace_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
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
                    } else if metric_index % 5 == 0 {
                        // histogram datapoint
                        let mut datapoints = vec![];
                        for _ in 0..datapoint_count {
                            datapoints.push(HistogramDataPoint {
                                attributes: vec![KeyValue {
                                    key: "datapoint_k1".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue("k1 value".to_string())),
                                    }),
                                }],
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_000_000_000,
                                explicit_bounds: vec![],
                                bucket_counts: vec![1, 2],
                                sum: Some(56.0),
                                count: 0,
                                flags: 0,
                                min: Some(12.0),
                                max: Some(100.1),
                                exemplars: vec![Exemplar {
                                    time_unix_nano: 1_000_000_000,
                                    span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                                    trace_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
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

                        Data::Histogram(Histogram {
                            data_points: datapoints.clone(),
                            aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
                        })
                    } else if metric_index % 7 == 0 {
                        // exponential histogram datapoint
                        let mut datapoints = vec![];
                        for _ in 0..datapoint_count {
                            datapoints.push(ExponentialHistogramDataPoint {
                                attributes: vec![KeyValue {
                                    key: "datapoint_k1".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue("k1 value".to_string())),
                                    }),
                                }],
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_000_000_000,
                                sum: Some(56.0),
                                count: 0,
                                flags: 0,
                                min: Some(12.0),
                                max: Some(100.1),
                                exemplars: vec![Exemplar {
                                    time_unix_nano: 1_000_000_000,
                                    span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                                    trace_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                                    value: Some(ExemplarValue::AsDouble(22.2)),
                                    filtered_attributes: vec![KeyValue {
                                        key: "************".to_string(),
                                        value: Some(AnyValue {
                                            value: Some(Value::BoolValue(true)),
                                        }),
                                    }],
                                }],
                                scale: 1,
                                positive: Some(Buckets {
                                    offset: 0,
                                    bucket_counts: vec![0, 0, 0],
                                }),
                                negative: Some(Buckets {
                                    offset: 0,
                                    bucket_counts: vec![0, 0, 0],
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
                                start_time_unix_nano: 0,
                                time_unix_nano: 1_000_000_000,
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
                        name: "metric name".to_string(),
                        description: "metric description".to_string(),
                        unit: "metric unit".to_string(),
                        metadata: vec![],
                        data: Some(metric_data),
                    });
                }
                let mut instrumentation_scope_attributes = vec![
                    KeyValue {
                        key: "instrumentation_scope_k1".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k1 value".to_string())),
                        }),
                    },
                    KeyValue {
                        key: "instrumentation_scope_k2".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k2 value".to_string())),
                        }),
                    },
                ];
                scope_metrics.push(ScopeMetrics {
                    schema_url: "http://schema.opentelemetry.io".to_string(),
                    scope: Some(InstrumentationScope {
                        name: "library".to_string(),
                        version: "v1".to_string(),
                        attributes: instrumentation_scope_attributes.clone(),
                        dropped_attributes_count: 5,
                    }),
                    metrics: metrics.clone(),
                });
            }

            let mut resource_metrics_attributes = vec![
                KeyValue {
                    key: "k1".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k1 value".to_string())),
                    }),
                },
                KeyValue {
                    key: "k2".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k2 value".to_string())),
                    }),
                },
            ];

            resource_metrics.push(ResourceMetrics {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                resource: Some(Resource {
                    attributes: resource_metrics_attributes.clone(),
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

    fn create_otlp_trace(
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
                    let mut span_attributes = vec![
                        KeyValue {
                            key: "span_attribute_key1".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("k1 value".to_string())),
                            }),
                        },
                        KeyValue {
                            key: "span_attribute_key2".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("k2 value".to_string())),
                            }),
                        },
                    ];
                    let mut links: Vec<Link> = vec![];
                    for _ in 0..link_count {
                        links.push(Link {
                            trace_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                            span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                            attributes: vec![KeyValue {
                                key: "link_attribute_key1".to_string(),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue("k1 value".to_string())),
                                }),
                            }],
                            trace_state: "trace states".to_string(),
                            dropped_attributes_count: 0,
                            flags: 4,
                        });
                    }
                    let mut events: Vec<Event> = vec![];
                    for _ in 0..event_count {
                        events.push(Event {
                            time_unix_nano: 2_000_000_000,
                            name: "event name".to_string(),
                            attributes: vec![KeyValue {
                                key: "event_attribute_key1".to_string(),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue("k1 value".to_string())),
                                }),
                            }],
                            dropped_attributes_count: 0,
                        });
                    }
                    spans.push(Span {
                        end_time_unix_nano: 2_000_000_000,
                        start_time_unix_nano: 1_000_000,
                        name: "trace name".to_string(),
                        kind: 4,
                        trace_state: "trace states".to_string(),
                        status: Some(Status {
                            code: 3,
                            message: "status_message".to_string(),
                        }),
                        links: links.clone(),
                        events: events.clone(),
                        attributes: span_attributes.clone(),
                        trace_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                        span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                        parent_span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                        dropped_attributes_count: 0,
                        flags: 4,
                        dropped_events_count: 0,
                        dropped_links_count: 0,
                    });
                }
                let mut instrumentation_scope_attributes = vec![
                    KeyValue {
                        key: "instrumentation_scope_k1".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k1 value".to_string())),
                        }),
                    },
                    KeyValue {
                        key: "instrumentation_scope_k2".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k2 value".to_string())),
                        }),
                    },
                ];
                scope_spans.push(ScopeSpans {
                    schema_url: "http://schema.opentelemetry.io".to_string(),
                    scope: Some(InstrumentationScope {
                        name: "library".to_string(),
                        version: "v1".to_string(),
                        attributes: instrumentation_scope_attributes.clone(),
                        dropped_attributes_count: 5,
                    }),
                    spans: spans.clone(),
                });
            }

            let mut resource_spans_attributes = vec![
                KeyValue {
                    key: "k1".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k1 value".to_string())),
                    }),
                },
                KeyValue {
                    key: "k2".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k2 value".to_string())),
                    }),
                },
            ];

            resource_spans.push(ResourceSpans {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                resource: Some(Resource {
                    attributes: resource_spans_attributes.clone(),
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

    fn create_otlp_log(
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
                    let mut log_records_attributes = vec![
                        KeyValue {
                            key: "log_attribute_key1".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("k1 value".to_string())),
                            }),
                        },
                        KeyValue {
                            key: "log_attribute_key2".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("k2 value".to_string())),
                            }),
                        },
                    ];

                    log_records.push(LogRecord {
                        time_unix_nano: 2_000_000_000,
                        observed_time_unix_nano: 1_000_000,
                        severity_text: "Severity info".to_string(),
                        severity_number: 2,
                        event_name: "event1".to_string(),
                        attributes: log_records_attributes.clone(),
                        trace_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                        span_id: Vec::from("EEE19B7EC3C1B174".as_bytes()),
                        body: Some(AnyValue {
                            value: Some(Value::StringValue("log_body".to_string())),
                        }),
                        flags: 8,
                        dropped_attributes_count: 0,
                    });
                }
                let mut instrumentation_scope_attributes = vec![
                    KeyValue {
                        key: "instrumentation_scope_k1".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k1 value".to_string())),
                        }),
                    },
                    KeyValue {
                        key: "instrumentation_scope_k2".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k2 value".to_string())),
                        }),
                    },
                ];
                scope_logs.push(ScopeLogs {
                    schema_url: "http://schema.opentelemetry.io".to_string(),
                    scope: Some(InstrumentationScope {
                        name: "library".to_string(),
                        version: "v1".to_string(),
                        attributes: instrumentation_scope_attributes.clone(),
                        dropped_attributes_count: 5,
                    }),
                    log_records: log_records.clone(),
                });
            }

            let mut resource_logs_attributes = vec![
                KeyValue {
                    key: "k1".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k1 value".to_string())),
                    }),
                },
                KeyValue {
                    key: "k2".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k2 value".to_string())),
                    }),
                },
            ];

            resource_logs.push(ResourceLogs {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                resource: Some(Resource {
                    attributes: resource_logs_attributes.clone(),
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

    fn create_otlp_profile(
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
                    let mut profile_attributes = vec![
                        KeyValue {
                            key: "profile_attribute_key1".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("k1 value".to_string())),
                            }),
                        },
                        KeyValue {
                            key: "profile_attribute_key2".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("k2 value".to_string())),
                            }),
                        },
                    ];

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
                let mut instrumentation_scope_attributes = vec![
                    KeyValue {
                        key: "instrumentation_scope_k1".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k1 value".to_string())),
                        }),
                    },
                    KeyValue {
                        key: "instrumentation_scope_k2".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("k2 value".to_string())),
                        }),
                    },
                ];
                scope_profiles.push(ScopeProfiles {
                    schema_url: "http://schema.opentelemetry.io".to_string(),
                    scope: Some(InstrumentationScope {
                        name: "library".to_string(),
                        version: "v1".to_string(),
                        attributes: instrumentation_scope_attributes.clone(),
                        dropped_attributes_count: 5,
                    }),
                    profiles: profiles.clone(),
                });
            }

            let mut resource_profiles_attributes = vec![
                KeyValue {
                    key: "k1".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k1 value".to_string())),
                    }),
                },
                KeyValue {
                    key: "k2".to_string(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("k2 value".to_string())),
                    }),
                },
            ];

            resource_profiles.push(ResourceProfiles {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                resource: Some(Resource {
                    attributes: resource_profiles_attributes.clone(),
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

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // send some messages to the exporter to calculate pipeline statistics
                for _ in 0..3 {
                    // // Send a data message
                    ctx.send_pdata(OTLPData::Metrics(create_otlp_metric(1, 1, 7, 1)))
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OTLPData::Traces(create_otlp_trace(1, 1, 7, 1, 1)))
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OTLPData::Logs(create_otlp_log(1, 1, 7)))
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OTLPData::Profiles(create_otlp_profile(1, 1, 1)))
                        .await
                        .expect("Failed to send data message");
                }

                // TODO ADD DELAY BETWEEN HERE
                _ = sleep(Duration::from_millis(5000));

                // send timertick to generate the report
                ctx.send_timer_tick()
                    .await
                    .expect("Failed to send TimerTick");

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        output_file: String,
    ) -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                // get a file to read and validate the output
                // open file
                // read the output file
                // assert each line accordingly
                let file = File::open(output_file).expect("failed to open file");
                let reader = BufReader::new(file);
                let mut lines = Vec::new();
                for line in reader.lines() {
                    lines.push(line.unwrap());
                }
            })
        }
    }

    #[test]
    fn test_debug_exporter_basic_verbosity() {
        let test_runtime = TestRuntime::new();
        let output_file = "debug_output_basic.txt".to_string();
        let config = Config::new(Verbosity::Basic);
        let exporter = ExporterWrapper::local(
            DebugExporter::new(config, Some(output_file.clone())),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        // remove the created file, prevent accidental check in of report
        // remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_exporter_normal_verbosity() {
        let test_runtime = TestRuntime::new();
        let output_file = "debug_output_normal.txt".to_string();
        let config = Config::new(Verbosity::Normal);
        let exporter = ExporterWrapper::local(
            DebugExporter::new(config, Some(output_file.clone())),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        // remove the created file, prevent accidental check in of report
        // remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_exporter_detailed_verbosity() {
        let test_runtime = TestRuntime::new();
        let output_file = "debug_output_detailed.txt".to_string();
        let config = Config::new(Verbosity::Detailed);
        let exporter = ExporterWrapper::local(
            DebugExporter::new(config, Some(output_file.clone())),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        // remove the created file, prevent accidental check in of report
        // remove_file(output_file).expect("Failed to remove file");
    }
}
