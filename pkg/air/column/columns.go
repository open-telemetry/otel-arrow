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

package column

import (
	"fmt"
	"io"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/memory"

	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/air/dictionary"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/air/stats"
)

// Column is a generic interface to interact with all types of column.
type Column interface {
	// Name returns the name of the column.
	Name() string
	// Type returns the type of the column.
	Type() arrow.DataType
	// Len returns the number of elements in the column.
	Len() int
	// Clear resets the column to its initial state.
	Clear()
	// PushFromValues adds the given values to the column.
	PushFromValues(fieldPath *rfield.FieldPath, data []rfield.Value)
	// NewArrowField returns an Arrow field for the column.
	NewArrowField() *arrow.Field
	// NewArray returns a new array for the column.
	NewArray(allocator *memory.GoAllocator) arrow.Array
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
	StructColumns []*StructColumn

	length int
}

type DictionaryMetadata struct {
	Card       int
	AvgLen     float64
	TotalEntry int
}

type ColumnMetadata struct {
	Field      *arrow.Field
	Len        int
	Children   map[string]*ColumnMetadata
	Dictionary *DictionaryMetadata
}

func (m ColumnMetadata) Dump(prefix string, f io.Writer) {
	_, err := fmt.Fprintf(f, "%s- %s(%s)", prefix, m.Field.Name, m.Field.Type)
	if err != nil {
		panic(err)
	}
	if m.Dictionary != nil {
		if m.Dictionary.Card > 0 {
			_, err := fmt.Fprintf(f, " dictionary-card=%d, entry-avg-len=%f, total-entry=%d", m.Dictionary.Card, m.Dictionary.AvgLen, m.Dictionary.TotalEntry)
			if err != nil {
				panic(err)
			}
		} else {
			_, err := fmt.Fprintf(f, " entry-avg-len=%f, total-entry=%d", m.Dictionary.AvgLen, m.Dictionary.TotalEntry)
			if err != nil {
				panic(err)
			}
		}
	}
	_, err = fmt.Fprintf(f, "\n")
	if err != nil {
		panic(err)
	}
	for _, child := range m.Children {
		child.Dump(prefix+"\t", f)
	}
}

func NewColumns(allocator *memory.GoAllocator, structType *arrow.StructType, fieldPath []int, config *config.Config, dictIdGen *dictionary.DictIdGenerator) (*Columns, []*rfield.FieldPath) {
	subFields := structType.Fields()
	fieldPaths := make([]*rfield.FieldPath, 0, len(subFields))
	columns := Columns{}
	for i := range subFields {
		subFieldPath := make([]int, len(fieldPath), len(fieldPath)+1)
		copy(subFieldPath, fieldPath)
		subFieldPath = append(subFieldPath, len(fieldPaths))
		colFieldPath := columns.CreateColumn(allocator, subFieldPath, subFields[i].Name, subFields[i].Type, config, dictIdGen)
		if colFieldPath != nil {
			fieldPaths = append(fieldPaths, colFieldPath)
		}
	}
	return &columns, fieldPaths
}

