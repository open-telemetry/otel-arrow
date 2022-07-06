package rbb_test

import (
	"github.com/apache/arrow/go/arrow"
	"github.com/apache/arrow/go/arrow/array"
	"math"
	"math/rand"
	"otel-arrow-adapter/pkg/rbb"
	config2 "otel-arrow-adapter/pkg/rbb/config"
	"testing"
	"time"
)

func TestAddRecord(t *testing.T) {
	t.Parallel()

	rbr := rbb.NewRecordBatchRepository(config2.NewDefaultConfig())
	rbr.AddRecord(GenSimpleRecord(0))
	rbr.AddRecord(GenComplexRecord(1))
	rbr.AddRecord(GenSimpleRecord(2))
	rbr.AddRecord(GenComplexRecord(3))

	if rbr.RecordBatchBuilderCount() != 2 {
		t.Errorf("Expected 2 RecordBatchBuilders, got %d", rbr.RecordBatchBuilderCount())
	}

	metadata := rbr.Metadata()
	for _, m := range metadata {
		switch m.SchemaId {
		case "a:Str,b:Str,c:Str,ts:I64":
			if m.RecordListLen != 0 {
				t.Errorf("Expected 0 RecordBatchCount, got %d", m.RecordListLen)
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
				t.Errorf("Expected 0 RecordBatchCount, got %d", m.RecordListLen)
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
	//spew.Dump(rbr.Metadata())
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
	rbr := rbb.NewRecordBatchRepository(&config)

	for i := 0; i < 100; i++ {
		rbr.AddRecord(GenRecord(int64(i), i%15, i%2, i))
	}
	rbr.Optimize()

	metadata := rbr.Metadata()
	for _, m := range metadata {
		switch m.SchemaId {
		case "a:Str,b:Str,c:Str,ts:I64":
			if m.RecordListLen != 0 {
				t.Errorf("Expected 0 RecordBatchCount, got %d", m.RecordListLen)
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
	//spew.Dump(rbr.Metadata())
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
	rbr := rbb.NewRecordBatchRepository(&config)

	// Generates 100 timestamps randomly sorted.
	tsValues := make([]int64, 0, 100)
	for i := 0; i < 100; i++ {
		tsValues = append(tsValues, int64(i))
	}
	rand.Seed(time.Now().UnixNano())
	rand.Shuffle(len(tsValues), func(i, j int) { tsValues[i], tsValues[j] = tsValues[j], tsValues[i] })

	// Inserts 100 records
	for _, ts := range tsValues {
		rbr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	rbr.Optimize() // Optimize will determine which string columns must be sorted (first batch only).
	records, err := rbr.Build()

	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	// Inserts 100 records
	for _, ts := range tsValues {
		rbr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	rbr.Optimize()
	records, err = rbr.Build() // Build will build an Arrow Record with the sorted columns determined in the first batch.

	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	// Columns "b" and "a" must be sorted.
	// "b" because it's cardinality is 2.
	// "a" because it's cardinality is 15 (satisfy the configuration).
	// "c" is not sorted because it's cardinality is 100 (doesn't satisfy the configuration).
	for schemaId, record := range records {
		if schemaId != "a:Str,b:Str,c:Str,ts:I64" {
			t.Errorf("Expected schemaId to be a:Str,b:Str,c:Str,ts:I64, got %s", schemaId)
		}
		if record.NumRows() != 100 {
			t.Errorf("Expected 100 rows, got %d", record.NumRows())
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

		//spew.Dump(record)
		record.Release()
	}
}
