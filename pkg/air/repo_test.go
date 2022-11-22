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
	"fmt"
	"math"
	"math/rand"
	"testing"
	"time"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/davecgh/go-spew/spew"
	"github.com/stretchr/testify/require"

	config2 "github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	arrow2 "github.com/f5/otel-arrow-adapter/pkg/arrow"
)

func TestAddRecord(t *testing.T) {
	t.Parallel()

	rr := NewRecordRepository(config2.NewUint8DefaultConfig())
	rr.AddRecord(GenSimpleRecord(0))
	rr.AddRecord(GenComplexRecord(1))
	rr.AddRecord(GenSimpleRecord(2))
	rr.AddRecord(GenComplexRecord(3))

	if rr.RecordBuilderCount() != 2 {
		t.Errorf("Expected 2 RecordBuilders, got %d", rr.RecordBuilderCount())
	}

	metadata := rr.Metadata()
	for _, m := range metadata {
		switch m.SchemaId {
		case "a:Str,b:Str,c:Str,ts:I64":
			if m.RecordListLen != 0 {
				t.Errorf("Expected 0 RecordCount, got %d", m.RecordListLen)
			}
			if len(m.Columns) != 4 {
				t.Errorf("Expected 4 columns, got %d", len(m.Columns))
			}
			for _, c := range m.Columns {
				if c.Len != 2 {
					t.Errorf("Expected 2 values, got %d", c.Len)
				}
			}
		case "a:{b:Str,c:Str},b:Str,c:Str,ts:I64":
			if m.RecordListLen != 0 {
				t.Errorf("Expected 0 RecordCount, got %d", m.RecordListLen)
			}
			if len(m.Columns) != 4 {
				t.Errorf("Expected 4 columns, got %d", len(m.Columns))
			}
			for _, c := range m.Columns {
				if c.Field.Type.ID() == arrow.STRUCT {
					if len(c.Children) != 2 {
						t.Errorf("Expected 2 children, got %d", len(c.Children))
					}
					for _, children := range c.Children {
						if children.Len != 2 {
							t.Errorf("Expected 2 values, got %d", children.Len)
						}
					}
				} else if c.Len != 2 {
					t.Errorf("Expected 2 values, got %d", c.Len)
				}
			}
		}
	}
	//spew.Dump(rr.Metadata())
}

func TestOptimize(t *testing.T) {
	t.Parallel()

	config := config2.Config{
		Dictionaries: config2.DictionariesConfig{
			StringColumns: config2.DictionaryConfig{
				MinRowCount:           10,
				MaxCard:               math.MaxUint8,
				MaxCardRatio:          0.5,
				MaxSortedDictionaries: 5,
			},
		},
	}
	rr := NewRecordRepository(&config)

	for i := 0; i < 100; i++ {
		rr.AddRecord(GenRecord(int64(i), i%15, i%2, i))
	}
	rr.Optimize()

	metadata := rr.Metadata()
	for _, m := range metadata {
		switch m.SchemaId {
		case "a:Str,b:Str,c:Str,ts:I64":
			if m.RecordListLen != 0 {
				t.Errorf("Expected 0 RecordCount, got %d", m.RecordListLen)
			}
			if len(m.Columns) != 4 {
				t.Errorf("Expected 4 columns, got %d", len(m.Columns))
			}
			for _, c := range m.Columns {
				if c.Len != 100 {
					t.Errorf("Expected 100 values, got %d", c.Len)
				}
			}

			// Check column "a"
			stats := m.DictionaryStats[0]
			if stats.NumPath[0] != 0 {
				t.Errorf("Expected 0 as first path, got %d", stats.NumPath[0])
			}
			if stats.AvgEntryLength != 3.3 {
				t.Errorf("Expected 3.3, got %f", stats.AvgEntryLength)
			}
			if stats.Cardinality != 15 {
				t.Errorf("Expected 15 cardinality, got %d", stats.Cardinality)
			}
			if stats.TotalEntry != 100 {
				t.Errorf("Expected 100 total entry, got %d", stats.TotalEntry)
			}

			// Check column "b"
			stats = m.DictionaryStats[1]
			if stats.NumPath[0] != 1 {
				t.Errorf("Expected 1 as first path, got %d", stats.NumPath[0])
			}
			if stats.AvgEntryLength != 4 {
				t.Errorf("Expected 4, got %f", stats.AvgEntryLength)
			}
			if stats.Cardinality != 2 {
				t.Errorf("Expected 2 cardinality, got %d", stats.Cardinality)
			}
			if stats.TotalEntry != 100 {
				t.Errorf("Expected 100 total entry, got %d", stats.TotalEntry)
			}

			// Check column "c"
			stats = m.DictionaryStats[2]
			if stats.NumPath[0] != 2 {
				t.Errorf("Expected 2 as first path, got %d", stats.NumPath[0])
			}
			if stats.AvgEntryLength != 5.9 {
				t.Errorf("Expected 5.9, got %f", stats.AvgEntryLength)
			}
			if stats.Cardinality != 100 {
				t.Errorf("Expected 100 cardinality, got %d", stats.Cardinality)
			}
			if stats.TotalEntry != 100 {
				t.Errorf("Expected 100 total entry, got %d", stats.TotalEntry)
			}
		}
	}
	//spew.Dump(rr.Metadata())
}

