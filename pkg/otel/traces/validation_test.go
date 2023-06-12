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

package traces_test

import (
	"encoding/json"
	"math"
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	tracesarrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
	tracesotlp "github.com/f5/otel-arrow-adapter/pkg/otel/traces/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)
var ProducerStats = stats.NewProducerStats()

// TestTracesEncodingDecoding tests the conversion of OTLP traces to OTel Arrow traces
// and back to OTLP. The initial OTLP traces are generated from a synthetic
// dataset.
//
// The validation process is based on the JSON comparison the OTLP traces generated
// and the OTLP traces decoded from the OTel Arrow traces. This comparison is strict
// and accept differences in the order of the fields.
func TestTracesEncodingDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	tracesGen := datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	expectedRequest := ptraceotlp.NewExportRequestFromTraces(tracesGen.Generate(100, 100))
	CheckEncodeDecode(t, expectedRequest)

	expectedRequest = ptraceotlp.NewExportRequestFromTraces(tracesGen.GenerateRandomTraces(100, 100))
	CheckEncodeDecode(t, expectedRequest)
}

// TestInvalidTracesDecoding is similar to TestLogsEncodingDecoding but introduces
// some random modification of the Arrow Records used to represent OTel traces.
// These modifications should be handled gracefully by the decoding process and
// generate an error but should never panic.
func TestInvalidTracesDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	tracesGen := datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	// Generate a random OTLP traces request.
	expectedRequest := ptraceotlp.NewExportRequestFromTraces(tracesGen.GenerateRandomTraces(2000, 100))

	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func CheckEncodeDecode(
	t *testing.T,
	expectedRequest ptraceotlp.ExportRequest,
) {
	// Convert the OTLP traces request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, tracesarrow.TracesSchema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		tb, err := tracesarrow.NewTracesBuilder(rBuilder, tracesarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(expectedRequest.Traces())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = tb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	relatedData, _, err := tracesotlp.RelatedDataFrom(relatedRecords, tracesarrow.NewConfig(conf))
	require.NoError(t, err)

	// Convert the Arrow record back to OTLP.
	traces, err := tracesotlp.TracesFrom(record, relatedData)
	require.NoError(t, err)

	record.Release()

	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)})
}

func MultiRoundOfCheckEncodeMessUpDecode(
	t *testing.T,
	expectedRequest ptraceotlp.ExportRequest,
) {
	rng := rand.New(rand.NewSource(int64(rand.Uint64())))

	for i := 0; i < 100; i++ {
		CheckEncodeMessUpDecode(t, expectedRequest, rng)
	}
}

func CheckEncodeMessUpDecode(
	t *testing.T,
	expectedRequest ptraceotlp.ExportRequest,
	rng *rand.Rand,
) {
	// Convert the OTLP traces request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, tracesarrow.TracesSchema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		tb, err := tracesarrow.NewTracesBuilder(rBuilder, tracesarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(expectedRequest.Traces())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = tb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	// Mix up the Arrow records in such a way as to make decoding impossible.
	mainRecordChanged, record, relatedRecords := common.MixUpArrowRecords(rng, record, relatedRecords)

	relatedData, _, err := tracesotlp.RelatedDataFrom(relatedRecords, tracesarrow.NewConfig(conf))

	// Convert the Arrow record back to OTLP.
	_, err = tracesotlp.TracesFrom(record, relatedData)

	if mainRecordChanged || relatedData == nil {
		require.Error(t, err)
	} else {
		require.NoError(t, err)
	}

	record.Release()
}

// TestConversionFromRealData tests the conversion of OTLP traces to Arrow and back to OTLP.
// The initial OTLP traces are generated from a real dataset (anonymized).
// This test is based on the JSON serialization of the initial generated OTLP traces compared to the JSON serialization
// of the OTLP traces generated from the Arrow records.
func TestConversionFromRealData(t *testing.T) {
	t.Parallel()
	t.Skip("Testing based on production data that is not stored in the")

	// Load a real OTLP traces request.
	ds := dataset.NewRealTraceDataset("../../../data/nth_first_otlp_traces.pb", []string{"trace_id"})

	batchSizes := []int{1, 10, 100, 1000, 5000, 10000}
	for _, batchSize := range batchSizes {
		tracesList := ds.Traces(0, batchSize)
		expectedRequest := ptraceotlp.NewExportRequestFromTraces(tracesList[0])

		// Convert the OTLP traces request to Arrow.
		checkTracesConversion(t, expectedRequest)
	}
}

func checkTracesConversion(t *testing.T, expectedRequest ptraceotlp.ExportRequest) { //nolint:unused // only used for testing
	t.Helper()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, tracesarrow.TracesSchema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		tb, err := tracesarrow.NewTracesBuilder(rBuilder, tracesarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		err = tb.Append(expectedRequest.Traces())
		require.NoError(t, err)
		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = tb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	relatedData, _, err := tracesotlp.RelatedDataFrom(relatedRecords, tracesarrow.NewConfig(conf))
	require.NoError(t, err)

	// Convert the Arrow records back to OTLP.
	traces, err := tracesotlp.TracesFrom(record, relatedData)
	if err != nil {
		t.Fatal(err)
	}

	defer record.Release()

	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)})
}
