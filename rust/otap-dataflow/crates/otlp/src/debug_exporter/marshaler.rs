use crate::proto::opentelemetry::collector::{
    logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceReques,
    profiles::v1development::ExportProfilesServiceRequest, trace::v1::ExportTraceServiceRequest,
};
pub trait PDataMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String;
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String;
    fn marshal_traces(&self, traces: ExportTracesServiceRequest) -> String;
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String;
}

pub struct NormalOTLPMarshaler;

impl PDataMarshaler for NormalOTLPMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_log) in log.resource_logs.iter().enumerate() {
            let resource_schema_url = resource_log.schema_url;

            // generate string for this
            let resource_attributes = resource_log.resource.attributes;

            // 		buffer.WriteString(fmt.Sprintf("ResourceLog #%d%s%s\n", i, writeResourceDetails(resourceLog.SchemaUrl()), writeAttributesString(resourceLog.Resource().Attributes())))

            for (index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                let scope_name = scope_log.scope.name;
                let scope_version = scope_log.scope.version;
                let scope_schema_url = scope_log.schema_url;
                let scope_attributes = scope_log.scope.attributes;
                // 			buffer.WriteString(fmt.Sprintf("ScopeLog #%d%s%s\n", i, writeScopeDetails(scopeLog.Scope().Name(), scopeLog.Scope().Version(), scopeLog.SchemaUrl()), writeAttributesString(scopeLog.Scope().Attributes())))

                for (index, log_record) in scope_log.log_records.iter().enumerate() {
                    let log_attributes = log_record.attributes;
                    let log_body = log_record.body.to_string();
                    // logString := fmt.Sprintf("%s %s", logRecord.Body().AsString(), strings.Join(logAttributes, " "))
                    // buffer.WriteString(logString)
                    // buffer.WriteString("\n")
                }
            }
        }

        return report;
    }
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_metric) in metric.resource_metrics.iter().enumerate() {
            let resource_schema_url = resource_metric.schema_url;
            let resource_attributes = resource_metric.resource.attributes;
            for (index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                let scope_name = scope_metric.scope.name;
                let scope_version = scope_metric.scope.version;
                let scope_schema_url = scope_metric.schema_url;
                let scope_attributes = scope_metric.scope.attributes;
                for (index, metric) in scope_metric.metrics.iter().enumerate() {
                    let metric_name = metric.name;

                    if let Some(data) = metric.data() {
                        let data_point_lines = match data {
                            metric::Data::Gauge(gauge) => {
                                writeNumberDataPoints(&metric, gauge.data_points);
                            }
                            metric::Data::Sum(sum) => {
                                writeNumberDataPoints(&metric, sum.data_points);
                            }
                            metric::Data::Histogram(histogram) => {
                                writeHistogramDataPoints(&metric);
                            }
                            metric::Data::ExponentialHistogram(exponential_histogram) => {
                                writeExponentialDataPoints(&metric);
                            }
                            metric::Data::Summary(summary) => {
                                writeSummaryDataPoints(&metric);
                            }
                        };
                    }
                    // ToDo: put data_points into report
                }
            }
        }
        return report;
    }
    fn marshal_traces(&self, traces: ExportTracesServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_span) in trace.resource_spans.iter().enumerate() {
            let resource_schema_url = resource_span.schema_url;
            let resource_attributes = resource_span.resource.attributes;
            for (index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                let scope_name = scope_span.scope.name;
                let scope_version = scope_span.scope.version;
                let scope_schema_url = scope_span.schema_url;
                let scope_attributes = scope_span.scope.attributes;
                for (index, span) in scope_span.spans.iter().enumerate() {
                    let span_name = span.name;
                    let span_trace_id = String::from_utf8(span.trace_id)?;
                    let span_span_id = String::from_utf8(span.span_id)?;

                    // check len of attributes
                    let span_attributes = span.attributes;
                    // if span.Attributes().Len() > 0 {
                    // 	spanAttributes := writeAttributes(span.Attributes())
                    // 	buffer.WriteString(" ")
                    // 	buffer.WriteString(strings.Join(spanAttributes, " "))
                    // }
                }
            }
        }
        return report;
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        // marshal_profiles to string based on verbosity
        let mut report = String::new();
        for (index, resource_profile) in profile.resource_profiles.iter().enumerate() {
            let resource_schema_url = resource_profile.schema_url;
            let resource_attributes = resource_profile.resource.attributes;
            for (index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                let scope_name = scope_profile.scope.name;
                let scope_version = scope_profile.scope.version;
                let scope_schema_url = scope_profile.schema_url;
                let scope_attributes = scope_profile.scope.attributes;
                for (index, profile) in scope_profile.profiles.iter().enumerate() {
                    let profile_id = String::from_utf8(profile.profile_id);
                    let profile_samples = profile.samples.len();
                    if profile.attribute_indices.len() > 0 {
                        // attrs := []string{}

                        for index in profile.attribute_indices {
                            attribute = profile.attribute_table[index];
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

pub struct DetailedOTLPMarshaler;

impl PDataMarshaler for DetailedOTLPMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_log) in log.resource_logs.iter().enumerate() {
            let resource_schema_url = resource_log.schema_url;

            // generate string for this
            let resource_attributes = resource_log.resource.attributes;

            for (index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                // let scope_name = scope_log.scope.name
                // let scope_version = scope_log.scope.version
                // let scope_attributes = scope_log.scope.attributes;
                let scope_schema_url = scope_log.schema_url;
                let instrumentation_scope = scope_log.scope;
                // 			buffer.WriteString(fmt.Sprintf("ScopeLog #%d%s%s\n", i, writeScopeDetails(scopeLog.Scope().Name(), scopeLog.Scope().Version(), scopeLog.SchemaUrl()), writeAttributesString(scopeLog.Scope().Attributes())))

                for (index, log_record) in scope_log.log_records.iter().enumerate() {
                    let log_attributes = log_record.attributes;
                    let log_body = log_record.body.to_string();

                    let observed_timestamp = log_record.observed_time_unix_nano;
                    let timestamp = log_record.time_unix_nano;
                    let severity_text = log_record.severity_text;
                    let severity_number = log_record.severity_number;
                    // check if event name is empty before adding to report
                    let event_name = log_record.event_name;
                    let trace_id = log_record.trace_id;
                    let span_id = log_record.span_id;
                    let flags = log_record.flags;
                }
            }
        }
        return report;
    }
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_metric) in metric.resource_metrics.iter().enumerate() {
            let resource_schema_url = resource_metric.schema_url;
            let resource_attributes = resource_metric.resource.attributes;
            for (index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                // let scope_name = scope_metric.scope.name
                // let scope_version = scope_metric.scope.version
                let scope_schema_url = scope_metric.schema_url;
                // let scope_attributes = scope_metric.scope.attributes;
                let instrumentation_scope = scope_metric.scope;
                for (index, metric) in scope_metric.metrics.iter().enumerate() {
                    let metric_name = metric.name;
                    // let metric_description = metric.description;
                    // let metric_unit = metric.unit;
                    // let metric_metadata = metric.metadata;
                    // let metric_data = metric.data;
                    // buf.logMetricDescriptor(metric)
                    // buf.logMetricDataPoints(metric)
                }
            }
        }
        return report;
    }
    fn marshal_traces(&self, traces: ExportTracesServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_span) in trace.resource_spans.iter().enumerate() {
            let resource_schema_url = resource_span.schema_url;
            let resource_attributes = resource_span.resource.attributes;
            for (index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                // let scope_name = scope_span.scope.name
                // let scope_version = scope_span.scope.version
                let scope_schema_url = scope_span.schema_url;
                // let scope_attributes = scope_span.scope.attributes;
                let instrumentation_scope = scope_span.scope;
                for (index, span) in scope_span.spans.iter().enumerate() {
                    let span_name = span.name;
                    let span_trace_id = span.trace_id;
                    let span_parent_span_id = span.parent_span_id;
                    let span_attributes = span.attributes;
                    let span_parent_id = span.parent_span_id;
                    let span_span_id = span.span_id;
                    let span_kind = span.kind;
                    // check len
                    // if ts := span.TraceState().AsRaw(); len(ts) != 0 {
                    // 	buf.logAttr("TraceState", ts)
                    // }
                    let span_trace_state = span.trace_state;

                    let span_start_timestamp = span.start_time_unix_nano;
                    let span_end_timestamp = span.end_time_unix_nano;
                    let span_events = span.events;
                    let span_links = span.links;

                    let span_status = span.status;
                    if let Some(status) = span.status {
                        let span_status_code = status.code;
                        let span_status_message = status.message;
                    }
                }
            }
        }
        return report;
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        let mut report = String::new();
        for (index, resource_profile) in profile.resource_profiles.iter().enumerate() {
            let resource_schema_url = resource_profile.schema_url;
            let resource_attributes = resource_profile.resource.attributes;
            for (index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                let instrumentation_scope = scope_profile.scope;
                let scope_schema_url = scope_profile.schema_url;
                for (index, profile) in scope_profile.profiles.iter().enumerate() {
                    let profile_id = String::from_utf8(profile.profile_id)?;
                    let profile_start_time = profile.time;
                    let profile_duration = profile.duration;
                    let profile_dropped_attributes_count = profile.dropped_attributes_count;
                    let profile_sample_type = profile.sample_type;

                    // buf.logEntry("    Location indices: %d", profile.LocationIndices().AsRaw())

                    // buf.logProfileSamples(profile.Sample(), dic.AttributeTable())
                    // buf.logComment(profile.CommentStrindices())
                    let profile_samples = profile.samples.len();
                    let profile_comment = profile.comment;
                    let profile_default_sample_type = profile.default_sample_type;
                }
            }
        }
        return report;
    }
}

