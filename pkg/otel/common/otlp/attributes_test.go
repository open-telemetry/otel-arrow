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
	"testing"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pcommon"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

func TestAttributes(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: "attributes", Type: carrow.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	maxIter := 10

	// Create Arrow record from OTLP attributes
	for {
		b := carrow.AttributesBuilderFrom(rBuilder.MapBuilder("attributes"))
		for i := 0; i < maxIter; i++ {
			err = b.Append(internal.Attrs1())
			require.NoError(t, err)
			err = b.Append(internal.Attrs2())
			require.NoError(t, err)
			err = b.Append(internal.Attrs3())
			require.NoError(t, err)
			err = b.Append(internal.Attrs4())
			require.NoError(t, err)
			err = b.Append(internal.Attrs5())
			require.NoError(t, err)
		}

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	// Retrieve the Arrow Map representing the attributes
	arr := record.Columns()[0].(*array.Map)

	// Check the OTLP Arrow encoding and OTLP decoding by
	// comparing the original and decoded attributes.
	row := 0
	for i := 0; i < maxIter; i++ {

		value := pcommon.NewMap()
		err = UpdateAttributesFrom(value, arr, row)
		require.NoError(t, err)
		assert.Equal(t, internal.Attrs1(), value)
		row++

		value = pcommon.NewMap()
		err = UpdateAttributesFrom(value, arr, row)
		require.NoError(t, err)
		assert.Equal(t, internal.Attrs2(), value)
		row++

		value = pcommon.NewMap()
		err = UpdateAttributesFrom(value, arr, row)
		require.NoError(t, err)
		assert.Equal(t, internal.Attrs3(), value)
		row++

		value = pcommon.NewMap()
		err = UpdateAttributesFrom(value, arr, row)
		require.NoError(t, err)
		assert.Equal(t, internal.Attrs4(), value)
		row++

		value = pcommon.NewMap()
		err = UpdateAttributesFrom(value, arr, row)
		require.NoError(t, err)
		assert.Equal(t, internal.Attrs5(), value)
		row++
	}
}
