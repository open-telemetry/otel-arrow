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

	"github.com/lquerel/otel-arrow-adapter/pkg/air/rfield"
)

// F32Column is a column of float32 data.
type F32Column struct {
	field   *arrow.Field
	builder *array.Float32Builder
}

// F64Column is a column of float64 data.
type F64Column struct {
	field   *arrow.Field
	builder *array.Float64Builder
}

// MakeF32Column creates a new F32 column.
func MakeF32Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) F32Column {
	return F32Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Float32, Metadata: metadata},
		builder: array.NewFloat32Builder(allocator),
	}
}

// MakeF64Column creates a new F64 column.
func MakeF64Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) F64Column {
	return F64Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Float64, Metadata: metadata},
		builder: array.NewFloat64Builder(allocator),
	}
}

// Push adds a new value to the column.
func (c *F32Column) Push(data *float32) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
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
	return c.field.Name
}

func (c *F32Column) Type() arrow.DataType {
	return c.field.Type
}

// Len returns the number of elements in the column.
func (c *F32Column) Len() int {
	return c.builder.Len()
}

// Clear clears the f32 data in the column but keep the original memory buffer allocated.
func (c *F32Column) Clear() {
}

// NewArrowField creates a F32 schema field.
func (c *F32Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *F32Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

func (c *F32Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Push adds a new value to the column.
func (c *F64Column) Push(data *float64) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
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
	return c.field.Name
}

func (c *F64Column) Type() arrow.DataType {
	return c.field.Type
}

// Len returns the number of elements in the column.
func (c *F64Column) Len() int {
	return c.builder.Len()
}

// Clear clears the f64 data in the column but keep the original memory buffer allocated.
func (c *F64Column) Clear() {
}

// NewArrowField creates a F64 schema field.
func (c *F64Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *F64Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

func (c *F64Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}
