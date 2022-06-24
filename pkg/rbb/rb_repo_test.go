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
	spew.Dump(rbr)
}
