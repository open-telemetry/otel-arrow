use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value},
    metrics::v1::{
        ExponentialHistogramDataPoint, HistogramDataPoint, Metric, NumberDataPoint,
        SummaryDataPoint, metric::Data, number_data_point::Value as NumberValue,
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
            // generate string for this
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_log.resource {
                resource_attributes = write_attributes(&resource.attributes);
            }

            // let resource_string = format!(
            //     "ResourceLog #{index} {schema} {attributes}",
            //     index = resource_index,
            //     schema = resource_log.schema_url.clone(),
            //     attributes = resource_attributes,
            // );
            // report.push_str(&resource_string);

            write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_log.schema_url.clone(),
                attributes = resource_attributes
            )
            .unwrap();

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                // let scope_schema_url = scope_log.schema_url;
                let mut scope_attributes = String::new();
                if let Some(scope) = &scope_log.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                // let scope_string = format!(
                //     "ScopeLog #{index} {name} @{version} [{schema}] {attributes}",
                //     index = scope_index,
                //     name = scope_name,
                //     version = scope_version,
                //     schema = scope_log.schema_url.clone(),
                //     attributes = scope_attributes,
                // );
                // report.push_str(&scope_string);

                write!(
                    &mut report,
                    "ScopeLog #{index} {name} @{version} [{schema}] {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_log.schema_url.clone(),
                    attributes = scope_attributes
                )
                .unwrap();

                for log_record in scope_log.log_records.iter() {
                    let mut log_body = String::new();
                    if let Some(body) = &log_record.body {
                        log_body = body.to_string();
                    }

                    // let string = format!(
                    //     "{body} {attributes}",
                    //     body = log_body,
                    //     attributes = write_attributes(&log_record.attributes)
                    // );
                    // report.push_str(&string);
                    write!(
                        &mut report,
                        " {body} {attributes}",
                        body = log_body,
                        attributes = write_attributes(&log_record.attributes)
                    )
                    .unwrap();
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

            // let resource_string = format!(
            //     "ResourceLog #{index} {schema} {attributes}",
            //     index = resource_index,
            //     schema = resource_metric.schema_url.clone(),
            //     attributes = resource_attributes,
            // );
            // report.push_str(&resource_string);

            write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_metric.schema_url.clone(),
                attributes = resource_attributes
            )
            .unwrap();

            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();
                if let Some(scope) = &scope_metric.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                // let scope_string = format!(
                //     "ScopeLog #{index} {name} {version} {schema} {attributes}",
                //     index = scope_index,
                //     name = scope_name,
                //     version = scope_version,
                //     schema = scope_metric.schema_url.clone(),
                //     attributes = scope_attributes
                // );
                // report.push_str(&scope_string);
                write!(
                    &mut report,
                    "ScopeLog #{index} {name} {version} {schema} {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_metric.schema_url.clone(),
                    attributes = scope_attributes
                )
                .unwrap();

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
                        write!(
                            &mut report,
                            "{datapoint_lines}",
                            datapoint_lines = data_point_lines
                        )
                        .unwrap();
                        // report.push_str(&data_point_lines);
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
            // let resource_string = format!(
            //     "ResourceLog #{index} {schema} {attributes}",
            //     index = resource_index,
            //     schema = resource_span.schema_url.clone(),
            //     attributes = resource_attributes,
            // );
            // report.push_str(&resource_string);
            write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_span.schema_url.clone(),
                attributes = resource_attributes
            )
            .unwrap();

            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();
                // let scope_schema_url = scope_span.schema_url;
                if let Some(scope) = &scope_span.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }

                // let scope_string = format!(
                //     "ScopeLog #{index} {name} {version} {schema} {attributes}",
                //     index = scope_index,
                //     name = scope_name,
                //     version = scope_version,
                //     schema = scope_span.schema_url.clone(),
                //     attributes = scope_attributes,
                // );
                // report.push_str(&scope_string);
                write!(
                    &mut report,
                    "ScopeLog #{index} {name} {version} {schema} {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_span.schema_url.clone(),
                    attributes = scope_attributes
                )
                .unwrap();
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

                    // let span_string = format!(
                    //     "{name} {trace_id} {span_id} {attributes}",
                    //     name = &span.name,
                    //     trace_id = span_trace_id,
                    //     span_id = span_span_id,
                    //     attributes = span_attributes
                    // );
                    // report.push_str(&span_string);
                    write!(
                        &mut report,
                        "{name} {trace_id} {span_id} {attributes}",
                        name = &span.name,
                        trace_id = span_trace_id,
                        span_id = span_span_id,
                        attributes = span_attributes
                    )
                    .unwrap();
                }
            }
        }
        return report;
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        // marshal_profiles to string based on verbosity
        let mut report = String::new();
        for (resource_index, resource_profile) in profiles.resource_profiles.iter().enumerate() {
            // let resource_schema_url = resource_profile.schema_url;
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_profile.resource {
                resource_attributes = write_attributes(&resource.attributes);
            }
            // let resource_string = format!(
            //     "ResourceLog #{index} {schema} {attributes}",
            //     index = resource_index,
            //     schema = resource_profile.schema_url.clone(),
            //     attributes = resource_attributes,
            // );
            // report.push_str(&resource_string);
            write!(
                &mut report,
                "ResourceLog #{index} {schema} {attributes}",
                index = resource_index,
                schema = resource_profile.schema_url.clone(),
                attributes = resource_attributes
            )
            .unwrap();
            for (scope_index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                let mut scope_name = String::new();
                let mut scope_version = String::new();
                let mut scope_attributes = String::new();
                // let scope_schema_url = scope_profile.schema_url;
                if let Some(scope) = &scope_profile.scope {
                    scope_name = scope.name.clone();
                    scope_version = scope.version.clone();
                    scope_attributes = write_attributes(&scope.attributes);
                }
                // let scope_string = format!(
                //     "ScopeLog #{index} {name} {version} {schema} {attributes}",
                //     index = scope_index,
                //     name = scope_name,
                //     version = scope_version,
                //     schema = scope_profile.schema_url.clone(),
                //     attributes = scope_attributes
                // );
                // report.push_str(&scope_string);
                write!(
                    &mut report,
                    "ScopeLog #{index} {name} {version} {schema} {attributes}",
                    index = scope_index,
                    name = scope_name,
                    version = scope_version,
                    schema = scope_profile.schema_url.clone(),
                    attributes = scope_attributes
                )
                .unwrap();
                for profile in scope_profile.profiles.iter() {
                    let profile_id = String::from_utf8(profile.profile_id.clone());
                    let profile_samples = profile.sample.len();
                    if profile.attribute_indices.len() > 0 {
                        // attrs := []string{}

                        for index in profile.attribute_indices {
                            let attribute = resource_profile.attribute_table[index];
                            let attribute_key = attribute.key;
                            let attribute_value = attribute.value.to_string();
                            // attrs = append(attrs, fmt.Sprintf("%s=%s", a.Key(), a.Value().AsString()))
                        }

                        // buffer.WriteString(" ")
                        // buffer.WriteString(strings.Join(attrs, " "))
                    }
                }
            }
        }
        return report;
    }
}

