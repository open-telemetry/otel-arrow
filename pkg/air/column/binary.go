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

	"otel-arrow-adapter/pkg/air/config"
)

// BinaryColumn is a column of binary data.
type BinaryColumn struct {
	// Name of the column.
	name string
	// Data of the column.
	data []*[]byte
	// Dictionary config of the column.
	config *config.DictionaryConfig
	// Dictionary ID of the column.
	dictId int
	// Optional dictionary containing the unique values of the column (used to build Arrow Dictionary).
	dictionary map[string]int
	// Total number of rows in the column.
	totalRowCount int
}

// MakeBinaryColumn creates a new Binary column.
func MakeBinaryColumn(name string, config *config.DictionaryConfig, dictId int) BinaryColumn {
	var dictionary map[string]int
	if config.MaxCard > 0 {
		dictionary = make(map[string]int)
	}

	return BinaryColumn{
		name:          name,
		data:          []*[]byte{},
		config:        config,
		dictId:        dictId,
		dictionary:    dictionary,
		totalRowCount: 0,
	}
}

// Name returns the name of the column.
func (c *BinaryColumn) Name() string {
	return c.name
}

// Push adds a new value to the column.
func (c *BinaryColumn) Push(value *[]byte) {
	// Maintains a dictionary of unique values
	if c.dictionary != nil {
		if value != nil {
			if _, ok := c.dictionary[string(*value)]; !ok {
				c.dictionary[string(*value)] = len(c.dictionary)
				if len(c.dictionary) > c.config.MaxCard {
					c.dictionary = nil
				}
			}
		}
	}

	c.totalRowCount++
	c.data = append(c.data, value)
}

// Len returns the number of values in the column.
func (c *BinaryColumn) Len() int {
	return len(c.data)
}

// DictionaryLen returns the number of unique values in the column.
func (c *BinaryColumn) DictionaryLen() int {
	if c.dictionary != nil {
		return len(c.dictionary)
	} else {
		return 0
	}
}

// Clear clears the bool data in the column but keep the original memory buffer allocated.
func (c *BinaryColumn) Clear() {
	c.data = c.data[:0]
}

// NewBinarySchemaField creates a Binary schema field.
func (c *BinaryColumn) NewBinarySchemaField() *arrow.Field {
	if c.dictionary != nil && c.config.IsDictionary(c.totalRowCount, c.DictionaryLen()) {
		return &arrow.Field{Name: c.name, Type: &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: arrow.BinaryTypes.Binary,
			Ordered:   false,
		}}
	} else {
		return &arrow.Field{Name: c.name, Type: arrow.BinaryTypes.Binary}
	}
}

// NewBinaryArray creates and initializes a new Arrow Array for the column.
func (c *BinaryColumn) NewBinaryArray(allocator *memory.GoAllocator) arrow.Array {
	if c.dictionary != nil && c.config.IsDictionary(c.totalRowCount, c.DictionaryLen()) {
		dictBuilder := array.NewBinaryBuilder(allocator, arrow.BinaryTypes.Binary)
		dictBuilder.Reserve(len(c.dictionary))
		for data := range c.dictionary {
			dictBuilder.Append([]byte(data))
		}
		builder := array.NewDictionaryBuilderWithDict(
			allocator,
			&arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8, // ToDo add support for uint16, uint32, uint64
				ValueType: arrow.BinaryTypes.Binary,
				Ordered:   false, // ToDo do test with ordered dictionaries
			},
			dictBuilder.NewArray())
		valuesBuilder := array.NewBinaryBuilder(allocator, arrow.BinaryTypes.Binary)
		builder.Reserve(len(c.data))
		for _, value := range c.data {
			if value != nil {
				valuesBuilder.Append(*value)
			} else {
				valuesBuilder.AppendNull()
			}
		}
		err := builder.AppendArray(valuesBuilder.NewArray())
		if err != nil {
			panic(err)
		}
		c.Clear()
		return builder.NewArray()
	} else {
		builder := array.NewBinaryBuilder(allocator, arrow.BinaryTypes.Binary)
		builder.Reserve(len(c.data))
		for _, v := range c.data {
			if v == nil {
				builder.AppendNull()
			} else {
				builder.Append(*v)
			}
		}
		c.Clear()
		return builder.NewArray()
	}
}
