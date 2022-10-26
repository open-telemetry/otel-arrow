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

package air

import (
	"fmt"
	"sort"
	"strings"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/air/common"
)

type SortableField struct {
	name  *string
	field *arrow.Field
}

type Fields []SortableField

// Sort interface
func (d Fields) Less(i, j int) bool {
	return *d[i].name < *d[j].name
}
func (d Fields) Len() int      { return len(d) }
func (d Fields) Swap(i, j int) { d[i], d[j] = d[j], d[i] }

func SchemaToId(schema *arrow.Schema) string {
	schemaId := ""
	fields := sortedFields(schema.Fields())
	for i := range fields {
		field := &fields[i]
		if i != 0 {
			schemaId += ","
		}
		schemaId += FieldToId(field.field)
	}
	return schemaId
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

func FieldToId(field *arrow.Field) string {
	return field.Name + ":" + DataTypeToId(field.Type)
}

func DataTypeToId(dt arrow.DataType) string {
	id := ""
	switch t := dt.(type) {
	case *arrow.BooleanType:
		id += common.BOOL_SIG
	case *arrow.Int8Type:
		id += common.I8_SIG
	case *arrow.Int16Type:
		id += common.I16_SIG
	case *arrow.Int32Type:
		id += common.I32_SIG
	case *arrow.Int64Type:
		id += common.I64_SIG
	case *arrow.Uint8Type:
		id += common.U8_SIG
	case *arrow.Uint16Type:
		id += common.U16_SIG
	case *arrow.Uint32Type:
		id += common.U32_SIG
	case *arrow.Uint64Type:
		id += common.U64_SIG
	case *arrow.Float32Type:
		id += common.F32_SIG
	case *arrow.Float64Type:
		id += common.F64_SIG
	case *arrow.StringType:
		id += common.STRING_SIG
	case *arrow.BinaryType:
		id += common.BINARY_SIG
	case *arrow.StructType:
		id += "{"
		fields := sortedFields(t.Fields())
		for i := range fields {
			if i > 0 {
				id += ","
			}
			id += FieldToId(fields[i].field)
		}
		id += "}"
	case *arrow.ListType:
		id += "["
		elemField := t.ElemField()
		id += DataTypeToId(elemField.Type)
		id += "]"
	case *arrow.DictionaryType:
		id += "Dic<"
		id += DataTypeToId(t.IndexType)
		id += ","
		id += DataTypeToId(t.ValueType)
		id += ">"
	default:
		panic("unsupported data type")
	}

	return id
}

func FieldArray(record arrow.Record, column string) (*arrow.Field, arrow.Array, error) {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil, nil, fmt.Errorf("column %q not found", column)
	}
	if len(fieldIdsWithSameName) != 1 {
		return nil, nil, fmt.Errorf("column %q is ambiguous (multiple columns with the same name)", column)
	}
	field := record.Schema().Field(fieldIdsWithSameName[0])
	return &field, record.Column(fieldIdsWithSameName[0]), nil
}

func OptionalFieldArray(record arrow.Record, column string) (*arrow.Field, arrow.Array, error) {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil, nil, nil
	}
	if len(fieldIdsWithSameName) != 1 {
		return nil, nil, fmt.Errorf("column %q is ambiguous (multiple columns with the same name)", column)
	}
	field := record.Schema().Field(fieldIdsWithSameName[0])
	return &field, record.Column(fieldIdsWithSameName[0]), nil
}

func StructFromRecord(record arrow.Record, column string) (*arrow.StructType, *array.Struct, error) {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil, nil, fmt.Errorf("column %q not found", column)
	}
	if len(fieldIdsWithSameName) != 1 {
		return nil, nil, fmt.Errorf("column %q is ambiguous (multiple columns with the same name)", column)
	}
	field := record.Schema().Field(fieldIdsWithSameName[0])
	if dt := field.Type.(*arrow.StructType); dt != nil {
		return dt, record.Column(fieldIdsWithSameName[0]).(*array.Struct), nil
	} else {
		return nil, nil, fmt.Errorf("column %q is not a struct", column)
	}
}