// pub struct DetailedOTLPMarshaler;

// impl PDataMarshaler for DetailedOTLPMarshaler {
//     fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
//         let mut report = String::new();
//         for (index, resource_log) in log.resource_logs.iter().enumerate() {
//             let resource_schema_url = resource_log.schema_url;

//             // generate string for this
//             let resource_attributes = resource_log.resource.attributes;

//             for (index, scope_log) in resource_log.scope_logs.iter().enumerate() {
//                 // let scope_name = scope_log.scope.name
//                 // let scope_version = scope_log.scope.version
//                 // let scope_attributes = scope_log.scope.attributes;
//                 let scope_schema_url = scope_log.schema_url;
//                 let instrumentation_scope = scope_log.scope;
//                 // 			buffer.WriteString(fmt.Sprintf("ScopeLog #%d%s%s\n", i, writeScopeDetails(scopeLog.Scope().Name(), scopeLog.Scope().Version(), scopeLog.SchemaUrl()), write_attributesString(scopeLog.Scope().Attributes())))

//                 for (index, log_record) in scope_log.log_records.iter().enumerate() {
//                     let log_attributes = log_record.attributes;
//                     let log_body = log_record.body.to_string();

//                     let observed_timestamp = log_record.observed_time_unix_nano;
//                     let timestamp = log_record.time_unix_nano;
//                     let severity_text = log_record.severity_text;
//                     let severity_number = log_record.severity_number;
//                     // check if event name is empty before adding to report
//                     let event_name = log_record.event_name;
//                     let trace_id = log_record.trace_id;
//                     let span_id = log_record.span_id;
//                     let flags = log_record.flags;
//                 }
//             }
//         }
//         return report;
//     }
//     fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
//         let mut report = String::new();
//         for (index, resource_metric) in metric.resource_metrics.iter().enumerate() {
//             let resource_schema_url = resource_metric.schema_url;
//             let resource_attributes = resource_metric.resource.attributes;
//             for (index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
//                 // let scope_name = scope_metric.scope.name
//                 // let scope_version = scope_metric.scope.version
//                 let scope_schema_url = scope_metric.schema_url;
//                 // let scope_attributes = scope_metric.scope.attributes;
//                 let instrumentation_scope = scope_metric.scope;
//                 for (index, metric) in scope_metric.metrics.iter().enumerate() {
//                     let metric_name = metric.name;
//                     // let metric_description = metric.description;
//                     // let metric_unit = metric.unit;
//                     // let metric_metadata = metric.metadata;
//                     // let metric_data = metric.data;
//                     // buf.logMetricDescriptor(metric)
//                     // buf.logMetricDataPoints(metric)
//                 }
//             }
//         }
//         return report;
//     }
//     fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String {
//         let mut report = String::new();
//         for (index, resource_span) in trace.resource_spans.iter().enumerate() {
//             let resource_schema_url = resource_span.schema_url;
//             let resource_attributes = resource_span.resource.attributes;
//             for (index, scope_span) in resource_span.scope_spans.iter().enumerate() {
//                 // let scope_name = scope_span.scope.name
//                 // let scope_version = scope_span.scope.version
//                 let scope_schema_url = scope_span.schema_url;
//                 // let scope_attributes = scope_span.scope.attributes;
//                 let instrumentation_scope = scope_span.scope;
//                 for (index, span) in scope_span.spans.iter().enumerate() {
//                     let span_name = span.name;
//                     let span_trace_id = span.trace_id;
//                     let span_parent_span_id = span.parent_span_id;
//                     let span_attributes = span.attributes;
//                     let span_parent_id = span.parent_span_id;
//                     let span_span_id = span.span_id;
//                     let span_kind = span.kind;
//                     // check len
//                     // if ts := span.TraceState().AsRaw(); len(ts) != 0 {
//                     // 	buf.logAttr("TraceState", ts)
//                     // }
//                     let span_trace_state = span.trace_state;

