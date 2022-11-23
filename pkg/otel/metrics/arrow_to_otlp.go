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

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"

	arrow2 "github.com/f5/otel-arrow-adapter/pkg/arrow"
	commonarrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"
)

func ArrowRecordsToOtlpMetrics(record arrow.Record) (pmetric.Metrics, error) {
	request := pmetric.NewMetrics()

	resourceMetrics := map[string]pmetric.ResourceMetrics{}
	scopeMetrics := map[string]pmetric.ScopeMetrics{}

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resource, err := commonarrow.NewResourceFromOld(record, i)
		if err != nil {
			return request, err
		}
		resId := commonarrow.ResourceID(resource)
		rm, ok := resourceMetrics[resId]
		if !ok {
			rm = request.ResourceMetrics().AppendEmpty()
			// TODO: SchemaURL
			resource.CopyTo(rm.Resource())
			resourceMetrics[resId] = rm
		}

		scope, err := otlp.NewScopeFromRecord(record, i, constants.SCOPE_METRICS)
		if err != nil {
			return request, err
		}
		scopeSpanId := resId + "|" + commonarrow.ScopeID(scope)
		sm, ok := scopeMetrics[scopeSpanId]
		if !ok {
			sm = rm.ScopeMetrics().AppendEmpty()
			scope.CopyTo(sm.Scope())
			// TODO: SchemaURL
			scopeMetrics[scopeSpanId] = sm
		}
		if err := SetMetricsFrom(sm.Metrics(), record, i); err != nil {
			return request, err
		}
	}

	return request, nil
}

func SetMetricsFrom(metrics pmetric.MetricSlice, record arrow.Record, row int) error {
	timeUnixNano, err := arrow2.U64FromRecord(record, row, constants.TIME_UNIX_NANO)
	if err != nil {
		return err
	}
	startTimeUnixNano, err := arrow2.U64FromRecord(record, row, constants.START_TIME_UNIX_NANO)
	if err != nil {
		return err
	}
	flags, err := arrow2.OptionalU32FromRecord(record, row, constants.FLAGS)
	if err != nil {
		return err
	}
	metricsField, arr, err := arrow2.FieldArray(record, constants.METRICS)
	if err != nil {
		return err
	}
	metricsType, ok := metricsField.Type.(*arrow.StructType)
	if !ok {
		return fmt.Errorf("metrics type is not a struct")
	}
	metricsArr, ok := arr.(*array.Struct)
	if !ok {
		return fmt.Errorf("metrics array is not a struct")
	}

	attrsField, attrsArray, err := arrow2.OptionalFieldArray(record, constants.ATTRIBUTES)
	if err != nil {
		return err
	}
	attributes := pcommon.NewMap()
	if attrsField != nil {
		if err := commonarrow.CopyAttributesFrom(attributes, attrsField.Type, attrsArray, row); err != nil {
			return err
		}
	}
	for i := range metricsType.Fields() {
		field := &metricsType.Fields()[i]
		metricType := metricMetadata(field, constants.METADATA_METRIC_TYPE)
		metricArr := metricsArr.Field(i)

		switch metricType {
		case constants.SUM_METRICS:
			err := collectSumMetrics(metrics, timeUnixNano, startTimeUnixNano, flags, field, metricArr, row, attributes)
			if err != nil {
				return err
			}
		case constants.GAUGE_METRICS:
			err := collectGaugeMetrics(metrics, timeUnixNano, startTimeUnixNano, flags, field, metricArr, row, attributes)
			if err != nil {
				return err
			}
		default:
			return fmt.Errorf("unsupported metric type: %s", metricType)
		}
	}

	return nil
}

