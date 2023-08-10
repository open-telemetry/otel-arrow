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

package metrics

import (
	"encoding/json"
	"math"
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"

	"github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/datagen"
	"github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/builder"
	cfg "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/config"
	ametrics "github.com/open-telemetry/otel-arrow/pkg/otel/metrics/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/otel/metrics/otlp"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
	"github.com/open-telemetry/otel-arrow/pkg/record_message"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)

// TestMetricsEncodingDecoding tests the conversion of OTLP metrics to Arrow and back to OTLP.
// The initial OTLP metrics are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP metrics compared to the JSON serialization
// of the OTLP metrics generated from the Arrow records.
func TestMetricsEncodingDecoding(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateRandomMetrics(50, 100))

	CheckEncodeDecode(t, expectedRequest)
}

func TestInvalidMetricsDecoding(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateAllKindOfMetrics(100, 100))

	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func TestGauges(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateGauges(100, 100))

	CheckEncodeDecode(t, expectedRequest)
	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func TestSums(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateSums(100, 100))

	CheckEncodeDecode(t, expectedRequest)
	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func TestSummaries(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateSummaries(100, 100))

	CheckEncodeDecode(t, expectedRequest)
	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func TestHistograms(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateHistograms(100, 100))

	CheckEncodeDecode(t, expectedRequest)
	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func TestExponentialHistograms(t *testing.T) {
	t.Parallel()

	metricsGen := MetricsGenerator()
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.GenerateExponentialHistograms(100, 100))

	CheckEncodeDecode(t, expectedRequest)
	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func MetricsGenerator() *datagen.MetricsGenerator {
	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	dg := datagen.NewDataGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes()).
		WithConfig(datagen.Config{
			ProbMetricDescription: 0.5,
			ProbMetricUnit:        0.5,
			ProbHistogramHasSum:   0.5,
			ProbHistogramHasMin:   0.5,
			ProbHistogramHasMax:   0.5,
		})
	return datagen.NewMetricsGeneratorWithDataGenerator(dg)
}

func CheckEncodeDecode(t *testing.T, expectedRequest pmetricotlp.ExportRequest) {
	stdTesting := assert.NewStdUnitTest(t)

	// Convert the OTLP metrics request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, ametrics.MetricsSchema, DefaultDictConfig, stats.NewProducerStats())
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		lb, err := ametrics.NewMetricsBuilder(rBuilder, ametrics.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer lb.Release()

		err = lb.Append(expectedRequest.Metrics())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = lb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, schema.ErrSchemaNotUpToDate)
	}

	relatedData, _, err := otlp.RelatedDataFrom(relatedRecords)
	require.NoError(t, err)

	// Convert the Arrow records back to OTLP.
	metrics, err := otlp.MetricsFrom(record, relatedData)
	require.NoError(t, err)

	record.Release()

	assert.Equiv(stdTesting, []json.Marshaler{expectedRequest}, []json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(metrics)})
}

// MultiRoundOfCheckEncodeMessUpDecode tests the robustness of the conversion of
// OTel Arrow records to OTLP metrics. These tests should never trigger a panic.
// For every main record, and related records (if any), we mix up the Arrow
// records in order to test the robustness of the conversion. In this situation,
// the conversion can generate errors, but should never panic.
func MultiRoundOfCheckEncodeMessUpDecode(t *testing.T, expectedRequest pmetricotlp.ExportRequest) {
	rng := rand.New(rand.NewSource(int64(rand.Uint64())))

	for i := 0; i < 100; i++ {
		OneRoundOfMessUpArrowRecords(t, expectedRequest, rng)
	}
}

func OneRoundOfMessUpArrowRecords(t *testing.T, expectedRequest pmetricotlp.ExportRequest, rng *rand.Rand) {
	// Convert the OTLP metrics request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer func() {
		pool.AssertSize(t, 0)
	}()

	rBuilder := builder.NewRecordBuilderExt(pool, ametrics.MetricsSchema, DefaultDictConfig, stats.NewProducerStats())
	defer func() {
		rBuilder.Release()
	}()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		lb, err := ametrics.NewMetricsBuilder(rBuilder, ametrics.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer lb.Release()

		err = lb.Append(expectedRequest.Metrics())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = lb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, schema.ErrSchemaNotUpToDate)
	}

	// Mix up the Arrow records in such a way as to make decoding impossible.
	mainRecordChanged, record, relatedRecords := common.MixUpArrowRecords(rng, record, relatedRecords)

	relatedData, _, err := otlp.RelatedDataFrom(relatedRecords)

	// Convert the Arrow records back to OTLP.
	_, err = otlp.MetricsFrom(record, relatedData)

	if mainRecordChanged || relatedData == nil {
		require.Error(t, err)
	} else {
		require.NoError(t, err)
	}

	record.Release()
}
