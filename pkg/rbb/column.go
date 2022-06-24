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

package rbb

import "github.com/apache/arrow/go/arrow"

type BoolColumn struct {
	Name string
	Data []*bool
}

type I8Column struct {
	Name string
	Data []*int8
}

type I16Column struct {
	Name string
	Data []*int16
}

type I32Column struct {
	Name string
	Data []*int32
}

type I64Column struct {
	Name string
	Data []*int64
}

type U8Column struct {
	Name string
	Data []*uint8
}

type U16Column struct {
	Name string
	Data []*uint16
}

type U32Column struct {
	Name string
	Data []*uint32
}

type U64Column struct {
	Name string
	Data []*uint64
}

type F32Column struct {
	Name string
	Data []*float32
}

type F64Column struct {
	Name string
	Data []*float64
}

type StringColumn struct {
	Name             string
	config           *DictionaryConfig
	fieldPath        []int
	dictId           int
	dictionary       map[string]bool
	Data             []*string
	totalValueLength int
	totalRowCount    int
}

type BinaryColumn struct {
	Name string
	Data []*[]byte
}

type ListColumn struct {
	Name string
	Type arrow.DataType
	Data [][]Value
}

type StructColumn struct {
	Name    string
	Type    arrow.DataType
	Columns Columns
}

type Columns struct {
	BooleanColumns []BoolColumn

	I8Columns  []I8Column
	I16Columns []I16Column
	I32Columns []I32Column
	I64Columns []I64Column

	U8Columns  []U8Column
	U16Columns []U16Column
	U32Columns []U32Column
	U64Columns []U64Column

	F32Columns []F32Column
	F64Columns []F64Column

	StringColumns []StringColumn
	BinaryColumns []BinaryColumn

	ListColumns   []ListColumn
	StructColumns []StructColumn
}

// Create a column with a field based on its field type and field name.
func (c *Columns) CreateColumn(path []int, field *Field, config *Config, dictIdGen *DictIdGenerator) *FieldPath {
	switch field.Value.(type) {
	case *Bool:
		c.BooleanColumns = append(c.BooleanColumns, BoolColumn{
			Name: field.Name,
			Data: []*bool{&field.Value.(*Bool).Value},
		})
		return NewFieldPath(len(c.BooleanColumns) - 1)
	case *I8:
		c.I8Columns = append(c.I8Columns, I8Column{
			Name: field.Name,
			Data: []*int8{&field.Value.(*I8).Value},
		})
		return NewFieldPath(len(c.I8Columns) - 1)
	case *I16:
		c.I16Columns = append(c.I16Columns, I16Column{
			Name: field.Name,
			Data: []*int16{&field.Value.(*I16).Value},
		})
		return NewFieldPath(len(c.I16Columns) - 1)
	case *I32:
		c.I32Columns = append(c.I32Columns, I32Column{
			Name: field.Name,
			Data: []*int32{&field.Value.(*I32).Value},
		})
		return NewFieldPath(len(c.I32Columns) - 1)
	case *I64:
		c.I64Columns = append(c.I64Columns, I64Column{
			Name: field.Name,
			Data: []*int64{&field.Value.(*I64).Value},
		})
		return NewFieldPath(len(c.I64Columns) - 1)
	case *U8:
		c.U8Columns = append(c.U8Columns, U8Column{
			Name: field.Name,
			Data: []*uint8{&field.Value.(*U8).Value},
		})
		return NewFieldPath(len(c.U8Columns) - 1)
	case *U16:
		c.U16Columns = append(c.U16Columns, U16Column{
			Name: field.Name,
			Data: []*uint16{&field.Value.(*U16).Value},
		})
		return NewFieldPath(len(c.U16Columns) - 1)
	case *U32:
		c.U32Columns = append(c.U32Columns, U32Column{
			Name: field.Name,
			Data: []*uint32{&field.Value.(*U32).Value},
		})
		return NewFieldPath(len(c.U32Columns) - 1)
	case *U64:
		c.U64Columns = append(c.U64Columns, U64Column{
			Name: field.Name,
			Data: []*uint64{&field.Value.(*U64).Value},
		})
		return NewFieldPath(len(c.U64Columns) - 1)
	case *F32:
		c.F32Columns = append(c.F32Columns, F32Column{
			Name: field.Name,
			Data: []*float32{&field.Value.(*F32).Value},
		})
		return NewFieldPath(len(c.F32Columns) - 1)
	case *F64:
		c.F64Columns = append(c.F64Columns, F64Column{
			Name: field.Name,
			Data: []*float64{&field.Value.(*F64).Value},
		})
		return NewFieldPath(len(c.F64Columns) - 1)
	case *String:
		c.StringColumns = append(c.StringColumns, StringColumn{
			Name:             field.Name,
			config:           &config.Dictionaries.StringColumns,
			fieldPath:        path,
			dictId:           dictIdGen.NextId(),
			Data:             []*string{&field.Value.(*String).Value},
			totalValueLength: 0,
			totalRowCount:    0,
		})
		return NewFieldPath(len(c.StringColumns) - 1)
	case *Binary:
		c.BinaryColumns = append(c.BinaryColumns, BinaryColumn{
			Name: field.Name,
			Data: []*[]byte{&field.Value.(*Binary).Value},
		})
		return NewFieldPath(len(c.BinaryColumns) - 1)
	case *List:
		dataType := ListDataType(field.Value.(*List).values)
		c.ListColumns = append(c.ListColumns, ListColumn{
			Name: field.Name,
			Type: dataType,
			Data: [][]Value{field.Value.(*List).values},
		})
		return NewFieldPath(len(c.ListColumns) - 1)
	case *Struct:
		dataType := StructDataType(field.Value.(*Struct).fields)
		fieldPaths := make([]*FieldPath, 0, len(field.Value.(*Struct).fields))
		columns := Columns{}
		for _, field := range field.Value.(*Struct).fields {
			updatedPath := make([]int, 0, len(path)+1)
			copy(updatedPath, path)
			updatedPath = append(updatedPath, len(fieldPaths))
			fieldPath := columns.CreateColumn(updatedPath, &field, config, dictIdGen)
			if fieldPath != nil {
				fieldPaths = append(fieldPaths, fieldPath)
			}
		}
		if !columns.IsEmpty() {
			c.StructColumns = append(c.StructColumns, StructColumn{
				Name:    field.Name,
				Type:    dataType,
				Columns: columns,
			})
			return NewFieldPathWithChildren(len(c.StructColumns)-1, fieldPaths)
		} else {
			return nil
		}
	default:
		panic("unsupported field type")
	}
}

func (c *Columns) IsEmpty() bool {
	return len(c.I8Columns) == 0 && len(c.I16Columns) == 0 && len(c.I32Columns) == 0 && len(c.I64Columns) == 0 && len(c.U8Columns) == 0 && len(c.U16Columns) == 0 && len(c.U32Columns) == 0 && len(c.U64Columns) == 0 && len(c.F32Columns) == 0 && len(c.F64Columns) == 0 && len(c.BooleanColumns) == 0 && len(c.StringColumns) == 0 && len(c.BinaryColumns) == 0 && len(c.ListColumns) == 0 && len(c.StructColumns) == 0
}
