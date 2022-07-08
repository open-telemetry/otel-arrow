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
	"otel-arrow-adapter/pkg/rbb/field_value"
	"otel-arrow-adapter/pkg/rbb/stats"
)

// StructColumn is a column of struct data.
type StructColumn struct {
	name       string
	structType arrow.DataType
	columns    Columns
}

// MakeStructColumn creates a new Struct column.
func MakeStructColumn(name string, structType arrow.DataType, columns Columns) StructColumn {
	return StructColumn{
		name:       name,
		structType: structType,
		columns:    columns,
	}
}

// Push pushes the value to the column.
func (c *StructColumn) Push(fieldPath *field_value.FieldPath, field *field_value.Field) {
	c.columns.UpdateColumn(fieldPath, field)
}

// Name returns the name of the column.
func (c *StructColumn) Name() string {
	return c.name
}

// Build builds the column.
func (c *StructColumn) Build(allocator *memory.GoAllocator) ([]arrow.Field, []array.Builder, error) {
	return c.columns.Build(allocator)
}

// DictionaryStats returns the dictionary statistics of the column.
func (c *StructColumn) DictionaryStats() []*stats.DictionaryStats {
	return c.columns.DictionaryStats()
}

// Type returns the type of the column.
func (c *StructColumn) Type() arrow.DataType {
	return c.structType
}

// Metadata returns the metadata of the column.
func (c *StructColumn) Metadata() []*ColumnMetadata {
	return c.columns.Metadata()
}
