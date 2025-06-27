use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value},
    metrics::v1::{
        ExponentialHistogramDataPoint, HistogramDataPoint, Metric, NumberDataPoint,
        SummaryDataPoint, exemplar::Value as ExemplarValue, metric::Data,
        number_data_point::Value as NumberValue,
    },
};
use std::fmt;
use std::fmt::Write;

pub trait PDataMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String;
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String;
    fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String;
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String;
}

/// The Normal Marshaler takes OTLP messages and converts them to a string by extracting their informations
/// the finalized string will be the output for a normal verbosity level
#[derive(Default)]
pub struct NormalOTLPMarshaler;

impl PDataMarshaler for NormalOTLPMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_log) in logs.resource_logs.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_log.resource {
                resource_attributes = write_attributes(&resource.attributes);
            }

            _ = write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_log.schema_url.clone(),
                attributes = resource_attributes
            );

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();
                if let Some(scope) = &scope_log.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                _ = write!(
                    &mut report,
                    "ScopeLog #{index} {name} @{version} [{schema}] {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_log.schema_url.clone(),
                    attributes = scope_attributes
                );

                for log_record in scope_log.log_records.iter() {
                    let mut log_body = String::new();
                    if let Some(body) = &log_record.body {
                        log_body = body.to_string();
                    }

                    _ = write!(
                        &mut report,
                        " {body} {attributes}",
                        body = log_body,
                        attributes = write_attributes(&log_record.attributes)
                    );
                }
            }
        }

        return report;
    }
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_metric) in metrics.resource_metrics.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_metric.resource {
                resource_attributes = write_attributes(&resource.attributes);
            }

            _ = write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_metric.schema_url.clone(),
                attributes = resource_attributes
            );

            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();
                if let Some(scope) = &scope_metric.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                _ = write!(
                    &mut report,
                    "ScopeLog #{index} {name} {version} {schema} {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_metric.schema_url.clone(),
                    attributes = scope_attributes
                );

                for (metric_index, metric) in scope_metric.metrics.iter().enumerate() {
                    let metric_name = metric.name.clone();

                    if let Some(data) = &metric.data {
                        let data_point_lines = match data {
                            Data::Gauge(gauge) => {
                                write_number_datapoints(&metric, &gauge.data_points)
                            }
                            Data::Sum(sum) => write_number_datapoints(&metric, &sum.data_points),
                            Data::Histogram(histogram) => {
                                write_histogram_datapoints(&metric, &histogram.data_points)
                            }
                            Data::ExponentialHistogram(exponential_histogram) => {
                                write_exponential_histogram_datapoints(
                                    &metric,
                                    &exponential_histogram.data_points,
                                )
                            }
                            Data::Summary(summary) => {
                                write_summary_datapoints(&metric, &summary.data_points)
                            }
                        };
                        _ = write!(
                            &mut report,
                            "{datapoint_lines}",
                            datapoint_lines = data_point_lines
                        );
                    }
                }
            }
        }
        return report;
    }
    fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_span) in traces.resource_spans.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_span.resource {
                resource_attributes = write_attributes(&resource.attributes);
            }
            _ = write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_span.schema_url.clone(),
                attributes = resource_attributes
            );

            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();
                if let Some(scope) = &scope_span.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                _ = write!(
                    &mut report,
                    "ScopeLog #{index} {name} {version} {schema} {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_span.schema_url.clone(),
                    attributes = scope_attributes
                );

                for span in scope_span.spans.iter() {
                    let mut span_trace_id = String::new();
                    let mut span_span_id = String::new();
                    if let Ok(trace_id) = String::from_utf8(span.trace_id.clone()) {
                        span_trace_id = trace_id;
                    }
                    if let Ok(span_id) = String::from_utf8(span.span_id.clone()) {
                        span_span_id = span_id;
                    }

                    let span_attributes = write_attributes(&span.attributes);

                    _ = write!(
                        &mut report,
                        "{name} {trace_id} {span_id} {attributes}",
                        name = &span.name,
                        trace_id = span_trace_id,
                        span_id = span_span_id,
                        attributes = span_attributes
                    );
                }
            }
        }
        return report;
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        // marshal_profiles to string based on verbosity
        let mut report = String::new();
        for (resource_index, resource_profile) in profiles.resource_profiles.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_profile.resource {
                resource_attributes = write_attributes(&resource.attributes);
            }

            _ = write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_profile.schema_url.clone(),
                attributes = resource_attributes
            );
            for (scope_index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();

                if let Some(scope) = &scope_profile.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                _ = write!(
                    &mut report,
                    "ScopeLog #{index} {name} {version} {schema} {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_profile.schema_url.clone(),
                    attributes = scope_attributes
                );
                for profile in scope_profile.profiles.iter() {
                    // let profile_id = String::from_utf8(profile.profile_id.clone());
                    // let profile_samples = profile.sample.len();
                    // if profile.attribute_indices.len() > 0 {
                    //     // attrs := []string{}

                    //     for index in profile.attribute_indices {
                    //         let attribute = resource_profile.attribute_table[index];
                    //         let attribute_key = attribute.key;
                    //         let attribute_value = attribute.value.to_string();
                    //         // attrs = append(attrs, fmt.Sprintf("%s=%s", a.Key(), a.Value().AsString()))
                    //     }

                    //     // buffer.WriteString(" ")
                    //     // buffer.WriteString(strings.Join(attrs, " "))
                    // }
                }
            }
        }
        return report;
    }
}