// CreateColumn creates a column with a field based on its field type and field name.
func (c *Columns) CreateColumn(allocator *memory.GoAllocator, path []int, fieldName string, fieldType arrow.DataType, config *config.Config, dictIdGen *dictionary.DictIdGenerator) *rfield.FieldPath {
	switch t := fieldType.(type) {
	case *arrow.BooleanType:
		c.BooleanColumns = append(c.BooleanColumns, MakeBoolColumn(allocator, fieldName))
		return rfield.NewFieldPath(len(c.BooleanColumns) - 1)
	case *arrow.Int8Type:
		c.I8Columns = append(c.I8Columns, MakeI8Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.I8Columns) - 1)
	case *arrow.Int16Type:
		c.I16Columns = append(c.I16Columns, MakeI16Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.I16Columns) - 1)
	case *arrow.Int32Type:
		c.I32Columns = append(c.I32Columns, MakeI32Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.I32Columns) - 1)
	case *arrow.Int64Type:
		c.I64Columns = append(c.I64Columns, MakeI64Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.I64Columns) - 1)
	case *arrow.Uint8Type:
		c.U8Columns = append(c.U8Columns, MakeU8Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.U8Columns) - 1)
	case *arrow.Uint16Type:
		c.U16Columns = append(c.U16Columns, MakeU16Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.U16Columns) - 1)
	case *arrow.Uint32Type:
		c.U32Columns = append(c.U32Columns, MakeU32Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.U32Columns) - 1)
	case *arrow.Uint64Type:
		c.U64Columns = append(c.U64Columns, MakeU64Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.U64Columns) - 1)
	case *arrow.Float32Type:
		c.F32Columns = append(c.F32Columns, MakeF32Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.F32Columns) - 1)
	case *arrow.Float64Type:
		c.F64Columns = append(c.F64Columns, MakeF64Column(allocator, fieldName))
		return rfield.NewFieldPath(len(c.F64Columns) - 1)
	case *arrow.StringType:
		stringColumn := NewStringColumn(allocator, fieldName, &config.Dictionaries.StringColumns, path, dictIdGen.NextId())
		c.StringColumns = append(c.StringColumns, *stringColumn)
		return rfield.NewFieldPath(len(c.StringColumns) - 1)
	case *arrow.BinaryType:
		c.BinaryColumns = append(c.BinaryColumns, MakeBinaryColumn(allocator, fieldName, &config.Dictionaries.BinaryColumns, path, dictIdGen.NextId()))
		return rfield.NewFieldPath(len(c.BinaryColumns) - 1)
	case *arrow.ListType:
		etype := t.Elem()
		// Dictionary are not yet supported for list columns (arrow implementation limitation).
		listColumn, fieldPaths := MakeListColumn(allocator, fieldName, path, etype, config.ConfigWithoutDictionarySupport(), dictIdGen)
		c.ListColumns = append(c.ListColumns, listColumn)
		if fieldPaths == nil {
			return rfield.NewFieldPath(len(c.ListColumns) - 1)
		} else {
			return rfield.NewFieldPathWithChildren(len(c.ListColumns)-1, fieldPaths)
		}
	case *arrow.StructType:
		columns, fieldPaths := NewColumns(allocator, t, path, config, dictIdGen)
		if !columns.IsEmpty() {
			c.StructColumns = append(c.StructColumns, NewStructColumn(fieldName, fieldType, columns))
			return rfield.NewFieldPathWithChildren(len(c.StructColumns)-1, fieldPaths)
		} else {
			return nil
		}
	default:
		panic("unsupported field type")
	}
}

func (c *Columns) UpdateColumn(fieldPath *rfield.FieldPath, field *rfield.Field) {
	switch t := field.Value.(type) {
	case *rfield.I8:
		c.I8Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.I8Columns[fieldPath.Current].Len()
	case *rfield.I16:
		c.I16Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.I16Columns[fieldPath.Current].Len()
	case *rfield.I32:
		c.I32Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.I32Columns[fieldPath.Current].Len()
	case *rfield.I64:
		c.I64Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.I64Columns[fieldPath.Current].Len()
	case *rfield.U8:
		c.U8Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.U8Columns[fieldPath.Current].Len()
	case *rfield.U16:
		c.U16Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.U16Columns[fieldPath.Current].Len()
	case *rfield.U32:
		c.U32Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.U32Columns[fieldPath.Current].Len()
	case *rfield.U64:
		c.U64Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.U64Columns[fieldPath.Current].Len()
	case *rfield.F32:
		c.F32Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.F32Columns[fieldPath.Current].Len()
	case *rfield.F64:
		c.F64Columns[fieldPath.Current].Push(&t.Value)
		c.length = c.F64Columns[fieldPath.Current].Len()
	case *rfield.String:
		c.StringColumns[fieldPath.Current].Push(&t.Value)
		c.length = c.StringColumns[fieldPath.Current].Len()
	case *rfield.Binary:
		c.BinaryColumns[fieldPath.Current].Push(t.Value)
		c.length = c.BinaryColumns[fieldPath.Current].Len()
	case *rfield.Bool:
		c.BooleanColumns[fieldPath.Current].Push(&t.Value)
		c.length = c.BooleanColumns[fieldPath.Current].Len()
	case *rfield.List:
		c.ListColumns[fieldPath.Current].Push(fieldPath, t.Values)
		c.length = c.ListColumns[fieldPath.Current].Len()
	case *rfield.Struct:
		for fieldPos := range t.Fields {
			c.StructColumns[fieldPath.Current].Push(fieldPath.Children[fieldPos], t.Fields[fieldPos])
		}
		c.length = c.StructColumns[fieldPath.Current].Len()
	default:
		panic("unsupported field type")
	}
}

