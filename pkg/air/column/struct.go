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
	columns    Columns
}

// MakeStructColumn creates a new Struct column.
func MakeStructColumn(name string, structType arrow.DataType, columns Columns) StructColumn {
	return StructColumn{
		name:       name,
		structType: structType,
		columns:    columns,
	}
}

// Push pushes the value to the column.
func (c *StructColumn) Push(fieldPath *rfield.FieldPath, field *rfield.Field) {
	c.columns.UpdateColumn(fieldPath, field)
}

// Name returns the name of the column.
func (c *StructColumn) Name() string {
	return c.name
}

// Build builds the column.
func (c *StructColumn) Build(allocator *memory.GoAllocator) (*arrow.Field, array.Builder, error) {
	fields, fieldBuilders, err := c.columns.Build(allocator)
	if err != nil {
		return nil, nil, err
	}
	structField := arrow.Field{Name: c.name, Type: arrow.StructOf(fields...)}
	structBuilder := array.UnsafeNewStructBuilderFromFields(allocator, fields, fieldBuilders)
	return &structField, structBuilder, nil
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

/*
func buildStruct(fields []arrow.Field, builders []array.Builder, memory *memory.GoAllocator) (*arrow.Field, array.Builder, error) {
	children := make([]arrow.ArrayData, len(builders))
	for i, b := range builders {
		arr := b.NewArray()
		defer arr.Release()
		children[i] = arr.Data()
	}

	data := array.NewData(arrow.StructOf(fields...), children[0].Len(), []*memory.Buffers{nil, nil}, children, 0, 0)
	defer data.Release()
	final := array.NewStructData(data)
}

type MyStructBuilder struct {}


// Retain increases the reference count by 1.
// Retain may be called simultaneously from multiple goroutines.
func (b *MyStructBuilder) Retain() {}

// Release decreases the reference count by 1.
func (b *MyStructBuilder) Release() {}

// Len returns the number of elements in the array builder.
func (b *MyStructBuilder) Len() int {}

// Cap returns the total number of elements that can be stored
// without allocating additional memory.
func (b *MyStructBuilder) Cap() int {}

// NullN returns the number of null values in the array builder.
func (b *MyStructBuilder) NullN() int {}

// AppendNull adds a new null value to the array being built.
func (b *MyStructBuilder) AppendNull() {}

// Reserve ensures there is enough space for appending n elements
// by checking the capacity and calling Resize if necessary.
func (b *MyStructBuilder) Reserve(n int) {}

// Resize adjusts the space allocated by b to n elements. If n is greater than b.Cap(),
// additional memory will be allocated. If n is smaller, the allocated memory may reduced.
func (b *MyStructBuilder) Resize(n int) {}

// NewArray creates a new array from the memory buffers used
// by the builder and resets the Builder so it can be used to build
// a new array.
func (b *MyStructBuilder) NewArray() arrow.Array {}

func (b *MyStructBuilder) init(capacity int) {}
func (b *MyStructBuilder) resize(newBits int, init func(int)) {}

func (b *MyStructBuilder) unmarshalOne(*json.Decoder) error {}
func (b *MyStructBuilder) unmarshal(*json.Decoder) error {}
*/