//func StructFromStruct(fieldType *arrow.StructType, fieldArr arrow.Array, column string) (*arrow.StructType, arrow.Array, error) {
//	fieldIdx, ok := fieldType.FieldIdx(column)
//	if !ok {
//		return nil, nil, fmt.Errorf("column %q not found", column)
//	}
//	fieldArr.
//	field := record.Schema().Field(fieldIdsWithSameName[0])
//	if dt := field.Type.(*arrow.StructType); dt != nil {
//		return dt, record.Column(fieldIdsWithSameName[0]), nil
//	} else {
//		return nil, nil, fmt.Errorf("column %q is not a struct", column)
//	}
//}

type ListOfStructs struct {
	dt    *arrow.StructType
	arr   *array.Struct
	start int
	end   int
}

// ListOfStructsFromRecord returns the struct type and an array of structs for a given field name.
func ListOfStructsFromRecord(record arrow.Record, field string, row int) (*ListOfStructs, error) {
	arr, err := Array(record, field)
	if err != nil {
		return nil, err
	}
	switch listArr := arr.(type) {
	case *array.List:
		if listArr.IsNull(row) {
			return nil, nil
		}
		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt := structArr.DataType().(*arrow.StructType)
			start := int(listArr.Offsets()[row])
			end := int(listArr.Offsets()[row+1])

			return &ListOfStructs{
				dt:    dt,
				arr:   structArr,
				start: start,
				end:   end,
			}, nil
		default:
			return nil, fmt.Errorf("field %q is not a list of structs", field)
		}
	default:
		return nil, fmt.Errorf("field %q is not a list", field)
	}
}

func (los *ListOfStructs) Start() int {
	return los.start
}

func (los *ListOfStructs) End() int {
	return los.end
}

func (los *ListOfStructs) FieldIdx(name string) (int, bool) {
	return los.dt.FieldIdx(name)
}

func (los *ListOfStructs) StringFieldById(fieldId int, row int) (string, error) {
	column := los.arr.Field(fieldId)
	return StringFromArray(column, row)
}

func (los *ListOfStructs) U32FieldById(fieldId int, row int) (uint32, error) {
	column := los.arr.Field(fieldId)
	return U32FromArray(column, row)
}

func (los *ListOfStructs) U64FieldById(fieldId int, row int) (uint64, error) {
	column := los.arr.Field(fieldId)
	return U64FromArray(column, row)
}

func (los *ListOfStructs) I32FieldById(fieldId int, row int) (int32, error) {
	column := los.arr.Field(fieldId)
	return I32FromArray(column, row)
}

func (los *ListOfStructs) I64FieldById(fieldId int, row int) (int64, error) {
	column := los.arr.Field(fieldId)
	return I64FromArray(column, row)
}

func (los *ListOfStructs) F64FieldById(fieldId int, row int) (float64, error) {
	column := los.arr.Field(fieldId)
	return F64FromArray(column, row)
}

func (los *ListOfStructs) BoolFieldById(fieldId int, row int) (bool, error) {
	column := los.arr.Field(fieldId)
	return BoolFromArray(column, row)
}

func (los *ListOfStructs) BinaryFieldById(fieldId int, row int) ([]byte, error) {
	column := los.arr.Field(fieldId)
	return BinaryFromArray(column, row)
}

func (los *ListOfStructs) StringFieldByName(name string, row int) (string, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return "", nil
	}
	column := los.arr.Field(fieldId)
	return StringFromArray(column, row)
}

func (los *ListOfStructs) U32FieldByName(name string, row int) (uint32, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldId)
	return U32FromArray(column, row)
}

func (los *ListOfStructs) U64FieldByName(name string, row int) (uint64, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldId)
	return U64FromArray(column, row)
}

func (los *ListOfStructs) I32FieldByName(name string, row int) (int32, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldId)
	return I32FromArray(column, row)
}

func (los *ListOfStructs) I64FieldByName(name string, row int) (int64, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldId)
	return I64FromArray(column, row)
}

func (los *ListOfStructs) F64FieldByName(name string, row int) (float64, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return 0.0, nil
	}
	column := los.arr.Field(fieldId)
	return F64FromArray(column, row)
}

