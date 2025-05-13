/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

import (
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/array"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/assert"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/open-telemetry/otel-arrow/pkg/otel/constants"
	"github.com/open-telemetry/otel-arrow/pkg/otel/internal"
)

type ExpectedAttributes struct {
	ParentId uint32
	Attrs    pcommon.Map
}

func TestDynAttrs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	dynAttrs := NewDynAttrsBuilder(PayloadTypes.SpanAttrs, pool)
	defer dynAttrs.Release()

	expected := make(map[uint32]ExpectedAttributes)
	expected[0] = ExpectedAttributes{0, internal.Attrs1()}
	expected[1] = ExpectedAttributes{1, internal.Attrs2()}
	expected[2] = ExpectedAttributes{2, internal.Attrs3()}
	expected[3] = ExpectedAttributes{3, internal.Attrs4()}
	expected[4] = ExpectedAttributes{4, internal.Attrs5()}

	for _, attrs := range expected {
		err := dynAttrs.Append(attrs.ParentId, attrs.Attrs)
		assert.NoError(t, err)
	}

	record, err := dynAttrs.Build(nil)
	defer record.Release()
	assert.NoError(t, err)

	assert.Equal(t, 1, dynAttrs.SchemaUpdateCount())
	assertAttributes(t, expected, record)

	// ------------------------------------------------------------------------
	expected = make(map[uint32]ExpectedAttributes)
	expected[0] = ExpectedAttributes{0, internal.Attrs1()}
	expected[1] = ExpectedAttributes{1, internal.Attrs3()}
	expected[2] = ExpectedAttributes{2, internal.Attrs5()}

	for _, attrs := range expected {
		err := dynAttrs.Append(attrs.ParentId, attrs.Attrs)
		assert.NoError(t, err)
	}

	record, err = dynAttrs.Build(nil)
	defer record.Release()
	assert.NoError(t, err)

	assert.Equal(t, 1, dynAttrs.SchemaUpdateCount())
	assertAttributes(t, expected, record)

	// ------------------------------------------------------------------------
	expected = make(map[uint32]ExpectedAttributes)
	expected[0] = ExpectedAttributes{0, internal.Attrs5()}

	for _, attrs := range expected {
		err := dynAttrs.Append(attrs.ParentId, attrs.Attrs)
		assert.NoError(t, err)
	}

	record, err = dynAttrs.Build(nil)
	defer record.Release()
	assert.NoError(t, err)

	assert.Equal(t, 1, dynAttrs.SchemaUpdateCount())
	assertAttributes(t, expected, record)
}

func assertAttributes(t *testing.T, expected map[uint32]ExpectedAttributes, record arrow.Record) {
	assert.Equal(t, int64(len(expected)), record.NumRows())

	colIdx := make(map[string]int)
	for i, field := range record.Schema().Fields() {
		colIdx[field.Name] = i
	}

	parentID := uint32(0)
	for row := 0; row < int(record.NumRows()); row++ {
		parentIDCol := record.Column(colIdx[constants.ParentID])
		delta := parentIDCol.(*array.Uint32).Value(row)
		parentID += delta
		expectedAttrs := expected[parentID]
		cols := make(map[int]bool)
		for i := 1; i < int(record.NumCols()); i++ {
			cols[i] = true
		}

		// Check attributes values with their corresponding record columns
		expectedAttrs.Attrs.Range(func(k string, v pcommon.Value) bool {
			name, _ := colName(k, v)
			idx := colIdx[name]
			col := record.Column(idx)
			delete(cols, idx)
			switch v.Type() {
			case pcommon.ValueTypeInt:
				ev := v.Int()
				dict := col.(*array.Dictionary)
				av := dict.Dictionary().(*array.Int64).Value(dict.GetValueIndex(row))
				assert.Equal(t, ev, av)
			case pcommon.ValueTypeDouble:
				assert.Equal(t, v.Double(), col.(*array.Float64).Value(row))
			case pcommon.ValueTypeStr:
				ev := v.Str()
				dict := col.(*array.Dictionary)
				av := dict.Dictionary().(*array.String).Value(dict.GetValueIndex(row))
				assert.Equal(t, ev, av)
			case pcommon.ValueTypeBool:
				assert.Equal(t, v.Bool(), col.(*array.Boolean).Value(row))
			case pcommon.ValueTypeMap:
			// todo
			case pcommon.ValueTypeSlice:
			// todo
			case pcommon.ValueTypeBytes:
				ev := v.Bytes().AsRaw()
				dict := col.(*array.Dictionary)
				av := dict.Dictionary().(*array.Binary).Value(dict.GetValueIndex(row))
				assert.Equal(t, ev, av)
			default:
				t.Errorf("unexpected value type: %v", v.Type())
			}
			return true
		})

		// Check that all other columns are null
		for idx := range cols {
			col := record.Column(idx)
			assert.True(t, col.IsNull(row))
		}
	}
}
