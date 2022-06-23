package rbb

import (
	"github.com/apache/arrow/go/arrow"
	"testing"
)

func TestCoerceFromBool(t *testing.T) {
	// Test coerce on a scalar value
	dataType1 := (&Bool{Value: true}).DataType()
	dataType2 := (&I8{Value: 1}).DataType()
	dataType := CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT8 {
		t.Errorf("Expected INT8, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&U8{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT8 {
		t.Errorf("Expected UINT8, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&I16{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT16 {
		t.Errorf("Expected INT16, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&U16{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT16 {
		t.Errorf("Expected UINT16, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&I32{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&U32{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&I64{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&U64{Value: 1}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&Bool{Value: true}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.BOOL {
		t.Errorf("Expected BOOL, got %v", dataType.ID())
	}

	dataType1 = (&Bool{Value: true}).DataType()
	dataType2 = (&String{Value: "bla"}).DataType()
	dataType = CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected String, got %v", dataType.ID())
	}
}