func TestBuild(t *testing.T) {
	t.Parallel()

	config := config2.Config{
		Dictionaries: config2.DictionariesConfig{
			StringColumns: config2.DictionaryConfig{
				MinRowCount:           10,
				MaxCard:               math.MaxUint8,
				MaxCardRatio:          0.5,
				MaxSortedDictionaries: 5,
			},
		},
	}
	rr := NewRecordRepository(&config)

	recordCount := 100

	// Generates 100 timestamps randomly sorted.
	tsValues := make([]int64, 0, recordCount)
	for i := 0; i < recordCount; i++ {
		tsValues = append(tsValues, int64(i))
	}
	rand.Seed(time.Now().UnixNano())
	rand.Shuffle(len(tsValues), func(i, j int) { tsValues[i], tsValues[j] = tsValues[j], tsValues[i] })

	// Inserts `recordCount` records
	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	_, err := rr.BuildRecords()
	require.NoError(t, err)

	// Inserts `recordCount` records again, so the optimizations (e.g. sorting) will be applied.
	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	records, err := rr.BuildRecords() // BuildRecord will build an Arrow Record with the sorted columns determined in the first batch.
	require.NoError(t, err)

	// Columns "b" and "a" must be sorted.
	// "b" because it's cardinality is 2.
	// "a" because it's cardinality is 15 (satisfy the configuration).
	// "c" is not sorted because it's cardinality is 100 (doesn't satisfy the configuration).
	for _, record := range records {
		schemaId := arrow2.SchemaToId(record.Schema())
		if schemaId != "a:Dic<U8,Str>,b:Dic<U8,Str>,c:Dic<U8,Str>,d:{a:Dic<U8,Str>,b:Dic<U8,Str>,c:[I64],d:[{a:I64,b:F64,c:Dic<U8,Str>}]},ts:I64" {
			t.Errorf("Expected schemaId to be a:Str,b:Str,c:Str,d:{a:Str,b:Str},ts:I64, got %s", schemaId)
		}
		if record.NumRows() != int64(recordCount) {
			t.Errorf("Expected %d rows, got %d", recordCount, record.NumRows())
		}
		d := record.Column(4).(*array.Struct)
		if record.ColumnName(0) != "ts" {
			t.Errorf("Expected column name to be ts, got %s", record.ColumnName(0))
		}

		// Field b
		if record.ColumnName(2) != "b" {
			t.Errorf("Expected column name to be b, got %s", record.ColumnName(2))
		}
		if String(record.Column(2).(*array.Dictionary)) != "[\"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\"]" {
			t.Errorf("Column b is not sorted as expected %s", String(record.Column(2).(*array.Dictionary)))
		}
		// Field d.b
		dB := d.Field(1)
		if String(dB.(*array.Dictionary)) != "[\"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\"]" {
			t.Errorf("Column d.b is not sorted as expected %s", String(dB.(*array.Dictionary)))
		}
		// Field a
		if record.ColumnName(1) != "a" {
			t.Errorf("Expected column name to be a, got %s", record.ColumnName(1))
		}
		if String(record.Column(1).(*array.Dictionary)) != "[\"a___0\" \"a___0\" \"a___0\" \"a___0\" \"a___1\" \"a___1\" \"a___1\" \"a___10\" \"a___10\" \"a___10\" \"a___11\" \"a___11\" \"a___11\" \"a___12\" \"a___12\" \"a___12\" \"a___13\" \"a___13\" \"a___13\" \"a___14\" \"a___14\" \"a___14\" \"a___2\" \"a___2\" \"a___2\" \"a___2\" \"a___3\" \"a___3\" \"a___3\" \"a___4\" \"a___4\" \"a___4\" \"a___4\" \"a___5\" \"a___5\" \"a___5\" \"a___6\" \"a___6\" \"a___6\" \"a___6\" \"a___7\" \"a___7\" \"a___7\" \"a___8\" \"a___8\" \"a___8\" \"a___8\" \"a___9\" \"a___9\" \"a___9\" \"a___0\" \"a___0\" \"a___0\" \"a___1\" \"a___1\" \"a___1\" \"a___1\" \"a___10\" \"a___10\" \"a___10\" \"a___11\" \"a___11\" \"a___11\" \"a___12\" \"a___12\" \"a___12\" \"a___13\" \"a___13\" \"a___13\" \"a___14\" \"a___14\" \"a___14\" \"a___2\" \"a___2\" \"a___2\" \"a___3\" \"a___3\" \"a___3\" \"a___3\" \"a___4\" \"a___4\" \"a___4\" \"a___5\" \"a___5\" \"a___5\" \"a___5\" \"a___6\" \"a___6\" \"a___6\" \"a___7\" \"a___7\" \"a___7\" \"a___7\" \"a___8\" \"a___8\" \"a___8\" \"a___9\" \"a___9\" \"a___9\" \"a___9\"]" {
			t.Errorf("Column a is not sorted as expected %s", String(record.Column(1).(*array.Dictionary)))
		}
		// Field d.a
		dA := d.Field(0)
		if String(dA.(*array.Dictionary)) != "[\"a_0\" \"a_0\" \"a_0\" \"a_0\" \"a_1\" \"a_1\" \"a_1\" \"a_10\" \"a_10\" \"a_10\" \"a_11\" \"a_11\" \"a_11\" \"a_12\" \"a_12\" \"a_12\" \"a_13\" \"a_13\" \"a_13\" \"a_14\" \"a_14\" \"a_14\" \"a_2\" \"a_2\" \"a_2\" \"a_2\" \"a_3\" \"a_3\" \"a_3\" \"a_4\" \"a_4\" \"a_4\" \"a_4\" \"a_5\" \"a_5\" \"a_5\" \"a_6\" \"a_6\" \"a_6\" \"a_6\" \"a_7\" \"a_7\" \"a_7\" \"a_8\" \"a_8\" \"a_8\" \"a_8\" \"a_9\" \"a_9\" \"a_9\" \"a_0\" \"a_0\" \"a_0\" \"a_1\" \"a_1\" \"a_1\" \"a_1\" \"a_10\" \"a_10\" \"a_10\" \"a_11\" \"a_11\" \"a_11\" \"a_12\" \"a_12\" \"a_12\" \"a_13\" \"a_13\" \"a_13\" \"a_14\" \"a_14\" \"a_14\" \"a_2\" \"a_2\" \"a_2\" \"a_3\" \"a_3\" \"a_3\" \"a_3\" \"a_4\" \"a_4\" \"a_4\" \"a_5\" \"a_5\" \"a_5\" \"a_5\" \"a_6\" \"a_6\" \"a_6\" \"a_7\" \"a_7\" \"a_7\" \"a_7\" \"a_8\" \"a_8\" \"a_8\" \"a_9\" \"a_9\" \"a_9\" \"a_9\"]" {
			t.Errorf("Column d.a is not sorted as expected %s", String(dA.(*array.Dictionary)))
		}

		if record.ColumnName(3) != "c" {
			t.Errorf("Expected column name to be c, got %s", record.ColumnName(3))
		}

		if record.ColumnName(4) != "d" {
			t.Errorf("Expected column name to be d, got %s", record.ColumnName(4))
		}
		dC := d.Field(2)
		if dC.(*array.List).String() != "[[1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3]]" {
			t.Errorf("Column d.c does not match expected value")
		}
		dD := d.Field(3)
		if dD.(*array.List).String() != "[{[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }} {[1 4 7] [2 5 8] { dictionary: [\"3\" \"6\" \"9\"]\n  indices: [0 1 2] }}]" {
			t.Errorf("Column d.d does not match expected value (got %q)", dD.(*array.List).String())
		}

		record.Release()
	}
}

