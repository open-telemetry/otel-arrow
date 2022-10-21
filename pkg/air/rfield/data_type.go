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

package rfield

import (
	"sort"
	"strings"

	"github.com/apache/arrow/go/v9/arrow"

	"github.com/lquerel/otel-arrow-adapter/pkg/air/common"
)

type NameTypes []*NameType

// Sort interface for Arrow Fields
type ArrowFields []arrow.Field

// Sort interface
func (f ArrowFields) Less(i, j int) bool {
	return f[i].Name < f[j].Name
}
func (f ArrowFields) Len() int      { return len(f) }
func (f ArrowFields) Swap(i, j int) { f[i], f[j] = f[j], f[i] }

// Sort interface
func (f NameTypes) Less(i, j int) bool {
	return f[i].Name < f[j].Name
}
func (f NameTypes) Len() int      { return len(f) }
func (f NameTypes) Swap(i, j int) { f[i], f[j] = f[j], f[i] }

type NameType struct {
	Name string
	Type string
}

// WriteDataTypeSignature writes the canonical arrow.DataType signature of the data type.
func WriteDataTypeSignature(dataType arrow.DataType, sig *strings.Builder) {
	switch dataType.ID() {
	case arrow.BOOL:
		sig.WriteString(common.BOOL_SIG)
	case arrow.UINT8:
		sig.WriteString(common.U8_SIG)
	case arrow.UINT16:
		sig.WriteString(common.U16_SIG)
	case arrow.UINT32:
		sig.WriteString(common.U32_SIG)
	case arrow.UINT64:
		sig.WriteString(common.U64_SIG)
	case arrow.INT8:
		sig.WriteString(common.I8_SIG)
	case arrow.INT16:
		sig.WriteString(common.I16_SIG)
	case arrow.INT32:
		sig.WriteString(common.I32_SIG)
	case arrow.INT64:
		sig.WriteString(common.I64_SIG)
	case arrow.FLOAT32:
		sig.WriteString(common.F32_SIG)
	case arrow.FLOAT64:
		sig.WriteString(common.F64_SIG)
	case arrow.STRING:
		sig.WriteString(common.STRING_SIG)
	case arrow.BINARY:
		sig.WriteString(common.BINARY_SIG)
	case arrow.LIST:
		sig.WriteByte('[')
		WriteDataTypeSignature(dataType.(*arrow.ListType).Elem(), sig)
		sig.WriteByte(']')
	case arrow.STRUCT:
		structDataType := dataType.(*arrow.StructType)
		sig.WriteByte('{')
		for i, field := range structDataType.Fields() {
			if i > 0 {
				sig.WriteByte(',')
			}
			sig.WriteString(field.Name)
			sig.WriteByte(':')
			WriteDataTypeSignature(field.Type, sig)
		}
		sig.WriteByte('}')
	case arrow.DATE32, arrow.DATE64, arrow.DECIMAL128, arrow.DECIMAL256, arrow.DENSE_UNION, arrow.SPARSE_UNION,
		arrow.INTERVAL, arrow.TIME32, arrow.TIME64, arrow.DICTIONARY, arrow.FIXED_SIZE_LIST, arrow.MAP,
		arrow.FIXED_SIZE_BINARY, arrow.INTERVAL_DAY_TIME, arrow.INTERVAL_MONTHS, arrow.INTERVAL_MONTH_DAY_NANO,
		arrow.DURATION, arrow.EXTENSION, arrow.FLOAT16, arrow.LARGE_LIST, arrow.LARGE_STRING, arrow.LARGE_BINARY,
		arrow.NULL, arrow.TIMESTAMP:
		fallthrough
	default:
		panic("unknown data type '" + dataType.ID().String() + "'")
	}
}

// DataTypeSignature returns the canonical arrow.DataType signature of the data type.
func DataTypeSignature(dataType arrow.DataType) string {
	var sig strings.Builder
	WriteDataTypeSignature(dataType, &sig)
	return sig.String()
}

// CoerceDataType coerces an heterogeneous set of [`DataType`] into a single one. Rules:
// * `Int64` and `Float64` are `Float64`
// * Lists and scalars are coerced to a list of a compatible scalar
// * Structs contain the union of all fields
// * All other types are coerced to `Utf8`.
func CoerceDataType(dataTypes []arrow.DataType) arrow.DataType {
	areAllStructs := true
	for _, otherDataType := range dataTypes {
		if otherDataType.ID() != arrow.STRUCT {
			areAllStructs = false
			break
		}
	}
	if areAllStructs {
		fields := map[string]arrow.DataType{}
		for _, dataType := range dataTypes {
			for _, field := range dataType.(*arrow.StructType).Fields() {
				if fieldDataType, found := fields[field.Name]; found {
					fields[field.Name] = CoerceDataType([]arrow.DataType{fieldDataType, field.Type})
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
		// ToDo check if it's possible to get rid of this sort as the incoming dataTypes are already sorted. Note: the fields map removes this incoming sort.
		sort.Sort(ArrowFields(structFields))
		dataType := arrow.StructOf(structFields...)
		return dataType
	} else {
		dataType := dataTypes[0]
		for i := 1; i < len(dataTypes); i++ {
			dataType = CoerceDataTypes(dataType, dataTypes[i])
		}

		return dataType
	}
}

func CoerceDataTypes(dataType1 arrow.DataType, dataType2 arrow.DataType) arrow.DataType {
	//exhaustive:ignore
	switch dt1 := dataType1.(type) {
	case *arrow.Uint8Type:
		//exhaustive:ignore
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
	case *arrow.Int8Type:
		//exhaustive:ignore
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
	case *arrow.Uint16Type:
		//exhaustive:ignore
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
	case *arrow.Int16Type:
		//exhaustive:ignore
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
	case *arrow.Uint32Type:
		//exhaustive:ignore
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
	case *arrow.Int32Type:
		//exhaustive:ignore
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
	case *arrow.Uint64Type:
		//exhaustive:ignore
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
	case *arrow.Int64Type:
		//exhaustive:ignore
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
	case *arrow.Float32Type:
		//exhaustive:ignore
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Float32.ID():
			return arrow.PrimitiveTypes.Float32
		case arrow.PrimitiveTypes.Float64.ID():
			return arrow.PrimitiveTypes.Float64
		default:
			return arrow.BinaryTypes.String
		}
	case *arrow.Float64Type:
		//exhaustive:ignore
		switch dataType2.ID() {
		case arrow.PrimitiveTypes.Float32.ID():
			return arrow.PrimitiveTypes.Float64
		case arrow.PrimitiveTypes.Float64.ID():
			return arrow.PrimitiveTypes.Float64
		default:
			return arrow.BinaryTypes.String
		}
	case *arrow.BooleanType:
		//exhaustive:ignore
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
	case *arrow.BinaryType:
		return arrow.BinaryTypes.Binary
	case *arrow.StructType:
		switch dataType2.(type) {
		case *arrow.StructType:
			return CoerceDataType([]arrow.DataType{dataType1, dataType2})
		default:
			panic("coercion not implemented")
		}
	case *arrow.ListType:
		switch dt2 := dataType2.(type) {
		case *arrow.ListType:
			return arrow.ListOf(CoerceDataType([]arrow.DataType{dt1.Elem(), dt2.Elem()}))
		default:
			panic("coercion not implemented")
		}
	default:
		return arrow.BinaryTypes.String
	}
}
