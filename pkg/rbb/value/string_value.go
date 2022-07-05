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

package value

import (
	"otel-arrow-adapter/pkg/rbb/config"
	"otel-arrow-adapter/pkg/rbb/stats"
)

// StringColumn is a column of optional string values.
type StringColumn struct {
	name             string
	config           *config.DictionaryConfig
	fieldPath        []int
	dictId           int
	dictionary       map[string]bool
	data             []*string
	totalValueLength int
	totalRowCount    int
}

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

func (c *StringColumn) ColumnName() *string {
	return &c.name
}

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

func (c *StringColumn) DictionaryLen() int {
	if c.dictionary != nil {
		return len(c.dictionary)
	} else {
		return 0
	}
}

func (c *StringColumn) AvgValueLength() float64 {
	if c.totalValueLength == 0 || c.totalRowCount == 0 {
		return 0.0
	}
	return float64(c.totalValueLength) / float64(c.totalRowCount)
}

func (c *StringColumn) Len() int {
	return len(c.data)
}
