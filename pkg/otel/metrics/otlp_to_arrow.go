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

func NewMultivariateMetricsConfig() *MultivariateMetricsConfig {
	return &MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
}

// OtlpMetricsToArrowRecords converts an OTLP ResourceMetrics to one or more Arrow records.
func OtlpMetricsToArrowRecords(rr *air.RecordRepository, request *collogspb.ExportMetricsServiceRequest, multivariateConf *MultivariateMetricsConfig) ([]arrow.Record, error) {
	result := []arrow.Record{}
	for _, resourceMetrics := range request.ResourceMetrics {
		for _, scopeMetrics := range resourceMetrics.ScopeMetrics {
			for _, metric := range scopeMetrics.Metrics {
				if metric.Data != nil {
					switch t := metric.Data.(type) {
					case *metricspb.Metric_Gauge:
						err := addGaugeOrSum(rr, resourceMetrics, scopeMetrics, metric.Name, t.Gauge.DataPoints, constants.GAUGE_METRICS, multivariateConf)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Sum:
						err := addGaugeOrSum(rr, resourceMetrics, scopeMetrics, metric.Name, t.Sum.DataPoints, constants.SUM_METRICS, multivariateConf)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Histogram:
						err := addHistogram(rr, resourceMetrics, scopeMetrics, metric.Name, t.Histogram)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_Summary:
						err := addSummary(rr, resourceMetrics, scopeMetrics, metric.Name, t.Summary)
						if err != nil {
							return nil, err
						}
					case *metricspb.Metric_ExponentialHistogram:
						err := addExpHistogram(rr, resourceMetrics, scopeMetrics, metric.Name, t.ExponentialHistogram)
						if err != nil {
							return nil, err
						}
					default:
						panic(fmt.Sprintf("Unsupported metric type: %v", metric.Data))
					}
				}
			}
			records, err := rr.BuildRecords()
			if err != nil {
				return nil, err
			}
			result = append(result, records...)
		}
	}
	return result, nil
}

func addGaugeOrSum(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, dataPoints []*metricspb.NumberDataPoint, metric_type string, config *MultivariateMetricsConfig) error {
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

func addSummary(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, summary *metricspb.Summary) error {
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

		if attributes := common.NewAttributes(sdp.Attributes); attributes != nil {
			record.AddField(attributes)
		}

		var summaryFields []*rfield.Field

		summaryFields = append(summaryFields, rfield.NewU64Field(constants.SUMMARY_COUNT, sdp.Count))
		summaryFields = append(summaryFields, rfield.NewF64Field(constants.SUMMARY_SUM, sdp.Sum))

		var items []rfield.Value
		for _, quantile := range sdp.QuantileValues {
			items = append(items, &rfield.Struct{
				Fields: []*rfield.Field{
					rfield.NewF64Field(constants.SUMMARY_QUANTILE, quantile.Quantile),
					rfield.NewF64Field(constants.SUMMARY_VALUE, quantile.Value),
				},
			})
		}
		summaryFields = append(summaryFields, rfield.NewListField(constants.SUMMARY_QUANTILE_VALUES, rfield.List{Values: items}))

		record.StructField(fmt.Sprintf("%s_%s", constants.SUMMARY_METRICS, metricName), rfield.Struct{Fields: summaryFields})

		if sdp.Flags > 0 {
			record.U32Field(constants.FLAGS, sdp.Flags)
		}

		rr.AddRecord(record)
	}
	return nil
}

