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

package arrow

import (
	"fmt"
	"sort"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
)

// This file contains a set of utility functions to help extract data from Arrow records, arrays, structs, ...

// Constants used to create schema id signature.

const BoolSig = "Bol"
const U8Sig = "U8"
const U16Sig = "U16"
const U32Sig = "U32"
const U64Sig = "U64"
const I8Sig = "I8"
const I16Sig = "I16"
const I32Sig = "I32"
const I64Sig = "I64"
const F32Sig = "F32"
const F64Sig = "F64"
const BinarySig = "Bin"
const StringSig = "Str"
const Timestamp = "Tns" // Timestamp in nanoseconds.

// SortableField is a wrapper around arrow.Field that implements sort.Interface.
type SortableField struct {
	name  *string
	field *arrow.Field
}

type Fields []SortableField

func (d Fields) Less(i, j int) bool {
	return *d[i].name < *d[j].name
}
func (d Fields) Len() int      { return len(d) }
func (d Fields) Swap(i, j int) { d[i], d[j] = d[j], d[i] }

// SchemaToID creates a unique id for a schema.
// Fields are sorted by name before creating the id (done at each nested level).
func SchemaToID(schema *arrow.Schema) string {
	schemaID := ""
	fields := sortedFields(schema.Fields())

	for i := range fields {
		field := &fields[i]
		if i != 0 {
			schemaID += ","
		}
		schemaID += FieldToID(field.field)
	}

	return schemaID
}

func sortedFields(fields []arrow.Field) []SortableField {
	sortedField := make([]SortableField, len(fields))
	for i := 0; i < len(fields); i++ {
		sortedField[i] = SortableField{
			name:  &fields[i].Name,
			field: &fields[i],
		}
	}
	sort.Sort(Fields(sortedField))

	return sortedField
}

// FieldToID creates a unique id for a field.
func FieldToID(field *arrow.Field) string {
	return field.Name + ":" + DataTypeToID(field.Type)
}

// DataTypeToID creates a unique id for a data type.
func DataTypeToID(dt arrow.DataType) string {
	id := ""
	switch t := dt.(type) {
	case *arrow.BooleanType:
		id += BoolSig
	case *arrow.Int8Type:
		id += I8Sig
	case *arrow.Int16Type:
		id += I16Sig
	case *arrow.Int32Type:
		id += I32Sig
	case *arrow.Int64Type:
		id += I64Sig
	case *arrow.Uint8Type:
		id += U8Sig
	case *arrow.Uint16Type:
		id += U16Sig
	case *arrow.Uint32Type:
		id += U32Sig
	case *arrow.Uint64Type:
		id += U64Sig
	case *arrow.Float32Type:
		id += F32Sig
	case *arrow.Float64Type:
		id += F64Sig
	case *arrow.StringType:
		id += StringSig
	case *arrow.BinaryType:
		id += BinarySig
	case *arrow.TimestampType:
		id += Timestamp
	case *arrow.StructType:
		id += "{"
		fields := sortedFields(t.Fields())

		for i := range fields {
			if i > 0 {
				id += ","
			}
			id += FieldToID(fields[i].field)
		}
		id += "}"
	case *arrow.ListType:
		id += "["

		elemField := t.ElemField()

		id += DataTypeToID(elemField.Type)
		id += "]"
	case *arrow.DictionaryType:
		id += "Dic<"
		id += DataTypeToID(t.IndexType)
		id += ","
		id += DataTypeToID(t.ValueType)
		id += ">"
	case *arrow.DenseUnionType:
		// TODO implement
		id += "DenseUnion<>"
	case *arrow.SparseUnionType:
		// TODO implement
		id += "SparseUnion<>"
	case *arrow.MapType:
		// TODO implement
		id += "Map<>"
	case *arrow.FixedSizeBinaryType:
		// TODO implement
		id += "FixedSizeBinary<>"
	default:
		panic("unsupported data type " + dt.String())
	}

	return id
}

