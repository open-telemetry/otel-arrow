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

	"github.com/f5/otel-arrow-adapter/pkg/air"
	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/apache/arrow/go/v11/arrow"
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
func OtlpMetricsToArrowRecords(rr *air.RecordRepository, request pmetric.Metrics, multivariateConf *MultivariateMetricsConfig, cfg *config.Config) ([]arrow.Record, error) {
	var result []arrow.Record

	for i := 0; i < request.ResourceMetrics().Len(); i++ {
		resourceMetrics := request.ResourceMetrics().At(i)
		resource := resourceMetrics.Resource()

		for j := 0; j < resourceMetrics.ScopeMetrics().Len(); j++ {
			scopeMetrics := resourceMetrics.ScopeMetrics().At(j)
			scope := scopeMetrics.Scope()

			for k := 0; k < scopeMetrics.Metrics().Len(); k++ {
				metric := scopeMetrics.Metrics().At(k)

				switch metric.Type() {
				case pmetric.MetricTypeGauge:
					err := addGaugeOrSum(rr, resource, scope, metric, metric.Gauge().DataPoints(), constants.GAUGE_METRICS, multivariateConf, cfg)
					if err != nil {
						return nil, err
					}
				case pmetric.MetricTypeSum:
					err := addGaugeOrSum(rr, resource, scope, metric, metric.Sum().DataPoints(), constants.SUM_METRICS, multivariateConf, cfg)
					if err != nil {
						return nil, err
					}
				case pmetric.MetricTypeHistogram:
					err := addHistogram(rr, resource, scope, metric, metric.Histogram(), cfg)
					if err != nil {
						return nil, err
					}
				case pmetric.MetricTypeExponentialHistogram:
					err := addExpHistogram(rr, resource, scope, metric, metric.ExponentialHistogram(), cfg)
					if err != nil {
						return nil, err
					}
				case pmetric.MetricTypeSummary:
					err := addSummary(rr, resource, scope, metric, metric.Summary(), cfg)
					if err != nil {
						return nil, err
					}
				default:
					panic(fmt.Sprintf("Unsupported metric type: %v", metric.Type()))
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

func addGaugeOrSum(rr *air.RecordRepository, res pcommon.Resource, scope pcommon.InstrumentationScope, metric pmetric.Metric, dataPoints pmetric.NumberDataPointSlice, metricType string, config *MultivariateMetricsConfig, cfg *config.Config) error {
	// TODO: Missing Temporality and IsMonotonic for Sums here.
	if mvKey, ok := config.Metrics[metric.Name()]; ok {
		return multivariateMetric(rr, res, scope, metric, dataPoints, metricType, mvKey, cfg)
	}
	univariateMetric(rr, res, scope, metric, dataPoints, metricType, cfg)
	return nil
}

func multivariateMetric(rr *air.RecordRepository, res pcommon.Resource, scope pcommon.InstrumentationScope, metric pmetric.Metric, dataPoints pmetric.NumberDataPointSlice, metricType string, multivariateKey string, cfg *config.Config) error {
	records := make(map[string]*MultivariateRecord)

	for i := 0; i < dataPoints.Len(); i++ {
		ndp := dataPoints.At(i)
		sig := DataPointSig[pmetric.NumberDataPoint](ndp, multivariateKey)
		newEntry := false
		stringSig := string(sig)
		record := records[stringSig]

		var multivariateMetricName string

		if record == nil {
			newEntry = true
			record = &MultivariateRecord{
				fields:  []*rfield.Field{},
				metrics: []*rfield.Field{},
			}
			records[stringSig] = record
		}

		if newEntry {
			record.fields = append(record.fields, common.ResourceField(res, cfg))
			record.fields = append(record.fields, common.ScopeField(constants.SCOPE_METRICS, scope, cfg))
			timeUnixNanoField := rfield.NewU64Field(constants.TIME_UNIX_NANO, uint64(ndp.Timestamp()))
			record.fields = append(record.fields, timeUnixNanoField)
			if ts := ndp.StartTimestamp(); ts > 0 {
				startTimeUnixNano := rfield.NewU64Field(constants.START_TIME_UNIX_NANO, uint64(ts))
				record.fields = append(record.fields, startTimeUnixNano)
			}
			ma, err := AddMultivariateValue(ndp.Attributes(), multivariateKey, &record.fields)
			if err != nil {
				return err
			}
			multivariateMetricName = ma
		} else {
			ma, err := ExtractMultivariateValue(ndp.Attributes(), multivariateKey)
			if err != nil {
				return err
			}
			multivariateMetricName = ma
		}

		switch ndp.ValueType() {
		case pmetric.NumberDataPointValueTypeDouble:
			field := rfield.NewF64Field(multivariateMetricName, ndp.DoubleValue())
			field.AddMetadata(constants.METADATA_METRIC_MULTIVARIATE_ATTR, multivariateKey)
			record.metrics = append(record.metrics, field)
		case pmetric.NumberDataPointValueTypeInt:
			field := rfield.NewI64Field(multivariateMetricName, ndp.IntValue())
			field.AddMetadata(constants.METADATA_METRIC_MULTIVARIATE_ATTR, multivariateKey)
			record.metrics = append(record.metrics, field)
		default:
			panic("Unsupported number data point value type")
		}
	}

	for _, record := range records {
		if len(record.fields) == 0 && len(record.metrics) == 0 {
			continue
		}
		metricField := rfield.NewStructField(metric.Name(), rfield.Struct{
			Fields: record.metrics,
		})
		metricField.AddMetadata(constants.METADATA_METRIC_TYPE, metricType)
		if metric.Description() != "" {
			metricField.AddMetadata(constants.METADATA_METRIC_DESCRIPTION, metric.Description())
		}
		if metric.Unit() != "" {
			metricField.AddMetadata(constants.METADATA_METRIC_UNIT, metric.Unit())
		}
		record.fields = append(record.fields, rfield.NewStructField(constants.METRICS, rfield.Struct{
			Fields: []*rfield.Field{metricField},
		}))
		rr.AddRecord(air.NewRecordFromFields(record.fields))
	}
	return nil
}

func univariateMetric(rr *air.RecordRepository, res pcommon.Resource, scope pcommon.InstrumentationScope, metric pmetric.Metric, dataPoints pmetric.NumberDataPointSlice, metricType string, cfg *config.Config) {
	for i := 0; i < dataPoints.Len(); i++ {
		ndp := dataPoints.At(i)
		record := air.NewRecord()

		common.AddResource(record, res, cfg)
		common.AddScope(record, constants.SCOPE_METRICS, scope, cfg)

		record.U64Field(constants.TIME_UNIX_NANO, uint64(ndp.Timestamp()))
		if ts := ndp.StartTimestamp(); ts > 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, uint64(ts))
		}

		if attributes := common.NewAttributes(ndp.Attributes(), cfg); attributes != nil {
			record.AddField(attributes)
		}

		switch ndp.ValueType() {
		case pmetric.NumberDataPointValueTypeDouble:
			metricField := rfield.NewF64Field(metric.Name(), ndp.DoubleValue())
			metricField.AddMetadata(constants.METADATA_METRIC_TYPE, metricType)
			if metric.Description() != "" {
				metricField.AddMetadata(constants.METADATA_METRIC_DESCRIPTION, metric.Description())
			}
			if metric.Unit() != "" {
				metricField.AddMetadata(constants.METADATA_METRIC_UNIT, metric.Unit())
			}
			record.StructField(constants.METRICS, rfield.Struct{
				Fields: []*rfield.Field{metricField},
			})
		case pmetric.NumberDataPointValueTypeInt:
			metricField := rfield.NewI64Field(metric.Name(), ndp.IntValue())
			metricField.AddMetadata(constants.METADATA_METRIC_TYPE, metricType)
			record.StructField(constants.METRICS, rfield.Struct{
				Fields: []*rfield.Field{metricField},
			})
		default:
			panic("Unsupported number data point value type")
		}

		// ToDo Exemplar

		if ndp.Flags() != 0 {
			record.U32Field(constants.FLAGS, uint32(ndp.Flags()))
		}

		rr.AddRecord(record)
	}
}

func addSummary(rr *air.RecordRepository, res pcommon.Resource, scope pcommon.InstrumentationScope, metric pmetric.Metric, summary pmetric.Summary, cfg *config.Config) error {
	for i := 0; i < summary.DataPoints().Len(); i++ {
		sdp := summary.DataPoints().At(i)

		record := air.NewRecord()

		common.AddResource(record, res, cfg)
		common.AddScope(record, constants.SCOPE_METRICS, scope, cfg)

		record.U64Field(constants.TIME_UNIX_NANO, uint64(sdp.Timestamp()))
		if sdp.StartTimestamp() > 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, uint64(sdp.StartTimestamp()))
		}

		if attributes := common.NewAttributes(sdp.Attributes(), cfg); attributes != nil {
			record.AddField(attributes)
		}

		var summaryFields []*rfield.Field

		summaryFields = append(summaryFields, rfield.NewU64Field(constants.SUMMARY_COUNT, sdp.Count()))
		summaryFields = append(summaryFields, rfield.NewF64Field(constants.SUMMARY_SUM, sdp.Sum()))

		var items []rfield.Value
		for j := 0; j < sdp.QuantileValues().Len(); j++ {
			quantile := sdp.QuantileValues().At(j)
			items = append(items, &rfield.Struct{
				Fields: []*rfield.Field{
					rfield.NewF64Field(constants.SUMMARY_QUANTILE, quantile.Quantile()),
					rfield.NewF64Field(constants.SUMMARY_VALUE, quantile.Value()),
				},
			})
		}
		summaryFields = append(summaryFields, rfield.NewListField(constants.SUMMARY_QUANTILE_VALUES, rfield.List{Values: items}))

		record.StructField(fmt.Sprintf("%s_%s", constants.SUMMARY_METRICS, metric.Name()), rfield.Struct{Fields: summaryFields})

		if sdp.Flags() != 0 {
			record.U32Field(constants.FLAGS, uint32(sdp.Flags()))
		}

		rr.AddRecord(record)
	}
	return nil
}

func addHistogram(rr *air.RecordRepository, res pcommon.Resource, scope pcommon.InstrumentationScope, metric pmetric.Metric, histogram pmetric.Histogram, cfg *config.Config) error {
	for i := 0; i < histogram.DataPoints().Len(); i++ {
		hdp := histogram.DataPoints().At(i)
		record := air.NewRecord()

		common.AddResource(record, res, cfg)
		common.AddScope(record, constants.SCOPE_METRICS, scope, cfg)

		record.U64Field(constants.TIME_UNIX_NANO, uint64(hdp.Timestamp()))
		if ts := hdp.StartTimestamp(); ts != 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, uint64(ts))
		}

		if attributes := common.NewAttributes(hdp.Attributes(), cfg); attributes != nil {
			record.AddField(attributes)
		}

		// Builds fields of the histogram struct
		var histoFields []*rfield.Field

		histoFields = append(histoFields, rfield.NewU64Field(constants.HISTOGRAM_COUNT, hdp.Count()))
		if hdp.HasSum() {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_SUM, hdp.Sum()))
		}
		if hdp.HasMin() {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MIN, hdp.Min()))
		}
		if hdp.HasMax() {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MAX, hdp.Max()))
		}
		var bucketCounts []rfield.Value
		for i := 0; i < hdp.BucketCounts().Len(); i++ {
			count := hdp.BucketCounts().At(i)
			bucketCounts = append(bucketCounts, rfield.NewU64(count))
		}
		if bucketCounts != nil {
			histoFields = append(histoFields, rfield.NewListField(constants.HISTOGRAM_BUCKET_COUNTS, rfield.List{Values: bucketCounts}))
		}
		var explicitBounds []rfield.Value
		for i := 0; i < hdp.ExplicitBounds().Len(); i++ {
			count := hdp.ExplicitBounds().At(i)
			explicitBounds = append(explicitBounds, rfield.NewF64(count))
		}
		if explicitBounds != nil {
			histoFields = append(histoFields, rfield.NewListField(constants.HISTOGRAM_EXPLICIT_BOUNDS, rfield.List{Values: explicitBounds}))
		}

		record.StructField(fmt.Sprintf("%s_%s", constants.HISTOGRAM, metric.Name()), rfield.Struct{Fields: histoFields})

		if hdp.Flags() != 0 {
			record.U32Field(constants.FLAGS, uint32(hdp.Flags()))
		}

		rr.AddRecord(record)
	}

	// ToDo aggregation temporality
	// ToDo Exemplar
	return nil
}

