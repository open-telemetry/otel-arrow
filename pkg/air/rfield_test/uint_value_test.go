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

	"github.com/apache/arrow/go/v11/arrow"

	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
)

func TestCoerceFromU8(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (rfield.NewU8(1)).DataType()
	dataType2 := (rfield.NewI8(1)).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewU8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT8 {
		t.Errorf("Expected UINT8, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewI16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewU16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT16 {
		t.Errorf("Expected UINT16, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewI32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewU32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewI64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewU64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewBool(true)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT8 {
		t.Errorf("Expected UINT8, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU8(1)).DataType()
	dataType2 = (rfield.NewString("bla")).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected String, got %v", dataType.ID())
	}
}

func TestCoerceFromU16(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (rfield.NewU16(1)).DataType()
	dataType2 := (rfield.NewI16(1)).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewU8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT16 {
		t.Errorf("Expected UINT16, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewI8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewU16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT16 {
		t.Errorf("Expected UINT16, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewI32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewU32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewI64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewU64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewBool(true)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT16 {
		t.Errorf("Expected UINT16, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU16(1)).DataType()
	dataType2 = (rfield.NewString("bla")).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}
}

func TestCoerceFromU32(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (rfield.NewU32(1)).DataType()
	dataType2 := (rfield.NewI32(1)).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewU8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewI8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewU16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewI16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewU32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewI64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewU64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewBool(true)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT32 {
		t.Errorf("Expected UINT32, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU32(1)).DataType()
	dataType2 = (rfield.NewString("bla")).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}
}

func TestCoerceFromU64(t *testing.T) {
	t.Parallel()

	// Test coerce on a scalar value
	dataType1 := (rfield.NewU64(1)).DataType()
	dataType2 := (rfield.NewI64(1)).DataType()
	dataType := rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewU8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewI8(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewU16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewI16(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewU32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewI32(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewU64(1)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewBool(true)).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.UINT64 {
		t.Errorf("Expected UINT64, got %v", dataType.ID())
	}

	dataType1 = (rfield.NewU64(1)).DataType()
	dataType2 = (rfield.NewString("bla")).DataType()
	dataType = rfield.CoerceDataTypes(dataType1, dataType2)
	if dataType.ID() != arrow.STRING {
		t.Errorf("Expected STRING, got %v", dataType.ID())
	}
}
