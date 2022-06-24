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

package rbb

// DictIdGenerator defines a dictionary id generator.
type DictIdGenerator struct {
	id int
}

func (g *DictIdGenerator) NextId() int {
	id := g.id
	g.id += 1
	return id
}

// FieldPath defines a field path.
type FieldPath struct {
	Current  int
	Children []*FieldPath
}

func NewFieldPathWithChildren(current int, children []*FieldPath) *FieldPath {
	return &FieldPath{
		Current:  current,
		Children: children,
	}
}

func NewFieldPath(current int) *FieldPath {
	return &FieldPath{
		Current:  current,
		Children: []*FieldPath{},
	}
}

func (fp *FieldPath) ChildPath(current int) *FieldPath {
	return &FieldPath{
		Current:  current,
		Children: fp.Children,
	}
}

type OrderBy struct {
	FieldPaths [][]uint64
}

type RecordList struct {
	Records []Record
}

// A RecordBatch builder.
// Must be fed with homogeneous records.
type RecordBatchBuilder struct {
	// The configuration of the builder.
	config *Config

	// The dictionary id generator.
	dictIdGen DictIdGenerator

	// The columns of the RecordBatch builder.
	columns Columns

	// The path for each fields.
	fieldPaths []*FieldPath

	// Optional order by clause
	orderBy *OrderBy

	// Non ordered records
	recordList []RecordList

	// Flag to indicate if the builder has been optimized.
	optimized bool
}

// Constructs a new `RecordBatchBuilder` from a Record.
func NewRecordBatchBuilderWithRecord(record *Record, config *Config) *RecordBatchBuilder {
	fieldPath := make([]*FieldPath, 0, record.FieldCount())
	builder := RecordBatchBuilder{
		config:     config,
		dictIdGen:  DictIdGenerator{id: 0},
		columns:    Columns{},
		fieldPaths: fieldPath,
		orderBy:    nil,
		recordList: []RecordList{},
		optimized:  config.Dictionaries.StringColumns.MaxSortedDictionaries == 0,
	}

	for fieldIdx, field := range record.fields {
		fieldPath := builder.columns.CreateColumn([]int{fieldIdx}, &field, config, &builder.dictIdGen)
		if fieldPath != nil {
			builder.fieldPaths = append(builder.fieldPaths, fieldPath)
		}
	}
	return &builder
}
