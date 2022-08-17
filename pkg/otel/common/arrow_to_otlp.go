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

package common

import (
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"

	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	"otel-arrow-adapter/pkg/air"
)

func AttributesFrom(dt arrow.DataType, arr arrow.Array, row int) ([]*commonpb.KeyValue, error) {
	structType, ok := dt.(*arrow.StructType)
	if !ok {
		return nil, fmt.Errorf("attributes is not a struct")
	}
	attrArray := arr.(*array.Struct)
	kvs := make([]*commonpb.KeyValue, 0, attrArray.NumField())
	for i := 0; i < attrArray.NumField(); i++ {
		valueField := structType.Field(i)
		value, err := KeyValueFrom(&valueField, attrArray.Field(i), row)
		if err != nil {
			return nil, err
		}
		kvs = append(kvs, value)
	}
	return kvs, nil
}

func AnyValueFrom(dt arrow.DataType, arr arrow.Array, row int) (*commonpb.AnyValue, error) {
	switch t := dt.(type) {
	case *arrow.BooleanType:
		v, err := air.BoolFromArray(arr, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: v}}, nil
	case *arrow.Float64Type:
		v, err := air.F64FromArray(arr, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{DoubleValue: v}}, nil
	case *arrow.Int64Type:
		v, err := air.I64FromArray(arr, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: v}}, nil
	case *arrow.StringType:
		v, err := air.StringFromArray(arr, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: v}}, nil
	case *arrow.BinaryType:
		v, err := air.BinaryFromArray(arr, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_BytesValue{BytesValue: v}}, nil
	case *arrow.StructType:
		structKvs, err := AttributesFrom(dt, arr, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_KvlistValue{KvlistValue: &commonpb.KeyValueList{
			Values: structKvs,
		}}}, nil
	case *arrow.ListType:
		arrList, ok := arr.(*array.List)
		if !ok {
			return nil, fmt.Errorf("array %is not a list")
		}
		values, err := ArrayValueFrom(arrList, row)
		if err != nil {
			return nil, err
		}
		return &commonpb.AnyValue{Value: &commonpb.AnyValue_ArrayValue{ArrayValue: &commonpb.ArrayValue{
			Values: values,
		}}}, nil
	case *arrow.DictionaryType:
		switch t.ValueType.(type) {
		case *arrow.StringType:
			v, err := air.StringFromArray(arr, row)
			if err != nil {
				return nil, err
			}
			return &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: v}}, nil
		case *arrow.BinaryType:
			v, err := air.BinaryFromArray(arr, row)
			if err != nil {
				return nil, err
			}
			return &commonpb.AnyValue{Value: &commonpb.AnyValue_BytesValue{BytesValue: v}}, nil
		default:
			return nil, fmt.Errorf("unsupported dictionary value type %T", t.ValueType)
		}
	default:
		return nil, fmt.Errorf("%T is not a supported value type", t)
	}
}

func KeyValueFrom(field *arrow.Field, arr arrow.Array, row int) (*commonpb.KeyValue, error) {
	value, err := AnyValueFrom(field.Type, arr, row)
	if err != nil {
		return nil, err
	}
	return &commonpb.KeyValue{Key: field.Name, Value: value}, nil
}

func ArrayValueFrom(arrList *array.List, row int) ([]*commonpb.AnyValue, error) {
	start := int(arrList.Offsets()[row])
	end := int(arrList.Offsets()[row+1])
	result := make([]*commonpb.AnyValue, 0, end-start)

	arrItems := arrList.ListValues()
	for ; start < end; start++ {
		if arrList.IsNull(start) {
			result = append(result, nil)
			continue
		}
		v, err := AnyValueFrom(arrList.DataType(), arrItems, start)
		if err != nil {
			return nil, err
		}
		result = append(result, v)
	}

	return result, nil
}
