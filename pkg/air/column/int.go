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

// I8Column is a column of int8 data.
type I8Column struct {
	field   *arrow.Field
	builder *array.Int8Builder
}

// U8Column is a column of int8 data.
type I16Column struct {
	field   *arrow.Field
	builder *array.Int16Builder
}

// I32Column is a column of int32 data.
type I32Column struct {
	field   *arrow.Field
	builder *array.Int32Builder
}

// I64Column is a column of int64 data.
type I64Column struct {
	field   *arrow.Field
	builder *array.Int64Builder
}

// MakeI8Column creates a new I8 column.
func MakeI8Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) I8Column {
	return I8Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Int8, Metadata: metadata},
		builder: array.NewInt8Builder(allocator),
	}
}

// MakeI16Column creates a new I16 column.
func MakeI16Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) I16Column {
	return I16Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Int16, Metadata: metadata},
		builder: array.NewInt16Builder(allocator),
	}
}

// MakeI32Column creates a new I32 column.
func MakeI32Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) I32Column {
	return I32Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Int32, Metadata: metadata},
		builder: array.NewInt32Builder(allocator),
	}
}

// MakeI64Column creates a new I64 column.
func MakeI64Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) I64Column {
	return I64Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Int64, Metadata: metadata},
		builder: array.NewInt64Builder(allocator),
	}
}

// Name returns the name of the column.
func (c *I8Column) Name() string {
	return c.field.Name
}

func (c *I8Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *I8Column) Push(data *int8) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *I8Column) Len() int {
	return c.builder.Len()
}

// NewArrowField creates a I8 schema field.
func (c *I8Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *I8Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

// Clear clears the int8 data in the column but keep the original memory buffer allocated.
func (c *I8Column) Clear() {
}

func (c *I8Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsI8()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}
func (c *I8Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Name returns the name of the column.
func (c *I16Column) Name() string {
	return c.field.Name
}

func (c *I16Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *I16Column) Push(data *int16) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *I16Column) Len() int {
	return c.builder.Len()
}

// NewArrowField creates a I16 schema field.
func (c *I16Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *I16Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

// Clear clears the int16 data in the column but keep the original memory buffer allocated.
func (c *I16Column) Clear() {
}

func (c *I16Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsI16()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}
func (c *I16Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Name returns the name of the column.
func (c *I32Column) Name() string {
	return c.field.Name
}

func (c *I32Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *I32Column) Push(data *int32) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *I32Column) Len() int {
	return c.builder.Len()
}

// Clear clears the int32 data in the column but keep the original memory buffer allocated.
func (c *I32Column) Clear() {
}

func (c *I32Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsI32()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}

// NewArrowField creates a I32 schema field.
func (c *I32Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *I32Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}
func (c *I32Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Name returns the name of the column.
func (c *I64Column) Name() string {
	return c.field.Name
}

// Push adds a new value to the column.
func (c *I64Column) Push(data *int64) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

func (c *I64Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		i64, err := value.AsI64()
		if err != nil {
			panic(err)
		}
		c.Push(i64)
	}
}

// Len returns the number of values in the column.
func (c *I64Column) Len() int {
	return c.builder.Len()
}

// Clear clears the int64 data in the column but keep the original memory buffer allocated.
func (c *I64Column) Clear() {
}

// NewArrowField creates a I64 schema field.
func (c *I64Column) NewArrowField() *arrow.Field {
	return c.field
}

func (c *I64Column) Type() arrow.DataType {
	return c.field.Type
}

func (c *I64Column) Build(allocator *memory.GoAllocator) (*arrow.Field, arrow.Array, error) {
	return c.NewArrowField(), c.NewArray(allocator), nil
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *I64Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}
func (c *I64Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}
