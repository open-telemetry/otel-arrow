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
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pmetric"

	jsonassert "github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
	ametrics "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
)

var producerStats = stats.NewProducerStats()

func TestMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, ametrics.Schema, DefaultDictConfig, producerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var err error

	maxIter := 10
	encoder := &pmetric.JSONMarshaler{}

	// Create Arrow record from OTLP metrics.
	for {
		b, err := ametrics.NewMetricsBuilder(rBuilder, false)
		require.NoError(t, err)
		for i := 0; i < maxIter; i++ {
			err = b.Append(internal.Metrics1())
			require.NoError(t, err)
			err = b.Append(internal.Metrics2())
			require.NoError(t, err)
		}

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, schema.ErrSchemaNotUpToDate)
	}
	defer record.Release()

	// Convert the Arrow record to OTLP metrics.
	metrics, err := MetricsFrom(record)
	require.NoError(t, err)

	// Generate expected metrics and build a JSON representation.
	expectedMetrics := internal.Metrics(maxIter)
	expectedJson, err := encoder.MarshalMetrics(expectedMetrics)
	require.NoError(t, err)

	// Build a JSON representation of the metrics from the Arrow record.
	metricsJson, err := encoder.MarshalMetrics(metrics)
	require.NoError(t, err)

	// Compare the JSON representations.
	jsonassert.EquivFromBytes(t, expectedJson, metricsJson)
}
