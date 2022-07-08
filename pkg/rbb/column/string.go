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
	"otel-arrow-adapter/pkg/rbb/config"
	"otel-arrow-adapter/pkg/rbb/stats"
)

// StringColumn is a column of optional string values.
type StringColumn struct {
	// Name of the column.
	name string
	// Dictionary config of the column.
	config *config.DictionaryConfig
	// Field path of the column (used to ref this column in the DictionaryStats).
	fieldPath []int
	// Dictionary ID of the column.
	dictId int
	// Optional dictionary containing the unique values of the column (used to build Arrow Dictionary).
	dictionary map[string]bool
	// Data of the column.
	data []*string
	// Total length of the values in the column.
	totalValueLength int
	// Total number of rows in the column.
	totalRowCount int
}

// NewStringColumn creates a new StringColumn.
func NewStringColumn(fieldName string, config *config.DictionaryConfig, fieldPath []int, dictId int) *StringColumn {
	return &StringColumn{
		name:             fieldName,
		config:           config,
		fieldPath:        fieldPath,
		dictId:           dictId,
		data:             []*string{},
		totalValueLength: 0,
		totalRowCount:    0,
		dictionary:       make(map[string]bool),
	}
}

// ColumnName returns the name of the column.
func (c *StringColumn) Name() *string {
	return &c.name
}

// Push adds a new value to the column.
func (c *StringColumn) Push(value *string) {
	// Maintains a dictionary of unique values
	if c.dictionary != nil {
		if value != nil {
			if _, ok := c.dictionary[*value]; !ok {
				c.dictionary[*value] = true
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

// DictionaryStats returns the DictionaryStats of the column.
func (c *StringColumn) DictionaryStats() *stats.DictionaryStats {
	if c.dictionary != nil {
		return &stats.DictionaryStats{
			Path:           c.fieldPath,
			Cardinality:    c.DictionaryLen(),
			AvgEntryLength: c.AvgValueLength(),
			TotalEntry:     c.totalRowCount,
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

// Len returns the number of values in the column.
func (c *StringColumn) Len() int {
	return len(c.data)
}

// Clear resets the column to its initial state.
func (c *StringColumn) Clear() {
	c.data = c.data[:0]
}

// MakeSchemaField creates a schema field
func (c *StringColumn) MakeSchemaField() arrow.Field {
	return arrow.Field{Name: c.name, Type: arrow.BinaryTypes.String}
}

// NewStringBuilder creates and initializes a new StringBuilder for the column.
func (c *StringColumn) NewStringBuilder(allocator *memory.GoAllocator) *array.StringBuilder {
	builder := array.NewStringBuilder(allocator)
	builder.Reserve(c.Len())
	for _, v := range c.data {
		if v == nil {
			builder.AppendNull()
		} else {
			builder.Append(*v)
		}
	}
	c.Clear()
	return builder
}