// // writeAttributes returns a slice of strings in the form "attrKey=attrValue"
// func writeAttributes(attributes pcommon.Map) (attributeStrings []string) {
// 	for k, v := range attributes.All() {
// 		attribute := fmt.Sprintf("%s=%s", k, v.AsString())
// 		attributeStrings = append(attributeStrings, attribute)
// 	}
// 	return attributeStrings
// }

// // writeAttributesString returns a string in the form " attrKey=attrValue attr2=value2"
// func writeAttributesString(attributesMap pcommon.Map) (attributesString string) {
// 	attributes := writeAttributes(attributesMap)
// 	if len(attributes) > 0 {
// 		attributesString = " " + strings.Join(attributes, " ")
// 	}
// 	return attributesString
// }

// func writeNumberDataPoints(metric pmetric.Metric, dataPoints pmetric.NumberDataPointSlice) (lines []string) {
// 	for i := 0; i < dataPoints.Len(); i++ {
// 		dataPoint := dataPoints.At(i)
// 		dataPointAttributes := writeAttributes(dataPoint.Attributes())

// 		var value string
// 		switch dataPoint.ValueType() {
// 		case pmetric.NumberDataPointValueTypeInt:
// 			value = strconv.FormatInt(dataPoint.IntValue(), 10)
// 		case pmetric.NumberDataPointValueTypeDouble:
// 			value = fmt.Sprintf("%v", dataPoint.DoubleValue())
// 		}