func (los *ListOfStructs) BoolFieldByName(name string, row int) (bool, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return false, nil
	}
	column := los.arr.Field(fieldId)
	return BoolFromArray(column, row)
}

func (los *ListOfStructs) BinaryFieldByName(name string, row int) ([]byte, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	column := los.arr.Field(fieldId)
	return BinaryFromArray(column, row)
}

func (los *ListOfStructs) StructArray(name string, row int) (*arrow.StructType, *array.Struct, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil, nil
	}
	column := los.arr.Field(fieldId)
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

func (los *ListOfStructs) IsNull(row int) bool {
	return los.arr.IsNull(row)
}

func (los *ListOfStructs) CopyAttributesFrom(attr pcommon.Map) error {
	attr.EnsureCapacity(los.end - los.start)
	for i := los.start; i < los.end; i++ {
		key, err := los.StringFieldByName("key", i)
		if err != nil {
			return err
		}
		// TODO replace this separator with a constant
		idx := strings.Index(key, "|")
		if idx == -1 {
			return fmt.Errorf("invalid key %q, the signature prefix is missing", key)
		}
		sig := key[:idx]
		key = key[idx+1:]
		// TODO replace field name strings with constants
		switch sig {
		case common.STRING_SIG:
			value, err := los.StringFieldByName("string", i)
			if err != nil {
				return err
			}
			attr.PutStr(key, value)
		case common.BINARY_SIG:
			value, err := los.BinaryFieldByName("binary", i)
			if err != nil {
				return err
			}
			attr.PutEmptyBytes(key).FromRaw(value)
		case common.I64_SIG:
			value, err := los.I64FieldByName("i64", i)
			if err != nil {
				return err
			}
			attr.PutInt(key, value)
		case common.F64_SIG:
			value, err := los.F64FieldByName("f64", i)
			if err != nil {
				return err
			}
			attr.PutDouble(key, value)
		case common.BOOL_SIG:
			value, err := los.BoolFieldByName("bool", i)
			if err != nil {
				return err
			}
			attr.PutBool(key, value)
		}
	}
	return nil
}

func (los *ListOfStructs) ListOfStructsById(row int, fieldId int, fieldName string) (*ListOfStructs, error) {
	column := los.arr.Field(fieldId)
	switch listArr := column.(type) {
	case *array.List:
		if listArr.IsNull(row) {
			return nil, nil
		}
		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt := structArr.DataType().(*arrow.StructType)
			start := int(listArr.Offsets()[row])
			end := int(listArr.Offsets()[row+1])

			return &ListOfStructs{
				dt:    dt,
				arr:   structArr,
				start: start,
				end:   end,
			}, nil
		default:
			return nil, fmt.Errorf("field %q is not a list of structs", fieldName)
		}
	default:
		return nil, fmt.Errorf("field %q is not a list", fieldName)
	}
}

func (los *ListOfStructs) ListOfStructsByName(name string, row int) (*ListOfStructs, error) {
	fieldId, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	return los.ListOfStructsById(row, fieldId, name)
}

func ListOfStructsFromStruct(fieldType *arrow.StructType, structArr *array.Struct, row int, name string) (*ListOfStructs, error) {
	fieldId, found := fieldType.FieldIdx(name)
	if !found {
		return nil, nil
	}
	column := structArr.Field(fieldId)
	switch listArr := column.(type) {
	case *array.List:
		if listArr.IsNull(row) {
			return nil, nil
		}
		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt := structArr.DataType().(*arrow.StructType)
			start := int(listArr.Offsets()[row])
			end := int(listArr.Offsets()[row+1])

			return &ListOfStructs{
				dt:    dt,
				arr:   structArr,
				start: start,
				end:   end,
			}, nil
		default:
			return nil, fmt.Errorf("field %q is not a list of structs", name)
		}
	default:
		return nil, fmt.Errorf("field %q is not a list", name)
	}
}

func FieldArrayOfStruct(fieldType *arrow.StructType, arr arrow.Array, column string) (*arrow.Field, arrow.Array, error) {
	if structArr := arr.(*array.Struct); structArr != nil {
		fieldOfStruct, id, found := FieldOfStruct(fieldType, column)
		if !found {
			return nil, nil, nil
		}
		return fieldOfStruct, structArr.Field(id), nil
	} else {
		return nil, nil, fmt.Errorf("column array is not of type struct")
	}
}

