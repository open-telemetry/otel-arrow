// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package metrics

import (
	"fmt"

	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"

	"github.com/apache/arrow/go/v9/arrow"
)

type MultivariateMetricsConfig struct {
	Metrics map[string]string
}

type MultivariateRecord struct {
	fields  []*rfield.Field
	metrics []*rfield.Field
}

// OtlpMetricsToArrowRecords converts an OTLP ResourceMetrics to one or more Arrow records.
func OtlpMetricsToArrowRecords(rr *air.RecordRepository, request *collogspb.ExportMetricsServiceRequest, multivariateConf *MultivariateMetricsConfig) (map[string][]arrow.Record, error) {
	result := make(map[string][]arrow.Record)
	for _, resourceMetrics := range request.ResourceMetrics {
		for _, scopeMetrics := range resourceMetrics.ScopeMetrics {
			for _, metric := range scopeMetrics.Metrics {
				if metric.Data != nil {
					switch t := metric.Data.(type) {
					case *metricspb.Metric_Gauge:
						err := addMetric(rr, resourceMetrics, scopeMetrics, metric.Name, t.Gauge.DataPoints, constants.GAUGE_METRICS, multivariateConf)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Sum:
						err := addMetric(rr, resourceMetrics, scopeMetrics, metric.Name, t.Sum.DataPoints, constants.SUM_METRICS, multivariateConf)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Histogram:
						// ToDo Metric Histogram
						return nil, nil
					case *metricspb.Metric_Summary:
						err := addSummaryMetric(rr, resourceMetrics, scopeMetrics, metric.Name, t.Summary)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_ExponentialHistogram:
						// ToDo Metric Exponential Histogram
						return nil, nil
					default:
						errorString := fmt.Sprintf("Unsupported metric type: %v", metric.Data)
						panic(errorString)
					}
				}
			}
			records, err := rr.Build()
			if err != nil {
				return nil, err
			}
			for schemaId, record := range records {
				allRecords := result[schemaId]
				result[schemaId] = append(allRecords, record)
			}
		}
	}
	return result, nil
}

func addMetric(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, dataPoints []*metricspb.NumberDataPoint, metric_type string, config *MultivariateMetricsConfig) error {
	if mvKey, ok := config.Metrics[metricName]; ok {
		return multivariateMetric(rr, resMetrics, scopeMetrics, metricName, dataPoints, metric_type, mvKey)
	}
	univariateMetric(rr, resMetrics, scopeMetrics, metricName, dataPoints, metric_type)
	return nil
}

// ToDo initial metric name is lost, it should be recorded as metadata or constant column
func multivariateMetric(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, dataPoints []*metricspb.NumberDataPoint, metric_type string, multivariateKey string) error {
	records := make(map[string]*MultivariateRecord)

	for _, ndp := range dataPoints {
		sig := DataPointSig(ndp, multivariateKey)
		newEntry := false
		stringSig := string(sig)
		record := records[stringSig]

		var multivariateMetricName *string

		if record == nil {
			newEntry = true
			record = &MultivariateRecord{
				fields:  []*rfield.Field{},
				metrics: []*rfield.Field{},
			}
			records[stringSig] = record
		}

		if newEntry {
			if resMetrics.Resource != nil {
				record.fields = append(record.fields, common.ResourceField(resMetrics.Resource))
			}
			if scopeMetrics.Scope != nil {
				record.fields = append(record.fields, common.ScopeField(constants.SCOPE_METRICS, scopeMetrics.Scope))
			}
			timeUnixNanoField := rfield.NewU64Field(constants.TIME_UNIX_NANO, ndp.TimeUnixNano)
			record.fields = append(record.fields, timeUnixNanoField)
			if ndp.StartTimeUnixNano > 0 {
				startTimeUnixNano := rfield.NewU64Field(constants.START_TIME_UNIX_NANO, ndp.StartTimeUnixNano)
				record.fields = append(record.fields, startTimeUnixNano)
			}
			ma, err := AddMultivariateValue(ndp.Attributes, multivariateKey, &record.fields)
			if err != nil {
				return err
			}
			multivariateMetricName = ma
		} else {
			ma, err := ExtractMultivariateValue(ndp.Attributes, multivariateKey)
			if err != nil {
				return err
			}
			multivariateMetricName = ma
		}

		if multivariateMetricName == nil {
			multivariateMetricName = new(string)
		}

		switch t := ndp.Value.(type) {
		case *metricspb.NumberDataPoint_AsDouble:
			record.metrics = append(record.metrics, rfield.NewF64Field(*multivariateMetricName, t.AsDouble))
		case *metricspb.NumberDataPoint_AsInt:
			record.metrics = append(record.metrics, rfield.NewI64Field(*multivariateMetricName, t.AsInt))
		default:
			panic("Unsupported number data point value type")
		}
	}

	for _, record := range records {
		if len(record.fields) == 0 && len(record.metrics) == 0 {
			continue
		}
		record.fields = append(record.fields, rfield.NewStructField(fmt.Sprintf("%s_%s", metric_type, metricName), rfield.Struct{
			Fields: record.metrics,
		}))
		rr.AddRecord(air.NewRecordFromFields(record.fields))
	}
	return nil
}