#[derive(Default)]
pub struct DetailedOTLPMarshaler;

impl PDataMarshaler for DetailedOTLPMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_log) in logs.resource_logs.iter().enumerate() {
            _ = write!(
                &mut report,
                "ResourceLog #{index}\n",
                index = resource_index
            );
            _ = write!(
                &mut report,
                "Resource SchemaURL: {schema_url}\n",
                schema_url = resource_log.schema_url
            );
            if let Some(resource) = &resource_log.resource {
                _ = write!(
                    &mut report,
                    "Resource attributes: {attributes}\n",
                    attributes = write_attributes_detailed(&resource.attributes)
                );
            }

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                _ = write!(&mut report, "ScopeLogs #{index}\n", index = scope_index);
                _ = write!(
                    &mut report,
                    "ScopeLogs SchemaURL: {schema_url}\n",
                    schema_url = scope_log.schema_url
                );
                if let Some(scope) = &scope_log.scope {
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope {name} {version}\n",
                        name = scope.name,
                        version = scope.version
                    );
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope Attributes:\n {attributes}",
                        attributes = write_attributes_detailed(&scope.attributes)
                    );
                }

                for (record_index, log_record) in scope_log.log_records.iter().enumerate() {
                    _ = write!(&mut report, "LogRecord #{index}\n", index = record_index);
                    _ = write!(
                        &mut report,
                        "ObservedTimestamp: {timestamp}\n",
                        timestamp = log_record.observed_time_unix_nano
                    );
                    _ = write!(
                        &mut report,
                        "Timestamp: {timestamp}\n",
                        timestamp = log_record.time_unix_nano
                    );
                    _ = write!(
                        &mut report,
                        "SeverityText: {severity}\n",
                        severity = log_record.severity_text
                    );
                    _ = write!(
                        &mut report,
                        "SeverityNumber: {severity_number}\n",
                        severity_number = log_record.severity_number
                    );

                    if !log_record.event_name.is_empty() {
                        _ = write!(
                            &mut report,
                            "EventName: {event_name}\n",
                            event_name = log_record.event_name
                        );
                    }
                    if let Some(body) = &log_record.body {
                        _ = write!(&mut report, "Body: {body}\n", body = body.to_string());
                    }
                    _ = write!(
                        &mut report,
                        "Attributes: {attributes}\n",
                        attributes = write_attributes_detailed(&log_record.attributes)
                    );
                    if let Ok(trace_id) = std::str::from_utf8(&log_record.trace_id) {
                        _ = write!(&mut report, "Trace ID: {trace_id}\n", trace_id = trace_id);
                    }

                    if let Ok(span_id) = std::str::from_utf8(&log_record.span_id) {
                        _ = write!(&mut report, "Span ID: {span_id}\n", span_id = span_id);
                    }

                    _ = write!(&mut report, "Flags: {flags}\n", flags = log_record.flags);
                }
            }
        }
        return report;
    }
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_metric) in metrics.resource_metrics.iter().enumerate() {
            _ = write!(
                &mut report,
                "ResourceMetric #{index}\n",
                index = resource_index
            );
            _ = write!(
                &mut report,
                "Resource SchemaURL: {schema_url}\n",
                schema_url = resource_metric.schema_url
            );

            if let Some(resource) = &resource_metric.resource {
                _ = write!(
                    &mut report,
                    "Resource attributes: {attributes}\n",
                    attributes = write_attributes_detailed(&resource.attributes)
                );
            }
            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                _ = write!(&mut report, "ScopeMetrics #{index}\n", index = scope_index);
                _ = write!(
                    &mut report,
                    "ScopeMetrics SchemaURL: {schema_url}\n",
                    schema_url = scope_metric.schema_url
                );
                if let Some(scope) = &scope_metric.scope {
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope {name} {version}\n",
                        name = scope.name,
                        version = scope.version
                    );
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope Attributes:\n {attributes}",
                        attributes = write_attributes_detailed(&scope.attributes)
                    );
                }

                for (metric_index, metric) in scope_metric.metrics.iter().enumerate() {
                    _ = write!(&mut report, "Metric #{index}\n", index = metric_index);
                    _ = write!(&mut report, "Descriptor:\n");
                    _ = write!(&mut report, "     -> Name: {name}\n", name = metric.name);
                    _ = write!(
                        &mut report,
                        "     -> Description: {description}\n",
                        description = metric.description
                    );
                    _ = write!(&mut report, "     -> Unit: {unit}\n", unit = metric.unit);
                    if let Some(data) = &metric.data {
                        match data {
                            Data::Gauge(gauge) => {
                                _ = write!(&mut report, "     -> DataType: Gauge\n");
                                for (datapoint_index, datapoint) in
                                    gauge.data_points.iter().enumerate()
                                {
                                    _ = write!(
                                        &mut report,
                                        "NumberDataPoints #{index}\n",
                                        index = datapoint_index
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Attributes: {attributes}\n",
                                        attributes =
                                            write_attributes_detailed(&datapoint.attributes)
                                    );
                                    _ = write!(
                                        &mut report,
                                        "StartTimestamp: {timestamp}\n",
                                        timestamp = datapoint.start_time_unix_nano
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Timestamp: {timestamp}\n",
                                        timestamp = datapoint.time_unix_nano
                                    );
                                    if let Some(value) = &datapoint.value {
                                        match value {
                                            NumberValue::AsInt(int) => {
                                                _ = write!(
                                                    &mut report,
                                                    "Value: {value}\n",
                                                    value = int
                                                );
                                            }
                                            NumberValue::AsDouble(double) => {
                                                _ = write!(
                                                    &mut report,
                                                    "Value: {value}\n",
                                                    value = double
                                                );
                                            }
                                        }
                                    }
                                    if datapoint.exemplars.len() > 0 {
                                        _ = write!(&mut report, "Exemplars: \n");

                                        for (exemplar_index, exemplar) in
                                            datapoint.exemplars.iter().enumerate()
                                        {
                                            _ = write!(
                                                &mut report,
                                                "Exemplar #{index}\n",
                                                index = exemplar_index
                                            );
                                            if let Ok(trace_id) =
                                                std::str::from_utf8(&exemplar.trace_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Trace ID: {trace_id}\n",
                                                    trace_id = trace_id
                                                );
                                            }
                                            if let Ok(span_id) =
                                                std::str::from_utf8(&exemplar.span_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Span ID: {span_id}\n",
                                                    span_id = span_id
                                                );
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> Timestamp: {timestamp}\n",
                                                timestamp = exemplar.time_unix_nano
                                            );
                                            if let Some(value) = &exemplar.value {
                                                match value {
                                                    ExemplarValue::AsInt(int) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {int}\n",
                                                        );
                                                    }
                                                    ExemplarValue::AsDouble(double) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {double}\n",
                                                        );
                                                    }
                                                }
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> FilteredAttributes:\n{attributes}",
                                                attributes = write_attributes_detailed(
                                                    &exemplar.filtered_attributes
                                                )
                                            );
                                        }
                                    }
                                }
                            }
                            Data::Sum(sum) => {
                                _ = write!(&mut report, "     -> DataType: Sum\n");
                                _ = write!(
                                    &mut report,
                                    "     -> IsMonotonic: {}\n",
                                    sum.is_monotonic
                                );
                                _ = write!(
                                    &mut report,
                                    "     -> AggregationTemporality: {}\n",
                                    sum.aggregation_temporality
                                );
                                for (datapoint_index, datapoint) in
                                    sum.data_points.iter().enumerate()
                                {
                                    _ = write!(
                                        &mut report,
                                        "NumberDataPoints #{index}\n",
                                        index = datapoint_index
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Attributes: {attributes}\n",
                                        attributes =
                                            write_attributes_detailed(&datapoint.attributes)
                                    );
                                    _ = write!(
                                        &mut report,
                                        "StartTimestamp: {timestamp}\n",
                                        timestamp = datapoint.start_time_unix_nano
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Timestamp: {timestamp}\n",
                                        timestamp = datapoint.time_unix_nano
                                    );
                                    if let Some(value) = &datapoint.value {
                                        match value {
                                            NumberValue::AsInt(int) => {
                                                _ = write!(
                                                    &mut report,
                                                    "Value: {value}\n",
                                                    value = int
                                                );
                                            }
                                            NumberValue::AsDouble(double) => {
                                                _ = write!(
                                                    &mut report,
                                                    "Value: {value}\n",
                                                    value = double
                                                );
                                            }
                                        }
                                    }
                                    if datapoint.exemplars.len() > 0 {
                                        _ = write!(&mut report, "Exemplars: \n");

                                        for (exemplar_index, exemplar) in
                                            datapoint.exemplars.iter().enumerate()
                                        {
                                            _ = write!(
                                                &mut report,
                                                "Exemplar #{index}\n",
                                                index = exemplar_index
                                            );
                                            if let Ok(trace_id) =
                                                std::str::from_utf8(&exemplar.trace_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Trace ID: {trace_id}\n",
                                                    trace_id = trace_id
                                                );
                                            }
                                            if let Ok(span_id) =
                                                std::str::from_utf8(&exemplar.span_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Span ID: {span_id}\n",
                                                    span_id = span_id
                                                );
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> Timestamp: {timestamp}\n",
                                                timestamp = exemplar.time_unix_nano
                                            );
                                            if let Some(value) = &exemplar.value {
                                                match value {
                                                    ExemplarValue::AsInt(int) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {value}\n",
                                                            value = int
                                                        );
                                                    }
                                                    ExemplarValue::AsDouble(double) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {value}\n",
                                                            value = double
                                                        );
                                                    }
                                                }
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> FilteredAttributes:\n{attributes}",
                                                attributes = write_attributes_detailed(
                                                    &exemplar.filtered_attributes
                                                )
                                            );
                                        }
                                    }
                                }
                            }
                            Data::Histogram(histogram) => {
                                _ = write!(&mut report, "     -> DataType: Histogram\n");
                                _ = write!(
                                    &mut report,
                                    "     -> AggregationTemporality: {}",
                                    histogram.aggregation_temporality
                                );

                                for (index, datapoint) in histogram.data_points.iter().enumerate() {
                                    _ = write!(&mut report, "HistogramDataPoints {}", index);
                                    _ = write!(
                                        &mut report,
                                        "Attributes: {}",
                                        write_attributes_detailed(&datapoint.attributes)
                                    );

                                    _ = write!(
                                        &mut report,
                                        "StartTimestamp: {}",
                                        datapoint.start_time_unix_nano
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Timestamp: {}",
                                        datapoint.time_unix_nano
                                    );
                                    _ = write!(&mut report, "Count: {}", datapoint.count);

                                    if let Some(sum) = &datapoint.sum {
                                        _ = write!(&mut report, "Sum: {}", sum);
                                    }
                                    if let Some(min) = &datapoint.min {
                                        _ = write!(&mut report, "Min: {}", min);
                                    }
                                    if let Some(max) = &datapoint.max {
                                        _ = write!(&mut report, "Max: {}", max);
                                    }

                                    for (index, bound) in
                                        datapoint.explicit_bounds.iter().enumerate()
                                    {
                                        _ = write!(&mut report, "ExplicitBound {index}: {bound}",);
                                    }
                                    for (index, count) in datapoint.bucket_counts.iter().enumerate()
                                    {
                                        _ = write!(&mut report, "Buckets {index}, Count: {count}",);
                                    }

                                    if datapoint.exemplars.len() > 0 {
                                        _ = write!(&mut report, "Exemplars: \n");

                                        for (exemplar_index, exemplar) in
                                            datapoint.exemplars.iter().enumerate()
                                        {
                                            _ = write!(
                                                &mut report,
                                                "Exemplar #{index}\n",
                                                index = exemplar_index
                                            );
                                            if let Ok(trace_id) =
                                                std::str::from_utf8(&exemplar.trace_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Trace ID: {trace_id}\n",
                                                    trace_id = trace_id
                                                );
                                            }
                                            if let Ok(span_id) =
                                                std::str::from_utf8(&exemplar.span_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Span ID: {span_id}\n",
                                                    span_id = span_id
                                                );
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> Timestamp: {timestamp}\n",
                                                timestamp = exemplar.time_unix_nano
                                            );
                                            if let Some(value) = &exemplar.value {
                                                match value {
                                                    ExemplarValue::AsInt(int) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {value}\n",
                                                            value = int
                                                        );
                                                    }
                                                    ExemplarValue::AsDouble(double) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {value}\n",
                                                            value = double
                                                        );
                                                    }
                                                }
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> FilteredAttributes:\n{attributes}",
                                                attributes = write_attributes_detailed(
                                                    &exemplar.filtered_attributes
                                                )
                                            );
                                        }
                                    }
                                }
                            }
                            Data::ExponentialHistogram(exponential_histogram) => {
                                _ = write!(
                                    &mut report,
                                    "     -> DataType: Exponential Histogram\n"
                                );
                                _ = write!(
                                    &mut report,
                                    "     -> AggregationTemporality: {}",
                                    exponential_histogram.aggregation_temporality
                                );
                                for (datapoint_index, datapoint) in
                                    exponential_histogram.data_points.iter().enumerate()
                                {
                                    _ = write!(
                                        &mut report,
                                        "ExponentialHistogramDataPoints #{index}\n",
                                        index = datapoint_index
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Attributes: {}\n",
                                        write_attributes_detailed(&datapoint.attributes)
                                    );
                                    _ = write!(
                                        &mut report,
                                        "StartTimestamp: {}\n",
                                        datapoint.start_time_unix_nano
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Timestamp: {}\n",
                                        datapoint.time_unix_nano
                                    );
                                    _ = write!(&mut report, "Count: {}\n", datapoint.count);
                                    if let Some(sum) = &datapoint.sum {
                                        _ = write!(&mut report, "Sum: {}", sum);
                                    }
                                    if let Some(min) = &datapoint.min {
                                        _ = write!(&mut report, "Min: {}", min);
                                    }
                                    if let Some(max) = &datapoint.max {
                                        _ = write!(&mut report, "Max: {}", max);
                                    }

                                    // scale := int(p.Scale())
                                    // factor := math.Ldexp(math.Ln2, -scale)
                                    // // Note: the equation used here, which is
                                    // //   math.Exp(index * factor)
                                    // // reports +Inf as the _lower_ boundary of the bucket nearest
                                    // // infinity, which is incorrect and can be addressed in various
                                    // // ways.  The OTel-Go implementation of this histogram pending
                                    // // in https://github.com/open-telemetry/opentelemetry-go/pull/2393
                                    // // uses a lookup table for the last finite boundary, which can be
                                    // // easily computed using `math/big` (for scales up to 20).

                                    // negB := p.Negative().BucketCounts()
                                    // posB := p.Positive().BucketCounts()

                                    // for i := 0; i < negB.Len(); i++ {
                                    //     pos := negB.Len() - i - 1
                                    //     index := float64(p.Negative().Offset()) + float64(pos)
                                    //     lower := math.Exp(index * factor)
                                    //     upper := math.Exp((index + 1) * factor)
                                    //     b.logEntry("Bucket [%f, %f), Count: %d", -upper, -lower, negB.At(pos))
                                    // }

                                    // if p.ZeroCount() != 0 {
                                    //     b.logEntry("Bucket [0, 0], Count: %d", p.ZeroCount())
                                    // }

                                    // for pos := 0; pos < posB.Len(); pos++ {
                                    //     index := float64(p.Positive().Offset()) + float64(pos)
                                    //     lower := math.Exp(index * factor)
                                    //     upper := math.Exp((index + 1) * factor)
                                    //     b.logEntry("Bucket (%f, %f], Count: %d", lower, upper, posB.At(pos))
                                    // }

                                    if datapoint.exemplars.len() > 0 {
                                        _ = write!(&mut report, "Exemplars: \n");

                                        for (exemplar_index, exemplar) in
                                            datapoint.exemplars.iter().enumerate()
                                        {
                                            _ = write!(
                                                &mut report,
                                                "Exemplar #{index}\n",
                                                index = exemplar_index
                                            );
                                            if let Ok(trace_id) =
                                                std::str::from_utf8(&exemplar.trace_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Trace ID: {trace_id}\n",
                                                    trace_id = trace_id
                                                );
                                            }
                                            if let Ok(span_id) =
                                                std::str::from_utf8(&exemplar.span_id)
                                            {
                                                _ = write!(
                                                    &mut report,
                                                    "     -> Span ID: {span_id}\n",
                                                    span_id = span_id
                                                );
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> Timestamp: {timestamp}\n",
                                                timestamp = exemplar.time_unix_nano
                                            );
                                            if let Some(value) = &exemplar.value {
                                                match value {
                                                    ExemplarValue::AsInt(int) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {value}\n",
                                                            value = int
                                                        );
                                                    }
                                                    ExemplarValue::AsDouble(double) => {
                                                        _ = write!(
                                                            &mut report,
                                                            "     -> Value: {value}\n",
                                                            value = double
                                                        );
                                                    }
                                                }
                                            }
                                            _ = write!(
                                                &mut report,
                                                "     -> FilteredAttributes:\n{attributes}",
                                                attributes = write_attributes_detailed(
                                                    &exemplar.filtered_attributes
                                                )
                                            );
                                        }
                                    }
                                }
                            }
                            Data::Summary(summary) => {
                                _ = write!(&mut report, "     -> DataType: Summary\n");
                                for (datapoint_index, datapoint) in
                                    summary.data_points.iter().enumerate()
                                {
                                    _ = write!(
                                        &mut report,
                                        "SummaryDataPoints {index}",
                                        index = datapoint_index
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Attributes: {attributes}",
                                        attributes =
                                            write_attributes_detailed(&datapoint.attributes)
                                    );
                                    _ = write!(
                                        &mut report,
                                        "StartTimestamp: {timestamp}",
                                        timestamp = datapoint.start_time_unix_nano
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Timestamp: {timestamp}",
                                        timestamp = datapoint.time_unix_nano
                                    );
                                    _ = write!(
                                        &mut report,
                                        "Count: {count}",
                                        count = datapoint.count
                                    );
                                    _ = write!(&mut report, "Sum: {sum}", sum = datapoint.sum);
                                    for (quantile_index, quantile) in
                                        datapoint.quantile_values.iter().enumerate()
                                    {
                                        _ = write!(
                                            &mut report,
                                            "QuantileValue {index}: Quantile {quantile}, Value {value}",
                                            index = quantile_index,
                                            quantile = quantile.quantile,
                                            value = quantile.value
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return report;
    }
    fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_span) in traces.resource_spans.iter().enumerate() {
            _ = write!(
                &mut report,
                "ResourceSpan #{index}\n",
                index = resource_index
            );
            _ = write!(
                &mut report,
                "Resource SchemaURL: {schema_url}\n",
                schema_url = resource_span.schema_url
            );
            if let Some(resource) = &resource_span.resource {
                _ = write!(
                    &mut report,
                    "Resource attributes {attributes}\n",
                    attributes = write_attributes_detailed(&resource.attributes)
                );
            }
            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                _ = write!(&mut report, "ScopeSpans #{index}\n", index = scope_index);
                _ = write!(
                    &mut report,
                    "ScopeSpans SchemaURL: {schema_url}\n",
                    schema_url = scope_span.schema_url
                );
                if let Some(scope) = &scope_span.scope {
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope {name} {version}\n",
                        name = scope.name,
                        version = scope.version
                    );
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope Attributes:\n {attributes}",
                        attributes = write_attributes_detailed(&scope.attributes)
                    );
                }

                for (span_index, span) in scope_span.spans.iter().enumerate() {
                    _ = write!(&mut report, "Span {index}\n", index = span_index);
                    if let Ok(trace_id) = std::str::from_utf8(&span.trace_id) {
                        _ = write!(
                            &mut report,
                            "     -> Trace ID: {trace_id}\n",
                            trace_id = trace_id
                        );
                    }
                    if let Ok(parent_span_id) = std::str::from_utf8(&span.parent_span_id) {
                        _ = write!(
                            &mut report,
                            "     -> Parent ID: {parent_span_id}\n",
                            parent_span_id = parent_span_id
                        );
                    }
                    if let Ok(span_id) = std::str::from_utf8(&span.span_id) {
                        _ = write!(&mut report, "     -> ID: {span_id}\n", span_id = span_id);
                    }

                    _ = write!(&mut report, "Name: {name}\n", name = span.name);
                    _ = write!(&mut report, "Kind: {kind}\n", kind = span.kind);
                    if !span.trace_state.is_empty() {
                        _ = write!(
                            &mut report,
                            "TraceState: {trace_state}\n",
                            trace_state = span.trace_state
                        );
                    }

                    _ = write!(
                        &mut report,
                        "Start time: {start_time}\n",
                        start_time = span.start_time_unix_nano
                    );
                    _ = write!(
                        &mut report,
                        "End time: {end_time}\n",
                        end_time = span.end_time_unix_nano
                    );
                    if let Some(status) = &span.status {
                        _ = write!(
                            &mut report,
                            "Status code: {status_code}\n",
                            status_code = status.code
                        );
                        _ = write!(
                            &mut report,
                            "Status message: {status_message}\n",
                            status_message = status.message
                        );
                    }

                    _ = write!(
                        &mut report,
                        "Attributes: {attributes}\n",
                        attributes = write_attributes_detailed(&span.attributes)
                    );

                    if span.events.len() > 0 {
                        _ = write!(&mut report, "Events: \n");
                        for (event_index, event) in span.events.iter().enumerate() {
                            _ = write!(&mut report, "SpanEvent {index}\n", index = event_index);
                            _ = write!(&mut report, "Name: {name}\n", name = event.name);
                            _ = write!(
                                &mut report,
                                "Timestamp: {timestamp}\n",
                                timestamp = event.time_unix_nano
                            );
                            _ = write!(
                                &mut report,
                                "DroppedAttributesCount: {dropped_attributes_count}\n",
                                dropped_attributes_count = event.dropped_attributes_count
                            );
                            _ = write!(
                                &mut report,
                                "Attributes: {attributes}\n",
                                attributes = write_attributes_detailed(&event.attributes)
                            );
                        }
                    }

                    if span.links.len() > 0 {
                        _ = write!(&mut report, "Links: \n");
                        for (index, link) in span.links.iter().enumerate() {
                            _ = write!(&mut report, "SpanLink: {}\n", index);
                            if let Ok(trace_id) = std::str::from_utf8(&link.trace_id) {
                                _ = write!(&mut report, "     -> Trace ID: {}\n", trace_id);
                            }
                            if let Ok(span_id) = std::str::from_utf8(&link.span_id) {
                                _ = write!(&mut report, "     -> Span ID: {}\n", span_id);
                            }

                            _ = write!(&mut report, "     -> TraceState: {}\n", link.trace_state);
                            _ = write!(
                                &mut report,
                                "     -> DroppedAttributesCount: {}\n",
                                link.dropped_attributes_count
                            );
                            _ = write!(
                                &mut report,
                                "     -> Attributes: {}\n",
                                write_attributes_detailed(&link.attributes)
                            );
                        }
                    }
                }
            }
        }
        return report;
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        let mut report = String::new();

        // buf.logProfileMappings(dic.MappingTable())
        // buf.logProfileLocations(dic.LocationTable())
        // buf.logProfileFunctions(dic.FunctionTable())
        // buf.logAttributesWithIndentation(
        // 	"Attribute units",
        // 	attributeUnitsToMap(dic.AttributeUnits()),
        // 	0)

        // buf.logAttributesWithIndentation(
        // 	"Link table",
        // 	linkTableToMap(dic.LinkTable()),
        // 	0)

        // buf.logStringTable(dic.StringTable())
        for (resource_index, resource_profile) in profiles.resource_profiles.iter().enumerate() {
            _ = write!(
                &mut report,
                "ResourceProfile #{index}\n",
                index = resource_index
            );
            _ = write!(
                &mut report,
                "Resource SchemaURL: {schema_url}\n",
                schema_url = resource_profile.schema_url
            );
            if let Some(resource) = &resource_profile.resource {
                _ = write!(
                    &mut report,
                    "Resource attributes {attributes}\n",
                    attributes = write_attributes_detailed(&resource.attributes)
                );
            }
            for (scope_index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                _ = write!(&mut report, "ScopeProfiles #{index}\n", index = scope_index);
                _ = write!(
                    &mut report,
                    "ScopeProfiles SchemaURL: {schema_url}\n",
                    schema_url = scope_profile.schema_url
                );
                if let Some(scope) = &scope_profile.scope {
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope {name} {version}\n",
                        name = scope.name,
                        version = scope.version
                    );
                    _ = write!(
                        &mut report,
                        "Instrumentation Scope Attributes:\n {attributes}",
                        attributes = write_attributes_detailed(&scope.attributes)
                    );
                }

                for (profile_index, profile) in scope_profile.profiles.iter().enumerate() {
                    // buf.logProfileSamples(profile.Sample(), dic.AttributeTable())
                    // buf.logComment(profile.CommentStrindices())

                    _ = write!(&mut report, "Profile {index}", index = profile_index);
                    if let Ok(profile_id) = std::str::from_utf8(&profile.profile_id) {
                        _ = write!(&mut report, "     -> Profile ID: {}\n", profile_id);
                    }
                    _ = write!(
                        &mut report,
                        "Start time: {profile_start_time}\n",
                        profile_start_time = profile.time_nanos
                    );
                    _ = write!(
                        &mut report,
                        "Duration: {profile_duration}\n",
                        profile_duration = profile.duration_nanos
                    );
                    _ = write!(
                        &mut report,
                        "Dropped attributes count: {profile_dropped_attributes_count}\n",
                        profile_dropped_attributes_count = profile.dropped_attributes_count
                    );

                    // _ = write!(&mut report, "Location indices: {location_indices}\n", profile.location_indices);

                    // if profile.sample.len() > 0 {

                    // }
                    // _ = write!(&mut report, "Sample type: {profile_sample_type}\n", profile.sample_type);

                    // if profile.
                }
            }
        }
        return report;
    }
}

