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
	PushFromValues(data []rfield.Value)
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
}

type ColumnMetadata struct {
	Name     string
	Type     arrow.DataType
	Len      int
	Children []*ColumnMetadata
}

func NewColumns(allocator *memory.GoAllocator, field *rfield.Field, fieldPath []int, subFields []*rfield.Field, config *config.Config, dictIdGen *dictionary.DictIdGenerator) (*Columns, []*rfield.FieldPath) {
	fieldPaths := make([]*rfield.FieldPath, 0, len(subFields))
	columns := Columns{}
	for i := range subFields {
		subFieldPath := make([]int, 0, len(fieldPath)+1)
		copy(subFieldPath, fieldPath)
		subFieldPath = append(subFieldPath, len(fieldPaths))
		fieldPath := columns.CreateColumn(allocator, subFieldPath, field.Value.(*rfield.Struct).Fields[i], config, dictIdGen)
		if fieldPath != nil {
			fieldPaths = append(fieldPaths, fieldPath)
		}
	}
	return &columns, fieldPaths
}

// Create a column with a field based on its field type and field name.
func (c *Columns) CreateColumn(allocator *memory.GoAllocator, path []int, field *rfield.Field, config *config.Config, dictIdGen *dictionary.DictIdGenerator) *rfield.FieldPath {
	switch field.Value.(type) {
	case *rfield.Bool:
		c.BooleanColumns = append(c.BooleanColumns, MakeBoolColumn(field.Name, &field.Value.(*rfield.Bool).Value))
		return rfield.NewFieldPath(len(c.BooleanColumns) - 1)
	case *rfield.I8:
		c.I8Columns = append(c.I8Columns, MakeI8Column(field.Name, &field.Value.(*rfield.I8).Value))
		return rfield.NewFieldPath(len(c.I8Columns) - 1)
	case *rfield.I16:
		c.I16Columns = append(c.I16Columns, MakeI16Column(field.Name, &field.Value.(*rfield.I16).Value))
		return rfield.NewFieldPath(len(c.I16Columns) - 1)
	case *rfield.I32:
		c.I32Columns = append(c.I32Columns, MakeI32Column(field.Name, &field.Value.(*rfield.I32).Value))
		return rfield.NewFieldPath(len(c.I32Columns) - 1)
	case *rfield.I64:
		c.I64Columns = append(c.I64Columns, MakeI64Column(field.Name, &field.Value.(*rfield.I64).Value))
		return rfield.NewFieldPath(len(c.I64Columns) - 1)
	case *rfield.U8:
		c.U8Columns = append(c.U8Columns, MakeU8Column(field.Name, &field.Value.(*rfield.U8).Value))
		return rfield.NewFieldPath(len(c.U8Columns) - 1)
	case *rfield.U16:
		c.U16Columns = append(c.U16Columns, MakeU16Column(field.Name, &field.Value.(*rfield.U16).Value))
		return rfield.NewFieldPath(len(c.U16Columns) - 1)
	case *rfield.U32:
		c.U32Columns = append(c.U32Columns, MakeU32Column(field.Name, &field.Value.(*rfield.U32).Value))
		return rfield.NewFieldPath(len(c.U32Columns) - 1)
	case *rfield.U64:
		c.U64Columns = append(c.U64Columns, MakeU64Column(field.Name, &field.Value.(*rfield.U64).Value))
		return rfield.NewFieldPath(len(c.U64Columns) - 1)
	case *rfield.F32:
		c.F32Columns = append(c.F32Columns, MakeF32Column(field.Name, &field.Value.(*rfield.F32).Value))
		return rfield.NewFieldPath(len(c.F32Columns) - 1)
	case *rfield.F64:
		c.F64Columns = append(c.F64Columns, MakeF64Column(field.Name, &field.Value.(*rfield.F64).Value))
		return rfield.NewFieldPath(len(c.F64Columns) - 1)
	case *rfield.String:
		stringColumn := NewStringColumn(field.Name, &config.Dictionaries.StringColumns, path, dictIdGen.NextId())
		stringColumn.Push(&field.Value.(*rfield.String).Value)
		c.StringColumns = append(c.StringColumns, *stringColumn)
		return rfield.NewFieldPath(len(c.StringColumns) - 1)
	case *rfield.Binary:
		c.BinaryColumns = append(c.BinaryColumns, MakeBinaryColumn(field.Name, &field.Value.(*rfield.Binary).Value))
		return rfield.NewFieldPath(len(c.BinaryColumns) - 1)
	case *rfield.List:
		dataType := rfield.ListDataType(field.Value.(*rfield.List).Values)
		c.ListColumns = append(c.ListColumns, MakeListColumn(allocator, field, path, dataType, field.Value.(*rfield.List).Values, config, dictIdGen))
		return rfield.NewFieldPath(len(c.ListColumns) - 1)
	case *rfield.Struct:
		dataType := rfield.StructDataType(field.Value.(*rfield.Struct).Fields)
		columns, fieldPaths := NewColumns(allocator, field, path, field.Value.(*rfield.Struct).Fields, config, dictIdGen)
		if !columns.IsEmpty() {
			c.StructColumns = append(c.StructColumns, NewStructColumn(field.Name, dataType, columns))
			return rfield.NewFieldPathWithChildren(len(c.StructColumns)-1, fieldPaths)
		} else {
			return nil
		}
	default:
		panic("unsupported field type")
	}
}