// 		dataPointLine := fmt.Sprintf("%s{%s} %s\n", metric.Name(), strings.Join(dataPointAttributes, ","), value)
// 		lines = append(lines, dataPointLine)
// 	}
// 	return lines
// }

// func writeHistogramDataPoints(metric pmetric.Metric) (lines []string) {
// 	for i := 0; i < metric.Histogram().DataPoints().Len(); i++ {
// 		dataPoint := metric.Histogram().DataPoints().At(i)
// 		dataPointAttributes := writeAttributes(dataPoint.Attributes())

// 		var value string
// 		value = fmt.Sprintf("count=%d", dataPoint.Count())
// 		if dataPoint.HasSum() {
// 			value += fmt.Sprintf(" sum=%v", dataPoint.Sum())
// 		}
// 		if dataPoint.HasMin() {
// 			value += fmt.Sprintf(" min=%v", dataPoint.Min())
// 		}
// 		if dataPoint.HasMax() {
// 			value += fmt.Sprintf(" max=%v", dataPoint.Max())
// 		}

// 		for bucketIndex := 0; bucketIndex < dataPoint.BucketCounts().Len(); bucketIndex++ {
// 			bucketBound := ""
// 			if bucketIndex < dataPoint.ExplicitBounds().Len() {
// 				bucketBound = fmt.Sprintf("le%v=", dataPoint.ExplicitBounds().At(bucketIndex))
// 			}
// 			bucketCount := dataPoint.BucketCounts().At(bucketIndex)
// 			value += fmt.Sprintf(" %s%d", bucketBound, bucketCount)
// 		}

// 		dataPointLine := fmt.Sprintf("%s{%s} %s\n", metric.Name(), strings.Join(dataPointAttributes, ","), value)
// 		lines = append(lines, dataPointLine)
// 	}
// 	return lines
// }