func addExpHistogram(rr *air.RecordRepository, res pcommon.Resource, scope pcommon.InstrumentationScope, metric pmetric.Metric, histogram pmetric.ExponentialHistogram, cfg *config.Config) error {
	for i := 0; i < histogram.DataPoints().Len(); i++ {
		hdp := histogram.DataPoints().At(i)
		record := air.NewRecord()

		common.AddResource(record, res, cfg)
		common.AddScope(record, constants.SCOPE_METRICS, scope, cfg)

		record.U64Field(constants.TIME_UNIX_NANO, uint64(hdp.Timestamp()))
		if hdp.StartTimestamp() > 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, uint64(hdp.StartTimestamp()))
		}

		if attributes := common.NewAttributes(hdp.Attributes(), cfg); attributes != nil {
			record.AddField(attributes)
		}

		// Builds fields of the histogram struct
		var histoFields []*rfield.Field

		histoFields = append(histoFields, rfield.NewU64Field(constants.HISTOGRAM_COUNT, hdp.Count()))
		if hdp.HasSum() {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_SUM, hdp.Sum()))
		}
		if hdp.HasMin() {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MIN, hdp.Min()))
		}
		if hdp.HasMax() {
			histoFields = append(histoFields, rfield.NewF64Field(constants.HISTOGRAM_MAX, hdp.Max()))
		}
		histoFields = append(histoFields, rfield.NewI32Field(constants.EXP_HISTOGRAM_SCALE, hdp.Scale()))
		histoFields = append(histoFields, rfield.NewU64Field(constants.EXP_HISTOGRAM_ZERO_COUNT, hdp.ZeroCount()))

		if hdp.Positive().BucketCounts().Len() != 0 {
			var bucketCounts []rfield.Value
			for i := 0; i < hdp.Positive().BucketCounts().Len(); i++ {
				count := hdp.Positive().BucketCounts().At(i)
				bucketCounts = append(bucketCounts, rfield.NewU64(count))
			}
			if bucketCounts != nil {
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_POSITIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, hdp.Positive().Offset()),
					rfield.NewListField(constants.HISTOGRAM_BUCKET_COUNTS, rfield.List{Values: bucketCounts}),
				}}))
			} else {
				// Reviewers notes: not sure what the
				// OTLP means in this case, having
				// Offset and no bucket counts?
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_POSITIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, hdp.Positive().Offset()),
				}}))
			}
		}

		if hdp.Negative().BucketCounts().Len() != 0 {
			var bucketCounts []rfield.Value
			for i := 0; i < hdp.Negative().BucketCounts().Len(); i++ {
				count := hdp.Negative().BucketCounts().At(i)
				bucketCounts = append(bucketCounts, rfield.NewU64(count))
			}
			if bucketCounts != nil {
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_NEGATIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, hdp.Negative().Offset()),
					rfield.NewListField(constants.HISTOGRAM_BUCKET_COUNTS, rfield.List{Values: bucketCounts}),
				}}))
			} else {
				// Note: See comments above; probably safe to eliminate the offset
				// if the counts are empty.
				histoFields = append(histoFields, rfield.NewStructField(constants.EXP_HISTOGRAM_NEGATIVE, rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI32Field(constants.EXP_HISTOGRAM_OFFSET, hdp.Negative().Offset()),
				}}))
			}
		}

		record.StructField(fmt.Sprintf("%s_%s", constants.EXP_HISTOGRAM, metric.Name()), rfield.Struct{Fields: histoFields})

		if hdp.Flags() != 0 {
			record.U32Field(constants.FLAGS, uint32(hdp.Flags()))
		}

		rr.AddRecord(record)
	}

	// ToDo aggregation temporality
	// ToDo Exemplar
	return nil
}

