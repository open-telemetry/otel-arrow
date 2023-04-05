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
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
	marrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)

func TestGauges(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: "gauges", Type: marrow.UnivariateGaugeDT, Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig, false)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	maxIter := 1

	// Create Arrow record from OTLP univariate gauges
	for {
		smdata := &marrow.ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &marrow.MetricSharedData{Attributes: &common.SharedAttributes{}}
		b := marrow.UnivariateGaugeBuilderFrom(rBuilder.StructBuilder("gauges"))
		for i := 0; i < maxIter; i++ {
			err = b.Append(internal.Gauge1(), smdata, mdata)
			require.NoError(t, err)
			err = b.Append(internal.Gauge2(), smdata, mdata)
			require.NoError(t, err)
		}

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	// Retrieve the Arrow struct representing the gauges
	arr := record.Columns()[0].(*array.Struct)

	// Check the OTLP Arrow encoding and OTLP decoding by
	// comparing the original and decoded gauges.
	row := 0
	smdata := &SharedData{Attributes: pcommon.NewMap()}
	mdata := &SharedData{Attributes: pcommon.NewMap()}
	gaugeIds, err := NewUnivariateGaugeIds(arr.DataType().(*arrow.StructType))
	require.NoError(t, err)
	for i := 0; i < maxIter; i++ {

		value := pmetric.NewGauge()
		err = UpdateUnivariateGaugeFrom(value, arr, row, gaugeIds, smdata, mdata)
		require.NoError(t, err)
		assert.Equal(t, internal.Gauge1(), value)
		row++

		value = pmetric.NewGauge()
		err = UpdateUnivariateGaugeFrom(value, arr, row, gaugeIds, smdata, mdata)
		require.NoError(t, err)
		assert.Equal(t, internal.Gauge2(), value)
		row++
	}
}
