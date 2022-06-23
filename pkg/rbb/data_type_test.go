package rbb

import (
	"github.com/apache/arrow/go/arrow"
	"testing"
)

func TestWriteDataTypeSignature(t *testing.T) {
	// UINT
	sig := DataTypeSignature(arrow.PrimitiveTypes.Uint8)
	if sig != "U8" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.PrimitiveTypes.Uint16)
	if sig != "U16" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.PrimitiveTypes.Uint32)
	if sig != "U32" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.PrimitiveTypes.Uint64)
	if sig != "U64" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	// INT
	sig = DataTypeSignature(arrow.PrimitiveTypes.Int8)
	if sig != "I8" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.PrimitiveTypes.Int16)
	if sig != "I16" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.PrimitiveTypes.Int32)
	if sig != "I32" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.PrimitiveTypes.Int64)
	if sig != "I64" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	sig = DataTypeSignature(arrow.BinaryTypes.String)
	if sig != "Str" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.BinaryTypes.Binary)
	if sig != "Bin" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = DataTypeSignature(arrow.FixedWidthTypes.Boolean)
	if sig != "Bol" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	sig = DataTypeSignature(arrow.ListOfField(arrow.Field{Name: "item", Type: arrow.PrimitiveTypes.Uint8}))
	if sig != "[U8]" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	sig = DataTypeSignature(arrow.StructOf(
		arrow.Field{Name: "c", Type: arrow.PrimitiveTypes.Uint8},
		arrow.Field{Name: "a", Type: arrow.PrimitiveTypes.Int8},
		arrow.Field{Name: "b", Type: arrow.BinaryTypes.String},
		arrow.Field{Name: "e", Type: arrow.StructOf(
			arrow.Field{Name: "g", Type: arrow.BinaryTypes.String},
			arrow.Field{Name: "f", Type: arrow.FixedWidthTypes.Boolean},
		)},
		arrow.Field{Name: "d", Type: arrow.ListOfField(arrow.Field{Name: "item", Type: arrow.PrimitiveTypes.Uint8})},
	))
	if sig != "{a:I8,b:Str,c:U8,d:[U8],e:{f:Bol,g:Str}}" {
		t.Errorf("Unexpected signature: %s", sig)
	}

}
