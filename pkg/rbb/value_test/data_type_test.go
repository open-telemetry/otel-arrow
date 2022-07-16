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

package value_test

import (
	"github.com/apache/arrow/go/v9/arrow"
	"otel-arrow-adapter/pkg/rbb/rfield"
	"testing"
)

func TestWriteDataTypeSignature(t *testing.T) {
	t.Parallel()

	// UINT
	sig := rfield.DataTypeSignature(arrow.PrimitiveTypes.Uint8)
	if sig != "U8" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Uint16)
	if sig != "U16" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Uint32)
	if sig != "U32" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Uint64)
	if sig != "U64" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	// INT
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Int8)
	if sig != "I8" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Int16)
	if sig != "I16" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Int32)
	if sig != "I32" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.PrimitiveTypes.Int64)
	if sig != "I64" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	sig = rfield.DataTypeSignature(arrow.BinaryTypes.String)
	if sig != "Str" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.BinaryTypes.Binary)
	if sig != "Bin" {
		t.Errorf("Unexpected signature: %s", sig)
	}
	sig = rfield.DataTypeSignature(arrow.FixedWidthTypes.Boolean)
	if sig != "Bol" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	sig = rfield.DataTypeSignature(arrow.ListOfField(arrow.Field{Name: "item", Type: arrow.PrimitiveTypes.Uint8}))
	if sig != "[U8]" {
		t.Errorf("Unexpected signature: %s", sig)
	}

	sig = rfield.DataTypeSignature(arrow.StructOf(
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
