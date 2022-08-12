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
	"otel-arrow-adapter/pkg/air/stats"
)

// BinaryColumn is a column of binary data.
type BinaryColumn struct {
	// Name of the column.
	name string
	// Data of the column.
	data []*[]byte
	// Dictionary config of the column.
	config *config.DictionaryConfig
	// Field path of the column (used to ref this column in the DictionaryStats).
	fieldPath []int
	// Dictionary ID of the column.
	dictId int
	// Optional dictionary containing the unique values of the column (used to build Arrow Dictionary).
	dictionary map[string]int
	// Total number of rows in the column.
	totalRowCount int
	// Total length of the values in the column.
	totalValueLength int

	field          *arrow.Field
	binaryBuilder  *array.BinaryBuilder
	dicoBuilder    *array.BinaryDictionaryBuilder
	dictionaryType *arrow.DictionaryType
}

// MakeBinaryColumn creates a new Binary column.
func MakeBinaryColumn(allocator *memory.GoAllocator, name string, config *config.DictionaryConfig, fieldPath []int, dictId int) BinaryColumn {
	var dictionary map[string]int
	if config.MaxCard > 0 {
		dictionary = make(map[string]int)
	}

	dicoType := &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint8, // ToDo add support for uint16, uint32, uint64
		ValueType: arrow.BinaryTypes.Binary,
		Ordered:   false, // ToDo do test with ordered dictionaries
	}

	return BinaryColumn{
		name:             name,
		data:             []*[]byte{},
		config:           config,
		fieldPath:        fieldPath,
		dictId:           dictId,
		dictionary:       dictionary,
		totalRowCount:    0,
		totalValueLength: 0,
		binaryBuilder:    array.NewBinaryBuilder(allocator, arrow.BinaryTypes.Binary),
		dicoBuilder:      array.NewDictionaryBuilder(allocator, dicoType).(*array.BinaryDictionaryBuilder),
		dictionaryType:   dicoType,
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
	if value != nil {
		c.totalValueLength += len(*value)
	}

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

func (c *BinaryColumn) TotalEntry() int {
	return c.totalRowCount
}

// AvgValueLength returns the average length of the values in the column.
func (c *BinaryColumn) AvgValueLength() float64 {
	if c.totalValueLength == 0 || c.totalRowCount == 0 {
		return 0.0
	}
	return float64(c.totalValueLength) / float64(c.totalRowCount)
}

// DictionaryStats returns the DictionaryStats of the column.
func (c *BinaryColumn) DictionaryStats(parentPath string) *stats.DictionaryStats {
	if c.dictionary != nil {
		stringPath := c.name
		if len(parentPath) > 0 {
			stringPath = parentPath + "." + c.name
		}
		return &stats.DictionaryStats{
			Type:           stats.BinaryDic,
			NumPath:        c.fieldPath,
			StringPath:     stringPath,
			Cardinality:    c.DictionaryLen(),
			AvgEntryLength: c.AvgValueLength(),
			TotalEntry:     c.totalRowCount,
		}
	}
	return nil
}

// Clear clears the bool data in the column but keep the original memory buffer allocated.
func (c *BinaryColumn) Clear() {
	c.data = c.data[:0]
}

// NewArrowField creates a Binary schema field.
func (c *BinaryColumn) NewArrowField() *arrow.Field {
	if c.dictionary != nil && c.config.IsDictionary(c.totalRowCount, c.DictionaryLen()) {
		return &arrow.Field{Name: c.name, Type: c.dictionaryType}
	} else {
		return &arrow.Field{Name: c.name, Type: arrow.BinaryTypes.Binary}
	}
}

// NewBinaryArray creates and initializes a new Arrow Array for the column.
func (c *BinaryColumn) NewBinaryArray(_ *memory.GoAllocator) arrow.Array {
	if c.dictionary != nil && c.config.IsDictionary(c.totalRowCount, c.DictionaryLen()) {
		c.dicoBuilder.Reserve(len(c.data))
		for _, value := range c.data {
			if value != nil {
				err := c.dicoBuilder.Append(*value)
				if err != nil {
					panic(err)
				}
			} else {
				c.dicoBuilder.AppendNull()
			}
		}
		c.Clear()
		return c.dicoBuilder.NewArray()
	} else {
		c.binaryBuilder.Reserve(len(c.data))
		for _, v := range c.data {
			if v == nil {
				c.binaryBuilder.AppendNull()
			} else {
				c.binaryBuilder.Append(*v)
			}
		}
		c.Clear()
		return c.binaryBuilder.NewArray()
	}
}
