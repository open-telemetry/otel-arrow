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
)

// F32Column is a column of float32 data.
type F32Column struct {
	// Name of the column.
	name string
	// Data of the column.
	data []*float32
}

// F64Column is a column of float64 data.
type F64Column struct {
	// Name of the column.
	name string
	// Data of the column.
	data []*float64
}

// MakeF32Column creates a new F32 column.
func MakeF32Column(name string) F32Column {
	return F32Column{
		name: name,
		data: []*float32{},
	}
}

// MakeF64Column creates a new F64 column.
func MakeF64Column(name string) F64Column {
	return F64Column{
		name: name,
		data: []*float64{},
	}
}

// Push adds a new value to the column.
func (c *F32Column) Push(data *float32) {
	c.data = append(c.data, data)
}

// PushFromValues adds the given values to the column.
func (c *F32Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, v := range data {
		fv, err := v.AsF32()
		if err != nil {
			panic(err)
		}
		c.Push(fv)
	}
}

// Name returns the name of the column.
func (c *F32Column) Name() string {
	return c.name
}

func (c *F32Column) Type() arrow.DataType {
	return arrow.PrimitiveTypes.Float32
}

// Len returns the number of elements in the column.
func (c *F32Column) Len() int {
	return len(c.data)
}

// Clear clears the f32 data in the column but keep the original memory buffer allocated.
func (c *F32Column) Clear() {
	c.data = c.data[:0]
}

// NewArrowField creates a F32 schema field.
func (c *F32Column) NewArrowField() *arrow.Field {
	return &arrow.Field{Name: c.name, Type: arrow.PrimitiveTypes.Float32}
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *F32Column) NewArray(allocator *memory.GoAllocator) arrow.Array {
	builder := array.NewFloat32Builder(allocator)
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

// Push adds a new value to the column.
func (c *F64Column) Push(data *float64) {
	c.data = append(c.data, data)
}

// PushFromValues adds the given values to the column.
func (c *F64Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, v := range data {
		fv, err := v.AsF64()
		if err != nil {
			panic(err)
		}
		c.Push(fv)
	}
}

// Name returns the name of the column.
func (c *F64Column) Name() string {
	return c.name
}

func (c *F64Column) Type() arrow.DataType {
	return arrow.PrimitiveTypes.Float64
}

// Len returns the number of elements in the column.
func (c *F64Column) Len() int {
	return len(c.data)
}

// Clear clears the f64 data in the column but keep the original memory buffer allocated.
func (c *F64Column) Clear() {
	c.data = c.data[:0]
}

// NewArrowField creates a F64 schema field.
func (c *F64Column) NewArrowField() *arrow.Field {
	return &arrow.Field{Name: c.name, Type: arrow.PrimitiveTypes.Float64}
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *F64Column) NewArray(allocator *memory.GoAllocator) arrow.Array {
	builder := array.NewFloat64Builder(allocator)
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
