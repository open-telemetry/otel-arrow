// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package rbb

import (
	"github.com/apache/arrow/go/arrow"
	"sort"
)

type Value interface {
	Normalize()
	DataType() arrow.DataType
}

type CommonValue struct{}

func (cv *CommonValue) Normalize() {}

type Bool struct {
	CommonValue
	Value bool
}

func (v *Bool) DataType() arrow.DataType { return arrow.FixedWidthTypes.Boolean }

type I8 struct {
	CommonValue
	Value int8
}

func (v *I8) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int8 }

type I16 struct {
	CommonValue
	Value int16
}

func (v *I16) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int16 }

type I32 struct {
	CommonValue
	Value int32
}

func (v *I32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int32 }

type I64 struct {
	CommonValue
	Value int64
}

func (v *I64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int64 }

type U8 struct {
	CommonValue
	Value uint8
}

func (v *U8) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint8 }

type U16 struct {
	CommonValue
	Value uint16
}

func (v *U16) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint16 }

type U32 struct {
	CommonValue
	Value uint32
}

func (v *U32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint32 }

type U64 struct {
	CommonValue
	Value uint64
}

func (v *U64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint64 }

type F32 struct {
	CommonValue
	Value float32
}

func (v *F32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Float32 }

type F64 struct {
	CommonValue
	Value float64
}

func (v *F64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Float64 }

type String struct {
	CommonValue
	Value string
}

func (v *String) DataType() arrow.DataType { return arrow.BinaryTypes.String }

type Binary struct {
	CommonValue
	Value []byte
}

func (v *Binary) DataType() arrow.DataType { return arrow.BinaryTypes.Binary }

type Struct struct {
	fields []Field
}

func (v *Struct) DataType() arrow.DataType {
	fields := make([]arrow.Field, 0, len(v.fields))
	for _, field := range v.fields {
		arrowField := arrow.Field{Name: field.Name, Type: field.Value.DataType(), Nullable: true, Metadata: arrow.Metadata{}}
		fields = append(fields, arrowField)
	}
	return arrow.StructOf(fields...)
}
func (v *Struct) Normalize() {
	// Sort all the fields by name
	sort.Slice(v.fields, func(i, j int) bool {
		return v.fields[i].Name < v.fields[j].Name
	})
	// Normalize recursively all the fields
	for _, field := range v.fields {
		field.Normalize()
	}
}

type List struct {
	values []Value
}

func (v *List) DataType() arrow.DataType {
	return arrow.ListOf(ListDataType(v.values))
}
func (v *List) Normalize() {
	// Normalize recursively all the value
	for _, value := range v.values {
		value.Normalize()
	}
}

func ListDataType(values []Value) arrow.DataType {
	dataTypeSet := map[arrow.DataType]bool{}

	// Deduplicate data types
	for _, value := range values {
		dataType := value.DataType()
		if dataType.ID() != arrow.NULL {
			dataTypeSet[dataType] = true
		}
	}

	if len(dataTypeSet) > 0 {
		dataTypes := make([]arrow.DataType, 0, len(dataTypeSet))
		for dataType, _ := range dataTypeSet {
			dataTypes = append(dataTypes, dataType)
		}
		return CoerceDataType(&dataTypes)
	} else {
		return arrow.Null
	}
}

// ToDo what about list mixing struct, uint, string, ... items?

// CoerceDataType coerces an heterogeneous set of [`DataType`] into a single one. Rules:
// * `Int64` and `Float64` are `Float64`
// * Lists and scalars are coerced to a list of a compatible scalar
// * Structs contain the union of all fields
// * All other types are coerced to `Utf8`
func CoerceDataType(dataTypes *[]arrow.DataType) arrow.DataType {
	dataType := (*dataTypes)[0]

	areAllStructs := true
	for _, otherDataType := range *dataTypes {
		if otherDataType.ID() != arrow.STRUCT {
			areAllStructs = false
			break
		}
	}
	if areAllStructs {
		fields := map[string]arrow.DataType{}
		for _, dataType := range *dataTypes {
			for _, field := range dataType.(*arrow.StructType).Fields() {
				if dataType, found := fields[field.Name]; found {
					fields[field.Name] = CoerceDataTypes(dataType, field.Type)
				} else {
					fields[field.Name] = field.Type
				}
			}
		}
		structFields := make([]arrow.Field, 0, len(fields))

		for fieldName, fieldType := range fields {
			structFields = append(structFields, arrow.Field{
				Name:     fieldName,
				Type:     fieldType,
				Nullable: true,
				Metadata: arrow.Metadata{},
			})
		}
		return arrow.StructOf(structFields...)
	} else {
		areAllEqual := true
		for _, otherDataType := range *dataTypes {
			if dataType.ID() != otherDataType.ID() {
				areAllEqual = false
				break
			}
		}

		if areAllEqual {
			return dataType
		}

		for _, otherDataType := range *dataTypes {
			dataType = CoerceDataTypes(dataType, otherDataType)
		}
		return dataType
	}
}