func collectSumMetrics(metrics pmetric.MetricSlice, timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricField *arrow.Field, metricArr arrow.Array, row int, attributes pcommon.Map) error {
	metricName := metricField.Name

	if _, is := metricField.Type.(*arrow.StructType); is {
		return collectMultivariateSumMetrics(
			metrics, timeUnixNano, startTimeUnixNano, flags,
			metricField, metricArr, metricName, row, attributes,
		)
	}

	m := metrics.AppendEmpty()
	m.SetName(metricName)
	m.SetDescription(metricMetadata(metricField, constants.METADATA_METRIC_DESCRIPTION))
	m.SetUnit(metricMetadata(metricField, constants.METADATA_METRIC_UNIT))

	sum := m.SetEmptySum()
	// TODO: Add isMonotonic
	// TODO: Add temporality
	// sum.SetIsMonotonic(true)
	// sum.SetAggregationTemporality(pmetric.MetricAggregationTemporalityCumulative)

	switch dt := metricField.Type.(type) {
	case *arrow.Int64Type:
		return collectI64NumberDataPoint(
			sum.DataPoints(), timeUnixNano, startTimeUnixNano,
			flags, metricArr, row, attributes,
		)

	case *arrow.Float64Type:
		return collectF64NumberDataPoint(
			sum.DataPoints(), timeUnixNano, startTimeUnixNano,
			flags, metricArr, row, attributes,
		)

	default:
		return fmt.Errorf("unsupported metric type: %T", dt)
	}
}

func collectGaugeMetrics(metrics pmetric.MetricSlice, timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricField *arrow.Field, metricArr arrow.Array, row int, attributes pcommon.Map) error {
	metricName := metricField.Name

	if _, is := metricField.Type.(*arrow.StructType); is {
		return collectMultivariateGaugeMetrics(
			metrics, timeUnixNano, startTimeUnixNano, flags,
			metricField, metricArr, metricName, row, attributes,
		)
	}

	m := metrics.AppendEmpty()
	m.SetName(metricName)
	m.SetDescription(metricMetadata(metricField, constants.METADATA_METRIC_DESCRIPTION))
	m.SetUnit(metricMetadata(metricField, constants.METADATA_METRIC_UNIT))

	gauge := m.SetEmptyGauge()

	switch dt := metricField.Type.(type) {
	case *arrow.Int64Type:
		return collectI64NumberDataPoint(
			gauge.DataPoints(), timeUnixNano, startTimeUnixNano,
			flags, metricArr, row, attributes,
		)

	case *arrow.Float64Type:
		return collectF64NumberDataPoint(
			gauge.DataPoints(), timeUnixNano, startTimeUnixNano,
			flags, metricArr, row, attributes,
		)

	default:
		return fmt.Errorf("unsupported metric type: %T", dt)
	}

}

func collectI64NumberDataPoint(points pmetric.NumberDataPointSlice, timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricArr arrow.Array, row int, attributes pcommon.Map) error {
	v, err := arrow2.I64FromArray(metricArr, row)
	if err != nil {
		return err
	}
	p := points.AppendEmpty()
	attributes.CopyTo(p.Attributes())
	p.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
	p.SetTimestamp(pcommon.Timestamp(timeUnixNano))
	p.SetFlags(pmetric.DataPointFlags(flags))
	p.SetIntValue(v)
	// TODO: Exemplars
	return nil
}

func collectF64NumberDataPoint(points pmetric.NumberDataPointSlice, timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, metricArr arrow.Array, row int, attributes pcommon.Map) error {
	v, err := arrow2.F64FromArray(metricArr, row)
	if err != nil {
		return err
	}
	p := points.AppendEmpty()
	attributes.CopyTo(p.Attributes())
	p.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
	p.SetTimestamp(pcommon.Timestamp(timeUnixNano))
	p.SetFlags(pmetric.DataPointFlags(flags))
	p.SetDoubleValue(v)
	// TODO: Exemplars
	return nil
}

