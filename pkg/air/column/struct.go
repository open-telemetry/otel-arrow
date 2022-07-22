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
	"github.com/apache/arrow/go/v9/arrow/array"
	"github.com/apache/arrow/go/v9/arrow/memory"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/air/stats"
)

// StructColumn is a column of struct data.
type StructColumn struct {
	name       string
	structType arrow.DataType
	columns    *Columns
	length     int
}

// NewStructColumn creates a new Struct column.
func NewStructColumn(name string, structType arrow.DataType, columns *Columns) *StructColumn {
	return &StructColumn{
		name:       name,
		structType: structType,
		columns:    columns,
		length:     0,
	}
}

// Push pushes the value to the column.
func (c *StructColumn) Push(fieldPath *rfield.FieldPath, field *rfield.Field) {
	c.columns.UpdateColumn(fieldPath, field)
	c.length++
}

// Name returns the name of the column.
func (c *StructColumn) Name() string {
	return c.name
}

// Len returns the number of elements in the column.
func (c *StructColumn) Len() int {
	return c.length
}

// Clear resets the column to its initial state.
func (c *StructColumn) Clear() {
	c.length = 0
}

// PushFromValues adds the given values to the column.
func (c *StructColumn) PushFromValues(data []rfield.Value) {
	//for _, value := range data {
	//	c.Push(nil, value)
	//}
}

// NewArrowField returns an Arrow field for the column.
func (c *StructColumn) NewArrowField() *arrow.Field {
	panic("not implemented")
}

// NewArray returns a new array for the column.
func (c *StructColumn) NewArray(allocator *memory.GoAllocator) arrow.Array {
	panic("not implemented")
}

// Build builds the column.
func (c *StructColumn) Build(allocator *memory.GoAllocator) (*arrow.Field, arrow.Array, error) {
	fieldRefs, fieldArrays, err := c.columns.Build(allocator)
	if err != nil {
		return nil, nil, err
	}

	// Create a struct field.
	fields := make([]arrow.Field, len(fieldRefs))
	for i, field := range fieldRefs {
		fields[i] = *field
	}
	structField := arrow.Field{Name: c.name, Type: arrow.StructOf(fields...)}

	// Create a struct array.
	children := make([]arrow.ArrayData, len(fieldArrays))
	for i, fieldArray := range fieldArrays {
		defer fieldArray.Release()
		children[i] = fieldArray.Data()
	}
	data := array.NewData(arrow.StructOf(fields...), children[0].Len(), []*memory.Buffer{nil, nil}, children, 0, 0)
	defer data.Release()
	structArray := array.NewStructData(data)

	c.Clear()

	return &structField, structArray, nil
}

// DictionaryStats returns the dictionary statistics of the column.
func (c *StructColumn) DictionaryStats() []*stats.DictionaryStats {
	return c.columns.DictionaryStats()
}

// Type returns the type of the column.
func (c *StructColumn) Type() arrow.DataType {
	return c.structType
}

// Metadata returns the metadata of the column.
func (c *StructColumn) Metadata() []*ColumnMetadata {
	return c.columns.Metadata()
}
