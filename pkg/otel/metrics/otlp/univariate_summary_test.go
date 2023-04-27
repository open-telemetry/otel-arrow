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

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
	marrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
)

func TestSummaries(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: "summaries", Type: marrow.UnivariateSummaryDT, Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig, producerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	maxIter := 10

	// Create Arrow record from OTLP univariate summaries
	for {
		smdata := &marrow.ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &marrow.MetricSharedData{Attributes: &common.SharedAttributes{}}
		b := marrow.UnivariateSummaryBuilderFrom(rBuilder.StructBuilder("summaries"))
		for i := 0; i < maxIter; i++ {
			err = b.Append(internal.Summary1(), smdata, mdata)
			require.NoError(t, err)
			err = b.Append(internal.Summary2(), smdata, mdata)
			require.NoError(t, err)
		}

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	// Retrieve the Arrow struct representing the summaries
	arr := record.Columns()[0].(*array.Struct)

	// Check the OTLP Arrow encoding and OTLP decoding by
	// comparing the original and decoded summaries.
	row := 0
	smdata := &SharedData{Attributes: pcommon.NewMap()}
	mdata := &SharedData{Attributes: pcommon.NewMap()}
	summaryIds, err := NewUnivariateSummaryIds(arr.DataType().(*arrow.StructType))
	require.NoError(t, err)
	for i := 0; i < maxIter; i++ {

		value := pmetric.NewSummary()
		err = UpdateUnivariateSummaryFrom(value, arr, row, summaryIds, smdata, mdata)
		require.NoError(t, err)
		assert.Equal(t, internal.Summary1(), value)
		row++

		value = pmetric.NewSummary()
		err = UpdateUnivariateSummaryFrom(value, arr, row, summaryIds, smdata, mdata)
		require.NoError(t, err)
		assert.Equal(t, internal.Summary2(), value)
		row++
	}
}