fn write_attributes(attributes: &Vec<KeyValue>) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        if let Some(value) = &attribute.value {
            let attribute_value = value.to_string();
            _ = write!(
                &mut attribute_string,
                "{key}={value} ",
                key = attribute.key,
                value = attribute_value
            );
        }
    }

    attribute_string
}

fn write_attributes_detailed(attributes: &Vec<KeyValue>) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        if let Some(value) = &attribute.value {
            let attribute_value = value.to_string();
            _ = write!(
                &mut attribute_string,
                "{key}: {value} ",
                key = attribute.key,
                value = attribute_value
            );
        }
    }

    attribute_string
}

fn write_number_datapoints(metric: &Metric, datapoints: &Vec<NumberDataPoint>) -> String {
    let mut lines = String::new();

    for datapoint in datapoints.iter() {
        let datapoint_attributes = write_attributes(&datapoint.attributes);
        if let Some(value) = datapoint.value {
            match value {
                NumberValue::AsDouble(double) => {
                    _ = write!(
                        &mut lines,
                        "{name} {attributes} {value}\n",
                        name = metric.name,
                        attributes = datapoint_attributes,
                        value = double
                    );
                }
                NumberValue::AsInt(int) => {
                    _ = write!(
                        &mut lines,
                        "{name} {attributes} {value}\n",
                        name = metric.name,
                        attributes = datapoint_attributes,
                        value = int
                    );
                }
            }
        }
    }

    return lines;
}

