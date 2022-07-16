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

package field_value

import (
	"github.com/apache/arrow/go/v9/arrow"
	"sort"
	"strings"
)

const BOOL_SIG = "Bol"
const U8_SIG = "U8"
const U16_SIG = "U16"
const U32_SIG = "U32"
const U64_SIG = "U64"
const I8_SIG = "I8"
const I16_SIG = "I16"
const I32_SIG = "I32"
const I64_SIG = "I64"
const F32_SIG = "F32"
const F64_SIG = "F64"
const BINARY_SIG = "Bin"
const STRING_SIG = "Str"

type NameType struct {
	Name string
	Type string
}

// DataTypeSignature returns the canonical arrow.DataType signature of the data type.
func DataTypeSignature(dataType arrow.DataType) string {
	switch dataType.ID() {
	case arrow.BOOL:
		return BOOL_SIG
	case arrow.UINT8:
		return U8_SIG
	case arrow.UINT16:
		return U16_SIG
	case arrow.UINT32:
		return U32_SIG
	case arrow.UINT64:
		return U64_SIG
	case arrow.INT8:
		return I8_SIG
	case arrow.INT16:
		return I16_SIG
	case arrow.INT32:
		return I32_SIG
	case arrow.INT64:
		return I64_SIG
	case arrow.FLOAT32:
		return F32_SIG
	case arrow.FLOAT64:
		return F64_SIG
	case arrow.STRING:
		return STRING_SIG
	case arrow.BINARY:
		return BINARY_SIG
	case arrow.LIST:
		return "[" + DataTypeSignature(dataType.(*arrow.ListType).Elem()) + "]"
	case arrow.STRUCT:
		var fields []NameType
		structDataType := dataType.(*arrow.StructType)
		for _, field := range structDataType.Fields() {
			fields = append(fields, NameType{
				Name: field.Name,
				Type: DataTypeSignature(field.Type),
			})
		}
		sort.Slice(fields, func(i, j int) bool {
			return fields[i].Name < fields[j].Name
		})
		fieldSigs := make([]string, 0, len(fields))
		for _, field := range fields {
			fieldSigs = append(fieldSigs, field.Name+":"+field.Type)
		}
		return "{" + strings.Join(fieldSigs, ",") + "}"
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

func StructDataType(fields []*Field) arrow.DataType {
	arrowFields := make([]arrow.Field, 0, len(fields))
	for _, field := range fields {
		arrowFields = append(arrowFields, arrow.Field{
			Name:     field.Name,
			Type:     field.DataType(),
			Nullable: true,
			Metadata: arrow.Metadata{},
		})
	}
	return arrow.StructOf(arrowFields...)
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
		for dataType := range dataTypeSet {
			dataTypes = append(dataTypes, dataType)
		}
		return CoerceDataType(&dataTypes)
	} else {
		return arrow.Null
	}
}

// CoerceDataType coerces an heterogeneous set of [`DataType`] into a single one. Rules:
// * `Int64` and `Float64` are `Float64`
// * Lists and scalars are coerced to a list of a compatible scalar
// * Structs contain the union of all fields
// * All other types are coerced to `Utf8`.
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
	//exhaustive:ignore
	switch dataType1.ID() {
	case arrow.PrimitiveTypes.Uint8.ID():
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
	case arrow.PrimitiveTypes.Int8.ID():
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
	case arrow.PrimitiveTypes.Uint16.ID():
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
	case arrow.PrimitiveTypes.Int16.ID():
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
	case arrow.PrimitiveTypes.Uint32.ID():
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
	case arrow.PrimitiveTypes.Int32.ID():
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
	case arrow.PrimitiveTypes.Uint64.ID():
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
	case arrow.PrimitiveTypes.Int64.ID():
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
	case arrow.FixedWidthTypes.Boolean.ID():
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
	case arrow.BinaryTypes.Binary.ID():
		return arrow.BinaryTypes.Binary
	default:
		return arrow.BinaryTypes.String
	}
}
