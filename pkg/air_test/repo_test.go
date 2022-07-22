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

package air_test

import (
	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"math"
	"math/rand"
	"otel-arrow-adapter/pkg/air"
	config2 "otel-arrow-adapter/pkg/air/config"
	"testing"
	"time"
)

func TestAddRecord(t *testing.T) {
	t.Parallel()

	rr := air.NewRecordRepository(config2.NewDefaultConfig())
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
				if c.Type.ID() == arrow.STRUCT {
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
	rr := air.NewRecordRepository(&config)

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
			if stats.Path[0] != 0 {
				t.Errorf("Expected 0 as first path, got %d", stats.Path[0])
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
			if stats.Path[0] != 1 {
				t.Errorf("Expected 1 as first path, got %d", stats.Path[0])
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
			if stats.Path[0] != 2 {
				t.Errorf("Expected 2 as first path, got %d", stats.Path[0])
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
	rr := air.NewRecordRepository(&config)

	recordCount := 100

	// Generates 100 timestamps randomly sorted.
	tsValues := make([]int64, 0, recordCount)
	for i := 0; i < recordCount; i++ {
		tsValues = append(tsValues, int64(i))
	}
	rand.Seed(time.Now().UnixNano())
	rand.Shuffle(len(tsValues), func(i, j int) { tsValues[i], tsValues[j] = tsValues[j], tsValues[i] })

	// Inserts 100 records
	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	rr.Optimize() // Optimize will determine which string columns must be sorted (first batch only).
	records, err := rr.Build()

	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	// Inserts 100 records
	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	rr.Optimize()
	records, err = rr.Build() // Build will build an Arrow Record with the sorted columns determined in the first batch.

	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	// Columns "b" and "a" must be sorted.
	// "b" because it's cardinality is 2.
	// "a" because it's cardinality is 15 (satisfy the configuration).
	// "c" is not sorted because it's cardinality is 100 (doesn't satisfy the configuration).
	for schemaId, record := range records {
		if schemaId != "a:Str,b:Str,c:Str,d:{a:Str,b:Str,c:[I64]},ts:I64" {
			t.Errorf("Expected schemaId to be a:Str,b:Str,c:Str,d:{a:Str,b:Str},ts:I64, got %s", schemaId)
		}
		if record.NumRows() != int64(recordCount) {
			t.Errorf("Expected %d rows, got %d", recordCount, record.NumRows())
		}
		if record.ColumnName(0) != "ts" {
			t.Errorf("Expected column name to be ts, got %s", record.ColumnName(0))
		}
		if record.ColumnName(1) != "a" {
			t.Errorf("Expected column name to be a, got %s", record.ColumnName(1))
		}
		if record.Column(1).(*array.String).String() != "[\"a_0\" \"a_0\" \"a_0\" \"a_0\" \"a_1\" \"a_1\" \"a_1\" \"a_10\" \"a_10\" \"a_10\" \"a_11\" \"a_11\" \"a_11\" \"a_12\" \"a_12\" \"a_12\" \"a_13\" \"a_13\" \"a_13\" \"a_14\" \"a_14\" \"a_14\" \"a_2\" \"a_2\" \"a_2\" \"a_2\" \"a_3\" \"a_3\" \"a_3\" \"a_4\" \"a_4\" \"a_4\" \"a_4\" \"a_5\" \"a_5\" \"a_5\" \"a_6\" \"a_6\" \"a_6\" \"a_6\" \"a_7\" \"a_7\" \"a_7\" \"a_8\" \"a_8\" \"a_8\" \"a_8\" \"a_9\" \"a_9\" \"a_9\" \"a_0\" \"a_0\" \"a_0\" \"a_1\" \"a_1\" \"a_1\" \"a_1\" \"a_10\" \"a_10\" \"a_10\" \"a_11\" \"a_11\" \"a_11\" \"a_12\" \"a_12\" \"a_12\" \"a_13\" \"a_13\" \"a_13\" \"a_14\" \"a_14\" \"a_14\" \"a_2\" \"a_2\" \"a_2\" \"a_3\" \"a_3\" \"a_3\" \"a_3\" \"a_4\" \"a_4\" \"a_4\" \"a_5\" \"a_5\" \"a_5\" \"a_5\" \"a_6\" \"a_6\" \"a_6\" \"a_7\" \"a_7\" \"a_7\" \"a_7\" \"a_8\" \"a_8\" \"a_8\" \"a_9\" \"a_9\" \"a_9\" \"a_9\"]" {
			t.Errorf("Column a is not sorted as expected")
		}

		if record.ColumnName(2) != "b" {
			t.Errorf("Expected column name to be b, got %s", record.ColumnName(2))
		}
		if record.Column(2).(*array.String).String() != "[\"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__0\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\" \"b__1\"]" {
			t.Errorf("Column b is not sorted as expected")
		}

		if record.ColumnName(3) != "c" {
			t.Errorf("Expected column name to be c, got %s", record.ColumnName(3))
		}

		if record.ColumnName(4) != "d" {
			t.Errorf("Expected column name to be d, got %s", record.ColumnName(4))
		}
		d := record.Column(4).(*array.Struct)
		dA := d.Field(0)
		if dA.(*array.String).String() != "[\"a_0\" \"a_0\" \"a_0\" \"a_0\" \"a_1\" \"a_1\" \"a_1\" \"a_10\" \"a_10\" \"a_10\" \"a_11\" \"a_11\" \"a_11\" \"a_12\" \"a_12\" \"a_12\" \"a_13\" \"a_13\" \"a_13\" \"a_14\" \"a_14\" \"a_14\" \"a_2\" \"a_2\" \"a_2\" \"a_2\" \"a_3\" \"a_3\" \"a_3\" \"a_4\" \"a_4\" \"a_4\" \"a_4\" \"a_5\" \"a_5\" \"a_5\" \"a_6\" \"a_6\" \"a_6\" \"a_6\" \"a_7\" \"a_7\" \"a_7\" \"a_8\" \"a_8\" \"a_8\" \"a_8\" \"a_9\" \"a_9\" \"a_9\" \"a_0\" \"a_0\" \"a_0\" \"a_1\" \"a_1\" \"a_1\" \"a_1\" \"a_10\" \"a_10\" \"a_10\" \"a_11\" \"a_11\" \"a_11\" \"a_12\" \"a_12\" \"a_12\" \"a_13\" \"a_13\" \"a_13\" \"a_14\" \"a_14\" \"a_14\" \"a_2\" \"a_2\" \"a_2\" \"a_3\" \"a_3\" \"a_3\" \"a_3\" \"a_4\" \"a_4\" \"a_4\" \"a_5\" \"a_5\" \"a_5\" \"a_5\" \"a_6\" \"a_6\" \"a_6\" \"a_7\" \"a_7\" \"a_7\" \"a_7\" \"a_8\" \"a_8\" \"a_8\" \"a_9\" \"a_9\" \"a_9\" \"a_9\"]" {
			t.Errorf("Column d.a is not sorted as expected")
		}
		dB := d.Field(1)
		if dB.(*array.String).String() != "[\"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_0\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\" \"b_1\"]" {
			t.Errorf("Column d.b is not sorted as expected")
		}
		dC := d.Field(2)
		if dC.(*array.List).String() != "[[1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3] [1 2 3]]" {
			t.Errorf("Column d.c does not match expected value")
		}

		record.Release()
	}
}
