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
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/otel/constants"
)

func NewAttributes(attributes []*commonpb.KeyValue) *rfield.Field {
	if attributes == nil || len(attributes) == 0 {
		return nil
	}

	attributeFields := make([]*rfield.Field, 0, len(attributes))

	for _, attribute := range attributes {
		value := OtlpAnyValueToValue(attribute.Value)
		if value != nil {
			attributeFields = append(attributeFields, &rfield.Field{
				Name:  attribute.Key,
				Value: value,
			})
		}
	}
	if len(attributeFields) > 0 {
		attrs := rfield.NewStructField(constants.ATTRIBUTES, rfield.Struct{
			Fields: attributeFields,
		})
		return attrs
	}
	return nil
}

func AttributesValue(attributes []*commonpb.KeyValue) rfield.Value {
	if attributes == nil || len(attributes) == 0 {
		return nil
	}

	attributeFields := make([]*rfield.Field, 0, len(attributes))

	for _, attribute := range attributes {
		value := OtlpAnyValueToValue(attribute.Value)
		if value != nil {
			attributeFields = append(attributeFields, &rfield.Field{
				Name:  attribute.Key,
				Value: value,
			})
		}
	}
	if len(attributeFields) > 0 {
		return &rfield.Struct{
			Fields: attributeFields,
		}
	}
	return nil
}

func AddResource(record *air.Record, resource *resourcepb.Resource) {
	resourceField := ResourceField(resource)
	if resourceField != nil {
		record.AddField(resourceField)
	}
}

func ResourceField(resource *resourcepb.Resource) *rfield.Field {
	var resourceFields []*rfield.Field

	attributes := NewAttributes(resource.Attributes)
	if attributes != nil {
		resourceFields = append(resourceFields, attributes)
	}

	if resource.DroppedAttributesCount > 0 {
		resourceFields = append(resourceFields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, resource.DroppedAttributesCount))
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

func AddScope(record *air.Record, scopeKey string, scope *commonpb.InstrumentationScope) {
	scopeField := ScopeField(scopeKey, scope)
	if scopeField != nil {
		// ToDo check optimization for when fields are always pointers or interfaces instead of structs as today.
		record.AddField(scopeField)
	}
}

func ScopeField(scopeKey string, scope *commonpb.InstrumentationScope) *rfield.Field {
	var fields []*rfield.Field

	fields = append(fields, rfield.NewStringField(constants.NAME, scope.Name))
	fields = append(fields, rfield.NewStringField(constants.VERSION, scope.Version))
	attributes := NewAttributes(scope.Attributes)
	if attributes != nil {
		fields = append(fields, attributes)
	}
	if scope.DroppedAttributesCount > 0 {
		fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, scope.DroppedAttributesCount))
	}

	field := rfield.NewStructField(scopeKey, rfield.Struct{
		Fields: fields,
	})
	return field
}

func OtlpAnyValueToValue(value *commonpb.AnyValue) rfield.Value {
	if value != nil {
		switch value.Value.(type) {
		case *commonpb.AnyValue_BoolValue:
			return &rfield.Bool{Value: value.GetBoolValue()}
		case *commonpb.AnyValue_IntValue:
			return &rfield.I64{Value: value.GetIntValue()}
		case *commonpb.AnyValue_DoubleValue:
			return &rfield.F64{Value: value.GetDoubleValue()}
		case *commonpb.AnyValue_StringValue:
			return &rfield.String{Value: value.GetStringValue()}
		case *commonpb.AnyValue_BytesValue:
			return &rfield.Binary{Value: value.GetBytesValue()}
		case *commonpb.AnyValue_ArrayValue:
			values := value.GetArrayValue()
			fieldValues := make([]rfield.Value, 0, len(values.Values))
			for _, value := range values.Values {
				v := OtlpAnyValueToValue(value)
				if v != nil {
					fieldValues = append(fieldValues, v)
				}
			}
			return &rfield.List{Values: fieldValues}
		case *commonpb.AnyValue_KvlistValue:
			values := value.GetKvlistValue()
			if values == nil || len(values.Values) == 0 {
				return nil
			} else {
				fields := make([]*rfield.Field, 0, len(values.Values))
				for _, kv := range values.Values {
					v := OtlpAnyValueToValue(kv.Value)
					if v != nil {
						fields = append(fields, &rfield.Field{
							Name:  kv.Key,
							Value: v,
						})
					}
				}
				return &rfield.Struct{Fields: fields}
			}
		default:
			return nil
		}
	} else {
		return nil
	}
}