// ListOfStructsFieldIDFromSchema returns the field id of a list of structs field from an Arrow schema.
func ListOfStructsFieldIDFromSchema(schema *arrow.Schema, fieldName string) (int, *arrow.StructType, error) {
	ids := schema.FieldIndices(fieldName)
	if len(ids) == 0 {
		return 0, nil, fmt.Errorf("no field %q in schema", fieldName)
	}
	if len(ids) > 1 {
		return 0, nil, fmt.Errorf("more than one field %q in schema", fieldName)
	}

	if lt, ok := schema.Field(ids[0]).Type.(*arrow.ListType); ok {
		st, ok := lt.ElemField().Type.(*arrow.StructType)
		if !ok {
			return 0, nil, fmt.Errorf("field %q is not a list of structs", fieldName)
		}
		return ids[0], st, nil
	} else {
		return 0, nil, fmt.Errorf("field %q is not a list", fieldName)
	}
}

// ListOfStructsFieldIDFromStruct returns the field id of a list of structs field from an Arrow struct.
func ListOfStructsFieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.StructType, error) {
	id, ok := dt.FieldIdx(fieldName)
	if !ok {
		return 0, nil, fmt.Errorf("field %q not found", fieldName)
	}

	if lt, ok := dt.Field(id).Type.(*arrow.ListType); ok {
		st, ok := lt.ElemField().Type.(*arrow.StructType)
		if !ok {
			return 0, nil, fmt.Errorf("field %q is not a list of structs", fieldName)
		}
		return id, st, nil
	} else {
		return 0, nil, fmt.Errorf("field %q is not a list", fieldName)
	}
}

// StructFieldIDFromStruct returns the field id of a struct field from an Arrow struct.
func StructFieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.StructType, error) {
	id, found := dt.FieldIdx(fieldName)
	if !found {
		return 0, nil, fmt.Errorf("no field %q in struct type", fieldName)
	}
	if st, ok := dt.Field(id).Type.(*arrow.StructType); ok {
		return id, st, nil
	} else {
		return 0, nil, fmt.Errorf("field %q is not a struct", fieldName)
	}
}

// FieldIDFromStruct returns the field id of a named field from an Arrow struct.
func FieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.DataType, error) {
	id, found := dt.FieldIdx(fieldName)
	if !found {
		return 0, nil, fmt.Errorf("no field %q in struct type", fieldName)
	}
	field := dt.Field(id)
	return id, &field.Type, nil
}

// OptionalFieldIDFromStruct returns the field id of a named field from an Arrow struct or -1 if the field is unknown.
func OptionalFieldIDFromStruct(dt *arrow.StructType, fieldName string) (id int) {
	id, found := dt.FieldIdx(fieldName)
	if !found {
		id = -1
	}
	return
}

// ListOfStructs is a wrapper around an Arrow list of structs used to expose utility functions.
type ListOfStructs struct {
	dt    *arrow.StructType
	arr   *array.Struct
	start int
	end   int
}

// ListOfStructsFromRecord returns the struct type and an array of structs for a given field id.
func ListOfStructsFromRecord(record arrow.Record, fieldID int, row int) (*ListOfStructs, error) {
	arr := record.Column(fieldID)
	switch listArr := arr.(type) {
	case *array.List:
		if listArr.IsNull(row) {
			return nil, nil
		}

		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt, ok := structArr.DataType().(*arrow.StructType)
			if !ok {
				return nil, fmt.Errorf("field id %d is not a list of structs", fieldID)
			}
			start := int(listArr.Offsets()[row])
			end := int(listArr.Offsets()[row+1])

			return &ListOfStructs{
				dt:    dt,
				arr:   structArr,
				start: start,
				end:   end,
			}, nil
		default:
			return nil, fmt.Errorf("field id %d is not a list of structs", fieldID)
		}
	default:
		return nil, fmt.Errorf("field id %d is not a list", fieldID)
	}
}