fn write_histogram_datapoints(metric: &Metric, datapoints: &Vec<HistogramDataPoint>) -> String {
    let mut lines = String::new();
    for datapoint in datapoints.iter() {
        let datapoint_attributes = write_attributes(&datapoint.attributes);
        let mut values = String::new();
        _ = write!(&mut values, "count={} ", datapoint.count);
        if let Some(sum) = datapoint.sum {
            _ = write!(&mut values, "sum={} ", sum);
        }
        if let Some(min) = datapoint.min {
            _ = write!(&mut values, "min={} ", min);
        }
        if let Some(max) = datapoint.max {
            _ = write!(&mut values, "max={} ", max);
        }

        for (i, bucket) in datapoint.bucket_counts.iter().enumerate() {
            let mut bucket_bound = String::new();
            if i < datapoint.explicit_bounds.len() {
                bucket_bound = format!("le{}=", datapoint.explicit_bounds[i]);
            }
            _ = write!(&mut values, "{}{} ", bucket_bound, bucket);
        }

        _ = write!(
            &mut lines,
            "{name} {attributes} {values}\n",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        );
    }

    return lines;
}

fn write_exponential_histogram_datapoints(
    metric: &Metric,
    datapoints: &Vec<ExponentialHistogramDataPoint>,
) -> String {
    let mut lines = String::new();
    for datapoint in datapoints.iter() {
        let datapoint_attributes = write_attributes(&datapoint.attributes);

        let mut values = String::new();
        _ = write!(&mut values, "count={} ", datapoint.count);

        if let Some(sum) = datapoint.sum {
            _ = write!(&mut values, "sum={} ", sum);
        }
        if let Some(min) = datapoint.min {
            _ = write!(&mut values, "min={} ", min);
        }
        if let Some(max) = datapoint.max {
            _ = write!(&mut values, "max={} ", max);
        }

        _ = write!(
            &mut lines,
            "{name} {attributes} {values}\n",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        );
    }
    return lines;
}

