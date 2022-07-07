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
	"github.com/apache/arrow/go/arrow"
	"github.com/apache/arrow/go/arrow/array"
	"github.com/apache/arrow/go/arrow/memory"
)

// BoolColumn is a column of boolean data.
type BoolColumn struct {
	// name of the column.
	name string

	// ToDo replace []*bool by []bool + bitset
	// data of the column.
	data []*bool
}

// MakeBoolColumn creates a new bool column.
func MakeBoolColumn(name string, data *bool) BoolColumn {
	return BoolColumn{
		name: name,
		data: []*bool{data},
	}
}

// NewBoolBuilder creates and initializes a new BooleanBuilder for the column.
func (c *BoolColumn) NewBoolBuilder(allocator *memory.GoAllocator) *array.BooleanBuilder {
	builder := array.NewBooleanBuilder(allocator)
	builder.Reserve(len(c.data))
	for _, v := range c.data {
		if v == nil {
			builder.AppendNull()
		} else {
			builder.UnsafeAppend(*v)
		}
	}
	c.Clear()
	return builder
}

// Name returns the name of the column.
func (c *BoolColumn) Name() string {
	return c.name
}

// Push adds a new value to the column.
func (c *BoolColumn) Push(data *bool) {
	c.data = append(c.data, data)
}

// Len returns the number of values in the column.
func (c *BoolColumn) Len() int {
	return len(c.data)
}

// Clear clears the int64 data in the column but keep the original memory buffer allocated.
func (c *BoolColumn) Clear() {
	c.data = c.data[:0]
}

// MakeBoolSchemaField creates a Bool schema field.
func (c *BoolColumn) MakeBoolSchemaField() arrow.Field {
	return arrow.Field{Name: c.name, Type: arrow.FixedWidthTypes.Boolean}
}
