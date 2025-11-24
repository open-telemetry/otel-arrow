// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// disable some of the rust/clippy lints that we are not able to control via prost codegen
#![allow(
    clippy::must_use_candidate,
    unused_qualifications,
    missing_docs,
    unused_results
)]

pub mod consts;

#[path = "."]
#[rustfmt::skip]
pub mod opentelemetry {
    #[path = "."]
    pub mod trace {
        #[path = "opentelemetry.proto.trace.v1.rs"]
        pub mod v1;
    }
    #[path = "."]
    pub mod logs {
        #[path = "opentelemetry.proto.logs.v1.rs"]
        pub mod v1;
    }
    #[path = "."]
    pub mod metrics {
        #[path = "opentelemetry.proto.metrics.v1.rs"]
        pub mod v1;
    }
    #[path = "."]
    pub mod common {
        #[path = "opentelemetry.proto.common.v1.rs"]
        pub mod v1;
    }
    #[path = "."]
    pub mod resource {
        #[path = "opentelemetry.proto.resource.v1.rs"]
        pub mod v1;
    }
    #[path = "."]
    pub mod profiles {
        #[path = "opentelemetry.proto.profiles.v1development.rs"]
        pub mod v1development;
    }
    #[path = "."]
    pub mod collector {
        #[path = "."]
        pub mod trace {
            #[path = "opentelemetry.proto.collector.trace.v1.rs"]
            pub mod v1;
        }
        #[path = "."]
        pub mod logs {
            #[path = "opentelemetry.proto.collector.logs.v1.rs"]
            pub mod v1;
        }
        #[path = "."]
        pub mod metrics {
            #[path = "opentelemetry.proto.collector.metrics.v1.rs"]
            pub mod v1;
        }
        #[path = "."]
        pub mod profiles {
            #[path = "opentelemetry.proto.collector.profiles.v1development.rs"]
            pub mod v1development;
        }
    }

    #[path = "."]
    pub mod arrow {
        #[path = "opentelemetry.proto.experimental.arrow.v1.rs"]
        pub mod v1;
    }
}

/// Protocol message data of some type.
///
/// Generally, callers should use the OtlpProtoBytes type defined in
/// crate::otlp instead, this is only useful where proto::Message
/// objects are required. OtlpProtoBytes has an efficient translation
/// into OtapArrowRecords, this type does not.
///
///
/// Note this could be considered for #[cfg(test)], however we are
/// aware of uses in otap-df-otap's fake_signal_generator and
/// debug_processor.
#[derive(Clone, Debug)]
pub enum OtlpProtoMessage {
    /// Logs data. This is equivalent to ExportLogsServiceRequest.
    Logs(opentelemetry::logs::v1::LogsData),
    /// Metrics data. This is equivalent to ExportMetricsServiceRequest.
    Metrics(opentelemetry::metrics::v1::MetricsData),
    /// Traces data. This is equivalent to ExportTraceServiceRequest.
    Traces(opentelemetry::trace::v1::TracesData),
}

impl OtlpProtoMessage {
    /// Compute the batch length.  This returns the same value as
    /// OtapArrowRecords::batch_length().
    ///
    /// TODO: The OpenTelemetry Collector has no standard function
    /// name for this, it has no multi-signal type so uses
    /// Logs.NumRecords, Traces.NumSpans, and Metrics.NumDataPoints.
    ///
    /// This was named to match the OtapArrowRecords::batch_length()
    /// function; we may conceive of renaming all of these methods to
    /// be more descriptive, for example num_items() or batch_items()
    /// which is a standard concept in Collector batch configuration.
    #[must_use]
    pub fn batch_length(&self) -> usize {
        match self {
            Self::Metrics(data) => metrics_batch_length(data),
            Self::Logs(data) => logs_batch_length(data),
            Self::Traces(data) => traces_batch_length(data),
        }
    }

    /// Get the signal type.
    #[must_use]
    pub fn signal_type(&self) -> otap_df_config::SignalType {
        use otap_df_config::SignalType;
        match self {
            Self::Logs(_) => SignalType::Logs,
            Self::Metrics(_) => SignalType::Metrics,
            Self::Traces(_) => SignalType::Traces,
        }
    }
}

fn logs_batch_length(logs: &opentelemetry::logs::v1::LogsData) -> usize {
    logs.resource_logs
        .iter()
        .flat_map(|rl| &rl.scope_logs)
        .map(|sl| sl.log_records.len())
        .sum()
}

fn traces_batch_length(traces: &opentelemetry::trace::v1::TracesData) -> usize {
    traces
        .resource_spans
        .iter()
        .flat_map(|rs| &rs.scope_spans)
        .map(|ss| ss.spans.len())
        .sum()
}

fn metrics_batch_length(metrics: &opentelemetry::metrics::v1::MetricsData) -> usize {
    use opentelemetry::metrics::v1::metric::Data;
    metrics
        .resource_metrics
        .iter()
        .flat_map(|rm| &rm.scope_metrics)
        .flat_map(|sm| &sm.metrics)
        .map(|metric| match &metric.data {
            Some(Data::Gauge(gauge)) => gauge.data_points.len(),
            Some(Data::Sum(sum)) => sum.data_points.len(),
            Some(Data::Histogram(histogram)) => histogram.data_points.len(),
            Some(Data::ExponentialHistogram(exp_histogram)) => exp_histogram.data_points.len(),
            Some(Data::Summary(summary)) => summary.data_points.len(),
            None => 0,
        })
        .sum()
}

