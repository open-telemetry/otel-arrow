/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package metrics

import (
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"

	colmetrics "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
)

func ArrowRecordsToOtlpMetrics(record arrow.Record) (*colmetrics.ExportMetricsServiceRequest, error) {
	request := colmetrics.ExportMetricsServiceRequest{
		ResourceMetrics: []*metricspb.ResourceMetrics{},
	}

	resourceMetrics := map[string]*metricspb.ResourceMetrics{}
	scopeMetrics := map[string]*metricspb.ScopeMetrics{}

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resource, err := common.NewResourceFrom(record, i)
		if err != nil {
			return nil, err
		}
		resId := common.ResourceId(resource)
		if _, ok := resourceMetrics[resId]; !ok {
			rs := &metricspb.ResourceMetrics{
				Resource:     resource,
				ScopeMetrics: []*metricspb.ScopeMetrics{},
				SchemaUrl:    "",
			}
			resourceMetrics[resId] = rs
		}
		rs := resourceMetrics[resId]

		scope, err := common.NewInstrumentationScopeFrom(record, i, constants.SCOPE_METRICS)
		if err != nil {
			return nil, err
		}
		scopeSpanId := resId + "|" + common.ScopeId(scope)
		if _, ok := scopeMetrics[scopeSpanId]; !ok {
			ss := &metricspb.ScopeMetrics{
				Scope:     scope,
				Metrics:   []*metricspb.Metric{},
				SchemaUrl: "",
			}
			scopeMetrics[scopeSpanId] = ss
			rs.ScopeMetrics = append(rs.ScopeMetrics, ss)
		}
		ss := scopeMetrics[scopeSpanId]

		metrics, err := NewMetrics(record, i)
		if err != nil {
			return nil, err
		}
		ss.Metrics = append(ss.Metrics, metrics...)
	}

	for _, resMetrics := range resourceMetrics {
		request.ResourceMetrics = append(request.ResourceMetrics, resMetrics)
	}

	return &request, nil
}

func NewMetrics(record arrow.Record, row int) ([]*metricspb.Metric, error) {
	metrics := []*metricspb.Metric{}

	timeUnixNano, err := air.U64FromRecord(record, row, constants.TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}
	startTimeUnixNano, err := air.U64FromRecord(record, row, constants.START_TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}
	flags, err := air.U32FromRecord(record, row, constants.FLAGS)
	if err != nil {
		return nil, err
	}
	metricsField, arr := air.FieldArray(record, constants.METRICS)
	if metricsField == nil {
		return nil, fmt.Errorf("no metrics found")
	}
	if metricsType, ok := metricsField.Type.(*arrow.StructType); ok {
		metricsArr, ok := arr.(*array.Struct)
		if !ok {
			return nil, fmt.Errorf("metrics array is not a struct")
		}

		attrsField, attrsArray := air.FieldArray(record, constants.ATTRIBUTES)
		var attributes []*v1.KeyValue
		if attrsField != nil {
			attrs, err := common.AttributesFrom(attrsField.Type, attrsArray, row)
			if err != nil {
				return nil, err
			}
			attributes = attrs
		}
		for i, metricField := range metricsType.Fields() {
			metricType := metricMetadata(&metricField, constants.METADATA_METRIC_TYPE)

			if metricType == constants.SUM_METRICS {
				metricArr := metricsArr.Field(i)
				sumMetrics, err := collectMetricsSum(timeUnixNano, startTimeUnixNano, flags, metricField, metricArr, row, attributes)
				if err != nil {
					return nil, err
				}
				metrics = append(metrics, sumMetrics...)
			} else {
				return nil, fmt.Errorf("unsupported metric type: %q", metricType)
			}
		}

		return metrics, nil

	} else {
		return nil, fmt.Errorf("metrics type is not a struct")
	}
}