func (c *Columns) UpdateColumn(fieldPath *rfield.FieldPath, field *rfield.Field) {
	switch field.Value.(type) {
	case *rfield.I8:
		c.I8Columns[fieldPath.Current].Push(&field.Value.(*rfield.I8).Value)
	case *rfield.I16:
		c.I16Columns[fieldPath.Current].Push(&field.Value.(*rfield.I16).Value)
	case *rfield.I32:
		c.I32Columns[fieldPath.Current].Push(&field.Value.(*rfield.I32).Value)
	case *rfield.I64:
		c.I64Columns[fieldPath.Current].Push(&field.Value.(*rfield.I64).Value)
	case *rfield.U8:
		c.U8Columns[fieldPath.Current].Push(&field.Value.(*rfield.U8).Value)
	case *rfield.U16:
		c.U16Columns[fieldPath.Current].Push(&field.Value.(*rfield.U16).Value)
	case *rfield.U32:
		c.U32Columns[fieldPath.Current].Push(&field.Value.(*rfield.U32).Value)
	case *rfield.U64:
		c.U64Columns[fieldPath.Current].Push(&field.Value.(*rfield.U64).Value)
	case *rfield.F32:
		c.F32Columns[fieldPath.Current].Push(&field.Value.(*rfield.F32).Value)
	case *rfield.F64:
		c.F64Columns[fieldPath.Current].Push(&field.Value.(*rfield.F64).Value)
	case *rfield.String:
		c.StringColumns[fieldPath.Current].Push(&field.Value.(*rfield.String).Value)
	case *rfield.Binary:
		c.BinaryColumns[fieldPath.Current].Push(&field.Value.(*rfield.Binary).Value)
	case *rfield.Bool:
		c.BooleanColumns[fieldPath.Current].Push(&field.Value.(*rfield.Bool).Value)
	case *rfield.List:
		c.ListColumns[fieldPath.Current].Push(field.Value.(*rfield.List).Values)
	case *rfield.Struct:
		for fieldPos := range field.Value.(*rfield.Struct).Fields {
			c.StructColumns[fieldPath.Current].Push(fieldPath.Children[fieldPos], field.Value.(*rfield.Struct).Fields[fieldPos])
		}
	default:
		panic("unsupported field type")
	}
}

