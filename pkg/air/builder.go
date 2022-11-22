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
	"io"
	"sort"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/ipc"
	"github.com/apache/arrow/go/v11/arrow/memory"

	"github.com/f5/otel-arrow-adapter/pkg/air/column"
	config2 "github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/dictionary"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/air/stats"
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

	fieldDataTypes map[string]arrow.DataType

	// The columns of the Record builder.
	columns column.Columns

	// The path for each fields.
	fieldPaths []*rfield.FieldPath

	// Optional order by clause
	orderByPath *OrderBy

	orderByClause []string

	// Non ordered records
	recordList []*Record

	// Flag to indicate if the builder has been optimized.
	optimized bool

	output    bytes.Buffer
	ipcWriter *ipc.Writer
}

type RecordBuilderMetadata struct {
	SchemaId        string
	Columns         map[string]*column.ColumnMetadata
	RecordListLen   int
	Optimized       bool
	OrderBy         []string
	DictionaryStats []*stats.DictionaryStats
}

// Constructs a new `RecordBuilder` from a Record.
// Important: the record is supposed to be normalized before the call to this method.
func NewRecordBuilderWithRecord(allocator *memory.GoAllocator, record *Record, config *config2.Config) *RecordBuilder {
	var buf bytes.Buffer

	builder := RecordBuilder{
		config:         config,
		dictIdGen:      dictionary.DictIdGenerator{Id: 0},
		fieldDataTypes: record.FieldDataTypes(),
		columns:        column.Columns{},
		fieldPaths:     make([]*rfield.FieldPath, 0, record.FieldCount()),
		orderByPath:    nil,
		recordList:     nil,
		optimized:      config.Dictionaries.StringColumns.MaxSortedDictionaries == 0,
		output:         buf,
		ipcWriter:      nil,
	}

	for fieldIdx, field := range record.fields {
		fieldName := field.Name
		fieldType := builder.fieldDataTypes[fieldName]
		fieldMetadata := field.Metadata()
		var arrowMetadata arrow.Metadata
		if fieldMetadata != nil {
			arrowMetadata = arrow.NewMetadata(fieldMetadata.Keys, fieldMetadata.Values)
		}

		stringPath := fieldName
		numPath := builder.columns.CreateColumn(allocator, []int{fieldIdx}, fieldName, stringPath, fieldType, arrowMetadata, config, &builder.dictIdGen)
		builder.columns.UpdateColumn(numPath, fieldType, record.fields[fieldIdx])
		if numPath != nil {
			builder.fieldPaths = append(builder.fieldPaths, numPath)
		}
	}
	return &builder
}

func (rb *RecordBuilder) AddRecord(record *Record) {
	if rb.recordList != nil {
		rb.recordList = append(rb.recordList, record)
	} else {
		for fieldIdx := range record.fields {
			field := record.fields[fieldIdx]
			rb.columns.UpdateColumn(rb.fieldPaths[fieldIdx], rb.fieldDataTypes[field.Name], field)
		}
	}
}

func (rb *RecordBuilder) IsEmpty() bool {
	return rb.columns.IsEmpty()
}

func (rb *RecordBuilder) BuildRecord(allocator *memory.GoAllocator) (arrow.Record, error) {
	// Sorts the string columns according to the order by clause.
	if rb.orderByPath != nil {
		recordList := rb.recordList
		capacity := 100
		if len(recordList) > capacity {
			capacity = len(recordList)
		}
		rb.recordList = make([]*Record, 0, capacity)
		sortByRecordList(recordList, rb.orderByPath)
		for _, record := range recordList {
			for pos := range record.fields {
				field := record.fields[pos]
				rb.columns.UpdateColumn(rb.fieldPaths[pos], rb.fieldDataTypes[field.Name], field)
			}
		}
	}

	// Creates a column builder for every column.
	fieldArrays, err := rb.columns.NewArrays(allocator)
	if err != nil {
		return nil, err
	}
	fieldRefs := rb.columns.NewArrowFields()
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

	columnsMetadata := rb.columns.Metadata()

	return &RecordBuilderMetadata{
		SchemaId:        schemaId,
		Columns:         columnsMetadata,
		RecordListLen:   recordListLen,
		Optimized:       rb.optimized,
		OrderBy:         rb.orderByClause,
		DictionaryStats: rb.columns.DictionaryStats(""),
	}
}

func (rb *RecordBuilder) DictionaryStats() []*stats.DictionaryStats {
	return rb.columns.DictionaryStats("")
}

func (rb *RecordBuilder) OrderBy(fieldPaths [][]int) {
	rb.orderByPath = &OrderBy{
		FieldPaths: fieldPaths,
	}
	rb.recordList = []*Record{}
}

func (rb *RecordBuilder) Optimize() bool {
	if rb.optimized {
		return true
	}

	if rb.orderByPath == nil {
		var dictionaryStats []*stats.DictionaryStats
		for _, ds := range rb.DictionaryStats() {
			if ds == nil || ds.Cardinality <= 1 {
				continue
			}

			if ds.Type == stats.StringDic && rb.config.Dictionaries.StringColumns.IsDictionary(ds.TotalEntry, ds.Cardinality, ds.TotalValueLength) {
				dictionaryStats = append(dictionaryStats, ds)
			} else if ds.Type == stats.BinaryDic && rb.config.Dictionaries.BinaryColumns.IsDictionary(ds.TotalEntry, ds.Cardinality, ds.TotalValueLength) {
				dictionaryStats = append(dictionaryStats, ds)
			}
		}
		if len(dictionaryStats) == 0 {
			rb.optimized = false
			return false
		}
		sort.Sort(stats.DictionaryStatsSlice(dictionaryStats))
		var numPaths [][]int
		var stringPaths []string
		for i, ds := range dictionaryStats {
			if i < rb.config.Dictionaries.StringColumns.MaxSortedDictionaries {
				path := make([]int, len(ds.NumPath))
				copy(path, ds.NumPath)
				numPaths = append(numPaths, path)
				stringPaths = append(stringPaths, ds.StringPath)
			} else {
				break
			}
		}
		if len(numPaths) > 0 {
			rb.orderByPath = &OrderBy{
				FieldPaths: numPaths,
			}
			orderByClause := make([]string, 0, len(numPaths))
			orderByClause = append(orderByClause, stringPaths...)
			rb.orderByClause = orderByClause
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

func (m *RecordBuilderMetadata) Dump(f io.Writer) {
	_, err := fmt.Fprintf(f, "Arrow Record Schema:\n")
	if err != nil {
		panic(err)
	}
	for _, col := range m.Columns {
		col.Dump("\t", f)
	}
	_, err = fmt.Fprintf(f, "Optimized: %t\n", m.Optimized)
	if err != nil {
		panic(err)
	}
	if m.OrderBy != nil {
		_, err = fmt.Fprintf(f, "OrderBy: %v\n", m.OrderBy)
		if err != nil {
			panic(err)
		}
	}
}