func addHistogram(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, histogram *metricspb.Histogram) error {
	for _, sdp := range histogram.DataPoints {
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

		if attributes := common.NewAttributes(sdp.Attributes); attributes != nil {
			record.AddField(attributes)
		}

		// Builds fields of the histogram struct
		var histoFields []*rfield.Field

		histoFields = append(histoFields, rfield.NewU64Field(constants.HISTOGRAM_COUNT, sdp.Count))
		if sdp.Sum != nil {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_SUM, *sdp.Sum))
		}
		if sdp.Min != nil {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MIN, *sdp.Min))
		}
		if sdp.Max != nil {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MAX, *sdp.Max))
		}
		var bucketCounts []rfield.Value
		for _, count := range sdp.BucketCounts {
			bucketCounts = append(bucketCounts, &rfield.U64{Value: count})
		}
		if bucketCounts != nil {
			histoFields = append(histoFields, rfield.NewListField(constants.HISTOGRAM_BUCKET_COUNTS, rfield.List{Values: bucketCounts}))
		}
		var explicitBounds []rfield.Value
		for _, count := range sdp.ExplicitBounds {
			explicitBounds = append(explicitBounds, &rfield.F64{Value: count})
		}
		if explicitBounds != nil {
			histoFields = append(histoFields, rfield.NewListField(constants.HISTOGRAM_EXPLICIT_BOUNDS, rfield.List{Values: explicitBounds}))
		}

		record.StructField(fmt.Sprintf("%s_%s", constants.HISTOGRAM, metricName), rfield.Struct{Fields: histoFields})

		if sdp.Flags > 0 {
			record.U32Field(constants.FLAGS, sdp.Flags)
		}

		rr.AddRecord(record)
	}

	// ToDo aggregation temporality
	// ToDo Exemplar
	return nil
}

func addExpHistogram(rr *air.RecordRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, histogram *metricspb.ExponentialHistogram) error {
	for _, sdp := range histogram.DataPoints {
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

		if attributes := common.NewAttributes(sdp.Attributes); attributes != nil {
			record.AddField(attributes)
		}

		// Builds fields of the histogram struct
		var histoFields []*rfield.Field

		histoFields = append(histoFields, rfield.NewU64Field(constants.HISTOGRAM_COUNT, sdp.Count))
		if sdp.Sum != nil {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_SUM, *sdp.Sum))
		}
		if sdp.Min != nil {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MIN, *sdp.Min))
		}
		if sdp.Max != nil {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MAX, *sdp.Max))
		}
		histoFields = append(histoFields, rfield.NewI32Field(constants.EXP_HISTOGRAM_SCALE, sdp.Scale))
		histoFields = append(histoFields, rfield.NewU64Field(constants.EXP_HISTOGRAM_ZERO_COUNT, sdp.ZeroCount))

		if sdp.Positive != nil {
			var bucketCounts []rfield.Value
			for _, count := range sdp.Positive.BucketCounts {
				bucketCounts = append(bucketCounts, &rfield.U64{Value: count})
			}
			if bucketCounts != nil {
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_POSITIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, sdp.Positive.Offset),
					rfield.NewListField(constants.HISTOGRAM_BUCKET_COUNTS, rfield.List{Values: bucketCounts}),
				}}))
			} else {
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_POSITIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, sdp.Positive.Offset),
				}}))
			}
		}

		if sdp.Negative != nil {
			var bucketCounts []rfield.Value
			for _, count := range sdp.Negative.BucketCounts {
				bucketCounts = append(bucketCounts, &rfield.U64{Value: count})
			}
			if bucketCounts != nil {
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_NEGATIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, sdp.Negative.Offset),
					rfield.NewListField(constants.HISTOGRAM_BUCKET_COUNTS, rfield.List{Values: bucketCounts}),
				}}))
			} else {
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_NEGATIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, sdp.Negative.Offset),
				}}))
			}
		}

		record.StructField(fmt.Sprintf("%s_%s", constants.EXP_HISTOGRAM, metricName), rfield.Struct{Fields: histoFields})

		if sdp.Flags > 0 {
			record.U32Field(constants.FLAGS, sdp.Flags)
		}

		rr.AddRecord(record)
	}

	// ToDo aggregation temporality
	// ToDo Exemplar
	return nil
}

/*
	Positive *ExponentialHistogramDataPoint_Buckets `protobuf:"bytes,8,opt,name=positive,proto3" json:"positive,omitempty"`
	Negative *ExponentialHistogramDataPoint_Buckets `protobuf:"bytes,9,opt,name=negative,proto3" json:"negative,omitempty"`
*/

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