// ListOfStructsFromStruct return a ListOfStructs from a struct field.
func ListOfStructsFromStruct(parent *array.Struct, fieldID int, row int) (*ListOfStructs, error) {
	arr := parent.Field(fieldID)
	if listArr, ok := arr.(*array.List); ok {
		if listArr.IsNull(row) {
			return nil, nil
		}

		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt, ok := structArr.DataType().(*arrow.StructType)
			if !ok {
				return nil, fmt.Errorf("field id %d is not a list of structs", fieldID)
			}
			start := int(listArr.Offsets()[row])
			end := int(listArr.Offsets()[row+1])

			return &ListOfStructs{
				dt:    dt,
				arr:   structArr,
				start: start,
				end:   end,
			}, nil
		default:
			return nil, fmt.Errorf("field id %d is not a list of structs", fieldID)
		}
	} else {
		return nil, fmt.Errorf("field id %d is not a list", fieldID)
	}
}

// Start returns the start index of the list of structs.
func (los *ListOfStructs) Start() int {
	return los.start
}

// End returns the end index of the list of structs.
func (los *ListOfStructs) End() int {
	return los.end
}

// FieldIdx returns the field id of a named field.
// The boolean return value indicates whether the field was found.
func (los *ListOfStructs) FieldIdx(name string) (int, bool) {
	return los.dt.FieldIdx(name)
}

// Field returns the field array of a named field.
// The boolean return value indicates whether the field was found.
func (los *ListOfStructs) Field(name string) (arrow.Array, bool) {
	id, ok := los.dt.FieldIdx(name)
	if !ok {
		return nil, false
	}
	return los.arr.Field(id), true
}

// FieldByID returns the field array of a field id.
func (los *ListOfStructs) FieldByID(id int) arrow.Array {
	return los.arr.Field(id)
}

// StringFieldByID returns the string value of a field id for a specific row.
func (los *ListOfStructs) StringFieldByID(fieldID int, row int) (string, error) {
	column := los.arr.Field(fieldID)
	return StringFromArray(column, row)
}

// U32FieldByID returns the uint32 value of a field id for a specific row.
func (los *ListOfStructs) U32FieldByID(fieldID int, row int) (uint32, error) {
	column := los.arr.Field(fieldID)
	return U32FromArray(column, row)
}

// U64FieldByID returns the uint64 value of a field id for a specific row.
func (los *ListOfStructs) U64FieldByID(fieldID int, row int) (uint64, error) {
	column := los.arr.Field(fieldID)
	return U64FromArray(column, row)
}

// TimestampFieldByID returns the timestamp value of a field id for a specific row.
func (los *ListOfStructs) TimestampFieldByID(fieldID int, row int) (arrow.Timestamp, error) {
	column := los.arr.Field(fieldID)
	return TimestampFromArray(column, row)
}

// OptionalTimestampFieldByID returns the timestamp value of a field id for a specific row or nil if the field is null.
func (los *ListOfStructs) OptionalTimestampFieldByID(fieldID int, row int) *pcommon.Timestamp {
	column := los.arr.Field(fieldID)
	if column.IsNull(row) {
		return nil
	}
	ts, err := TimestampFromArray(column, row)
	if err != nil {
		return nil
	}

	timestamp := pcommon.Timestamp(ts)
	return &timestamp
}

// I32FieldByID returns the int32 value of a field id for a specific row.
func (los *ListOfStructs) I32FieldByID(fieldID int, row int) (int32, error) {
	column := los.arr.Field(fieldID)
	return I32FromArray(column, row)
}

// I64FieldByID returns the int64 value of a field id for a specific row.
func (los *ListOfStructs) I64FieldByID(fieldID int, row int) (int64, error) {
	column := los.arr.Field(fieldID)
	return I64FromArray(column, row)
}

// F64FieldByID returns the float64 value of a field id for a specific row.
func (los *ListOfStructs) F64FieldByID(fieldID int, row int) (float64, error) {
	column := los.arr.Field(fieldID)
	return F64FromArray(column, row)
}

// F64OrNilFieldByID returns the float64 value of a field id for a specific row or nil if the field is null.
func (los *ListOfStructs) F64OrNilFieldByID(fieldID int, row int) (*float64, error) {
	column := los.arr.Field(fieldID)
	return F64OrNilFromArray(column, row)
}