func Array(record arrow.Record, column string) (arrow.Array, error) {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil, fmt.Errorf("column %q not found", column)
	}
	if len(fieldIdsWithSameName) != 1 {
		return nil, fmt.Errorf("column %q is ambiguous (multiple columns with the same name)", column)
	}
	return record.Column(fieldIdsWithSameName[0]), nil
}

func OptionalArray(record arrow.Record, column string) (arrow.Array, error) {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil, nil
	}
	if len(fieldIdsWithSameName) != 1 {
		return nil, fmt.Errorf("column %q is ambiguous (multiple columns with the same name)", column)
	}
	return record.Column(fieldIdsWithSameName[0]), nil
}

func FieldOfStruct(dt *arrow.StructType, column string) (*arrow.Field, int, bool) {
	idx, found := dt.FieldIdx(column)
	if !found {
		return nil, 0, false
	}
	field := dt.Field(idx)
	return &field, idx, true
}

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

//func BoolFromRecord(record arrow.Record, row int, column string) (bool, error) {
//	return BoolFromArray(Array(record, column), row)
//}

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

//func F64FromRecord(record arrow.Record, row int, column string) (float64, error) {
//	return F64FromArray(Array(record, column), row)
//}

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

func U64FromRecord(record arrow.Record, row int, column string) (uint64, error) {
	arr, err := Array(record, column)
	if err != nil {
		return 0, err
	}
	return U64FromArray(arr, row)
}

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

func U32FromRecord(record arrow.Record, row int, column string) (uint32, error) {
	arr, err := Array(record, column)
	if err != nil {
		return 0, err
	}
	return U32FromArray(arr, row)
}

func OptionalU32FromRecord(record arrow.Record, row int, column string) (uint32, error) {
	arr, err := OptionalArray(record, column)
	if err != nil {
		return 0, err
	}
	if arr == nil {
		return 0, nil
	}
	return U32FromArray(arr, row)
}

func U32FromStruct(fieldType *arrow.StructType, structArr *array.Struct, row int, column string) (uint32, error) {
	_, id, found := FieldOfStruct(fieldType, column)
	if !found {
		return 0, nil
	}
	return U32FromArray(structArr.Field(id), row)
}

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
		default:
			return 0, fmt.Errorf("column is not of type int32")
		}
	}
}

func I32FromRecord(record arrow.Record, row int, column string) (int32, error) {
	arr, err := Array(record, column)
	if err != nil {
		return 0, err
	}
	return I32FromArray(arr, row)
}

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

//func I64FromRecord(record arrow.Record, row int, column string) (int64, error) {
//	return I64FromArray(Array(record, column), row)
//}

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

func StringFromRecord(record arrow.Record, row int, column string) (string, error) {
	arr, err := Array(record, column)
	if err != nil {
		return "", err
	}

	return StringFromArray(arr, row)
}

func StringFromStruct(fieldType *arrow.StructType, arr arrow.Array, row int, column string) (string, error) {
	if structArr := arr.(*array.Struct); structArr != nil {
		_, id, found := FieldOfStruct(fieldType, column)
		if !found {
			return "", nil
		}
		return StringFromArray(structArr.Field(id), row)
	} else {
		return "", fmt.Errorf("column array is not of type struct")
	}
}

func I32FromStruct(fieldType *arrow.StructType, arr arrow.Array, row int, column string) (int32, error) {
	if structArr := arr.(*array.Struct); structArr != nil {
		_, id, found := FieldOfStruct(fieldType, column)
		if !found {
			return 0, nil
		}
		return I32FromArray(structArr.Field(id), row)
	} else {
		return 0, fmt.Errorf("column array is not of type struct")
	}
}

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

func BinaryFromRecord(record arrow.Record, row int, column string) ([]byte, error) {
	arr, err := Array(record, column)
	if err != nil {
		return nil, err
	}
	return BinaryFromArray(arr, row)
}
