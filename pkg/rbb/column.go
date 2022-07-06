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

import (
	"github.com/apache/arrow/go/arrow"
	"github.com/apache/arrow/go/arrow/array"
	"github.com/apache/arrow/go/arrow/memory"
	"otel-arrow-adapter/pkg/rbb/config"
	"otel-arrow-adapter/pkg/rbb/stats"
	"otel-arrow-adapter/pkg/rbb/value"
)

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

type BinaryColumn struct {
	Name string
	Data []*[]byte
}

type ListColumn struct {
	Name string
	Type arrow.DataType
	Data [][]value.Value
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

	StringColumns []value.StringColumn
	BinaryColumns []BinaryColumn

	ListColumns   []ListColumn
	StructColumns []StructColumn
}

type ColumnMetadata struct {
	Name     string
	Type     arrow.DataType
	Len      int
	Children []*ColumnMetadata
}

// Create a column with a field based on its field type and field name.
func (c *Columns) CreateColumn(path []int, field *value.Field, config *config.Config, dictIdGen *DictIdGenerator) *FieldPath {
	switch field.Value.(type) {
	case *value.Bool:
		c.BooleanColumns = append(c.BooleanColumns, BoolColumn{
			Name: field.Name,
			Data: []*bool{&field.Value.(*value.Bool).Value},
		})
		return NewFieldPath(len(c.BooleanColumns) - 1)
	case *value.I8:
		c.I8Columns = append(c.I8Columns, I8Column{
			Name: field.Name,
			Data: []*int8{&field.Value.(*value.I8).Value},
		})
		return NewFieldPath(len(c.I8Columns) - 1)
	case *value.I16:
		c.I16Columns = append(c.I16Columns, I16Column{
			Name: field.Name,
			Data: []*int16{&field.Value.(*value.I16).Value},
		})
		return NewFieldPath(len(c.I16Columns) - 1)
	case *value.I32:
		c.I32Columns = append(c.I32Columns, I32Column{
			Name: field.Name,
			Data: []*int32{&field.Value.(*value.I32).Value},
		})
		return NewFieldPath(len(c.I32Columns) - 1)
	case *value.I64:
		c.I64Columns = append(c.I64Columns, I64Column{
			Name: field.Name,
			Data: []*int64{&field.Value.(*value.I64).Value},
		})
		return NewFieldPath(len(c.I64Columns) - 1)
	case *value.U8:
		c.U8Columns = append(c.U8Columns, U8Column{
			Name: field.Name,
			Data: []*uint8{&field.Value.(*value.U8).Value},
		})
		return NewFieldPath(len(c.U8Columns) - 1)
	case *value.U16:
		c.U16Columns = append(c.U16Columns, U16Column{
			Name: field.Name,
			Data: []*uint16{&field.Value.(*value.U16).Value},
		})
		return NewFieldPath(len(c.U16Columns) - 1)
	case *value.U32:
		c.U32Columns = append(c.U32Columns, U32Column{
			Name: field.Name,
			Data: []*uint32{&field.Value.(*value.U32).Value},
		})
		return NewFieldPath(len(c.U32Columns) - 1)
	case *value.U64:
		c.U64Columns = append(c.U64Columns, U64Column{
			Name: field.Name,
			Data: []*uint64{&field.Value.(*value.U64).Value},
		})
		return NewFieldPath(len(c.U64Columns) - 1)
	case *value.F32:
		c.F32Columns = append(c.F32Columns, F32Column{
			Name: field.Name,
			Data: []*float32{&field.Value.(*value.F32).Value},
		})
		return NewFieldPath(len(c.F32Columns) - 1)
	case *value.F64:
		c.F64Columns = append(c.F64Columns, F64Column{
			Name: field.Name,
			Data: []*float64{&field.Value.(*value.F64).Value},
		})
		return NewFieldPath(len(c.F64Columns) - 1)
	case *value.String:
		stringColumn := value.NewStringColumn(field.Name, &config.Dictionaries.StringColumns, path, dictIdGen.NextId())
		stringColumn.Push(&field.Value.(*value.String).Value)
		c.StringColumns = append(c.StringColumns, *stringColumn)
		return NewFieldPath(len(c.StringColumns) - 1)
	case *value.Binary:
		c.BinaryColumns = append(c.BinaryColumns, BinaryColumn{
			Name: field.Name,
			Data: []*[]byte{&field.Value.(*value.Binary).Value},
		})
		return NewFieldPath(len(c.BinaryColumns) - 1)
	case *value.List:
		dataType := value.ListDataType(field.Value.(*value.List).Values)
		c.ListColumns = append(c.ListColumns, ListColumn{
			Name: field.Name,
			Type: dataType,
			Data: [][]value.Value{field.Value.(*value.List).Values},
		})
		return NewFieldPath(len(c.ListColumns) - 1)
	case *value.Struct:
		dataType := value.StructDataType(field.Value.(*value.Struct).Fields)
		fieldPaths := make([]*FieldPath, 0, len(field.Value.(*value.Struct).Fields))
		columns := Columns{}
		for i := range field.Value.(*value.Struct).Fields {
			updatedPath := make([]int, 0, len(path)+1)
			copy(updatedPath, path)
			updatedPath = append(updatedPath, len(fieldPaths))
			fieldPath := columns.CreateColumn(updatedPath, &field.Value.(*value.Struct).Fields[i], config, dictIdGen)
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

func (c *Columns) UpdateColumn(fieldPath *FieldPath, field *value.Field) {
	switch field.Value.(type) {
	case *value.I8:
		c.I8Columns[fieldPath.Current].Data = append(c.I8Columns[fieldPath.Current].Data, &field.Value.(*value.I8).Value)
	case *value.I16:
		c.I16Columns[fieldPath.Current].Data = append(c.I16Columns[fieldPath.Current].Data, &field.Value.(*value.I16).Value)
	case *value.I32:
		c.I32Columns[fieldPath.Current].Data = append(c.I32Columns[fieldPath.Current].Data, &field.Value.(*value.I32).Value)
	case *value.I64:
		c.I64Columns[fieldPath.Current].Data = append(c.I64Columns[fieldPath.Current].Data, &field.Value.(*value.I64).Value)
	case *value.U8:
		c.U8Columns[fieldPath.Current].Data = append(c.U8Columns[fieldPath.Current].Data, &field.Value.(*value.U8).Value)
	case *value.U16:
		c.U16Columns[fieldPath.Current].Data = append(c.U16Columns[fieldPath.Current].Data, &field.Value.(*value.U16).Value)
	case *value.U32:
		c.U32Columns[fieldPath.Current].Data = append(c.U32Columns[fieldPath.Current].Data, &field.Value.(*value.U32).Value)
	case *value.U64:
		c.U64Columns[fieldPath.Current].Data = append(c.U64Columns[fieldPath.Current].Data, &field.Value.(*value.U64).Value)
	case *value.F32:
		c.F32Columns[fieldPath.Current].Data = append(c.F32Columns[fieldPath.Current].Data, &field.Value.(*value.F32).Value)
	case *value.F64:
		c.F64Columns[fieldPath.Current].Data = append(c.F64Columns[fieldPath.Current].Data, &field.Value.(*value.F64).Value)
	case *value.String:
		c.StringColumns[fieldPath.Current].Push(&field.Value.(*value.String).Value)
	case *value.Binary:
		c.BinaryColumns[fieldPath.Current].Data = append(c.BinaryColumns[fieldPath.Current].Data, &field.Value.(*value.Binary).Value)
	case *value.Bool:
		c.BooleanColumns[fieldPath.Current].Data = append(c.BooleanColumns[fieldPath.Current].Data, &field.Value.(*value.Bool).Value)
	case *value.List:
		c.ListColumns[fieldPath.Current].Data = append(c.ListColumns[fieldPath.Current].Data, field.Value.(*value.List).Values)
	case *value.Struct:
		for fieldPos := range field.Value.(*value.Struct).Fields {
			c.StructColumns[fieldPath.Current].Columns.UpdateColumn(fieldPath.Children[fieldPos], &field.Value.(*value.Struct).Fields[fieldPos])
		}
	default:
		panic("unsupported field type")
	}
}

func (c *Columns) Build(allocator *memory.GoAllocator) ([]arrow.Field, []array.Builder, error) {
	columnCount := c.ColumnCount()
	fields := make([]arrow.Field, 0, columnCount)
	builders := make([]array.Builder, 0, columnCount)

	for i := range c.BooleanColumns {
		col := &c.BooleanColumns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.FixedWidthTypes.Boolean})
		builder := array.NewBooleanBuilder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.I8Columns {
		col := &c.I8Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Int8})
		builder := array.NewInt8Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.I16Columns {
		col := &c.I16Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Int16})
		builder := array.NewInt16Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.I32Columns {
		col := &c.I32Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Int32})
		builder := array.NewInt32Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.I64Columns {
		col := &c.I64Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Int64})
		builder := array.NewInt64Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.U8Columns {
		col := &c.U8Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Uint8})
		builder := array.NewUint8Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.U16Columns {
		col := &c.U16Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Uint16})
		builder := array.NewUint16Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.U32Columns {
		col := &c.U32Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Uint32})
		builder := array.NewUint32Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.U64Columns {
		col := &c.U64Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Uint64})
		builder := array.NewUint64Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.F32Columns {
		col := &c.F32Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Float32})
		builder := array.NewFloat32Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	for i := range c.F64Columns {
		col := &c.F64Columns[i]
		fields = append(fields, arrow.Field{Name: col.Name, Type: arrow.PrimitiveTypes.Float64})
		builder := array.NewFloat64Builder(allocator)
		builder.Reserve(len(col.Data))
		for _, v := range col.Data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.UnsafeAppend(*v)
			}
		}
		builders = append(builders, builder)
		col.Clear()
	}
	return fields, builders, nil
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
			Name: i8Column.Name,
			Type: arrow.PrimitiveTypes.Int8,
			Len:  len(i8Column.Data),
		})
	}
	for _, i16Column := range c.I16Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i16Column.Name,
			Type: arrow.PrimitiveTypes.Int16,
			Len:  len(i16Column.Data),
		})
	}
	for _, i32Column := range c.I32Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i32Column.Name,
			Type: arrow.PrimitiveTypes.Int32,
			Len:  len(i32Column.Data),
		})
	}
	for _, i64Column := range c.I64Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: i64Column.Name,
			Type: arrow.PrimitiveTypes.Int64,
			Len:  len(i64Column.Data),
		})
	}
	for _, u8Column := range c.U8Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u8Column.Name,
			Type: arrow.PrimitiveTypes.Uint8,
			Len:  len(u8Column.Data),
		})
	}
	for _, u16Column := range c.U16Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u16Column.Name,
			Type: arrow.PrimitiveTypes.Uint16,
			Len:  len(u16Column.Data),
		})
	}
	for _, u32Column := range c.U32Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u32Column.Name,
			Type: arrow.PrimitiveTypes.Uint32,
			Len:  len(u32Column.Data),
		})
	}
	for _, u64Column := range c.U64Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: u64Column.Name,
			Type: arrow.PrimitiveTypes.Uint64,
			Len:  len(u64Column.Data),
		})
	}
	for _, f32Column := range c.F32Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: f32Column.Name,
			Type: arrow.PrimitiveTypes.Float32,
			Len:  len(f32Column.Data),
		})
	}
	for _, f64Column := range c.F64Columns {
		metadata = append(metadata, &ColumnMetadata{
			Name: f64Column.Name,
			Type: arrow.PrimitiveTypes.Float64,
			Len:  len(f64Column.Data),
		})
	}
	for _, booleanColumn := range c.BooleanColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: booleanColumn.Name,
			Type: arrow.FixedWidthTypes.Boolean,
			Len:  len(booleanColumn.Data),
		})
	}
	for _, stringColumn := range c.StringColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: *stringColumn.ColumnName(),
			Type: arrow.BinaryTypes.String,
			Len:  stringColumn.Len(),
		})
	}
	for _, binaryColumn := range c.BinaryColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: binaryColumn.Name,
			Type: arrow.BinaryTypes.Binary,
			Len:  len(binaryColumn.Data),
		})
	}
	for _, listColumn := range c.ListColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name: listColumn.Name,
			Type: listColumn.Type,
			Len:  len(listColumn.Data),
		})
	}
	for _, structColumn := range c.StructColumns {
		metadata = append(metadata, &ColumnMetadata{
			Name:     structColumn.Name,
			Type:     structColumn.Type,
			Len:      0,
			Children: structColumn.Columns.Metadata(),
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

func (c *StructColumn) DictionaryStats() []*stats.DictionaryStats {
	return c.Columns.DictionaryStats()
}

func (c *BoolColumn) Clear() {
	c.Data = c.Data[:0]
}

func (c *I8Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *I16Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *I32Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *I64Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *U8Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *U16Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *U32Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *U64Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *F32Column) Clear() {
	c.Data = c.Data[:0]
}

func (c *F64Column) Clear() {
	c.Data = c.Data[:0]
}