// BoolFieldByID returns the bool value of a field id for a specific row.
func (los *ListOfStructs) BoolFieldByID(fieldID int, row int) (bool, error) {
	column := los.arr.Field(fieldID)
	return BoolFromArray(column, row)
}

// BinaryFieldByID returns the binary value of a field id for a specific row.
func (los *ListOfStructs) BinaryFieldByID(fieldID int, row int) ([]byte, error) {
	column := los.arr.Field(fieldID)
	return BinaryFromArray(column, row)
}

// FixedSizeBinaryFieldByID returns the fixed size binary value of a field id for a specific row.
func (los *ListOfStructs) FixedSizeBinaryFieldByID(fieldID int, row int) ([]byte, error) {
	column := los.arr.Field(fieldID)
	return FixedSizeBinaryFromArray(column, row)
}

// StringFieldByName returns the string value of a named field for a specific row.
func (los *ListOfStructs) StringFieldByName(name string, row int) (string, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return "", nil
	}
	column := los.arr.Field(fieldID)
	return StringFromArray(column, row)
}

// U32FieldByName returns the uint32 value of a named field for a specific row.
func (los *ListOfStructs) U32FieldByName(name string, row int) (uint32, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return U32FromArray(column, row)
}

// U64FieldByName returns the uint64 value of a named field for a specific row.
func (los *ListOfStructs) U64FieldByName(name string, row int) (uint64, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return U64FromArray(column, row)
}

// I32FieldByName returns the int32 value of a named field for a specific row.
func (los *ListOfStructs) I32FieldByName(name string, row int) (int32, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return I32FromArray(column, row)
}

// I64FieldByName returns the int64 value of a named field for a specific row.
func (los *ListOfStructs) I64FieldByName(name string, row int) (int64, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return I64FromArray(column, row)
}

// F64FieldByName returns the float64 value of a named field for a specific row.
func (los *ListOfStructs) F64FieldByName(name string, row int) (float64, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0.0, nil
	}
	column := los.arr.Field(fieldID)
	return F64FromArray(column, row)
}

// BoolFieldByName returns the bool value of a named field for a specific row.
func (los *ListOfStructs) BoolFieldByName(name string, row int) (bool, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return false, nil
	}
	column := los.arr.Field(fieldID)
	return BoolFromArray(column, row)
}

// BinaryFieldByName returns the binary value of a named field for a specific row.
func (los *ListOfStructs) BinaryFieldByName(name string, row int) ([]byte, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	column := los.arr.Field(fieldID)
	return BinaryFromArray(column, row)
}

// FixedSizeBinaryFieldByName returns the fixed size binary value of a named field for a specific row.
func (los *ListOfStructs) FixedSizeBinaryFieldByName(name string, row int) ([]byte, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	column := los.arr.Field(fieldID)
	return FixedSizeBinaryFromArray(column, row)
}

// StructArray returns the underlying arrow array for a named field for a specific row.
func (los *ListOfStructs) StructArray(name string, row int) (*arrow.StructType, *array.Struct, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil, nil
	}
	column := los.arr.Field(fieldID)

	switch structArr := column.(type) {
	case *array.Struct:
		if structArr.IsNull(row) {
			return nil, nil, nil
		}
		return structArr.DataType().(*arrow.StructType), structArr, nil
	default:
		return nil, nil, fmt.Errorf("field %q is not a struct", name)
	}
}

// StructByID returns the underlying arrow struct stype and arrow array for a field id for a specific row.
func (los *ListOfStructs) StructByID(fieldID int, row int) (*arrow.StructType, *array.Struct, error) {
	column := los.arr.Field(fieldID)
	switch structArr := column.(type) {
	case *array.Struct:
		if structArr.IsNull(row) {
			return nil, nil, nil
		}
		return structArr.DataType().(*arrow.StructType), structArr, nil
	default:
		return nil, nil, fmt.Errorf("field id %d is not a struct", fieldID)
	}
}

