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
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"github.com/apache/arrow/go/v9/arrow/bitutil"
	"github.com/apache/arrow/go/v9/arrow/memory"

	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/air/dictionary"
	"otel-arrow-adapter/pkg/air/rfield"
)

type ListColumn interface {
	Column
	Push(fieldPath *rfield.FieldPath, list []rfield.Value)
}

const (
	minBuilderCapacity = 1 << 5
)

// ListColumn is a column of list data.
type ListColumnBase struct {
	name       string         // name of the column.
	etype      arrow.DataType // data type of the list's elements.
	offsets    *I32Column     // offsets used to recover the initial sub-list from the flattened list of values.
	nullBitmap *memory.Buffer // null bitmap used to determine if a sub-list is valid or null.
	length     int
	capacity   int
	nulls      int // number of null sub-lists.
	mem        *memory.GoAllocator
	values     Column
}

func MakeListColumn(allocator *memory.GoAllocator, listName string, metadata arrow.Metadata, fieldPath []int, stringFieldPath string, etype arrow.DataType, config *config.Config, dictIdGen *dictionary.DictIdGenerator) (ListColumn, []*rfield.FieldPath) {
	var values Column
	fieldPaths := []*rfield.FieldPath(nil)
	switch t := etype.(type) {
	case *arrow.BooleanType:
		col := MakeBoolColumn(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Uint8Type:
		col := MakeU8Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Uint16Type:
		col := MakeU16Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Uint32Type:
		col := MakeU32Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Uint64Type:
		col := MakeU64Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Int8Type:
		col := MakeI8Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Int16Type:
		col := MakeI16Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Int32Type:
		col := MakeI32Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Int64Type:
		col := MakeI64Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Float32Type:
		col := MakeF32Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.Float64Type:
		col := MakeF64Column(allocator, etype.Name(), metadata)
		values = &col
	case *arrow.StringType:
		col := NewStringColumn(allocator, etype.Name(), metadata, &config.Dictionaries.StringColumns, fieldPath, dictIdGen.NextId())
		values = col
	case *arrow.BinaryType:
		col := MakeBinaryColumn(allocator, etype.Name(), metadata, &config.Dictionaries.StringColumns, fieldPath, dictIdGen.NextId())
		values = &col
	case *arrow.StructType:
		columns, fps := NewColumns(allocator, t, fieldPath, stringFieldPath, config, dictIdGen)
		fieldPaths = fps
		values = NewStructColumn(etype.Name(), metadata, etype, columns)

	// List of list of not yet supported

	default:
		panic(fmt.Sprintf("ListColumn: unsupported data type %s", etype.Name()))
	}
	return NewListColumnBase(allocator, listName, etype, values), fieldPaths
}

func NewListColumnBase(allocator *memory.GoAllocator, name string, dataType arrow.DataType, values Column) *ListColumnBase {
	// Initialize ListColumnBase
	nulls := 0

	offsets := MakeI32Column(allocator, "offsets", arrow.Metadata{})

	return &ListColumnBase{
		name:       name,
		etype:      dataType,
		offsets:    &offsets, // offsets used to recover the initial sub-list from the flattened list of values.
		nullBitmap: nil,      // null bitmap used to determine if a sub-list is valid or null.
		length:     0,        // number of sub-lists.
		capacity:   0,
		nulls:      nulls,
		mem:        allocator,
		values:     values,
	}
}

func (c *ListColumnBase) Name() string {
	return c.name
}

func (c *ListColumnBase) Type() arrow.DataType {
	return c.etype
}

func (c *ListColumnBase) Len() int {
	return c.length
}

func (c *ListColumnBase) PushFromValues(_ *rfield.FieldPath, _ []rfield.Value) {
	panic("not implemented")
}

func (c *ListColumnBase) appendNextOffset(offset int32) {
	c.offsets.Push(&offset)
}

func (c *ListColumnBase) Append(v bool, offset int32) {
	c.Reserve(1)
	c.unsafeAppendBoolToBitmap(v)
	c.appendNextOffset(offset)
}

func (c *ListColumnBase) init(capacity int) {
	toAlloc := bitutil.CeilByte(capacity) / 8
	c.nullBitmap = memory.NewResizableBuffer(c.mem)
	c.nullBitmap.Resize(toAlloc)
	c.capacity = capacity
	memory.Set(c.nullBitmap.Buf(), 0)
}

func (c *ListColumnBase) resizeHelper(n int) {
	if n < minBuilderCapacity {
		n = minBuilderCapacity
	}

	if c.capacity == 0 {
		c.init(n)
	} else {
		c.resize(n, c.init)
	}
}

// Reserve ensures there is enough space for appending n elements
// by checking the capacity and calling Resize if necessary.
func (c *ListColumnBase) Reserve(n int) {
	c.reserve(n, c.resizeHelper)
}

func (c *ListColumnBase) resize(newBits int, init func(int)) {
	if c.nullBitmap == nil {
		init(newBits)
		return
	}

	newBytesN := bitutil.CeilByte(newBits) / 8
	oldBytesN := c.nullBitmap.Len()
	c.nullBitmap.Resize(newBytesN)
	c.capacity = newBits
	if oldBytesN < newBytesN {
		memory.Set(c.nullBitmap.Buf()[oldBytesN:], 0)
	}
	if newBits < c.length {
		c.length = newBits
		c.nulls = newBits - bitutil.CountSetBits(c.nullBitmap.Buf(), 0, newBits)
	}
}

func (c *ListColumnBase) reserve(elements int, resize func(int)) {
	if c.nullBitmap == nil {
		c.nullBitmap = memory.NewResizableBuffer(c.mem)
	}
	if c.length+elements > c.capacity {
		newCap := bitutil.NextPowerOf2(c.length + elements)
		resize(newCap)
	}
}

func (c *ListColumnBase) unsafeAppendBoolToBitmap(isValid bool) {
	if isValid {
		bitutil.SetBit(c.nullBitmap.Bytes(), c.length)
	} else {
		c.nulls++
	}
	c.length++
}

func (c *ListColumnBase) Push(fieldPath *rfield.FieldPath, list []rfield.Value) {
	if list != nil {
		c.Append(true, int32(c.values.Len()))
	} else {
		c.Append(false, int32(c.values.Len()))
	}
	c.values.PushFromValues(fieldPath, list)
}

func (c *ListColumnBase) NewArrowField() *arrow.Field {
	return &arrow.Field{
		Name: c.name,
		Type: arrow.ListOf(c.etype),
	}
}

// Clear clears the list data in the column but keep the original memory buffer allocated.
func (c *ListColumnBase) Clear() {
	if c.nullBitmap != nil {
		c.nullBitmap.Release()
		c.nullBitmap = nil
	}

	c.nulls = 0
	c.length = 0
	c.capacity = 0

	c.offsets.Clear()
	c.values.Clear()
}

func (c *ListColumnBase) NewArray(allocator *memory.GoAllocator) arrow.Array {
	if c.offsets.Len() != c.length+1 {
		c.appendNextOffset(int32(c.values.Len()))
	}

	values := c.values.NewArray(allocator)
	defer values.Release()
	c.etype = c.values.NewArrowField().Type // Update etype as the schema of the items is not necessarily the same (string -> dictionary).

	var offsets *memory.Buffer
	if c.offsets != nil {
		arr := c.offsets.NewArray(allocator)
		defer arr.Release()
		offsets = arr.Data().Buffers()[1]
	}

	data := array.NewData(
		arrow.ListOf(c.etype), c.Len(),
		[]*memory.Buffer{
			c.nullBitmap,
			offsets,
		},
		[]arrow.ArrayData{values.Data()},
		c.nulls,
		0,
	)
	defer c.Clear()

	listArray := array.NewListData(data)
	data.Release()

	return listArray
}

// Metadata returns the metadata of the column.
func (c *ListColumnBase) Metadata() *ColumnMetadata {
	children := make(map[string]*ColumnMetadata)
	children[c.values.Name()] = c.values.Metadata()
	return &ColumnMetadata{
		Field:    c.NewArrowField(),
		Len:      c.Len(),
		Children: children,
	}
}
