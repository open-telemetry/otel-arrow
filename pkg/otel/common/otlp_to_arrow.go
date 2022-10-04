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

package common

import (
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
)

func NewAttributes(attributes pcommon.Map, cfg *config.Config) *rfield.Field {
	switch cfg.Attribute.Encoding {
	case config.AttributesAsStructs:
		return NewAttributesAsStructs(attributes)
	case config.AttributesAsLists:
		return NewAttributesAsLists(attributes)
	case config.AttributesAsListStructs:
		return NewAttributesAsListStructs(attributes)
	default:
		panic(fmt.Sprintf("unknown attribute encoding: %v", cfg.Attribute.Encoding))
	}
}

func NewAttributesAsStructs(attributes pcommon.Map) *rfield.Field {
	if attributes.Len() == 0 {
		return nil
	}

	attributeFields := make([]*rfield.Field, 0, attributes.Len())

	attributes.Range(func(key string, v pcommon.Value) bool {
		if value := OtlpAnyValueToValue(v); value != nil {
			attributeFields = append(attributeFields, &rfield.Field{
				Name:  key,
				Value: value,
			})
		}
		return true
	})
	if len(attributeFields) == 0 {
		return nil
	}
	return rfield.NewStructField(constants.ATTRIBUTES, rfield.Struct{
		Fields: attributeFields,
	})
}

func NewAttributesAsLists(attributes pcommon.Map) *rfield.Field {
	if attributes.Len() == 0 {
		return nil
	}

	attributeFields := make([]*rfield.Field, 0, attributes.Len())

	stringAttrKeys := make([]rfield.Value, 0, attributes.Len())
	stringAttrValues := make([]rfield.Value, 0, attributes.Len())

	i64AttrKeys := make([]rfield.Value, 0, attributes.Len())
	i64AttrValues := make([]rfield.Value, 0, attributes.Len())

	f64AttrKeys := make([]rfield.Value, 0, attributes.Len())
	f64AttrValues := make([]rfield.Value, 0, attributes.Len())

	boolAttrKeys := make([]rfield.Value, 0, attributes.Len())
	boolAttrValues := make([]rfield.Value, 0, attributes.Len())

	attributes.Range(func(key string, v pcommon.Value) bool {
		if value := OtlpAnyValueToValue(v); value != nil {
			switch v := value.(type) {
			case *rfield.String:
				stringAttrKeys = append(stringAttrKeys, rfield.NewString(key))
				stringAttrValues = append(stringAttrValues, &rfield.String{Value: v.Value})
			case *rfield.I64:
				i64AttrKeys = append(i64AttrKeys, rfield.NewString(key))
				i64AttrValues = append(i64AttrValues, &rfield.I64{Value: v.Value})
			case *rfield.F64:
				f64AttrKeys = append(f64AttrKeys, rfield.NewString(key))
				f64AttrValues = append(f64AttrValues, &rfield.F64{Value: v.Value})
			case *rfield.Bool:
				boolAttrKeys = append(boolAttrKeys, rfield.NewString(key))
				boolAttrValues = append(boolAttrValues, &rfield.Bool{Value: v.Value})
			default:
				panic(fmt.Sprintf("unsupported type: %T", value))
				//attributeFields = append(attributeFields, &rfield.Field{
				//	Name:  key,
				//	Value: value,
				//})
			}
		}
		return true
	})
	if len(stringAttrKeys) > 0 {
		attributeFields = append(attributeFields, rfield.NewListField("string_attr_keys", rfield.List{
			Values: stringAttrKeys,
		}))
		attributeFields = append(attributeFields, rfield.NewListField("string_attr_values", rfield.List{
			Values: stringAttrValues,
		}))
	}
	if len(i64AttrKeys) > 0 {
		attributeFields = append(attributeFields, rfield.NewListField("i64_attr_keys", rfield.List{
			Values: i64AttrKeys,
		}))
		attributeFields = append(attributeFields, rfield.NewListField("i64_attr_values", rfield.List{
			Values: i64AttrValues,
		}))
	}
	if len(f64AttrKeys) > 0 {
		attributeFields = append(attributeFields, rfield.NewListField("f64_attr_keys", rfield.List{
			Values: f64AttrKeys,
		}))
		attributeFields = append(attributeFields, rfield.NewListField("f64_attr_values", rfield.List{
			Values: f64AttrValues,
		}))
	}
	if len(boolAttrKeys) > 0 {
		attributeFields = append(attributeFields, rfield.NewListField("bool_attr_keys", rfield.List{
			Values: boolAttrKeys,
		}))
		attributeFields = append(attributeFields, rfield.NewListField("bool_attr_values", rfield.List{
			Values: boolAttrValues,
		}))
	}
	if len(attributeFields) > 0 {
		attrs := rfield.NewStructField(constants.ATTRIBUTES, rfield.Struct{
			Fields: attributeFields,
		})
		return attrs
	}
	return nil
}