// IsNull returns true if the row is null.
func (los *ListOfStructs) IsNull(row int) bool {
	return los.arr.IsNull(row)
}

// ListValuesById return the list array for a field id for a specific row.
func (los *ListOfStructs) ListValuesById(row int, fieldID int) (arr arrow.Array, start int, end int, err error) {
	column := los.arr.Field(fieldID)
	switch listArr := column.(type) {
	case *array.List:
		if listArr.IsNull(row) {
			return nil, 0, 0, nil
		}
		start = int(listArr.Offsets()[row])
		end = int(listArr.Offsets()[row+1])
		arr = listArr.ListValues()
	default:
		err = fmt.Errorf("field id %d is not a list", fieldID)
	}
	return
}

// ListOfStructsById returns the list of structs for a field id for a specific row.
func (los *ListOfStructs) ListOfStructsById(row int, fieldID int) (*ListOfStructs, error) {
	column := los.arr.Field(fieldID)
	switch listArr := column.(type) {
	case *array.List:
		if listArr.IsNull(row) {
			return nil, nil
		}

		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt, ok := structArr.DataType().(*arrow.StructType)
			if !ok {
				return nil, fmt.Errorf("field id %d is not a list of struct", fieldID)
			}
			start := int(listArr.Offsets()[row])
			end := int(listArr.Offsets()[row+1])

			return &ListOfStructs{
				dt:    dt,
				arr:   structArr,
				start: start,
				end:   end,
			}, nil
		default:
			return nil, fmt.Errorf("field id %d is not a list of structs", fieldID)
		}
	default:
		return nil, fmt.Errorf("field id %d is not a list", fieldID)
	}
}

// DataType returns the underlying arrow struct type.
func (los *ListOfStructs) DataType() *arrow.StructType {
	return los.dt
}

// Array returns the underlying arrow array.
func (los *ListOfStructs) Array() *array.Struct {
	return los.arr
}

// BoolFromArray returns the bool value for a specific row in an Arrow array.
func BoolFromArray(arr arrow.Array, row int) (bool, error) {
	if arr == nil {
		return false, nil
	} else {
		switch arr := arr.(type) {
		case *array.Boolean:
			if arr.IsNull(row) {
				return false, nil
			} else {
				return arr.Value(row), nil
			}
		default:
			return false, fmt.Errorf("column is not of type bool")
		}
	}
}

// F64FromArray returns the float64 value for a specific row in an Arrow array.
func F64FromArray(arr arrow.Array, row int) (float64, error) {
	if arr == nil {
		return 0.0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Float64:
			if arr.IsNull(row) {
				return 0.0, nil
			} else {
				return arr.Value(row), nil
			}
		default:
			return 0.0, fmt.Errorf("column is not of type f64")
		}
	}
}

// F64OrNilFromArray returns a pointer to the float64 value for a specific row in an Arrow array or nil if the value is nil.
func F64OrNilFromArray(arr arrow.Array, row int) (*float64, error) {
	if arr == nil {
		return nil, nil
	} else {
		switch arr := arr.(type) {
		case *array.Float64:
			if arr.IsNull(row) {
				return nil, nil
			} else {
				v := arr.Value(row)
				return &v, nil
			}
		default:
			return nil, fmt.Errorf("column is not of type f64")
		}
	}
}

// U64FromArray returns the uint64 value for a specific row in an Arrow array.
func U64FromArray(arr arrow.Array, row int) (uint64, error) {
	if arr == nil {
		return 0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Uint64:
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return arr.Value(row), nil
			}
		default:
			return 0, fmt.Errorf("column is not of type uint64")
		}
	}
}

// TimestampFromArray returns the timestamp value for a specific row in an Arrow array.
func TimestampFromArray(arr arrow.Array, row int) (arrow.Timestamp, error) {
	if arr == nil {
		return 0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Timestamp:
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return arr.Value(row), nil
			}
		default:
			return 0, fmt.Errorf("column is not of type timestamp")
		}
	}
}