//
// From<> conversions between Request types and Data types
//

impl From<opentelemetry::collector::logs::v1::ExportLogsServiceRequest>
    for opentelemetry::logs::v1::LogsData
{
    fn from(req: opentelemetry::collector::logs::v1::ExportLogsServiceRequest) -> Self {
        Self {
            resource_logs: req.resource_logs,
        }
    }
}

impl From<opentelemetry::logs::v1::LogsData>
    for opentelemetry::collector::logs::v1::ExportLogsServiceRequest
{
    fn from(data: opentelemetry::logs::v1::LogsData) -> Self {
        Self {
            resource_logs: data.resource_logs,
        }
    }
}

impl From<opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest>
    for opentelemetry::metrics::v1::MetricsData
{
    fn from(req: opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest) -> Self {
        Self {
            resource_metrics: req.resource_metrics,
        }
    }
}

impl From<opentelemetry::metrics::v1::MetricsData>
    for opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest
{
    fn from(data: opentelemetry::metrics::v1::MetricsData) -> Self {
        Self {
            resource_metrics: data.resource_metrics,
        }
    }
}

impl From<opentelemetry::collector::trace::v1::ExportTraceServiceRequest>
    for opentelemetry::trace::v1::TracesData
{
    fn from(req: opentelemetry::collector::trace::v1::ExportTraceServiceRequest) -> Self {
        Self {
            resource_spans: req.resource_spans,
        }
    }
}

impl From<opentelemetry::trace::v1::TracesData>
    for opentelemetry::collector::trace::v1::ExportTraceServiceRequest
{
    fn from(data: opentelemetry::trace::v1::TracesData) -> Self {
        Self {
            resource_spans: data.resource_spans,
        }
    }
}

//
// From<> conversions from Data types to OtlpProtoMessage enum
//

impl From<opentelemetry::logs::v1::LogsData> for OtlpProtoMessage {
    fn from(data: opentelemetry::logs::v1::LogsData) -> Self {
        Self::Logs(data)
    }
}

impl From<opentelemetry::metrics::v1::MetricsData> for OtlpProtoMessage {
    fn from(data: opentelemetry::metrics::v1::MetricsData) -> Self {
        Self::Metrics(data)
    }
}

impl From<opentelemetry::trace::v1::TracesData> for OtlpProtoMessage {
    fn from(data: opentelemetry::trace::v1::TracesData) -> Self {
        Self::Traces(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::InstrumentationScope;
    use crate::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
    use crate::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric::Data,
        number_data_point::Value,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData};

    #[test]
    fn test_logs_batch_length() {
        let logs = LogsData::new(vec![
            ResourceLogs::new(
                Resource::default(),
                vec![
                    ScopeLogs::new(
                        InstrumentationScope::default(),
                        vec![LogRecord::default(), LogRecord::default()],
                    ),
                    ScopeLogs::new(InstrumentationScope::default(), vec![LogRecord::default()]),
                ],
            ),
            ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![LogRecord::default(), LogRecord::default()],
                )],
            ),
        ]);

        assert_eq!(logs_batch_length(&logs), 5);
    }

    #[test]
    fn test_traces_batch_length() {
        let traces = TracesData::new(vec![
            ResourceSpans::new(
                Resource::default(),
                vec![
                    ScopeSpans::new(
                        InstrumentationScope::default(),
                        vec![Span::default(), Span::default(), Span::default()],
                    ),
                    ScopeSpans::new(InstrumentationScope::default(), vec![Span::default()]),
                ],
            ),
            ResourceSpans::new(
                Resource::default(),
                vec![ScopeSpans::new(
                    InstrumentationScope::default(),
                    vec![Span::default()],
                )],
            ),
        ]);

        assert_eq!(traces_batch_length(&traces), 5);
    }

    #[test]
    fn test_metrics_batch_length() {
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric {
                        name: "gauge1".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    value: Some(Value::AsDouble(1.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    value: Some(Value::AsDouble(2.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                        ..Default::default()
                    },
                    Metric {
                        name: "gauge2".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    value: Some(Value::AsDouble(3.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    value: Some(Value::AsDouble(4.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    value: Some(Value::AsDouble(5.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                        ..Default::default()
                    },
                ],
            )],
        )]);

        assert_eq!(metrics_batch_length(&metrics), 5);
    }

    #[test]
    fn test_empty_logs() {
        let logs = LogsData::new(vec![]);
        assert_eq!(logs_batch_length(&logs), 0);
    }

    #[test]
    fn test_empty_traces() {
        let traces = TracesData::new(vec![]);
        assert_eq!(traces_batch_length(&traces), 0);
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = MetricsData::new(vec![]);
        assert_eq!(metrics_batch_length(&metrics), 0);
    }
}