// func writeExponentialHistogramDataPoints(metric pmetric.Metric) (lines []string) {
// 	for i := 0; i < metric.ExponentialHistogram().DataPoints().Len(); i++ {
// 		dataPoint := metric.ExponentialHistogram().DataPoints().At(i)
// 		dataPointAttributes := writeAttributes(dataPoint.Attributes())

// 		var value string
// 		value = fmt.Sprintf("count=%d", dataPoint.Count())
// 		if dataPoint.HasSum() {
// 			value += fmt.Sprintf(" sum=%v", dataPoint.Sum())
// 		}
// 		if dataPoint.HasMin() {
// 			value += fmt.Sprintf(" min=%v", dataPoint.Min())
// 		}
// 		if dataPoint.HasMax() {
// 			value += fmt.Sprintf(" max=%v", dataPoint.Max())
// 		}

// 		// TODO display buckets

// 		dataPointLine := fmt.Sprintf("%s{%s} %s\n", metric.Name(), strings.Join(dataPointAttributes, ","), value)
// 		lines = append(lines, dataPointLine)
// 	}
// 	return lines
// }

// func writeSummaryDataPoints(metric pmetric.Metric) (lines []string) {
// 	for i := 0; i < metric.Summary().DataPoints().Len(); i++ {
// 		dataPoint := metric.Summary().DataPoints().At(i)
// 		dataPointAttributes := writeAttributes(dataPoint.Attributes())

// 		var value string
// 		value = fmt.Sprintf("count=%d", dataPoint.Count())
// 		value += fmt.Sprintf(" sum=%f", dataPoint.Sum())

// 		for quantileIndex := 0; quantileIndex < dataPoint.QuantileValues().Len(); quantileIndex++ {
// 			quantile := dataPoint.QuantileValues().At(quantileIndex)
// 			value += fmt.Sprintf(" q%v=%v", quantile.Quantile(), quantile.Value())
// 		}

// 		dataPointLine := fmt.Sprintf("%s{%s} %s\n", metric.Name(), strings.Join(dataPointAttributes, ","), value)
// 		lines = append(lines, dataPointLine)
// 	}
// 	return lines
// }

// func (normalMetricsMarshaler) MarshalMetrics(md pmetric.Metrics) ([]byte, error) {
// 	var buffer bytes.Buffer
// 	for i := 0; i < md.ResourceMetrics().Len(); i++ {
// 		resourceMetrics := md.ResourceMetrics().At(i)

// 		buffer.WriteString(fmt.Sprintf("ResourceMetrics #%d%s%s\n", i, writeResourceDetails(resourceMetrics.SchemaUrl()), writeAttributesString(resourceMetrics.Resource().Attributes())))

// 		for j := 0; j < resourceMetrics.ScopeMetrics().Len(); j++ {
// 			scopeMetrics := resourceMetrics.ScopeMetrics().At(j)

// 			buffer.WriteString(fmt.Sprintf("ScopeMetrics #%d%s%s\n", i, writeScopeDetails(scopeMetrics.Scope().Name(), scopeMetrics.Scope().Version(), scopeMetrics.SchemaUrl()), writeAttributesString(scopeMetrics.Scope().Attributes())))

// 			for k := 0; k < scopeMetrics.Metrics().Len(); k++ {
// 				metric := scopeMetrics.Metrics().At(k)

// 				var dataPointLines []string
// 				switch metric.Type() {
// 				case pmetric.MetricTypeGauge:
// 					dataPointLines = writeNumberDataPoints(metric, metric.Gauge().DataPoints())
// 				case pmetric.MetricTypeSum:
// 					dataPointLines = writeNumberDataPoints(metric, metric.Sum().DataPoints())
// 				case pmetric.MetricTypeHistogram:
// 					dataPointLines = writeHistogramDataPoints(metric)
// 				case pmetric.MetricTypeExponentialHistogram:
// 					dataPointLines = writeExponentialHistogramDataPoints(metric)
// 				case pmetric.MetricTypeSummary:
// 					dataPointLines = writeSummaryDataPoints(metric)
// 				}
// 				for _, line := range dataPointLines {
// 					buffer.WriteString(line)
// 				}
// 			}
// 		}
// 	}
// 	return buffer.Bytes(), nil
// }