// U32FromArray returns the uint32 value for a specific row in an Arrow array.
func U32FromArray(arr arrow.Array, row int) (uint32, error) {
	if arr == nil {
		return 0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Uint32:
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return arr.Value(row), nil
			}
		default:
			return 0, fmt.Errorf("column is not of type uint32")
		}
	}
}

// U32FromStruct returns the uint32 value for a specific row in an Arrow struct.
func U32FromStruct(structArr *array.Struct, row int, fieldID int) (uint32, error) {
	return U32FromArray(structArr.Field(fieldID), row)
}

// I32FromArray returns the int32 value for a specific row in an Arrow array.
func I32FromArray(arr arrow.Array, row int) (int32, error) {
	if arr == nil {
		return 0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Int32:
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return arr.Value(row), nil
			}
		case *array.Dictionary:
			i32Arr := arr.Dictionary().(*array.Int32)
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return i32Arr.Value(arr.GetValueIndex(row)), nil
			}
		default:
			return 0, fmt.Errorf("column is not of type int32")
		}
	}
}

// I64FromArray returns the int64 value for a specific row in an Arrow array.
func I64FromArray(arr arrow.Array, row int) (int64, error) {
	if arr == nil {
		return 0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Int64:
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return arr.Value(row), nil
			}
		default:
			return 0, fmt.Errorf("column is not of type int64")
		}
	}
}

// StringFromArray returns the string value for a specific row in an Arrow array.
func StringFromArray(arr arrow.Array, row int) (string, error) {
	if arr == nil {
		return "", nil
	} else {
		if arr.IsNull(row) {
			return "", nil
		}

		switch arr := arr.(type) {
		case *array.String:
			return arr.Value(row), nil
		case *array.Dictionary:
			return arr.Dictionary().(*array.String).Value(arr.GetValueIndex(row)), nil
		default:
			return "", fmt.Errorf("column is not of type string")
		}
	}
}

// StringFromStruct returns the string value for a specific row in an Arrow struct.
func StringFromStruct(arr arrow.Array, row int, id int) (string, error) {
	structArr, ok := arr.(*array.Struct)
	if !ok {
		return "", fmt.Errorf("array id %d is not of type struct", id)
	}
	if structArr != nil {
		return StringFromArray(structArr.Field(id), row)
	} else {
		return "", fmt.Errorf("column array is not of type struct")
	}
}

// I32FromStruct returns the int32 value for a specific row in an Arrow struct.
func I32FromStruct(arr arrow.Array, row int, id int) (int32, error) {
	structArr, ok := arr.(*array.Struct)
	if !ok {
		return 0, fmt.Errorf("array id %d is not of type struct", id)
	}
	if structArr != nil {
		return I32FromArray(structArr.Field(id), row)
	} else {
		return 0, fmt.Errorf("column array is not of type struct")
	}
}

// BinaryFromArray returns the binary value for a specific row in an Arrow array.
func BinaryFromArray(arr arrow.Array, row int) ([]byte, error) {
	if arr == nil {
		return nil, nil
	} else {
		if arr.IsNull(row) {
			return nil, nil
		}

		switch arr := arr.(type) {
		case *array.Binary:
			return arr.Value(row), nil
		case *array.Dictionary:
			return arr.Dictionary().(*array.Binary).Value(arr.GetValueIndex(row)), nil
		default:
			return nil, fmt.Errorf("column is not of type binary")
		}
	}
}

// FixedSizeBinaryFromArray returns the fixed size binary value for a specific row in an Arrow array.
func FixedSizeBinaryFromArray(arr arrow.Array, row int) ([]byte, error) {
	if arr == nil {
		return nil, nil
	} else {
		if arr.IsNull(row) {
			return nil, nil
		}

		switch arr := arr.(type) {
		case *array.FixedSizeBinary:
			return arr.Value(row), nil
		case *array.Dictionary:
			return arr.Dictionary().(*array.FixedSizeBinary).Value(arr.GetValueIndex(row)), nil
		default:
			return nil, fmt.Errorf("column is not of type binary")
		}
	}
}