type AttributeTuple struct {
	key    string
	i64    *int64
	f64    *float64
	str    *string
	bool   *bool
	binary []byte
}

type AttributeTuples []AttributeTuple

// Sort interface
func (f AttributeTuples) Less(i, j int) bool {
	return f[i].key < f[j].key
}
func (f AttributeTuples) Len() int      { return len(f) }
func (f AttributeTuples) Swap(i, j int) { f[i], f[j] = f[j], f[i] }

func NewAttributesAsListStructs(attributes pcommon.Map) *rfield.Field {
	if attributes.Len() == 0 {
		return nil
	}

	attributeTuples := make([]AttributeTuple, 0, attributes.Len())

	stringCount := 0
	i64Count := 0
	f64Count := 0
	boolCount := 0
	binaryCount := 0

	attributes.Range(func(key string, v pcommon.Value) bool {
		if value := OtlpAnyValueToValue(v); value != nil {
			switch v := value.(type) {
			case *rfield.String:
				attributeTuples = append(attributeTuples, AttributeTuple{
					key: key,
					str: v.Value,
				})
				stringCount++
			case *rfield.I64:
				attributeTuples = append(attributeTuples, AttributeTuple{
					key: key,
					i64: v.Value,
				})
				i64Count++
			case *rfield.F64:
				attributeTuples = append(attributeTuples, AttributeTuple{
					key: key,
					f64: v.Value,
				})
				f64Count++
			case *rfield.Bool:
				attributeTuples = append(attributeTuples, AttributeTuple{
					key:  key,
					bool: v.Value,
				})
				boolCount++
			case *rfield.Binary:
				attributeTuples = append(attributeTuples, AttributeTuple{
					key:    key,
					binary: v.Value,
				})
				binaryCount++
			default:
				panic(fmt.Sprintf("unexpected type: %T", v))
			}
		}
		return true
	})
	values := make([]rfield.Value, 0, attributes.Len())
	if len(attributeTuples) > 0 {
		// ToDo remove attribute tuples and create the structs directly as the sorting of attributeTuples doesn't improve compression ratio
		for _, attribute := range attributeTuples {
			fields := make([]*rfield.Field, 0, len(attributeTuples))
			fields = append(fields, &rfield.Field{Name: "key", Value: &rfield.String{Value: &attribute.key}})
			if stringCount > 0 {
				fields = append(fields, &rfield.Field{Name: "string", Value: &rfield.String{Value: attribute.str}})
			}
			if i64Count > 0 {
				fields = append(fields, &rfield.Field{Name: "i64", Value: &rfield.I64{Value: attribute.i64}})
			}
			if f64Count > 0 {
				fields = append(fields, &rfield.Field{Name: "f64", Value: &rfield.F64{Value: attribute.f64}})
			}
			if boolCount > 0 {
				fields = append(fields, &rfield.Field{Name: "bool", Value: &rfield.Bool{Value: attribute.bool}})
			}
			if binaryCount > 0 {
				fields = append(fields, &rfield.Field{Name: "binary", Value: &rfield.Binary{Value: attribute.binary}})
			}
			values = append(values, &rfield.Struct{Fields: fields})
		}
	}

	if len(values) > 0 {
		fieldStruct := make([]arrow.Field, 0, 6)
		if binaryCount > 0 {
			fieldStruct = append(fieldStruct, arrow.Field{Name: "binary", Type: arrow.BinaryTypes.Binary, Nullable: true, Metadata: arrow.Metadata{}})
		}
		if boolCount > 0 {
			fieldStruct = append(fieldStruct, arrow.Field{Name: "bool", Type: arrow.FixedWidthTypes.Boolean, Nullable: true, Metadata: arrow.Metadata{}})
		}
		if f64Count > 0 {
			fieldStruct = append(fieldStruct, arrow.Field{Name: "f64", Type: arrow.PrimitiveTypes.Float64, Nullable: true, Metadata: arrow.Metadata{}})
		}
		if i64Count > 0 {
			fieldStruct = append(fieldStruct, arrow.Field{Name: "i64", Type: arrow.PrimitiveTypes.Int64, Nullable: true, Metadata: arrow.Metadata{}})
		}
		fieldStruct = append(fieldStruct, arrow.Field{Name: "key", Type: arrow.BinaryTypes.String, Nullable: true, Metadata: arrow.Metadata{}})
		if stringCount > 0 {
			fieldStruct = append(fieldStruct, arrow.Field{Name: "string", Type: arrow.BinaryTypes.String, Nullable: true, Metadata: arrow.Metadata{}})
		}
		etype := arrow.StructOf(fieldStruct...)
		attrs := rfield.NewListField(constants.ATTRIBUTES, *rfield.UnsafeNewList(etype, values))
		return attrs
	}
	return nil
}

