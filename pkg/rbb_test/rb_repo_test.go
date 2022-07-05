package rbb_test

import (
	"github.com/apache/arrow/go/arrow"
	"math"
	"otel-arrow-adapter/pkg/rbb"
	config2 "otel-arrow-adapter/pkg/rbb/config"
	"testing"
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
