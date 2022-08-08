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

package air

import (
	"bytes"
	"fmt"
	"sort"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"github.com/apache/arrow/go/v9/arrow/ipc"
	"github.com/apache/arrow/go/v9/arrow/memory"

	"otel-arrow-adapter/pkg/air/column"
	config2 "otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/air/dictionary"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/air/stats"
)

type OrderBy struct {
	FieldPaths [][]int
}

// A Record builder.
// Must be fed with homogeneous records.
type RecordBuilder struct {
	// The configuration of the builder.
	config *config2.Config

	// The dictionary id generator.
	dictIdGen dictionary.DictIdGenerator

	// The columns of the Record builder.
	columns column.Columns

	// The path for each fields.
	fieldPaths []*rfield.FieldPath

	// Optional order by clause
	orderBy *OrderBy

	// Non ordered records
	recordList []*Record

	// Flag to indicate if the builder has been optimized.
	optimized bool

	output    bytes.Buffer
	ipcWriter *ipc.Writer
}

type RecordBuilderMetadata struct {
	SchemaId        string
	Columns         []*column.ColumnMetadata
	RecordListLen   int
	Optimized       bool
	DictionaryStats []*stats.DictionaryStats
}

// Constructs a new `RecordBuilder` from a Record.
func NewRecordBuilderWithRecord(allocator *memory.GoAllocator, record *Record, config *config2.Config) *RecordBuilder {
	var buf bytes.Buffer

	builder := RecordBuilder{
		config:     config,
		dictIdGen:  dictionary.DictIdGenerator{Id: 0},
		columns:    column.Columns{},
		fieldPaths: make([]*rfield.FieldPath, 0, record.FieldCount()),
		orderBy:    nil,
		recordList: nil,
		optimized:  config.Dictionaries.StringColumns.MaxSortedDictionaries == 0,
		output:     buf,
		ipcWriter:  nil,
	}

	for fieldIdx := range record.fields {
		fieldName := record.fields[fieldIdx].Name
		fieldType := record.fields[fieldIdx].DataType()
		fieldPath := builder.columns.CreateColumn(allocator, []int{fieldIdx}, fieldName, fieldType, config, &builder.dictIdGen)
		builder.columns.UpdateColumn(fieldPath, record.fields[fieldIdx])
		if fieldPath != nil {
			builder.fieldPaths = append(builder.fieldPaths, fieldPath)
		}
	}
	return &builder
}

func (rb *RecordBuilder) AddRecord(record *Record) {
	if rb.recordList != nil {
		rb.recordList = append(rb.recordList, record)
	} else {
		for fieldIdx := range record.fields {
			rb.columns.UpdateColumn(rb.fieldPaths[fieldIdx], record.fields[fieldIdx])
		}
	}
}

func (rb *RecordBuilder) IsEmpty() bool {
	return rb.columns.IsEmpty()
}

func (rb *RecordBuilder) BuildRecord(allocator *memory.GoAllocator) (arrow.Record, error) {
	// Sorts the string columns according to the order by clause.
	if rb.orderBy != nil {
		recordList := rb.recordList
		capacity := 100
		if len(recordList) > capacity {
			capacity = len(recordList)
		}
		rb.recordList = make([]*Record, 0, capacity)
		sortByRecordList(recordList, rb.orderBy)
		for _, record := range recordList {
			for pos := range record.fields {
				rb.columns.UpdateColumn(rb.fieldPaths[pos], record.fields[pos])
			}
		}
	}

	// Creates a column builder for every column.
	fieldRefs, fieldArrays, err := rb.columns.Build(allocator)
	if err != nil {
		return nil, err
	}
	if len(fieldRefs) == 0 {
		return nil, nil
	}

	// Creates an Arrow Schema from the fields returned by the build method.
	fields := make([]arrow.Field, len(fieldRefs))
	for i, fieldRef := range fieldRefs {
		fields[i] = *fieldRef
	}
	schema := arrow.NewSchema(fields, nil)
	cols := make([]arrow.Array, len(fieldRefs))
	rows := int64(0)

	defer func(cols []arrow.Array) {
		for _, col := range cols {
			if col == nil {
				continue
			}
			col.Release()
		}
	}(cols)

	// Creates the Record from the schema and columns.
	for i, fieldArray := range fieldArrays {
		cols[i] = fieldArray
		irow := int64(cols[i].Len())
		if i > 0 && irow != rows {
			panic(fmt.Errorf("arrow/array: field %d has %d rows. want=%d", i, irow, rows))
		}
		rows = irow
	}

	return array.NewRecord(schema, cols, rows), nil
}

func (rb *RecordBuilder) Metadata(schemaId string) *RecordBuilderMetadata {
	recordListLen := 0

	if rb.recordList != nil {
		recordListLen = len(rb.recordList)
	}

	return &RecordBuilderMetadata{
		SchemaId:        schemaId,
		Columns:         rb.columns.Metadata(),
		RecordListLen:   recordListLen,
		Optimized:       rb.optimized,
		DictionaryStats: rb.columns.DictionaryStats(),
	}
}

func (rb *RecordBuilder) DictionaryStats() []*stats.DictionaryStats {
	return rb.columns.DictionaryStats()
}

func (rb *RecordBuilder) OrderBy(fieldPaths [][]int) {
	rb.orderBy = &OrderBy{
		FieldPaths: fieldPaths,
	}
	rb.recordList = []*Record{}
}

func (rb *RecordBuilder) Optimize() bool {
	if rb.optimized {
		return true
	}

	if rb.orderBy == nil {
		var dictionaryStats []*stats.DictionaryStats
		for _, ds := range rb.DictionaryStats() {
			if ds.Cardinality > 1 && rb.config.Dictionaries.StringColumns.IsDictionary(ds.TotalEntry, ds.Cardinality) {
				dictionaryStats = append(dictionaryStats, ds)
			}
		}
		sort.Sort(stats.DictionaryStatsSlice(dictionaryStats))
		var paths [][]int
		for i, ds := range dictionaryStats {
			if i < rb.config.Dictionaries.StringColumns.MaxSortedDictionaries {
				path := make([]int, len(ds.Path))
				copy(path, ds.Path)
				paths = append(paths, path)
			} else {
				break
			}
		}
		if len(paths) > 0 {
			rb.orderBy = &OrderBy{
				FieldPaths: paths,
			}
			rb.optimized = true
			rb.recordList = []*Record{}
			return true
		}
	}
	return false
}

func sortByRecordList(recordList []*Record, orderBy *OrderBy) {
	if orderBy == nil {
		return
	}

	records := Records{
		records: recordList,
		orderBy: orderBy,
	}
	sort.Sort(&records)
}
