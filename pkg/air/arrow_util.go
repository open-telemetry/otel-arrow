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

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"

	"otel-arrow-adapter/pkg/air/common"
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

func FieldArray(record arrow.Record, column string) (*arrow.Field, arrow.Array) {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil, nil
	}
	if len(fieldIdsWithSameName) != 1 {
		panic(fmt.Sprintf("duplicate field with name %q", column))
	}
	field := record.Schema().Field(fieldIdsWithSameName[0])
	return &field, record.Column(fieldIdsWithSameName[0])
}

func Array(record arrow.Record, column string) arrow.Array {
	fieldIdsWithSameName := record.Schema().FieldIndices(column)
	if fieldIdsWithSameName == nil {
		return nil
	}
	if len(fieldIdsWithSameName) != 1 {
		panic(fmt.Sprintf("duplicate field with name %q", column))
	}
	return record.Column(fieldIdsWithSameName[0])
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

func BoolFromRecord(record arrow.Record, row int, column string) (bool, error) {
	return BoolFromArray(Array(record, column), row)
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

func F64FromRecord(record arrow.Record, row int, column string) (float64, error) {
	return F64FromArray(Array(record, column), row)
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
	return U64FromArray(Array(record, column), row)
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
	return U32FromArray(Array(record, column), row)
}

func U32FromStruct(field *arrow.Field, arr arrow.Array, row int, column string) (uint32, error) {
	if dt := field.Type.(*arrow.StructType); dt != nil {
		if structArr := arr.(*array.Struct); structArr != nil {
			_, id, found := FieldOfStruct(dt, column)
			if !found {
				return 0, nil
			}
			return U32FromArray(structArr.Field(id), row)
		} else {
			return 0, fmt.Errorf("column array is not of type struct")
		}
	} else {
		return 0, fmt.Errorf("field is not of type struct")
	}
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
	return I32FromArray(Array(record, column), row)
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

func I64FromRecord(record arrow.Record, row int, column string) (int64, error) {
	return I64FromArray(Array(record, column), row)
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

func StringFromRecord(record arrow.Record, row int, column string) (string, error) {
	return StringFromArray(Array(record, column), row)
}

func StringFromStruct(field *arrow.Field, arr arrow.Array, row int, column string) (string, error) {
	if dt := field.Type.(*arrow.StructType); dt != nil {
		if structArr := arr.(*array.Struct); structArr != nil {
			_, id, found := FieldOfStruct(dt, column)
			if !found {
				return "", nil
			}
			return StringFromArray(structArr.Field(id), row)
		} else {
			return "", fmt.Errorf("column array is not of type struct")
		}
	} else {
		return "", fmt.Errorf("field is not of type struct")
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
	return BinaryFromArray(Array(record, column), row)
}