//                     let span_start_timestamp = span.start_time_unix_nano;
//                     let span_end_timestamp = span.end_time_unix_nano;
//                     let span_events = span.events;
//                     let span_links = span.links;

//                     let span_status = span.status;
//                     if let Some(status) = span.status {
//                         let span_status_code = status.code;
//                         let span_status_message = status.message;
//                     }
//                 }
//             }
//         }
//         return report;
//     }
//     fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
//         let mut report = String::new();
//         for (index, resource_profile) in profile.resource_profiles.iter().enumerate() {
//             let resource_schema_url = resource_profile.schema_url;
//             let resource_attributes = resource_profile.resource.attributes;
//             for (index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
//                 let instrumentation_scope = scope_profile.scope;
//                 let scope_schema_url = scope_profile.schema_url;
//                 for (index, profile) in scope_profile.profiles.iter().enumerate() {
//                     let profile_id = String::from_utf8(profile.profile_id);
//                     let profile_start_time = profile.time;
//                     let profile_duration = profile.duration;
//                     let profile_dropped_attributes_count = profile.dropped_attributes_count;
//                     let profile_sample_type = profile.sample_type;

//                     // buf.logEntry("    Location indices: %d", profile.LocationIndices().AsRaw())

//                     // buf.logProfileSamples(profile.Sample(), dic.AttributeTable())
//                     // buf.logComment(profile.CommentStrindices())
//                     let profile_samples = profile.samples.len();
//                     let profile_comment = profile.comment;
//                     let profile_default_sample_type = profile.default_sample_type;
//                 }
//             }
//         }
//         return report;
//     }
// }