func (c *Columns) Build(allocator *memory.GoAllocator) ([]*arrow.Field, []arrow.Array, error) {
	columnCount := c.ColumnCount()
	fields := make([]*arrow.Field, 0, columnCount)
	arrays := make([]arrow.Array, 0, columnCount)

	for i := range c.BooleanColumns {
		col := &c.BooleanColumns[i]
		fields = append(fields, col.NewBoolSchemaField())
		arrays = append(arrays, col.NewBoolArray(allocator))
	}
	for i := range c.I8Columns {
		col := &c.I8Columns[i]
		fields = append(fields, col.NewI8SchemaField())
		arrays = append(arrays, col.NewI8Array(allocator))
	}
	for i := range c.I16Columns {
		col := &c.I16Columns[i]
		fields = append(fields, col.NewI16SchemaField())
		arrays = append(arrays, col.NewI16Array(allocator))
	}
	for i := range c.I32Columns {
		col := &c.I32Columns[i]
		fields = append(fields, col.NewI32SchemaField())
		arrays = append(arrays, col.NewI32Array(allocator))
	}
	for i := range c.I64Columns {
		col := &c.I64Columns[i]
		fields = append(fields, col.NewArrowField())
		arrays = append(arrays, col.NewArray(allocator))
	}
	for i := range c.U8Columns {
		col := &c.U8Columns[i]
		fields = append(fields, col.NewU8SchemaField())
		arrays = append(arrays, col.NewU8Array(allocator))
	}
	for i := range c.U16Columns {
		col := &c.U16Columns[i]
		fields = append(fields, col.NewU16SchemaField())
		arrays = append(arrays, col.NewU16Array(allocator))
	}
	for i := range c.U32Columns {
		col := &c.U32Columns[i]
		fields = append(fields, col.NewU32SchemaField())
		arrays = append(arrays, col.NewU32Array(allocator))
	}
	for i := range c.U64Columns {
		col := &c.U64Columns[i]
		fields = append(fields, col.NewU64SchemaField())
		arrays = append(arrays, col.NewU64Array(allocator))
	}
	for i := range c.F32Columns {
		col := &c.F32Columns[i]
		fields = append(fields, col.NewF32SchemaField())
		arrays = append(arrays, col.NewF32Array(allocator))
	}
	for i := range c.F64Columns {
		col := &c.F64Columns[i]
		fields = append(fields, col.NewF64SchemaField())
		arrays = append(arrays, col.NewF64Array(allocator))
	}
	for i := range c.StringColumns {
		col := &c.StringColumns[i]
		// ToDo implement dictionary builder when that makes sense
		fields = append(fields, col.NewStringSchemaField())
		arrays = append(arrays, col.NewStringArray(allocator))
	}
	for i := range c.BinaryColumns {
		col := &c.BinaryColumns[i]
		// ToDo implement dictionary builder when that makes sense
		fields = append(fields, col.NewBinarySchemaField())
		arrays = append(arrays, col.NewBinaryArray(allocator))
	}
	for i := range c.StructColumns {
		col := c.StructColumns[i]
		structField, structArray, err := col.Build(allocator)
		if err != nil {
			return nil, nil, err
		}
		fields = append(fields, structField)
		arrays = append(arrays, structArray)
	}
	for i := range c.ListColumns {
		col := c.ListColumns[i]
		listField := col.NewArrowField()
		listArray := col.NewArray(allocator)
		fields = append(fields, listField)
		arrays = append(arrays, listArray)
	}

	return fields, arrays, nil
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

func (c *Columns) IsEmpty() bool {
	return len(c.I8Columns) == 0 && len(c.I16Columns) == 0 && len(c.I32Columns) == 0 && len(c.I64Columns) == 0 && len(c.U8Columns) == 0 && len(c.U16Columns) == 0 && len(c.U32Columns) == 0 && len(c.U64Columns) == 0 && len(c.F32Columns) == 0 && len(c.F64Columns) == 0 && len(c.BooleanColumns) == 0 && len(c.StringColumns) == 0 && len(c.BinaryColumns) == 0 && len(c.ListColumns) == 0 && len(c.StructColumns) == 0
}

func (c *Columns) Metadata() []*ColumnMetadata {
	metadata := make([]*ColumnMetadata, 0, len(c.I8Columns)+len(c.I16Columns)+len(c.I32Columns)+len(c.I64Columns)+
		len(c.U8Columns)+len(c.U16Columns)+len(c.U32Columns)+len(c.U64Columns)+len(c.F32Columns)+len(c.F64Columns)+
		len(c.BooleanColumns)+len(c.StringColumns)+len(c.BinaryColumns)+len(c.ListColumns)+len(c.StructColumns))

	for _, i8Column := range c.I8Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i8Column.Name(),
			Type: arrow.PrimitiveTypes.Int8,
			Len:  i8Column.Len(),
		})
	}
	for _, i16Column := range c.I16Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i16Column.Name(),
			Type: arrow.PrimitiveTypes.Int16,
			Len:  i16Column.Len(),
		})
	}
	for _, i32Column := range c.I32Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i32Column.Name(),
			Type: arrow.PrimitiveTypes.Int32,
			Len:  i32Column.Len(),
		})
	}
	for _, i64Column := range c.I64Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i64Column.Name(),
			Type: arrow.PrimitiveTypes.Int64,
			Len:  i64Column.Len(),
		})
	}
	for _, u8Column := range c.U8Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u8Column.Name(),
			Type: arrow.PrimitiveTypes.Uint8,
			Len:  u8Column.Len(),
		})
	}
	for _, u16Column := range c.U16Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u16Column.Name(),
			Type: arrow.PrimitiveTypes.Uint16,
			Len:  u16Column.Len(),
		})
	}
	for _, u32Column := range c.U32Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u32Column.Name(),
			Type: arrow.PrimitiveTypes.Uint32,
			Len:  u32Column.Len(),
		})
	}
	for _, u64Column := range c.U64Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u64Column.Name(),
			Type: arrow.PrimitiveTypes.Uint64,
			Len:  u64Column.Len(),
		})
	}
	for _, f32Column := range c.F32Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: f32Column.Name(),
			Type: arrow.PrimitiveTypes.Float32,
			Len:  f32Column.Len(),
		})
	}
	for _, f64Column := range c.F64Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: f64Column.Name(),
			Type: arrow.PrimitiveTypes.Float64,
			Len:  f64Column.Len(),
		})
	}
	for _, booleanColumn := range c.BooleanColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: booleanColumn.Name(),
			Type: arrow.FixedWidthTypes.Boolean,
			Len:  booleanColumn.Len(),
		})
	}
	for _, stringColumn := range c.StringColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: *stringColumn.Name(),
			Type: arrow.BinaryTypes.String,
			Len:  stringColumn.Len(),
		})
	}
	for _, binaryColumn := range c.BinaryColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: binaryColumn.Name(),
			Type: arrow.BinaryTypes.Binary,
			Len:  binaryColumn.Len(),
		})
	}
	for _, listColumn := range c.ListColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: listColumn.Name(),
			Type: listColumn.Type(),
			Len:  listColumn.Len(),
		})
	}
	for _, structColumn := range c.StructColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name:     structColumn.Name(),
			Type:     structColumn.Type(),
			Len:      0,
			Children: structColumn.Metadata(),
		})
	}
	return metadata
}

func (c *Columns) DictionaryStats() []*stats.DictionaryStats {
	dictionaryStats := make([]*stats.DictionaryStats, 0, len(c.StringColumns)+len(c.StructColumns))

	for _, stringColumn := range c.StringColumns {
		dictionaryStats = append(dictionaryStats, stringColumn.DictionaryStats())
	}
	for _, structColumn := range c.StructColumns {
		dictionaryStats = append(dictionaryStats, structColumn.DictionaryStats()...)
	}
	return dictionaryStats
}
