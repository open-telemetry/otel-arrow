/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

// Utility functions to extract ids, and values from Struct data type or from
// Arrow arrays.

import (
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"

	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// U32FromStruct returns the uint32 value for a specific row in an Arrow struct
// or 0 if the field doesn't exist.
func U32FromStruct(structArr *array.Struct, row int, fieldID int) (uint32, error) {
	if fieldID == -1 {
		return 0, nil
	}
	return U32FromArray(structArr.Field(fieldID), row)
}

// ListOfStructsFieldIDFromStruct returns the field id of a list of structs
// field from an Arrow struct or -1 if the field is not found.
//
// An error is returned if the field is not a list of structs.
func ListOfStructsFieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.StructType, error) {
	if dt == nil {
		return -1, nil, nil
	}

	id, ok := dt.FieldIdx(fieldName)
	if !ok {
		return -1, nil, nil
	}

	if lt, ok := dt.Field(id).Type.(*arrow.ListType); ok {
		st, ok := lt.ElemField().Type.(*arrow.StructType)
		if !ok {
			return 0, nil, werror.WrapWithContext(ErrNotListOfStructsType, map[string]interface{}{"fieldName": fieldName})
		}
		return id, st, nil
	} else {
		return 0, nil, werror.WrapWithContext(ErrNotListType, map[string]interface{}{"fieldName": fieldName})
	}
}

// FieldIDFromStruct returns the field id of a named field from an Arrow struct
// or -1 for an unknown field.
func FieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.DataType) {
	if dt == nil {
		return -1, nil
	}

	id, found := dt.FieldIdx(fieldName)
	if !found {
		return -1, nil
	}
	field := dt.Field(id)
	return id, &field.Type
}

// StructFieldIDFromStruct returns the field id of a struct field from an Arrow
// struct or -1 for an unknown field.
//
// An error is returned if the field is not a struct.
func StructFieldIDFromStruct(dt *arrow.StructType, fieldName string) (int, *arrow.StructType, error) {
	if dt == nil {
		return -1, nil, nil
	}

	id, found := dt.FieldIdx(fieldName)
	if !found {
		return -1, nil, nil
	}
	if st, ok := dt.Field(id).Type.(*arrow.StructType); ok {
		return id, st, nil
	} else {
		return 0, nil, werror.WrapWithContext(ErrNotStructType, map[string]interface{}{"fieldName": fieldName})
	}
}

// StringFromStruct returns the string value for a specific row in an Arrow struct.
func StringFromStruct(arr arrow.Array, row int, id int) (string, error) {
	if id == -1 {
		return "", nil
	}

	structArr, ok := arr.(*array.Struct)
	if !ok {
		return "", werror.WrapWithContext(ErrNotArrayStruct, map[string]interface{}{"row": row, "id": id})
	}
	if structArr != nil {
		return StringFromArray(structArr.Field(id), row)
	} else {
		return "", werror.WrapWithContext(ErrNotArrayStruct, map[string]interface{}{"row": row, "id": id})
	}
}

// I32FromStruct returns the int32 value for a specific field+row in an Arrow
// Array struct.
func I32FromStruct(arr arrow.Array, row int, id int) (int32, error) {
	if id == -1 {
		return 0, nil
	}
	structArr, ok := arr.(*array.Struct)
	if !ok {
		return 0, werror.WrapWithContext(ErrNotArrayStruct, map[string]interface{}{"row": row, "id": id})
	}
	if structArr != nil {
		return I32FromArray(structArr.Field(id), row)
	} else {
		return 0, werror.WrapWithContext(ErrNotArrayStruct, map[string]interface{}{
			"row": row,
			"id":  id,
		})
	}
}

// OptionalFieldIDFromStruct returns the field id of a named field from an Arrow struct or -1 if the field is unknown.
func OptionalFieldIDFromStruct(dt *arrow.StructType, fieldName string) (id int) {
	if dt == nil {
		id = -1
		return
	}

	id, found := dt.FieldIdx(fieldName)
	if !found {
		id = -1
	}
	return
}

// ListOfStructsFromStruct return a ListOfStructs from a struct field.
func ListOfStructsFromStruct(parent *array.Struct, fieldID int, row int) (*ListOfStructs, error) {
	if fieldID == -1 {
		return nil, nil
	}

	arr := parent.Field(fieldID)
	if listArr, ok := arr.(*array.List); ok {
		if listArr.IsNull(row) {
			return nil, nil
		}

		switch structArr := listArr.ListValues().(type) {
		case *array.Struct:
			dt, ok := structArr.DataType().(*arrow.StructType)
			if !ok {
				return nil, werror.WrapWithContext(ErrNotStructType, map[string]interface{}{"fieldID": fieldID, "row": row})
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
			return nil, werror.WrapWithContext(ErrNotArrayStruct, map[string]interface{}{"fieldID": fieldID, "row": row})
		}
	} else {
		return nil, werror.WrapWithContext(ErrNotArrayList, map[string]interface{}{"fieldID": fieldID, "row": row})
	}
}