fn write_attributes(attributes: &Vec<KeyValue>) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        let mut attribute_value = String::new();
        if let Some(value) = &attribute.value {
            attribute_value = value.to_string();
        }
        write!(
            &mut attribute_string,
            "{key}={value} ",
            key = attribute.key,
            value = attribute_value
        )
        .unwrap();
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
                    // let datapoint_line = format!(
                    //     "{name} {attributes} {value}",
                    //     name = metric.name,
                    //     attributes = datapoint_attributes,
                    //     value = double
                    // );
                    // lines = lines.push_str(&datapoint_line);
                    write!(
                        &mut lines,
                        "{name} {attributes} {value}\n",
                        name = metric.name,
                        attributes = datapoint_attributes,
                        value = double
                    )
                    .unwrap();
                }
                NumberValue::AsInt(int) => {
                    // let datapoint_line = format!(
                    //     "{name} {attributes} {value}",
                    //     name = metric.name,
                    //     attributes = datapoint_attributes,
                    //     value = int
                    // );
                    // lines = lines.push_str(&datapoint_line);
                    write!(
                        &mut lines,
                        "{name} {attributes} {value}\n",
                        name = metric.name,
                        attributes = datapoint_attributes,
                        value = int
                    )
                    .unwrap();
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
        // values.push_str(&format!("count={} ", datapoint.count));
        write!(&mut values, "count={} ", datapoint.count).unwrap();
        if let Some(sum) = datapoint.sum {
            // values.push_str(&format!("sum={} ", sum));
            write!(&mut values, "sum={} ", sum).unwrap();
        }
        if let Some(min) = datapoint.min {
            // values.push_str(&format!("min={} ", min));
            write!(&mut values, "min={} ", min).unwrap();
        }
        if let Some(max) = datapoint.max {
            // values.push_str(&format!("max={} ", max));
            write!(&mut values, "max={} ", max).unwrap();
        }

        for (i, bucket) in datapoint.bucket_counts.iter().enumerate() {
            let mut bucket_bound = String::new();
            if i < datapoint.explicit_bounds.len() {
                bucket_bound = format!("le{}=", datapoint.explicit_bounds[i]);
            }
            write!(&mut values, "{}{} ", bucket_bound, bucket).unwrap();
            // values.push_str(&format!(" {}{} ", bucket_bound, bucket));
        }

        // let datapoint_line = format!("{} {} {}\n", metric.name, datapoint_attributes, values);
        // lines.push_str(&datapoint_line);
        write!(
            &mut lines,
            "{name} {attributes} {values}\n",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        )
        .unwrap();
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

        // values.push_str(&format!("count={}", datapoint.count));
        write!(&mut values, "count={} ", datapoint.count).unwrap();

        if let Some(sum) = datapoint.sum {
            // values.push_str(&format!(" sum={}", sum));
            write!(&mut values, "sum={} ", sum).unwrap();
        }
        if let Some(min) = datapoint.min {
            // values.push_str(&format!("min={} ", min));
            write!(&mut values, "min={} ", min).unwrap();
        }
        if let Some(max) = datapoint.max {
            // values.push_str(&format!("max={} ", max));
            write!(&mut values, "max={} ", max).unwrap();
        }

        // let datapoint_line = format!("{} {} {}\n", metric.name, datapoint_attributes, values);
        // lines.push_str(&datapoint_line);
        write!(
            &mut lines,
            "{name} {attributes} {values}\n",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        )
        .unwrap();
    }
    return lines;
}

