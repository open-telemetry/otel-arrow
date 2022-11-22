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

package arrow

import (
	"fmt"
	"sort"
	"strings"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
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

func FieldToID(field *arrow.Field) string {
	return field.Name + ":" + DataTypeToID(field.Type)
}

func DataTypeToID(dt arrow.DataType) string {
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

func FieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.DataType, error) {
	id, found := dt.FieldIdx(fieldName)
	if !found {
		return 0, nil, fmt.Errorf("no field %q in struct type", fieldName)
	}
	field := dt.Field(id)
	return id, &field.Type, nil
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
	dt, ok := field.Type.(*arrow.StructType)
	if !ok {
		return nil, nil, fmt.Errorf("column %q is not a struct", column)
	}
	if dt != nil {
		return dt, record.Column(fieldIdsWithSameName[0]).(*array.Struct), nil
	} else {
		return nil, nil, fmt.Errorf("column %q is not a struct", column)
	}
}

type ListOfStructs struct {
	dt    *arrow.StructType
	arr   *array.Struct
	start int
	end   int
}

// TODO remove bis once the other implementation is no longer used

// ListOfStructsFromRecordBis returns the struct type and an array of structs for a given field id.
func ListOfStructsFromRecordBis(record arrow.Record, fieldID int, row int) (*ListOfStructs, error) {
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

func (los *ListOfStructs) Start() int {
	return los.start
}

func (los *ListOfStructs) End() int {
	return los.end
}

func (los *ListOfStructs) FieldIdx(name string) (int, bool) {
	return los.dt.FieldIdx(name)
}

func (los *ListOfStructs) Field(name string) (arrow.Array, bool) {
	id, ok := los.dt.FieldIdx(name)
	if !ok {
		return nil, false
	}
	return los.arr.Field(id), true
}

func (los *ListOfStructs) FieldByID(id int) arrow.Array {
	return los.arr.Field(id)
}

func (los *ListOfStructs) StringFieldByID(fieldID int, row int) (string, error) {
	column := los.arr.Field(fieldID)
	return StringFromArray(column, row)
}

func (los *ListOfStructs) U32FieldByID(fieldID int, row int) (uint32, error) {
	column := los.arr.Field(fieldID)
	return U32FromArray(column, row)
}

func (los *ListOfStructs) U64FieldByID(fieldID int, row int) (uint64, error) {
	column := los.arr.Field(fieldID)
	return U64FromArray(column, row)
}

func (los *ListOfStructs) I32FieldByID(fieldID int, row int) (int32, error) {
	column := los.arr.Field(fieldID)
	return I32FromArray(column, row)
}

func (los *ListOfStructs) I64FieldByID(fieldID int, row int) (int64, error) {
	column := los.arr.Field(fieldID)
	return I64FromArray(column, row)
}

func (los *ListOfStructs) F64FieldByID(fieldID int, row int) (float64, error) {
	column := los.arr.Field(fieldID)
	return F64FromArray(column, row)
}

func (los *ListOfStructs) F64OrNilFieldByID(fieldID int, row int) (*float64, error) {
	column := los.arr.Field(fieldID)
	return F64OrNilFromArray(column, row)
}

func (los *ListOfStructs) BoolFieldByID(fieldID int, row int) (bool, error) {
	column := los.arr.Field(fieldID)
	return BoolFromArray(column, row)
}

func (los *ListOfStructs) BinaryFieldByID(fieldID int, row int) ([]byte, error) {
	column := los.arr.Field(fieldID)
	return BinaryFromArray(column, row)
}

func (los *ListOfStructs) FixedSizeBinaryFieldByID(fieldID int, row int) ([]byte, error) {
	column := los.arr.Field(fieldID)
	return FixedSizeBinaryFromArray(column, row)
}

func (los *ListOfStructs) StringFieldByName(name string, row int) (string, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return "", nil
	}
	column := los.arr.Field(fieldID)
	return StringFromArray(column, row)
}

func (los *ListOfStructs) U32FieldByName(name string, row int) (uint32, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return U32FromArray(column, row)
}

func (los *ListOfStructs) U64FieldByName(name string, row int) (uint64, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return U64FromArray(column, row)
}

func (los *ListOfStructs) I32FieldByName(name string, row int) (int32, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return I32FromArray(column, row)
}

func (los *ListOfStructs) I64FieldByName(name string, row int) (int64, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0, nil
	}
	column := los.arr.Field(fieldID)
	return I64FromArray(column, row)
}

func (los *ListOfStructs) F64FieldByName(name string, row int) (float64, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return 0.0, nil
	}
	column := los.arr.Field(fieldID)
	return F64FromArray(column, row)
}

func (los *ListOfStructs) BoolFieldByName(name string, row int) (bool, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return false, nil
	}
	column := los.arr.Field(fieldID)
	return BoolFromArray(column, row)
}

func (los *ListOfStructs) BinaryFieldByName(name string, row int) ([]byte, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	column := los.arr.Field(fieldID)
	return BinaryFromArray(column, row)
}

func (los *ListOfStructs) FixedSizeBinaryFieldByName(name string, row int) ([]byte, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	column := los.arr.Field(fieldID)
	return FixedSizeBinaryFromArray(column, row)
}

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

// TODO remove this function
func (los *ListOfStructs) OldListOfStructsById(row int, fieldID int, fieldName string) (*ListOfStructs, error) {
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
				return nil, fmt.Errorf("field %q is not a list of struct", fieldName)
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
			return nil, fmt.Errorf("field %q is not a list of structs", fieldName)
		}
	default:
		return nil, fmt.Errorf("field %q is not a list", fieldName)
	}
}

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

func (los *ListOfStructs) ListOfStructsByName(name string, row int) (*ListOfStructs, error) {
	fieldID, found := los.dt.FieldIdx(name)
	if !found {
		return nil, nil
	}
	return los.OldListOfStructsById(row, fieldID, name)
}

func (los *ListOfStructs) DataType() *arrow.StructType {
	return los.dt
}

func (los *ListOfStructs) Array() *array.Struct {
	return los.arr
}

func FieldArrayOfStruct(fieldType *arrow.StructType, arr arrow.Array, column string) (*arrow.Field, arrow.Array, error) {
	structArr, ok := arr.(*array.Struct)
	if !ok {
		return nil, nil, fmt.Errorf("array %q is not a struct", column)
	}
	if structArr != nil {
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

// TODO remove this function
func U32FromStructOld(fieldType *arrow.StructType, structArr *array.Struct, row int, column string) (uint32, error) {
	_, id, found := FieldOfStruct(fieldType, column)
	if !found {
		return 0, nil
	}
	return U32FromArray(structArr.Field(id), row)
}

func U32FromStruct(structArr *array.Struct, row int, fieldID int) (uint32, error) {
	return U32FromArray(structArr.Field(fieldID), row)
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

// TODO remove this function
func OldStringFromStruct(fieldType *arrow.StructType, arr arrow.Array, row int, column string) (string, error) {
	structArr, ok := arr.(*array.Struct)
	if !ok {
		return "", fmt.Errorf("array %q is not of type struct", column)
	}
	if structArr != nil {
		_, id, found := FieldOfStruct(fieldType, column)
		if !found {
			return "", nil
		}
		return StringFromArray(structArr.Field(id), row)
	} else {
		return "", fmt.Errorf("column array is not of type struct")
	}
}

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