func ExtractMultivariateValue(attributes pcommon.Map, multivariateKey string) (res string, err error) {
	attributes.Range(func(key string, value pcommon.Value) bool {
		if key != multivariateKey {
			return true
		}
		switch value.Type() {
		case pcommon.ValueTypeStr:
			res = value.Str()
		default:
			err = fmt.Errorf("unsupported multivariate value type: %v", value)
		}
		return false
	})
	return
}

func AddMultivariateValue(attributes pcommon.Map, multivariateKey string, fields *[]*rfield.Field) (res string, err error) {
	var multivariateValue string
	attributeFields := make([]*rfield.Field, 0, attributes.Len())
	attributes.Range(func(key string, value pcommon.Value) bool {
		if key == multivariateKey {
			switch value.Type() {
			case pcommon.ValueTypeStr:
				multivariateValue = value.Str()
				return true
			default:
				err = fmt.Errorf("unsupported multivariate value type: %v", value)
			}
		}

		attributeFields = append(attributeFields, rfield.NewField(key, common.OtlpAnyValueToValue(value)))
		return true
	})
	if len(attributeFields) > 0 {
		*fields = append(*fields, rfield.NewStructField(constants.ATTRIBUTES, rfield.Struct{Fields: attributeFields}))
	}
	return multivariateValue, nil
}
