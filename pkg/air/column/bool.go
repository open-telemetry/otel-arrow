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

// BoolColumn is a column of boolean data.
type BoolColumn struct {
	field   *arrow.Field
	builder *array.BooleanBuilder
}

// MakeBoolColumn creates a new bool column.
func MakeBoolColumn(allocator *memory.GoAllocator, name string, metadata arrow.Metadata) BoolColumn {
	return BoolColumn{
		field:   &arrow.Field{Name: name, Type: arrow.FixedWidthTypes.Boolean, Metadata: metadata},
		builder: array.NewBooleanBuilder(allocator),
	}
}

// Name returns the name of the column.
func (c *BoolColumn) Name() string {
	return c.field.Name
}

func (c *BoolColumn) Type() arrow.DataType {
	return c.field.Type
}

// Push adds a new value to the column.
func (c *BoolColumn) Push(data *bool) {
	if data == nil {
		c.builder.AppendNull()
	} else {
		c.builder.Append(*data)
	}
}

// PushFromValues adds the given values to the column.
func (c *BoolColumn) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, v := range data {
		bv, err := v.AsBool()
		if err != nil {
			panic(err)
		}
		c.Push(bv)
	}
}

// Len returns the number of values in the column.
func (c *BoolColumn) Len() int {
	return c.builder.Len()
}

// Clear clears the int64 data in the column but keep the original memory buffer allocated.
func (c *BoolColumn) Clear() {
}

// NewArrowField creates a Bool schema field.
func (c *BoolColumn) NewArrowField() *arrow.Field {
	return c.field
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *BoolColumn) NewArray(_ *memory.GoAllocator) arrow.Array {
	return c.builder.NewArray()
}

func (c *BoolColumn) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
	}
}