fn write_summary_datapoints(metric: &Metric, datapoints: &Vec<SummaryDataPoint>) -> String {
    let mut lines = String::new();
    for datapoint in datapoints.iter() {
        let datapoint_attributes = write_attributes(&datapoint.attributes);
        let mut values = String::new();
        // values.push_str(&format!("count={}", datapoint.count));
        // values.push_str(&format!(" sum={}", datapoint.sum));

        write!(&mut values, "count={} ", datapoint.count).unwrap();
        write!(&mut values, "sum={} ", datapoint.sum).unwrap();

        for quantile in datapoint.quantile_values.iter() {
            // values.push_str(&format!(" q{}={} ", quantile.quantile, quantile.value));
            write!(&mut values, "q{}={} ", quantile.quantile, quantile.value).unwrap();
        }

        // let datapoint_line = format!("{} {} {}\n", metric.name, datapoint_attributes, values);
        // lines.push_str(&datapoint_line);
        write!(
            &mut lines,
            "{name} {attributes} {values}\n",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        )
        .unwrap();
    }
    return lines;
}

// // func (normalMetricsMarshaler) MarshalMetrics(md pmetric.Metrics) ([]byte, error) {
// // 	var buffer bytes.Buffer
// // 	for i := 0; i < md.ResourceMetrics().Len(); i++ {
// // 		resourceMetrics := md.ResourceMetrics().At(i)

// // 		buffer.WriteString(fmt.Sprintf("ResourceMetrics #%d%s%s\n", i, writeResourceDetails(resourceMetrics.SchemaUrl()), writeAttributesString(resourceMetrics.Resource().Attributes())))

// // 		for j := 0; j < resourceMetrics.ScopeMetrics().Len(); j++ {
// // 			scopeMetrics := resourceMetrics.ScopeMetrics().At(j)

// // 			buffer.WriteString(fmt.Sprintf("ScopeMetrics #%d%s%s\n", i, writeScopeDetails(scopeMetrics.Scope().Name(), scopeMetrics.Scope().Version(), scopeMetrics.SchemaUrl()), writeAttributesString(scopeMetrics.Scope().Attributes())))

// // 			for k := 0; k < scopeMetrics.Metrics().Len(); k++ {
// // 				metric := scopeMetrics.Metrics().At(k)

// // 				var dataPointLines []string
// // 				switch metric.Type() {
// // 				case pmetric.MetricTypeGauge:
// // 					dataPointLines = writeNumberDataPoints(metric, metric.Gauge().DataPoints())
// // 				case pmetric.MetricTypeSum:
// // 					dataPointLines = writeNumberDataPoints(metric, metric.Sum().DataPoints())
// // 				case pmetric.MetricTypeHistogram:
// // 					dataPointLines = writeHistogramDataPoints(metric)
// // 				case pmetric.MetricTypeExponentialHistogram:
// // 					dataPointLines = writeExponentialHistogramDataPoints(metric)
// // 				case pmetric.MetricTypeSummary:
// // 					dataPointLines = writeSummaryDataPoints(metric)
// // 				}
// // 				for _, line := range dataPointLines {
// // 					buffer.WriteString(line)
// // 				}
// // 			}
// // 		}
// // 	}
// // 	return buffer.Bytes(), nil
// // }

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
