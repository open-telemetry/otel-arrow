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

type ListColumn interface {
	Column
	Push(list []rfield.Value)
}

// ListColumn is a column of list data.
type ListColumnBase struct {
	// name of the column.
	name string
	// dataType of the list items.
	dataType arrow.DataType
	// Data of the column.
	Data    [][]rfield.Value
	offsets []int
	valid   []bool
}

type I64ListColumn struct {
	ListColumnBase
	column I64Column
}

func MakeListColumn(name string, dataType arrow.DataType, data []rfield.Value) ListColumn {
	var validity bool
	if data != nil {
		validity = true
	} else {
		validity = false
	}
	base := ListColumnBase{
		name:     name,
		dataType: dataType,
		Data:     [][]rfield.Value{data},
		offsets:  []int{0},
		valid:    []bool{validity},
	}
	switch dataType.(type) {
	case *arrow.Int64Type:
		values := make([]*int64, len(data))
		for i, value := range data {
			if value == nil {
				values[i] = nil
			} else {
				v, err := value.AsI64()
				if err != nil {
					panic(err)
				}
				values[i] = v
			}
		}
		return &I64ListColumn{
			ListColumnBase: base,
			column:         MakeI64sColumn(name, values),
		}
	default:
		return &base
	}
}

func (c *ListColumnBase) Name() string {
	return c.name
}

func (c *ListColumnBase) Type() arrow.DataType {
	return c.dataType
}

func (c *ListColumnBase) Len() int {
	return len(c.Data)
}

// Clear clears the list data in the column but keep the original memory buffer allocated.
func (c *ListColumnBase) Clear() {
	c.Data = c.Data[:0]
}

func (c *ListColumnBase) Push(list []rfield.Value) {
	c.Data = append(c.Data, list)
	c.offsets = append(c.offsets, len(c.Data))
	if list != nil {
		c.valid = append(c.valid, true)
	} else {
		c.valid = append(c.valid, false)
	}
}

func (c *ListColumnBase) Build(allocator *memory.GoAllocator) (*arrow.Field, array.Builder, error) {
	var listBuilder array.Builder

	switch c.dataType.(type) {
	case *arrow.Int8Type:
		listBuilder = c.int8ListBuilder(allocator)
	case *arrow.Int16Type:
		listBuilder = c.int16ListBuilder(allocator)
	case *arrow.Int32Type:
		listBuilder = c.int32ListBuilder(allocator)
	case *arrow.Int64Type:
		listBuilder = c.int64ListBuilder(allocator)
	case *arrow.StructType:
		listBuilder = c.structListBuilder(allocator)
	}

	listField := &arrow.Field{
		Name: c.name,
		Type: arrow.ListOf(c.dataType),
	}
	c.Clear()
	return listField, listBuilder, nil
}

func (c *ListColumnBase) int8ListBuilder(allocator *memory.GoAllocator) *array.ListBuilder {
	listBuilder := array.NewListBuilder(allocator, c.dataType)
	valueBuilder := listBuilder.ValueBuilder().(*array.Int8Builder)

	for _, subList := range c.Data {
		if subList != nil || len(subList) > 0 {
			// Append all values of the sublist
			listBuilder.Append(true)
			for _, value := range subList {
				if value == nil {
					valueBuilder.AppendNull()
				} else {
					v, err := value.AsI8()
					if err != nil {
						panic(err)
					}
					valueBuilder.Append(*v)
				}
			}
		} else {
			listBuilder.Append(false)
		}
	}
	return listBuilder
}

func (c *ListColumnBase) int16ListBuilder(allocator *memory.GoAllocator) *array.ListBuilder {
	listBuilder := array.NewListBuilder(allocator, c.dataType)
	valueBuilder := listBuilder.ValueBuilder().(*array.Int16Builder)

	for _, subList := range c.Data {
		if subList != nil || len(subList) > 0 {
			// Append all values of the sublist
			listBuilder.Append(true)
			for _, value := range subList {
				if value == nil {
					valueBuilder.AppendNull()
				} else {
					v, err := value.AsI16()
					if err != nil {
						panic(err)
					}
					valueBuilder.Append(*v)
				}
			}
		} else {
			listBuilder.Append(false)
		}
	}
	return listBuilder
}