fn write_summary_datapoints(metric: &Metric, datapoints: &Vec<SummaryDataPoint>) -> String {
    let mut lines = String::new();
    for datapoint in datapoints.iter() {
        let datapoint_attributes = write_attributes(&datapoint.attributes);
        let mut values = String::new();

        _ = write!(&mut values, "count={} ", datapoint.count);
        _ = write!(&mut values, "sum={} ", datapoint.sum);

        for quantile in datapoint.quantile_values.iter() {
            write!(&mut values, "q{}={} ", quantile.quantile, quantile.value).unwrap();
        }

        _ = write!(
            &mut lines,
            "{name} {attributes} {values}\n",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        );
    }
    return lines;
}

impl fmt::Display for AnyValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(value) = &self.value {
            match value {
                Value::StringValue(string) => {
                    write!(f, "{}", string)?;
                }
                Value::BoolValue(bool) => {
                    write!(f, "{}", bool.to_string())?;
                }
                Value::IntValue(int) => {
                    write!(f, "{}", int.to_string())?;
                }
                Value::DoubleValue(double) => {
                    write!(f, "{}", double.to_string())?;
                }
                Value::ArrayValue(array) => {
                    let values = &array.values;
                    for value in values {
                        write!(f, "{}", value)?;
                    }
                }
                Value::KvlistValue(kvlist) => {
                    let string = write_attributes(&kvlist.values);
                    write!(f, "{}", string)?;
                }
                Value::BytesValue(bytes) => {
                    if let Ok(byte_string) = String::from_utf8(bytes.to_vec()) {
                        write!(f, "{}", byte_string)?;
                    }
                    write!(f, "")?;
                }
                _ => {
                    write!(f, "")?;
                }
            }
        } else {
            write!(f, "")?;
        }
        Ok(())
    }
}