func collectMetricsSum(timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricField arrow.Field, metricArr arrow.Array, row int, attributes []*v1.KeyValue) ([]*metricspb.Metric, error) {
	metricName := metricField.Name
	switch dt := metricField.Type.(type) {
	case *arrow.Int64Type:
		dp, err := collectI64NumberDataPoint(timeUnixNano, startTimeUnixNano, flags, metricArr, row, attributes)
		if err != nil {
			return nil, err
		}
		return []*metricspb.Metric{{
			Name:        metricName,
			Description: metricMetadata(&metricField, constants.METADATA_METRIC_DESCRIPTION),
			Unit:        metricMetadata(&metricField, constants.METADATA_METRIC_UNIT),
			Data: &metricspb.Metric_Sum{
				Sum: &metricspb.Sum{
					DataPoints:             []*metricspb.NumberDataPoint{dp},
					AggregationTemporality: 0,     // ToDo Add aggregation temporality
					IsMonotonic:            false, // ToDo Add is monotonic
				},
			},
		}}, nil
	case *arrow.Float64Type:
		dp, err := collectF64NumberDataPoint(timeUnixNano, startTimeUnixNano, flags, metricArr, row, attributes)
		if err != nil {
			return nil, err
		}
		return []*metricspb.Metric{{
			Name:        metricName,
			Description: metricMetadata(&metricField, constants.METADATA_METRIC_DESCRIPTION),
			Unit:        metricMetadata(&metricField, constants.METADATA_METRIC_UNIT),
			Data: &metricspb.Metric_Sum{
				Sum: &metricspb.Sum{
					DataPoints:             []*metricspb.NumberDataPoint{dp},
					AggregationTemporality: 0,     // ToDo Add aggregation temporality
					IsMonotonic:            false, // ToDo Add is monotonic
				},
			},
		}}, nil
	case *arrow.StructType:
		mm, err := NewMultivariateSumMetrics(timeUnixNano, startTimeUnixNano, flags, &metricField, metricArr, metricName, row, attributes)
		if err != nil {
			return nil, err
		}
		return mm, nil
	default:
		return nil, fmt.Errorf("unsupported metric type: %T", dt)
	}
}

func collectI64NumberDataPoint(timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricArr arrow.Array, row int, attributes []*v1.KeyValue) (*metricspb.NumberDataPoint, error) {
	v, err := air.I64FromArray(metricArr, row)
	if err != nil {
		return nil, err
	}
	return &metricspb.NumberDataPoint{
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNano,
		TimeUnixNano:      timeUnixNano,
		Value: &metricspb.NumberDataPoint_AsInt{
			AsInt: v,
		},
		Exemplars: nil, // ToDo Add exemplars
		Flags:     flags,
	}, nil
}

func collectF64NumberDataPoint(timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricArr arrow.Array, row int, attributes []*v1.KeyValue) (*metricspb.NumberDataPoint, error) {
	v, err := air.F64FromArray(metricArr, row)
	if err != nil {
		return nil, err
	}
	return &metricspb.NumberDataPoint{
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNano,
		TimeUnixNano:      timeUnixNano,
		Value: &metricspb.NumberDataPoint_AsDouble{
			AsDouble: v,
		},
		Exemplars: nil, // ToDo Add exemplars
		Flags:     flags,
	}, nil
}

func NewMultivariateSumMetrics(timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, field *arrow.Field, arr arrow.Array, name string, row int, attributes []*v1.KeyValue) ([]*metricspb.Metric, error) {
	metricFields := field.Type.(*arrow.StructType).Fields()
	multivariateArr, ok := arr.(*array.Struct)
	if !ok {
		return nil, fmt.Errorf("metrics array is not a struct")
	}

	dataPoints := make([]*metricspb.NumberDataPoint, 0, len(metricFields))
	for i, metricField := range metricFields {
		metricArr := multivariateArr.Field(i)

		extAttributes := make([]*v1.KeyValue, len(attributes)+1)
		copy(extAttributes, attributes)
		extAttributes[len(attributes)] = &v1.KeyValue{
			Key: metricMetadata(&metricField, constants.METADATA_METRIC_MULTIVARIATE_ATTR),
			Value: &v1.AnyValue{
				Value: &v1.AnyValue_StringValue{
					StringValue: metricField.Name,
				},
			},
		}

		switch dt := metricField.Type.(type) {
		case *arrow.Int64Type:
			dp, err := collectI64NumberDataPoint(timeUnixNano, startTimeUnixNano, flags, metricArr, row, extAttributes)
			if err != nil {
				return nil, err
			}
			dataPoints = append(dataPoints, dp)
		case *arrow.Float64Type:
			dp, err := collectF64NumberDataPoint(timeUnixNano, startTimeUnixNano, flags, metricArr, row, extAttributes)
			if err != nil {
				return nil, err
			}
			dataPoints = append(dataPoints, dp)
		default:
			return nil, fmt.Errorf("unsupported metric type: %T", dt)
		}
	}

	return []*metricspb.Metric{
		{
			Name:        name,
			Description: metricMetadata(field, constants.METADATA_METRIC_DESCRIPTION),
			Unit:        metricMetadata(field, constants.METADATA_METRIC_UNIT),
			Data: &metricspb.Metric_Sum{
				Sum: &metricspb.Sum{
					DataPoints:             dataPoints,
					AggregationTemporality: 0,     // ToDo Add aggregation temporality
					IsMonotonic:            false, // ToDo Add is monotonic
				},
			},
		},
	}, nil
}

func metricMetadata(field *arrow.Field, metadata string) string {
	if field.HasMetadata() {
		idx := field.Metadata.FindKey(metadata)
		if idx != -1 {
			return field.Metadata.Values()[idx]
		} else {
			return ""
		}
	} else {
		return ""
	}
}