func (c *ListColumnBase) int32ListBuilder(allocator *memory.GoAllocator) *array.ListBuilder {
	listBuilder := array.NewListBuilder(allocator, c.dataType)
	valueBuilder := listBuilder.ValueBuilder().(*array.Int32Builder)

	for _, subList := range c.Data {
		if subList != nil || len(subList) > 0 {
			// Append all values of the sublist
			listBuilder.Append(true)
			for _, value := range subList {
				if value == nil {
					valueBuilder.AppendNull()
				} else {
					v, err := value.AsI32()
					if err != nil {
						panic(err)
					}
					valueBuilder.Append(*v)
				}
			}
		} else {
			listBuilder.Append(false)
		}
	}
	return listBuilder
}

func (c *ListColumnBase) int64ListBuilder(allocator *memory.GoAllocator) *array.ListBuilder {
	listBuilder := array.NewListBuilder(allocator, c.dataType)
	valueBuilder := listBuilder.ValueBuilder().(*array.Int64Builder)

	for _, subList := range c.Data {
		if subList != nil || len(subList) > 0 {
			// Append all values of the sublist
			listBuilder.Append(true)
			for _, value := range subList {
				if value == nil {
					valueBuilder.AppendNull()
				} else {
					v, err := value.AsI64()
					if err != nil {
						panic(err)
					}
					valueBuilder.Append(*v)
				}
			}
		} else {
			listBuilder.Append(false)
		}
	}
	return listBuilder
}

func (c *ListColumnBase) structListBuilder(allocator *memory.GoAllocator) *array.StructBuilder {
	structBuilder := array.NewStructBuilder(allocator, c.dataType.(*arrow.StructType))
	//valueBuilder := structBuilder.ValueBuilder().(*array.StructBuilder)

	for _, subList := range c.Data {
		if subList != nil || len(subList) > 0 {
			// Append all values of the sublist
			structBuilder.Append(true)
			for _, value := range subList {
				if value == nil {
					//valueBuilder.AppendNull()
				} else {
					_, err := value.AsI64()
					if err != nil {
						panic(err)
					}
					//valueBuilder.Append(*v)
				}
			}
		} else {
			structBuilder.Append(false)
		}
	}
	return structBuilder
}

func (c *I64ListColumn) Push(list []rfield.Value) {
	c.Data = append(c.Data, list)
	c.offsets = append(c.offsets, c.column.Len())
	if list != nil {
		c.valid = append(c.valid, true)
	} else {
		c.valid = append(c.valid, false)
	}
	for _, value := range list {
		i64, err := value.AsI64()
		if err != nil {
			panic(err)
		}
		c.column.Push(i64)
	}
}

func (c *I64ListColumn) Build(allocator *memory.GoAllocator) (*arrow.Field, array.Builder, error) {
	listBuilder := array.NewListBuilder(allocator, c.dataType)
	valueBuilder := listBuilder.ValueBuilder().(*array.Int64Builder)

	for _, subList := range c.Data {
		if subList != nil || len(subList) > 0 {
			// Append all values of the sublist
			listBuilder.Append(true)
			for _, value := range subList {
				if value == nil {
					valueBuilder.AppendNull()
				} else {
					v, err := value.AsI64()
					if err != nil {
						panic(err)
					}
					valueBuilder.Append(*v)
				}
			}
		} else {
			listBuilder.Append(false)
		}
	}

	listField := &arrow.Field{
		Name: c.name,
		Type: arrow.ListOf(c.dataType),
	}
	c.Clear()
	return listField, listBuilder, nil
}