func TestBuildHeterogeneousListOfStructs(t *testing.T) {
	t.Parallel()

	config := config2.NewUint8DefaultConfig()
	rr := NewRecordRepository(config)

	rr.AddRecord(RecordWithHeterogeneousListOfStructs(1))

	records, err := rr.BuildRecords()

	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}
	spew.Dump(records)
}

func RecordWithHeterogeneousListOfStructs(ts uint64) *Record {
	record := NewRecord()
	record.U64Field("ts", ts)
	record.ListField("events", rfield.List{
		Values: []rfield.Value{
			&rfield.Struct{
				Fields: []*rfield.Field{
					{Name: "name", Value: rfield.NewString("event1.name")},
					{Name: "attributes", Value: &rfield.Struct{
						Fields: []*rfield.Field{
							{Name: "attr1", Value: rfield.NewString("event1.a1")},
							{Name: "attr2", Value: rfield.NewI8(2)},
							{Name: "attr4", Value: rfield.NewI32(4)},
							{Name: "attr7", Value: rfield.NewU16(7)},
							{Name: "attr9", Value: rfield.NewU64(9)},
							{Name: "attr11", Value: rfield.NewF64(11.0)},
							{Name: "attr13", Value: rfield.NewStruct([]*rfield.Field{
								{Name: "attr13_1", Value: rfield.NewString("event1.attr13.attr13_13")},
								{Name: "attr13_2", Value: rfield.NewI8(13)},
							})},
						},
					}},
				},
			},
			&rfield.Struct{
				Fields: []*rfield.Field{
					{Name: "name", Value: rfield.NewString("event2.name")},
					{Name: "attributes", Value: &rfield.Struct{
						Fields: []*rfield.Field{
							{Name: "attr1", Value: rfield.NewString("event2.a1")},
							{Name: "attr3", Value: rfield.NewI16(3)},
							{Name: "attr5", Value: rfield.NewI64(5)},
							{Name: "attr6", Value: rfield.NewU8(6)},
							{Name: "attr8", Value: rfield.NewU32(8)},
							{Name: "attr10", Value: rfield.NewF32(10.0)},
							{Name: "attr12", Value: rfield.NewBinary([]byte("12"))},
							{Name: "attr14", Value: rfield.UnsafeNewList(&arrow.Int8Type{}, []rfield.Value{
								rfield.NewI8(14),
								rfield.NewI8(14),
								&rfield.I8{
									Value: nil,
								},
							})},
						},
					}},
				},
			},
			&rfield.Struct{
				Fields: []*rfield.Field{
					{Name: "name", Value: rfield.NewString("event3.name")},
				},
			},
		},
	})
	return record
}

func String(dictionary *array.Dictionary) string {
	indices := dictionary.Indices().(*array.Uint8)
	dicValues := dictionary.Dictionary().(*array.String)
	output := "["
	for i := 0; i < dictionary.Len(); i++ {
		if i > 0 {
			output += " "
		}
		output += fmt.Sprintf("\"%s\"", dicValues.Value(int(indices.Value(i))))
	}
	output += "]"
	return output
}
