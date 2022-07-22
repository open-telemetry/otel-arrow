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
)

// U8Column is a column of uint8 data.
type U8Column struct {
	// name of the column.
	name string
	// data of the column.
	data []*uint8
}

// U16Column is a column of uint16 data.
type U16Column struct {
	// name of the column.
	name string
	// data of the column.
	data []*uint16
}

// U32Column is a column of uint32 data.
type U32Column struct {
	// name of the column.
	name string
	// data of the column.
	data []*uint32
}

// U64Column is a column of uint64 data.
type U64Column struct {
	// name of the column.
	name string
	// data of the column.
	data []*uint64
}

// MakeU8Column creates a new U8 column.
func MakeU8Column(name string, data *uint8) U8Column {
	return U8Column{
		name: name,
		data: []*uint8{data},
	}
}

// MakeU16Column creates a new U16 column.
func MakeU16Column(name string, data *uint16) U16Column {
	return U16Column{
		name: name,
		data: []*uint16{data},
	}
}

// MakeU32Column creates a new U32 column.
func MakeU32Column(name string, data *uint32) U32Column {
	return U32Column{
		name: name,
		data: []*uint32{data},
	}
}

// MakeU64Column creates a new U64 column.
func MakeU64Column(name string, data *uint64) U64Column {
	return U64Column{
		name: name,
		data: []*uint64{data},
	}
}

// Name returns the name of the column.
func (c *U8Column) Name() string {
	return c.name
}

// Push adds a new value to the column.
func (c *U8Column) Push(data *uint8) {
	c.data = append(c.data, data)
}

// Len returns the number of values in the column.
func (c *U8Column) Len() int {
	return len(c.data)
}

// NewU8SchemaField creates a U8 schema field.
func (c *U8Column) NewU8SchemaField() *arrow.Field {
	return &arrow.Field{Name: c.name, Type: arrow.PrimitiveTypes.Uint8}
}

// NewU8Array creates and initializes a new Arrow Array for the column.
func (c *U8Column) NewU8Array(allocator *memory.GoAllocator) arrow.Array {
	builder := array.NewUint8Builder(allocator)
	builder.Reserve(len(c.data))
	for _, v := range c.data {
		if v == nil {
			builder.AppendNull()
		} else {
			builder.UnsafeAppend(*v)
		}
	}
	c.Clear()
	return builder.NewArray()
}

// Clear clears the uint8 data in the column but keep the original memory buffer allocated.
func (c *U8Column) Clear() {
	c.data = c.data[:0]
}

// Name returns the name of the column.
func (c *U16Column) Name() string {
	return c.name
}

// Push adds a new value to the column.
func (c *U16Column) Push(data *uint16) {
	c.data = append(c.data, data)
}

// Len returns the number of values in the column.
func (c *U16Column) Len() int {
	return len(c.data)
}

// NewU16SchemaField creates a U16 schema field.
func (c *U16Column) NewU16SchemaField() *arrow.Field {
	return &arrow.Field{Name: c.name, Type: arrow.PrimitiveTypes.Uint16}
}

// NewU16Array creates and initializes a new Arrow Array for the column.
func (c *U16Column) NewU16Array(allocator *memory.GoAllocator) arrow.Array {
	builder := array.NewUint16Builder(allocator)
	builder.Reserve(len(c.data))
	for _, v := range c.data {
		if v == nil {
			builder.AppendNull()
		} else {
			builder.UnsafeAppend(*v)
		}
	}
	c.Clear()
	return builder.NewArray()
}

// Clear clears the uint16 data in the column but keep the original memory buffer allocated.
func (c *U16Column) Clear() {
	c.data = c.data[:0]
}

// Name returns the name of the column.
func (c *U32Column) Name() string {
	return c.name
}

// Push adds a new value to the column.
func (c *U32Column) Push(data *uint32) {
	c.data = append(c.data, data)
}

// Len returns the number of values in the column.
func (c *U32Column) Len() int {
	return len(c.data)
}

// Clear clears the uint32 data in the column but keep the original memory buffer allocated.
func (c *U32Column) Clear() {
	c.data = c.data[:0]
}

// NewU32SchemaField creates a U32 schema field.
func (c *U32Column) NewU32SchemaField() *arrow.Field {
	return &arrow.Field{Name: c.name, Type: arrow.PrimitiveTypes.Uint32}
}

// NewU32Array creates and initializes a new Arrow Array for the column.
func (c *U32Column) NewU32Array(allocator *memory.GoAllocator) arrow.Array {
	builder := array.NewUint32Builder(allocator)
	builder.Reserve(len(c.data))
	for _, v := range c.data {
		if v == nil {
			builder.AppendNull()
		} else {
			builder.UnsafeAppend(*v)
		}
	}
	c.Clear()
	return builder.NewArray()
}

// Name returns the name of the column.
func (c *U64Column) Name() string {
	return c.name
}

// Push adds a new value to the column.
func (c *U64Column) Push(data *uint64) {
	c.data = append(c.data, data)
}

// Len returns the number of values in the column.
func (c *U64Column) Len() int {
	return len(c.data)
}

// Clear clears the uint64 data in the column but keep the original memory buffer allocated.
func (c *U64Column) Clear() {
	c.data = c.data[:0]
}

// NewU64SchemaField creates a U64 schema field.
func (c *U64Column) NewU64SchemaField() *arrow.Field {
	return &arrow.Field{Name: c.name, Type: arrow.PrimitiveTypes.Uint64}
}

// NewU64Array creates and initializes a new Arrow Array for the column.
func (c *U64Column) NewU64Array(allocator *memory.GoAllocator) arrow.Array {
	builder := array.NewUint64Builder(allocator)
	builder.Reserve(len(c.data))
	for _, v := range c.data {
		if v == nil {
			builder.AppendNull()
		} else {
			builder.UnsafeAppend(*v)
		}
	}
	c.Clear()
	return builder.NewArray()
}
