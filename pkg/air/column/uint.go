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
	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"

	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
)

// U8Column is a column of uint8 data.
type U8Column struct {
	field   *arrow.Field
	builder *array.Uint8Builder
}

// U16Column is a column of uint16 data.
type U16Column struct {
	field   *arrow.Field
	builder *array.Uint16Builder
}

// U32Column is a column of uint32 data.
type U32Column struct {
	field   *arrow.Field
	builder *array.Uint32Builder
}

// U64Column is a column of uint64 data.
type U64Column struct {
	field   *arrow.Field
	builder *array.Uint64Builder
}

// MakeU8Column creates a new U8 column.
func MakeU8Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) U8Column {
	return U8Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Uint8, Metadata: metadata},
		builder: array.NewUint8Builder(allocator),
	}
}

// MakeU16Column creates a new U16 column.
func MakeU16Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) U16Column {
	return U16Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Uint16, Metadata: metadata},
		builder: array.NewUint16Builder(allocator),
	}
}

// MakeU32Column creates a new U32 column.
func MakeU32Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) U32Column {
	return U32Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Uint32, Metadata: metadata},
		builder: array.NewUint32Builder(allocator),
	}
}

// MakeU64Column creates a new U64 column.
func MakeU64Column(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) U64Column {
	return U64Column{
		field:   &arrow.Field{Name: name, Type: arrow.PrimitiveTypes.Uint64, Metadata: metadata},
		builder: array.NewUint64Builder(allocator),
	}
}

// Name returns the name of the column.
func (c *U8Column) Name() string {
	return c.field.Name
}

func (c *U8Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *U8Column) Push(data *uint8) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *U8Column) Len() int {
	return c.builder.Len()
}

// NewArrowField creates a U8 schema field.
func (c *U8Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *U8Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

// Clear clears the uint8 data in the column but keep the original memory buffer allocated.
func (c *U8Column) Clear() {
}

func (c *U8Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsU8()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}

func (c *U8Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Name returns the name of the column.
func (c *U16Column) Name() string {
	return c.field.Name
}

func (c *U16Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *U16Column) Push(data *uint16) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *U16Column) Len() int {
	return c.builder.Len()
}

// NewArrowField creates a U16 schema field.
func (c *U16Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *U16Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

// Clear clears the uint16 data in the column but keep the original memory buffer allocated.
func (c *U16Column) Clear() {
}

func (c *U16Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsU16()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}

func (c *U16Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Name returns the name of the column.
func (c *U32Column) Name() string {
	return c.field.Name
}

func (c *U32Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *U32Column) Push(data *uint32) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *U32Column) Len() int {
	return c.builder.Len()
}

// Clear clears the uint32 data in the column but keep the original memory buffer allocated.
func (c *U32Column) Clear() {
}

func (c *U32Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsU32()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}

// NewArrowField creates a U32 schema field.
func (c *U32Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *U32Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

func (c *U32Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}

// Name returns the name of the column.
func (c *U64Column) Name() string {
	return c.field.Name
}

func (c *U64Column) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *U64Column) Push(data *uint64) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// Len returns the number of values in the column.
func (c *U64Column) Len() int {
	return c.builder.Len()
}

// Clear clears the uint64 data in the column but keep the original memory buffer allocated.
func (c *U64Column) Clear() {
}

func (c *U64Column) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, value := range data {
		v, err := value.AsU64()
		if err != nil {
			panic(err)
		}
		c.Push(v)
	}
}

// NewArrowField creates a U64 schema field.
func (c *U64Column) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *U64Column) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

func (c *U64Column) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}