func (c *Columns) NewArrowFields() []*arrow.Field {
	fields := make([]*arrow.Field, 0, c.ColumnCount())

	for i := range c.BooleanColumns {
		col := &c.BooleanColumns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.I8Columns {
		col := &c.I8Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.I16Columns {
		col := &c.I16Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.I32Columns {
		col := &c.I32Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.I64Columns {
		col := &c.I64Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.U8Columns {
		col := &c.U8Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.U16Columns {
		col := &c.U16Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.U32Columns {
		col := &c.U32Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.U64Columns {
		col := &c.U64Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.F32Columns {
		col := &c.F32Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.F64Columns {
		col := &c.F64Columns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.StringColumns {
		col := &c.StringColumns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.BinaryColumns {
		col := &c.BinaryColumns[i]
		fields = append(fields, col.NewArrowField())
	}
	for i := range c.StructColumns {
		col := c.StructColumns[i]
		structField := col.NewArrowField()
		fields = append(fields, structField)
	}
	for i := range c.ListColumns {
		col := c.ListColumns[i]
		listField := col.NewArrowField()
		fields = append(fields, listField)
	}

	return fields
}

func (c *Columns) NewArrays(allocator *memory.GoAllocator) ([]arrow.Array, error) {
	columnCount := c.ColumnCount()
	arrays := make([]arrow.Array, 0, columnCount)

	for i := range c.BooleanColumns {
		col := &c.BooleanColumns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.I8Columns {
		col := &c.I8Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.I16Columns {
		col := &c.I16Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.I32Columns {
		col := &c.I32Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.I64Columns {
		col := &c.I64Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.U8Columns {
		col := &c.U8Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.U16Columns {
		col := &c.U16Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.U32Columns {
		col := &c.U32Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.U64Columns {
		col := &c.U64Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.F32Columns {
		col := &c.F32Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.F64Columns {
		col := &c.F64Columns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.StringColumns {
		col := &c.StringColumns[i]
		// ToDo implement dictionary builder when that makes sense
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.BinaryColumns {
		col := &c.BinaryColumns[i]
		// ToDo implement dictionary builder when that makes sense
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.StructColumns {
		col := c.StructColumns[i]
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.ListColumns {
		col := c.ListColumns[i]
		listArray := col.NewArray(allocator)
		arrays = append(arrays, listArray)
	}

	return arrays, nil
}

func (c *Columns) ColumnCount() int {
	return len(c.I8Columns) + len(c.I16Columns) + len(c.I32Columns) + len(c.I64Columns) +
		len(c.U8Columns) + len(c.U16Columns) + len(c.U32Columns) + len(c.U64Columns) +
		len(c.F32Columns) + len(c.F64Columns) +
		len(c.BooleanColumns) +
		len(c.StringColumns) +
		len(c.BinaryColumns) +
		len(c.ListColumns) +
		len(c.StructColumns)
}

func (c *Columns) Len() int {
	return c.length
}

func (c *Columns) Clear() {
	for i := range c.BooleanColumns {
		c.BooleanColumns[i].Clear()
	}
	for i := range c.I8Columns {
		c.I8Columns[i].Clear()
	}
	for i := range c.I16Columns {
		c.I16Columns[i].Clear()
	}
	for i := range c.I32Columns {
		c.I32Columns[i].Clear()
	}
	for i := range c.I64Columns {
		c.I64Columns[i].Clear()
	}
	for i := range c.U8Columns {
		c.U8Columns[i].Clear()
	}
	for i := range c.U16Columns {
		c.U16Columns[i].Clear()
	}
	for i := range c.U32Columns {
		c.U32Columns[i].Clear()
	}
	for i := range c.U64Columns {
		c.U64Columns[i].Clear()
	}
	for i := range c.F32Columns {
		c.F32Columns[i].Clear()
	}
	for i := range c.F64Columns {
		c.F64Columns[i].Clear()
	}
	for i := range c.StringColumns {
		c.StringColumns[i].Clear()
	}
	for i := range c.BinaryColumns {
		c.BinaryColumns[i].Clear()
	}
	for i := range c.StructColumns {
		c.StructColumns[i].Clear()
	}
	for i := range c.ListColumns {
		c.ListColumns[i].Clear()
	}
	c.length = 0
}

func (c *Columns) IsEmpty() bool {
	return len(c.I8Columns) == 0 && len(c.I16Columns) == 0 && len(c.I32Columns) == 0 && len(c.I64Columns) == 0 && len(c.U8Columns) == 0 && len(c.U16Columns) == 0 && len(c.U32Columns) == 0 && len(c.U64Columns) == 0 && len(c.F32Columns) == 0 && len(c.F64Columns) == 0 && len(c.BooleanColumns) == 0 && len(c.StringColumns) == 0 && len(c.BinaryColumns) == 0 && len(c.ListColumns) == 0 && len(c.StructColumns) == 0
}

func (c *Columns) Metadata() map[string]*ColumnMetadata {
	metadata := make(map[string]*ColumnMetadata)

	for _, column := range c.I8Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.I16Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.I32Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.I64Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.U8Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.U16Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.U32Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.U64Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.F32Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.F64Columns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.BooleanColumns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.StringColumns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
			Dictionary: &DictionaryMetadata{
				Card:       column.DictionaryLen(),
				AvgLen:     column.AvgValueLength(),
				TotalEntry: column.TotalEntry(),
			},
		}
	}
	for _, column := range c.BinaryColumns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
			Dictionary: &DictionaryMetadata{
				Card:       column.DictionaryLen(),
				AvgLen:     column.AvgValueLength(),
				TotalEntry: column.TotalEntry(),
			},
		}
	}
	for _, column := range c.ListColumns {
		metadata[column.Name()] = &ColumnMetadata{
			Field: column.NewArrowField(),
			Len:   column.Len(),
		}
	}
	for _, column := range c.StructColumns {
		metadata[column.Name()] = &ColumnMetadata{
			Field:    column.NewArrowField(),
			Len:      0,
			Children: column.Metadata(),
		}
	}
	return metadata
}

func (c *Columns) DictionaryStats(parentPath string) []*stats.DictionaryStats {
	dictionaryStats := make([]*stats.DictionaryStats, 0, len(c.StringColumns)+len(c.StructColumns))

	for _, column := range c.StringColumns {
		dictionaryStats = append(dictionaryStats, column.DictionaryStats(parentPath))
	}
	for _, column := range c.BinaryColumns {
		dictionaryStats = append(dictionaryStats, column.DictionaryStats(parentPath))
	}
	for _, structColumn := range c.StructColumns {
		dictionaryStats = append(dictionaryStats, structColumn.DictionaryStats(parentPath)...)
	}
	return dictionaryStats
}
