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
	"sort"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"

	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/constants"
)

type Attributes []*commonpb.KeyValue

// Sort interface
func (d Attributes) Less(i, j int) bool {
	return d[i].Key < d[j].Key
}
func (d Attributes) Len() int      { return len(d) }
func (d Attributes) Swap(i, j int) { d[i], d[j] = d[j], d[i] }

func AttributesId(attrs []*commonpb.KeyValue) string {
	sort.Sort(Attributes(attrs))
	attrsId := "{"
	for i, attr := range attrs {
		if i > 0 {
			attrsId += ","
		}
		attrsId += attr.Key + ":" + ValueId(attr.Value)
	}
	attrsId += "}"
	return attrsId
}

func NewResourceFrom(record arrow.Record, row int) (*resourcepb.Resource, error) {
	resourceField, resourceArray := air.FieldArray(record, constants.RESOURCE)
	if resourceArray == nil {
		return nil, nil
	}
	droppedAttributesCount, err := air.U32FromStruct(resourceField, resourceArray, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}
	attrField, attrArray, err := air.FieldArrayOfStruct(resourceField, resourceArray, constants.ATTRIBUTES)
	if err != nil {
		return nil, err
	}
	var attributes []*commonpb.KeyValue
	if attrField != nil {
		attributes, err = AttributesFrom(attrField.Type, attrArray, row)
	}
	if err != nil {
		return nil, err
	}
	return &resourcepb.Resource{
		Attributes:             attributes,
		DroppedAttributesCount: droppedAttributesCount,
	}, nil
}

func NewInstrumentationScopeFrom(record arrow.Record, row int, scope string) (*commonpb.InstrumentationScope, error) {
	scopeField, scopeArray := air.FieldArray(record, scope)
	if scopeArray == nil {
		return nil, nil
	}
	name, err := air.StringFromStruct(scopeField, scopeArray, row, constants.NAME)
	if err != nil {
		return nil, err
	}
	version, err := air.StringFromStruct(scopeField, scopeArray, row, constants.VERSION)
	if err != nil {
		return nil, err
	}
	droppedAttributesCount, err := air.U32FromStruct(scopeField, scopeArray, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}
	attrField, attrArray, err := air.FieldArrayOfStruct(scopeField, scopeArray, constants.ATTRIBUTES)
	if err != nil {
		return nil, err
	}
	var attributes []*commonpb.KeyValue
	if attrField != nil {
		attributes, err = AttributesFrom(attrField.Type, attrArray, row)
	}
	if err != nil {
		return nil, err
	}
	return &commonpb.InstrumentationScope{
		Name:                   name,
		Version:                version,
		Attributes:             attributes,
		DroppedAttributesCount: droppedAttributesCount,
	}, nil
}

func ResourceId(r *resourcepb.Resource) string {
	if r == nil {
		return ""
	}
	return AttributesId(r.Attributes) + "|" + fmt.Sprintf("dac:%d", r.DroppedAttributesCount)
}

func ScopeId(is *commonpb.InstrumentationScope) string {
	if is == nil {
		return ""
	}
	return "name:" + is.Name + "|version:" + is.Version + "|" + AttributesId(is.Attributes) + "|" + fmt.Sprintf("dac:%d", is.DroppedAttributesCount)
}

func ValueId(v *commonpb.AnyValue) string {
	switch v.Value.(type) {
	case *commonpb.AnyValue_StringValue:
		return v.GetStringValue()
	case *commonpb.AnyValue_IntValue:
		return fmt.Sprintf("%d", v.GetIntValue())
	case *commonpb.AnyValue_DoubleValue:
		return fmt.Sprintf("%f", v.GetDoubleValue())
	case *commonpb.AnyValue_BoolValue:
		return fmt.Sprintf("%t", v.GetBoolValue())
	case *commonpb.AnyValue_BytesValue:
		return fmt.Sprintf("%x", v.GetBytesValue())
	case *commonpb.AnyValue_KvlistValue:
		return AttributesId(v.GetKvlistValue().Values)
	case *commonpb.AnyValue_ArrayValue:
		values := v.GetArrayValue().Values
		valueId := "["
		for i := 0; i < len(values); i++ {
			if i > 0 {
				valueId += ","
			}
			valueId += ValueId(values[i])
		}
		valueId += "]"
		return valueId
	default:
		panic("unsupported value type")
	}
}

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
			return nil, fmt.Errorf("array is not a list")
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
