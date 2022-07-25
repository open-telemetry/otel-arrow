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
	"testing"

	"github.com/apache/arrow/go/v9/arrow"

	"otel-arrow-adapter/pkg/air/rfield"
)

func TestCoerceFromI8(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (&rfield.I8{Value: 1}).DataType()
	dataType2 := (&rfield.I8{Value: 1}).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT8 {
		t.Errorf("Expected INT8, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.U8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.I16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT16 {
		t.Errorf("Expected INT16, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.U16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.I32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.U32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.I64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.U64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.Bool{Value: true}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT8 {
		t.Errorf("Expected INT8, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I8{Value: 1}).DataType()
	dataType2 = (&rfield.String{Value: "bla"}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected String, got %v", dataType.ID())
	}
}

func TestCoerceFromI16(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (&rfield.I16{Value: 1}).DataType()
	dataType2 := (&rfield.I16{Value: 1}).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT16 {
		t.Errorf("Expected INT16, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.U8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.I8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT16 {
		t.Errorf("Expected INT16, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.U16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.I32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.U32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.I64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.U64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.Bool{Value: true}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT16 {
		t.Errorf("Expected INT16, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I16{Value: 1}).DataType()
	dataType2 = (&rfield.String{Value: "bla"}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected String, got %v", dataType.ID())
	}
}

func TestCoerceFromI32(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (&rfield.I32{Value: 1}).DataType()
	dataType2 := (&rfield.I32{Value: 1}).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.U8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.I8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.U16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.I16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.U32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.I64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.U64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.Bool{Value: true}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT32 {
		t.Errorf("Expected INT32, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I32{Value: 1}).DataType()
	dataType2 = (&rfield.String{Value: "bla"}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected String, got %v", dataType.ID())
	}
}

func TestCoerceFromI64(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (&rfield.I64{Value: 1}).DataType()
	dataType2 := (&rfield.I64{Value: 1}).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.U8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.I8{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.U16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.I16{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.U32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.I32{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.U64{Value: 1}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.Bool{Value: true}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.INT64 {
		t.Errorf("Expected INT64, got %v", dataType.ID())
	}

	dataType1 = (&rfield.I64{Value: 1}).DataType()
	dataType2 = (&rfield.String{Value: "bla"}).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}
}