func AttributesValue(attributes pcommon.Map) rfield.Value {
	if attributes.Len() == 0 {
		return nil
	}

	attributeFields := make([]*rfield.Field, 0, attributes.Len())

	attributes.Range(func(key string, v pcommon.Value) bool {
		if value := OtlpAnyValueToValue(v); value != nil {
			attributeFields = append(attributeFields, &rfield.Field{
				Name:  key,
				Value: value,
			})
		}
		return true
	})
	if len(attributeFields) == 0 {
		return nil
	}
	return &rfield.Struct{
		Fields: attributeFields,
	}
}

func AddResource(record *air.Record, resource pcommon.Resource, cfg *config.Config) {
	resourceField := ResourceField(resource, cfg)
	if resourceField != nil {
		record.AddField(resourceField)
	}
}

func ResourceField(resource pcommon.Resource, cfg *config.Config) *rfield.Field {
	var resourceFields []*rfield.Field

	// ToDo test attributes := NewAttributes(resource.Attributes)
	attributes := NewAttributes(resource.Attributes(), cfg)
	if attributes != nil {
		resourceFields = append(resourceFields, attributes)
	}

	if resource.DroppedAttributesCount() > 0 {
		resourceFields = append(resourceFields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, resource.DroppedAttributesCount()))
	}
	if len(resourceFields) > 0 {
		field := rfield.NewStructField(constants.RESOURCE, rfield.Struct{
			Fields: resourceFields,
		})
		return field
	} else {
		return nil
	}
}

func AddScope(record *air.Record, scopeKey string, scope pcommon.InstrumentationScope, cfg *config.Config) {
	scopeField := ScopeField(scopeKey, scope, cfg)
	if scopeField != nil {
		record.AddField(scopeField)
	}
}

func ScopeField(scopeKey string, scope pcommon.InstrumentationScope, cfg *config.Config) *rfield.Field {
	var fields []*rfield.Field

	if len(scope.Name()) > 0 {
		fields = append(fields, rfield.NewStringField(constants.NAME, scope.Name()))
	}
	if len(scope.Version()) > 0 {
		fields = append(fields, rfield.NewStringField(constants.VERSION, scope.Version()))
	}
	// todo attributes := NewAttributes(scope.Attributes)
	attributes := NewAttributes(scope.Attributes(), cfg)
	if attributes != nil {
		fields = append(fields, attributes)
	}
	if scope.DroppedAttributesCount() > 0 {
		fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, scope.DroppedAttributesCount()))
	}

	field := rfield.NewStructField(scopeKey, rfield.Struct{
		Fields: fields,
	})
	return field
}

func OtlpAnyValueToValue(value pcommon.Value) rfield.Value {
	switch value.Type() {
	case pcommon.ValueTypeEmpty:
		return nil
	case pcommon.ValueTypeString:
		return rfield.NewString(value.StringVal())
	case pcommon.ValueTypeInt:
		return rfield.NewI64(value.IntVal())
	case pcommon.ValueTypeDouble:
		return rfield.NewF64(value.DoubleVal())
	case pcommon.ValueTypeBool:
		return rfield.NewBool(value.BoolVal())
	case pcommon.ValueTypeBytes:
		return &rfield.Binary{Value: value.BytesVal().AsRaw()}
	case pcommon.ValueTypeSlice:
		values := value.SliceVal()
		fieldValues := make([]rfield.Value, 0, values.Len())

		for i := 0; i < values.Len(); i++ {
			if v := OtlpAnyValueToValue(values.At(i)); v != nil {
				fieldValues = append(fieldValues, v)
			}
		}
		if len(fieldValues) == 0 {
			return nil
		}
		return &rfield.List{Values: fieldValues}
	case pcommon.ValueTypeMap:
		values := value.MapVal()
		if values.Len() == 0 {
			return nil
		}
		fields := make([]*rfield.Field, 0, values.Len())
		values.Range(func(key string, v pcommon.Value) bool {
			if value := OtlpAnyValueToValue(v); value != nil {
				fields = append(fields, &rfield.Field{
					Name:  key,
					Value: value,
				})
			}
			return true
		})
		if len(fields) == 0 {
			return nil
		}
		return &rfield.Struct{Fields: fields}
	}
	return nil
}
