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

package otlp

import (
	"math"
	"testing"

	"github.com/apache/arrow/go/v17/arrow"
	"github.com/apache/arrow/go/v17/arrow/array"
	"github.com/apache/arrow/go/v17/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pcommon"

	carrow "github.com/open-telemetry/otel-arrow/pkg/otel/common/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/builder"
	cfg "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16, 0.0)
var producerStats = stats.NewProducerStats()

func TestEmptyAnyValue(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: "any_value", Type: carrow.AnyValueDT, Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig, producerStats, nil)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	for {
		b := carrow.AnyValueBuilderFrom(rBuilder.SparseUnionBuilder("any_value"))
		v := pcommon.NewValueEmpty()
		err = b.Append(&v)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	defer record.Release()

	expected := `[]`

	require.JSONEq(t, expected, string(json))
}

func TestMultipleAnyValues(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: "any_value", Type: carrow.AnyValueDT, Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig, producerStats, nil)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	for {
		b := carrow.AnyValueBuilderFrom(rBuilder.SparseUnionBuilder("any_value"))
		v := pcommon.NewValueBool(true)
		err := b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueInt(10)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueBool(false)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueBool(true)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueBool(true)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueStr("string")
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueInt(0)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueDouble(0.0)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueBool(false)
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueStr("")
		err = b.Append(&v)
		require.NoError(t, err)
		v = pcommon.NewValueBytes()
		err = b.Append(&v)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	arr := record.Columns()[0].(*array.SparseUnion)
	value := pcommon.NewValueEmpty()
	err = UpdateValueFrom(value, arr, 0)
	require.NoError(t, err)
	assert.True(t, value.Bool())

	err = UpdateValueFrom(value, arr, 1)
	require.NoError(t, err)
	assert.Equal(t, int64(10), value.Int())

	err = UpdateValueFrom(value, arr, 2)
	require.NoError(t, err)
	assert.False(t, value.Bool())

	err = UpdateValueFrom(value, arr, 3)
	require.NoError(t, err)
	assert.True(t, value.Bool())

	err = UpdateValueFrom(value, arr, 4)
	require.NoError(t, err)
	assert.True(t, value.Bool())

	err = UpdateValueFrom(value, arr, 5)
	require.NoError(t, err)
	assert.Equal(t, "string", value.Str())

	err = UpdateValueFrom(value, arr, 6)
	require.NoError(t, err)
	assert.Equal(t, int64(0), value.Int())

	err = UpdateValueFrom(value, arr, 7)
	require.NoError(t, err)
	assert.Equal(t, float64(0.0), value.Double())

	err = UpdateValueFrom(value, arr, 8)
	require.NoError(t, err)
	assert.Equal(t, false, value.Bool())

	err = UpdateValueFrom(value, arr, 9)
	require.NoError(t, err)
	assert.Equal(t, "", value.Str())

	err = UpdateValueFrom(value, arr, 10)
	require.NoError(t, err)
	assert.Equal(t, pcommon.NewByteSlice(), value.Bytes())
}
