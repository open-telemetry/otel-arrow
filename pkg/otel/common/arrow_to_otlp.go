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

func AttributesFrom(field *arrow.Field, arr arrow.Array, row int) ([]*commonpb.KeyValue, error) {
	structType, ok := field.Type.(*arrow.StructType)
	if !ok {
		return nil, fmt.Errorf("field %q is not a struct", field.Name)
	}
	attrArray := arr.(*array.Struct)
	kvs := make([]*commonpb.KeyValue, 0, attrArray.NumField())
	for i := 0; i < attrArray.NumField(); i++ {
		fieldName := structType.Field(i).Name
		switch t := structType.Field(i).Type.(type) {
		case *arrow.BooleanType:
			v, err := air.BoolFromArray(attrArray.Field(i), row)
			if err != nil {
				return nil, err
			}
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BoolValue{BoolValue: v}}})
		case *arrow.Float64Type:
			v, err := air.F64FromArray(attrArray.Field(i), row)
			if err != nil {
				return nil, err
			}
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_DoubleValue{DoubleValue: v}}})
		case *arrow.Int64Type:
			v, err := air.I64FromArray(attrArray.Field(i), row)
			if err != nil {
				return nil, err
			}
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: v}}})
		case *arrow.StringType:
			v, err := air.StringFromArray(attrArray.Field(i), row)
			if err != nil {
				return nil, err
			}
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: v}}})
		case *arrow.BinaryType:
			v, err := air.BinaryFromArray(attrArray.Field(i), row)
			if err != nil {
				return nil, err
			}
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_BytesValue{BytesValue: v}}})
		case *arrow.StructType:
			// ToDo
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_KvlistValue{KvlistValue: nil}}})
		case *arrow.ListType:
			// ToDo
			kvs = append(kvs, &commonpb.KeyValue{
				Key:   fieldName,
				Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_ArrayValue{ArrayValue: nil}}})
		case *arrow.DictionaryType:
			// ToDo
			println("attribute with dictionary type not yet implemented")
		default:
			return nil, fmt.Errorf("attribute %q is not a supported type (type: %T)", fieldName, t)
		}
	}
	return kvs, nil
}
