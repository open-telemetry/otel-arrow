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
	"math"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"github.com/apache/arrow/go/v9/arrow/memory"

	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/air/stats"
)

// StringColumn is a column of optional string values.
type StringColumn struct {
	// Dictionary config of the column.
	config *config.DictionaryConfig
	// Field path of the column (used to ref this column in the DictionaryStats).
	fieldPath []int
	// Dictionary ID of the column.
	dictId int
	// Optional dictionary containing the unique values of the column (used to build Arrow Dictionary).
	dictionary map[string]int
	// Data of the column.
	data []*string
	// Total length of the values in the column.
	totalValueLength int
	// Total number of rows in the column.
	totalRowCount int

	stringField *arrow.Field
	dicoField   *arrow.Field

	stringBuilder  *array.StringBuilder
	dicoBuilder    *array.BinaryDictionaryBuilder
	dictionaryType *arrow.DictionaryType
}

// NewStringColumn creates a new StringColumn.
func NewStringColumn(allocator *memory.GoAllocator, name string, metadata arrow.Metadata, config *config.DictionaryConfig, fieldPath []int, dictId int) *StringColumn {
	var dictionary map[string]int
	if config.MaxCard > 0 {
		dictionary = make(map[string]int)
	}
	indexType := arrow.PrimitiveTypes.Uint16
	if config.MaxCard <= math.MaxUint8 {
		indexType = arrow.PrimitiveTypes.Uint8
	}
	dicoType := &arrow.DictionaryType{
		IndexType: indexType,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false, // ToDo do test with ordered dictionaries
	}

	return &StringColumn{
		config:           config,
		fieldPath:        fieldPath,
		dictId:           dictId,
		data:             []*string{},
		totalValueLength: 0,
		totalRowCount:    0,
		dictionary:       dictionary,
		stringField:      &arrow.Field{Name: name, Type: arrow.BinaryTypes.String, Metadata: metadata},
		dicoField:        &arrow.Field{Name: name, Type: dicoType, Metadata: metadata},
		stringBuilder:    array.NewStringBuilder(allocator),
		dicoBuilder:      array.NewDictionaryBuilder(allocator, dicoType).(*array.BinaryDictionaryBuilder),
		dictionaryType:   dicoType,
	}
}

// ColumnName returns the name of the column.
func (c *StringColumn) Name() string {
	return c.stringField.Name
}

func (c *StringColumn) Type() arrow.DataType {
	return c.stringField.Type
}

// Push adds a new value to the column.
func (c *StringColumn) Push(value *string) {
	// Maintains a dictionary of unique values
	if c.dictionary != nil {
		if value != nil {
			if _, ok := c.dictionary[*value]; !ok {
				c.dictionary[*value] = len(c.dictionary)
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

// PushFromValues adds the given values to the column.
func (c *StringColumn) PushFromValues(_ *rfield.FieldPath, data []rfield.Value) {
	for _, v := range data {
		fv, err := v.AsString()
		if err != nil {
			panic(err)
		}
		c.Push(fv)
	}
}

// DictionaryStats returns the DictionaryStats of the column.
func (c *StringColumn) DictionaryStats(parentPath string) *stats.DictionaryStats {
	if c.dictionary != nil {
		stringPath := c.dicoField.Name
		if len(parentPath) > 0 {
			stringPath = parentPath + "." + c.dicoField.Name
		}
		return &stats.DictionaryStats{
			Type:             stats.StringDic,
			NumPath:          c.fieldPath,
			StringPath:       stringPath,
			Cardinality:      c.DictionaryLen(),
			AvgEntryLength:   c.AvgValueLength(),
			TotalEntry:       c.totalRowCount,
			TotalValueLength: c.totalValueLength,
		}
	}
	return nil
}

// DictionaryLen returns the number of unique values in the column.
func (c *StringColumn) DictionaryLen() int {
	if c.dictionary != nil {
		return len(c.dictionary)
	} else {
		return 0
	}
}

// AvgValueLength returns the average length of the values in the column.
func (c *StringColumn) AvgValueLength() float64 {
	if c.totalValueLength == 0 || c.totalRowCount == 0 {
		return 0.0
	}
	return float64(c.totalValueLength) / float64(c.totalRowCount)
}

func (c *StringColumn) TotalEntry() int {
	return c.totalRowCount
}

// Len returns the number of values in the column.
func (c *StringColumn) Len() int {
	return len(c.data)
}

// Clear resets the column to its initial state.
func (c *StringColumn) Clear() {
	c.data = c.data[:0]
}

// NewArrowField creates a schema field
func (c *StringColumn) NewArrowField() *arrow.Field {
	if c.dictionary != nil && c.config.IsDictionary(c.totalRowCount, c.DictionaryLen(), c.totalValueLength) {
		return c.dicoField
	} else {
		return c.stringField
	}
}

// NewArray creates and initializes a new Arrow Array for the column.
func (c *StringColumn) NewArray(_ *memory.GoAllocator) arrow.Array {
	if c.dictionary != nil && c.config.IsDictionary(c.totalRowCount, c.DictionaryLen(), c.totalValueLength) {
		c.dicoBuilder.Reserve(len(c.data))
		for _, value := range c.data {
			if value != nil {
				err := c.dicoBuilder.AppendString(*value)
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
		c.stringBuilder.Reserve(c.Len())
		for _, v := range c.data {
			if v == nil {
				c.stringBuilder.AppendNull()
			} else {
				c.stringBuilder.Append(*v)
			}
		}
		c.Clear()
		return c.stringBuilder.NewArray()
	}
}

func (c *StringColumn) Metadata() *ColumnMetadata {
	return &ColumnMetadata{
		Field: c.NewArrowField(),
		Len:   c.Len(),
		Dictionary: &DictionaryMetadata{
			Card:       c.DictionaryLen(),
			AvgLen:     c.AvgValueLength(),
			TotalEntry: c.TotalEntry(),
		},
	}
}
