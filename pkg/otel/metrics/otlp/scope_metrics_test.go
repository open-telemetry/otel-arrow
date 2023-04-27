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
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
	"golang.org/x/exp/rand"

	carrow "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	jsonassert "github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
	marrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
)

func TestScopeMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: constants.ScopeMetrics, Type: arrow.ListOf(marrow.ScopeMetricsDT), Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig, producerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	maxIter := 10

	// Create Arrow record from OTLP scope metrics.
	for {
		lb := rBuilder.ListBuilder(constants.ScopeMetrics)
		b := marrow.ScopeMetricsBuilderFrom(lb.StructBuilder())
		for i := 0; i < maxIter; i++ {
			err := lb.Append(2, func() error {
				err = b.Append(ToScopeMetricsGroup(internal.ScopeMetrics3()))
				require.NoError(t, err)
				err = b.Append(ToScopeMetricsGroup(internal.ScopeMetrics4()))
				require.NoError(t, err)
				return nil
			})
			require.NoError(t, err)
		}

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	// Retrieve the Arrow struct representing the scope metrics.
	arr := record.Columns()[0].(*array.List)

	// Check the OTLP Arrow encoding and OTLP decoding by
	// comparing the original and decoded scope metrics.
	row := 0
	scopeMetricsIds, err := NewScopeMetricsIds(arr.DataType().(*arrow.ListType).ElemField().Type.(*arrow.StructType))
	require.NoError(t, err)
	for i := 0; i < maxIter; i++ {

		los, err := carrow.ListOfStructsFromArray(arr, row)
		require.NoError(t, err)
		value := pmetric.NewResourceMetrics().ScopeMetrics()
		err = UpdateScopeMetricsFrom(value, los, scopeMetricsIds)
		require.NoError(t, err)
		assert.Equal(t, 2, value.Len())
		AssertJSONEq(t, internal.ScopeMetrics3(), value.At(0))
		AssertJSONEq(t, internal.ScopeMetrics4(), value.At(1))
		row++
	}
}

func ToScopeMetricsGroup(scopeMetrics pmetric.ScopeMetrics) *marrow.ScopeMetricsGroup {
	metrics := make([]*pmetric.Metric, 0, scopeMetrics.Metrics().Len())
	scope := scopeMetrics.Scope()

	metricsSlice := scopeMetrics.Metrics()
	for i := 0; i < metricsSlice.Len(); i++ {
		log := metricsSlice.At(i)
		metrics = append(metrics, &log)
	}
	return &marrow.ScopeMetricsGroup{
		Scope:          &scope,
		ScopeSchemaUrl: scopeMetrics.SchemaUrl(),
		Metrics:        metrics,
	}
}

func TestScopeMetricsWithGenerator(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	dg := datagen.NewDataGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes()).
		WithConfig(datagen.Config{
			ProbMetricDescription: 0.5,
			ProbMetricUnit:        0.5,
			ProbHistogramHasSum:   0.5,
			ProbHistogramHasMin:   0.5,
			ProbHistogramHasMax:   0.5,
		})
	metricsGen := datagen.NewMetricsGeneratorWithDataGenerator(dg)

	// Generate a random OTLP metrics request.
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.Generate(2 /*00*/, 1 /*00*/))

	// Build a list of scope metrics from the OTLP metrics request generated
	// above.
	var expectedScopeMetrics []pmetric.ScopeMetrics
	expectedResourceMetricsSlice := expectedRequest.Metrics().ResourceMetrics()

	for i := 0; i < expectedResourceMetricsSlice.Len(); i++ {
		expectedResourceMetrics := expectedResourceMetricsSlice.At(i)
		expectedScopeMetricsSlice := expectedResourceMetrics.ScopeMetrics()
		for j := 0; j < expectedScopeMetricsSlice.Len(); j++ {
			expectedScopeMetrics = append(expectedScopeMetrics, expectedScopeMetricsSlice.At(j))
		}
	}

	// Initialize the infrastructure to build an Arrow record from the
	// scope metrics and then retrieve the scope metrics from the Arrow
	// record.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	s := arrow.NewSchema([]arrow.Field{
		{Name: constants.ScopeMetrics, Type: arrow.ListOf(marrow.ScopeMetricsDT), Metadata: schema.Metadata(schema.Optional)},
	}, nil)

	rBuilder := builder.NewRecordBuilderExt(pool, s, DefaultDictConfig, producerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	maxIter := 1

	// Create Arrow record from OTLP scope metrics.
	for {
		lb := rBuilder.ListBuilder(constants.ScopeMetrics)
		b := marrow.ScopeMetricsBuilderFrom(lb.StructBuilder())
		for i := 0; i < maxIter; i++ {
			err := lb.Append(len(expectedScopeMetrics), func() error {
				for _, scopeMetrics := range expectedScopeMetrics {
					err := b.Append(ToScopeMetricsGroup(scopeMetrics))
					require.NoError(t, err)
				}
				return nil
			})
			require.NoError(t, err)
		}

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	// Retrieve the Arrow struct representing the scope metrics.
	arr := record.Columns()[0].(*array.List)

	// Check the OTLP Arrow encoding and OTLP decoding by
	// comparing the original and decoded scope metrics.
	row := 0
	scopeMetricsIds, err := NewScopeMetricsIds(arr.DataType().(*arrow.ListType).ElemField().Type.(*arrow.StructType))
	require.NoError(t, err)
	for i := 0; i < maxIter; i++ {
		los, err := carrow.ListOfStructsFromArray(arr, row)
		require.NoError(t, err)
		value := pmetric.NewResourceMetrics().ScopeMetrics()
		err = UpdateScopeMetricsFrom(value, los, scopeMetricsIds)
		require.NoError(t, err)
		assert.Equal(t, len(expectedScopeMetrics), value.Len())
		for i := 0; i < len(expectedScopeMetrics); i++ {
			AssertJSONEq(t, expectedScopeMetrics[i], value.At(i))
		}
		row++
	}
}

func AssertJSONEq(t *testing.T, expected pmetric.ScopeMetrics, actual pmetric.ScopeMetrics) {
	expectedJSON, err := Jsonify(expected)
	require.NoError(t, err)
	actualJSON, err := Jsonify(actual)
	require.NoError(t, err)
	jsonassert.EquivFromBytes(t, expectedJSON, actualJSON)
}

func Jsonify(scopeMetrics pmetric.ScopeMetrics) ([]byte, error) {
	metrics := pmetric.NewMetrics()
	rm := metrics.ResourceMetrics().AppendEmpty()
	sm := rm.ScopeMetrics().AppendEmpty()
	scopeMetrics.CopyTo(sm)
	encoder := &pmetric.JSONMarshaler{}
	return encoder.MarshalMetrics(metrics)
}