func collectMultivariateSumMetrics(metrics pmetric.MetricSlice, timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, inputField *arrow.Field, inputArr arrow.Array, name string, row int, attributes pcommon.Map) error {
	inputStruct, ok := inputField.Type.(*arrow.StructType)
	if !ok {
		return fmt.Errorf("expected struct type, got %T", inputField.Type)
	}
	multiFields := inputStruct.Fields()
	multiStruct, ok := inputArr.(*array.Struct) // Note: type assertion in caller
	if !ok {
		return fmt.Errorf("expected array struct, got %T", inputArr)
	}
	m := metrics.AppendEmpty()
	m.SetName(name)
	m.SetDescription(metricMetadata(inputField, constants.METADATA_METRIC_DESCRIPTION))
	m.SetUnit(metricMetadata(inputField, constants.METADATA_METRIC_UNIT))

	sum := m.SetEmptySum()
	// TODO: Add isMonotonic
	// TODO: Add temporality
	// sum.SetIsMonotonic(true)
	// sum.SetAggregationTemporality(pmetric.MetricAggregationTemporalityCumulative)

	for i := range multiFields {
		field := &multiFields[i]
		arr := multiStruct.Field(i)

		extAttributes := pcommon.NewMap()
		extAttributes.EnsureCapacity(attributes.Len() + 1)
		attributes.CopyTo(extAttributes)

		extAttributes.PutStr(
			metricMetadata(field, constants.METADATA_METRIC_MULTIVARIATE_ATTR),
			field.Name,
		)

		switch dt := field.Type.(type) {
		case *arrow.Int64Type:
			if err := collectI64NumberDataPoint(sum.DataPoints(), timeUnixNano, startTimeUnixNano, flags, arr, row, extAttributes); err != nil {
				return err
			}
		case *arrow.Float64Type:
			if err := collectF64NumberDataPoint(sum.DataPoints(), timeUnixNano, startTimeUnixNano, flags, arr, row, extAttributes); err != nil {
				return err
			}
		default:
			return fmt.Errorf("unsupported metric type: %T", dt)
		}
	}

	return nil
}

func collectMultivariateGaugeMetrics(metrics pmetric.MetricSlice, timeUnixNano uint64, startTimeUnixNano uint64, flags uint32, inputField *arrow.Field, inputArr arrow.Array, name string, row int, attributes pcommon.Map) error {
	inputStruct, ok := inputField.Type.(*arrow.StructType)
	if !ok {
		return fmt.Errorf("expected struct type, got %T", inputField.Type)
	}
	multiFields := inputStruct.Fields()
	multiStruct, ok := inputArr.(*array.Struct) // Note: type assertion in caller
	if !ok {
		return fmt.Errorf("expected array struct, got %T", inputArr)
	}
	m := metrics.AppendEmpty()
	m.SetName(name)
	m.SetDescription(metricMetadata(inputField, constants.METADATA_METRIC_DESCRIPTION))
	m.SetUnit(metricMetadata(inputField, constants.METADATA_METRIC_UNIT))

	gauge := m.SetEmptyGauge()

	for i := range multiFields {
		field := &multiFields[i]
		arr := multiStruct.Field(i)

		extAttributes := pcommon.NewMap()
		extAttributes.EnsureCapacity(attributes.Len() + 1)
		attributes.CopyTo(extAttributes)

		extAttributes.PutStr(
			metricMetadata(field, constants.METADATA_METRIC_MULTIVARIATE_ATTR),
			field.Name,
		)

		switch dt := field.Type.(type) {
		case *arrow.Int64Type:
			if err := collectI64NumberDataPoint(gauge.DataPoints(), timeUnixNano, startTimeUnixNano, flags, arr, row, extAttributes); err != nil {
				return err
			}
		case *arrow.Float64Type:
			if err := collectF64NumberDataPoint(gauge.DataPoints(), timeUnixNano, startTimeUnixNano, flags, arr, row, extAttributes); err != nil {
				return err
			}
		default:
			return fmt.Errorf("unsupported metric type: %T", dt)
		}
	}

	return nil
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
