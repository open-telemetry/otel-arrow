package rbb

import (
	"github.com/davecgh/go-spew/spew"
	"testing"
)

func TestAddRecord(t *testing.T) {
	rbr := NewRecordBatchRepository(NewDefaultConfig())
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
				if c.Len != 2 {
					t.Errorf("Expected 2 values, got %d", c.Len)
				}
			}
		}
	}

	spew.Dump(rbr.Metadata())
}