func univariateMetric(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, dataPoints []*metricspb.NumberDataPoint, metric_type string) {
	for _, ndp := range dataPoints {
		record := air.NewRecord()

		if resMetrics.Resource != nil {
			common.AddResource(record, resMetrics.Resource)
		}
		if scopeMetrics.Scope != nil {
			common.AddScope(record, constants.SCOPE_METRICS, scopeMetrics.Scope)
		}

		record.U64Field(constants.TIME_UNIX_NANO, ndp.TimeUnixNano)
		if ndp.StartTimeUnixNano > 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, ndp.StartTimeUnixNano)
		}

		if attributes := common.NewAttributes(ndp.Attributes); attributes != nil {
			record.AddField(attributes)
		}

		if ndp.Value != nil {
			switch t := ndp.Value.(type) {
			case *metricspb.NumberDataPoint_AsDouble:
				record.StructField(fmt.Sprintf("%s_%s", metric_type, metricName), rfield.Struct{
					Fields: []*rfield.Field{
						rfield.NewF64Field(constants.METRIC_VALUE, t.AsDouble),
					},
				})
			case *metricspb.NumberDataPoint_AsInt:
				record.StructField(fmt.Sprintf("%s_%s", metric_type, metricName), rfield.Struct{
					Fields: []*rfield.Field{
						rfield.NewI64Field(constants.METRIC_VALUE, t.AsInt),
					},
				})
			default:
				panic("Unsupported number data point value type")
			}
		}

		// ToDo Exemplar

		if ndp.Flags > 0 {
			record.U32Field(constants.FLAGS, ndp.Flags)
		}

		rr.AddRecord(record)
	}
}

func addSummaryMetric(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, summary *metricspb.Summary) error {
	for _, sdp := range summary.DataPoints {
		record := air.NewRecord()

		if resMetrics.Resource != nil {
			common.AddResource(record, resMetrics.Resource)
		}
		if scopeMetrics.Scope != nil {
			common.AddScope(record, constants.SCOPE_METRICS, scopeMetrics.Scope)
		}

		record.U64Field(constants.TIME_UNIX_NANO, sdp.TimeUnixNano)
		if sdp.StartTimeUnixNano > 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, sdp.StartTimeUnixNano)
		}

		record.U64Field(constants.SUMMARY_COUNT, sdp.Count)
		record.F64Field(constants.SUMMARY_SUM, sdp.Sum)

		if attributes := common.NewAttributes(sdp.Attributes); attributes != nil {
			record.AddField(attributes)
		}

		var items []rfield.Value
		for _, quantile := range sdp.QuantileValues {
			items = append(items, &rfield.Struct{
				Fields: []*rfield.Field{
					rfield.NewF64Field(constants.SUMMARY_QUANTILE, quantile.Quantile),
					rfield.NewF64Field(constants.SUMMARY_VALUE, quantile.Value),
				},
			})
		}
		record.ListField(fmt.Sprintf("%s_%s", constants.SUMMARY_QUANTILE_VALUES, metricName), rfield.List{Values: items})

		if sdp.Flags > 0 {
			record.U32Field(constants.FLAGS, sdp.Flags)
		}

		rr.AddRecord(record)
	}
	return nil
}

func ExtractMultivariateValue(attributes []*commonpb.KeyValue, multivariateKey string) (*string, error) {
	for _, attribute := range attributes {
		if attribute.GetKey() == multivariateKey {
			value := attribute.GetValue().Value
			switch t := value.(type) {
			case *commonpb.AnyValue_StringValue:
				return &t.StringValue, nil
			default:
				return nil, fmt.Errorf("Unsupported multivariate value type: %v", value)
			}
		}
	}
	return nil, nil
}

func AddMultivariateValue(attributes []*commonpb.KeyValue, multivariateKey string, fields *[]*rfield.Field) (*string, error) {
	var multivariateValue *string
	attributeFields := make([]*rfield.Field, 0, len(attributes))
	for _, attribute := range attributes {
		if attribute.Value != nil {
			if attribute.GetKey() == multivariateKey {
				value := attribute.GetValue().Value
				switch t := value.(type) {
				case *commonpb.AnyValue_StringValue:
					multivariateValue = &t.StringValue
				default:
					return nil, fmt.Errorf("Unsupported multivariate value type: %v", value)
				}
			}
		}
		attributeFields = append(attributeFields, rfield.NewField(attribute.GetKey(), common.OtlpAnyValueToValue(attribute.GetValue())))
	}
	if len(attributeFields) > 0 {
		*fields = append(*fields, rfield.NewStructField(constants.ATTRIBUTES, rfield.Struct{Fields: attributeFields}))
	}
	return multivariateValue, nil
}
