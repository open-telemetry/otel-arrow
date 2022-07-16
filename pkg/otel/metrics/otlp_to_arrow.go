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
	"github.com/apache/arrow/go/v9/arrow"
	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/rfield"
)

type MultivariateMetricsConfig struct {
	Metrics map[string]string
}

type MultivariateRecord struct {
	fields  []*rfield.Field
	metrics []*rfield.Field
}

func OtlpMetricsToArrowEvents(rbr *rbb.RecordRepository, request *collogspb.ExportMetricsServiceRequest, multivariateConf *MultivariateMetricsConfig) (map[string][]arrow.Record, error) {
	result := make(map[string][]arrow.Record)
	for _, resourceMetrics := range request.ResourceMetrics {
		for _, scopeMetrics := range resourceMetrics.ScopeMetrics {
			for _, metric := range scopeMetrics.Metrics {
				if metric.Data != nil {
					switch metric.Data.(type) {
					case *metricspb.Metric_Gauge:
						err := addMetric(rbr, resourceMetrics, scopeMetrics, metric.Name, metric.Data.(*metricspb.Metric_Gauge).Gauge.DataPoints, multivariateConf)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Sum:
						err := addMetric(rbr, resourceMetrics, scopeMetrics, metric.Name, metric.Data.(*metricspb.Metric_Sum).Sum.DataPoints, multivariateConf)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Histogram:
						// ToDo Metric Histogram
						return nil, nil
					case *metricspb.Metric_Summary:
						// ToDo Metric Summary
						return nil, nil
					case *metricspb.Metric_ExponentialHistogram:
						// ToDo Metric Exponential Histogram
						return nil, nil
					default:
						errorString := fmt.Sprintf("Unsupported metric type: %v", metric.Data)
						panic(errorString)
					}
				}
			}
			records, err := rbr.Build()
			if err != nil {
				return nil, err
			}
			for schemaId, record := range records {
				allRecords := result[schemaId]
				if allRecords == nil {
					result[schemaId] = []arrow.Record{record}
				} else {
					result[schemaId] = append(allRecords, record)
				}
			}
		}
	}
	return result, nil
}

func addMetric(rbr *rbb.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, dataPoints []*metricspb.NumberDataPoint, config *MultivariateMetricsConfig) error {
	if mvKey, ok := config.Metrics[metricName]; ok {
		return multivariateMetric(rbr, resMetrics, scopeMetrics, dataPoints, mvKey)
	} else {
		univariateMetric(rbr, resMetrics, scopeMetrics, metricName, dataPoints)
		return nil
	}
}

// ToDo initial metric name is lost, it should be recorded as metadata or constant column
func multivariateMetric(rbr *rbb.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, dataPoints []*metricspb.NumberDataPoint, multivariateKey string) error {
	records := make(map[string]*MultivariateRecord)

	for _, ndp := range dataPoints {
		sig := DataPointSig(ndp, multivariateKey)
		newEntry := false
		stringSig := string(sig)
		record := records[stringSig]

		if record == nil {
			newEntry = true
			record = &MultivariateRecord{
				fields:  []*rfield.Field{},
				metrics: []*rfield.Field{},
			}
			records[stringSig] = record
		}

		var multivariateMetricName *string
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
			emptyString := ""
			multivariateMetricName = &emptyString
		}

		switch ndp.Value.(type) {
		case *metricspb.NumberDataPoint_AsDouble:
			record.metrics = append(record.metrics, rfield.NewF64Field(*multivariateMetricName, ndp.Value.(*metricspb.NumberDataPoint_AsDouble).AsDouble))
		case *metricspb.NumberDataPoint_AsInt:
			record.metrics = append(record.metrics, rfield.NewI64Field(*multivariateMetricName, ndp.Value.(*metricspb.NumberDataPoint_AsInt).AsInt))
		default:
			panic("Unsupported number data point value type")
		}
	}

	for _, record := range records {
		if len(record.fields) == 0 && len(record.metrics) == 0 {
			continue
		}
		record.fields = append(record.fields, rfield.NewStructField(constants.METRICS, rfield.Struct{
			Fields: record.metrics,
		}))
		rbr.AddRecord(rbb.NewRecordFromFields(record.fields))
	}
	return nil
}

func univariateMetric(rbr *rbb.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, dataPoints []*metricspb.NumberDataPoint) {
	for _, ndp := range dataPoints {
		record := rbb.NewRecord()

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

		attributes := common.NewAttributes(ndp.Attributes)
		if attributes != nil {
			record.AddField(attributes)
		}

		if ndp.Value != nil {
			switch ndp.Value.(type) {
			case *metricspb.NumberDataPoint_AsDouble:
				record.StructField(constants.METRICS, rfield.Struct{
					Fields: []*rfield.Field{
						rfield.NewF64Field(metricName, ndp.Value.(*metricspb.NumberDataPoint_AsDouble).AsDouble),
					},
				})
			case *metricspb.NumberDataPoint_AsInt:
				record.StructField(constants.METRICS, rfield.Struct{
					Fields: []*rfield.Field{
						rfield.NewI64Field(metricName, ndp.Value.(*metricspb.NumberDataPoint_AsInt).AsInt),
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

		rbr.AddRecord(record)
	}
}

func ExtractMultivariateValue(attributes []*commonpb.KeyValue, multivariateKey string) (*string, error) {
	for _, attribute := range attributes {
		if attribute.GetKey() == multivariateKey {
			value := attribute.GetValue().Value
			switch value.(type) {
			case *commonpb.AnyValue_StringValue:
				return &value.(*commonpb.AnyValue_StringValue).StringValue, nil
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
				switch value.(type) {
				case *commonpb.AnyValue_StringValue:
					multivariateValue = &value.(*commonpb.AnyValue_StringValue).StringValue
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
