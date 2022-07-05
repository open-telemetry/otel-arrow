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

import (
	config2 "otel-arrow-adapter/pkg/rbb/config"
	"otel-arrow-adapter/pkg/rbb/stats"
	"sort"
)

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
	FieldPaths [][]int
}

// A RecordBatch builder.
// Must be fed with homogeneous records.
type RecordBatchBuilder struct {
	// The configuration of the builder.
	config *config2.Config

	// The dictionary id generator.
	dictIdGen DictIdGenerator

	// The columns of the RecordBatch builder.
	columns Columns

	// The path for each fields.
	fieldPaths []*FieldPath

	// Optional order by clause
	orderBy *OrderBy

	// Non ordered records
	recordList []*Record

	// Flag to indicate if the builder has been optimized.
	optimized bool
}

type RecordBatchBuilderMetadata struct {
	SchemaId        string
	Columns         []*ColumnMetadata
	RecordListLen   int
	Optimized       bool
	DictionaryStats []*stats.DictionaryStats
}

// Constructs a new `RecordBatchBuilder` from a Record.
func NewRecordBatchBuilderWithRecord(record *Record, config *config2.Config) *RecordBatchBuilder {
	fieldPath := make([]*FieldPath, 0, record.FieldCount())
	builder := RecordBatchBuilder{
		config:     config,
		dictIdGen:  DictIdGenerator{id: 0},
		columns:    Columns{},
		fieldPaths: fieldPath,
		orderBy:    nil,
		recordList: nil,
		optimized:  config.Dictionaries.StringColumns.MaxSortedDictionaries == 0,
	}

	for fieldIdx := range record.fields {
		fieldPath := builder.columns.CreateColumn([]int{fieldIdx}, &record.fields[fieldIdx], config, &builder.dictIdGen)
		if fieldPath != nil {
			builder.fieldPaths = append(builder.fieldPaths, fieldPath)
		}
	}
	return &builder
}

func (rbb *RecordBatchBuilder) AddRecord(record *Record) {
	if rbb.recordList != nil {
		rbb.recordList = append(rbb.recordList, record)
	} else {
		for fieldIdx := range record.fields {
			rbb.columns.UpdateColumn(rbb.fieldPaths[fieldIdx], &record.fields[fieldIdx])
		}
	}
}

func (rbb *RecordBatchBuilder) IsEmpty() bool {
	return rbb.columns.IsEmpty()
}

func (rbb *RecordBatchBuilder) Metadata(schemaId string) *RecordBatchBuilderMetadata {
	recordListLen := 0

	if rbb.recordList != nil {
		recordListLen = len(rbb.recordList)
	}

	return &RecordBatchBuilderMetadata{
		SchemaId:        schemaId,
		Columns:         rbb.columns.Metadata(),
		RecordListLen:   recordListLen,
		Optimized:       rbb.optimized,
		DictionaryStats: rbb.columns.DictionaryStats(),
	}
}

func (rbb *RecordBatchBuilder) DictionaryStats() []*stats.DictionaryStats {
	return rbb.columns.DictionaryStats()
}

func (rbb *RecordBatchBuilder) OrderBy(fieldPaths [][]int) {
	rbb.orderBy = &OrderBy{
		FieldPaths: fieldPaths,
	}
	rbb.recordList = []*Record{}
}

func (rbb *RecordBatchBuilder) Optimize() bool {
	if rbb.optimized {
		return true
	}

	if rbb.orderBy == nil {
		var dictionaryStats []*stats.DictionaryStats
		for _, ds := range rbb.DictionaryStats() {
			if ds.Cardinality > 1 && rbb.config.Dictionaries.StringColumns.IsDictionary(ds.TotalEntry, ds.Cardinality) {
				dictionaryStats = append(dictionaryStats, ds)
			}
		}
		sort.Slice(dictionaryStats, func(i, j int) bool {
			a := dictionaryStats[i]
			b := dictionaryStats[j]
			a_ratio := float64(a.Cardinality) / float64(a.TotalEntry)
			b_ratio := float64(b.Cardinality) / float64(b.TotalEntry)
			if a_ratio == b_ratio {
				return a.AvgEntryLength > b.AvgEntryLength
			} else {
				return a_ratio < b_ratio
			}
		})
		var paths [][]int
		for i, ds := range dictionaryStats {
			if i < rbb.config.Dictionaries.StringColumns.MaxSortedDictionaries {
				path := make([]int, len(ds.Path))
				copy(path, ds.Path)
				paths = append(paths, path)
			} else {
				break
			}
		}
		if len(paths) > 0 {
			rbb.orderBy = &OrderBy{
				FieldPaths: paths,
			}
			rbb.optimized = true
			rbb.recordList = []*Record{}
			return true
		}
	}
	return false
}