func CoerceDataTypes(dataType1 arrow.DataType, dataType2 arrow.DataType) arrow.DataType {
	switch dataType1.ID() {
	case arrow.PrimitiveTypes.Uint8.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Uint8.ID():
			return arrow.PrimitiveTypes.Uint8
		case arrow.PrimitiveTypes.Uint16.ID():
			return arrow.PrimitiveTypes.Uint16
		case arrow.PrimitiveTypes.Uint32.ID():
			return arrow.PrimitiveTypes.Uint32
		case arrow.PrimitiveTypes.Uint64.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Uint8
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Int8.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Int8.ID():
			return arrow.PrimitiveTypes.Int8
		case arrow.PrimitiveTypes.Int16.ID():
			return arrow.PrimitiveTypes.Int16
		case arrow.PrimitiveTypes.Int32.ID():
			return arrow.PrimitiveTypes.Int32
		case arrow.PrimitiveTypes.Int64.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Int8
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Uint16.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Uint8.ID():
			return arrow.PrimitiveTypes.Uint16
		case arrow.PrimitiveTypes.Uint16.ID():
			return arrow.PrimitiveTypes.Uint16
		case arrow.PrimitiveTypes.Uint32.ID():
			return arrow.PrimitiveTypes.Uint32
		case arrow.PrimitiveTypes.Uint64.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Uint16
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Int16.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Int8.ID():
			return arrow.PrimitiveTypes.Int16
		case arrow.PrimitiveTypes.Int16.ID():
			return arrow.PrimitiveTypes.Int16
		case arrow.PrimitiveTypes.Int32.ID():
			return arrow.PrimitiveTypes.Int32
		case arrow.PrimitiveTypes.Int64.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Int16
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Uint32.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Uint8.ID():
			return arrow.PrimitiveTypes.Uint32
		case arrow.PrimitiveTypes.Uint16.ID():
			return arrow.PrimitiveTypes.Uint32
		case arrow.PrimitiveTypes.Uint32.ID():
			return arrow.PrimitiveTypes.Uint32
		case arrow.PrimitiveTypes.Uint64.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Uint32
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Int32.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Int8.ID():
			return arrow.PrimitiveTypes.Int32
		case arrow.PrimitiveTypes.Int16.ID():
			return arrow.PrimitiveTypes.Int32
		case arrow.PrimitiveTypes.Int32.ID():
			return arrow.PrimitiveTypes.Int32
		case arrow.PrimitiveTypes.Int64.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Int32
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Uint64.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Uint8.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.PrimitiveTypes.Uint16.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.PrimitiveTypes.Uint32.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.PrimitiveTypes.Uint64.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Uint64
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.PrimitiveTypes.Int64.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Int8.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.PrimitiveTypes.Int16.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.PrimitiveTypes.Int32.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.PrimitiveTypes.Int64.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.PrimitiveTypes.Int64
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.FixedWidthTypes.Boolean.ID():
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Uint8.ID():
			return arrow.PrimitiveTypes.Uint8
		case arrow.PrimitiveTypes.Uint16.ID():
			return arrow.PrimitiveTypes.Uint16
		case arrow.PrimitiveTypes.Uint32.ID():
			return arrow.PrimitiveTypes.Uint32
		case arrow.PrimitiveTypes.Uint64.ID():
			return arrow.PrimitiveTypes.Uint64
		case arrow.PrimitiveTypes.Int8.ID():
			return arrow.PrimitiveTypes.Int8
		case arrow.PrimitiveTypes.Int16.ID():
			return arrow.PrimitiveTypes.Int16
		case arrow.PrimitiveTypes.Int32.ID():
			return arrow.PrimitiveTypes.Int32
		case arrow.PrimitiveTypes.Int64.ID():
			return arrow.PrimitiveTypes.Int64
		case arrow.FixedWidthTypes.Boolean.ID():
			return arrow.FixedWidthTypes.Boolean
		default:
			return arrow.BinaryTypes.String
		}
	case arrow.BinaryTypes.Binary.ID():
		return arrow.BinaryTypes.Binary
	default:
		return arrow.BinaryTypes.String
	}
}
