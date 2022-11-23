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
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	tracesarrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
	tracesotlp "github.com/f5/otel-arrow-adapter/pkg/otel/traces/otlp"
)

// TestConversionFromSyntheticData tests the conversion of OTLP traces to Arrow and back to OTLP.
// The initial OTLP traces are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP traces compared to the JSON serialization
// of the OTLP traces generated from the Arrow records.
func TestConversionFromSyntheticData(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	tracesGen := datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	// Generate a random OTLP traces request.
	expectedRequest := ptraceotlp.NewExportRequestFromTraces(tracesGen.Generate(10, 100))

	// Convert the OTLP traces request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	traceSchema := acommon.NewAdaptiveSchema(tracesarrow.Schema)
	defer traceSchema.Release()
	tb, err := tracesarrow.NewTracesBuilder(pool, traceSchema)
	if err != nil {
		t.Fatal(err)
	}
	err = tb.Append(expectedRequest.Traces())
	if err != nil {
		t.Fatal(err)
	}
	record, err := tb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer record.Release()

	// Convert the Arrow record back to OTLP.
	traces, err := tracesotlp.TracesFrom(record)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)})
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
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	traceSchema := acommon.NewAdaptiveSchema(tracesarrow.Schema)
	tb, err := tracesarrow.NewTracesBuilder(pool, traceSchema)
	if err != nil {
		t.Fatal(err)
	}
	err = tb.Append(expectedRequest.Traces())
	if err != nil {
		t.Fatal(err)
	}
	record, err := tb.Build()
	defer record.Release()
	if err != nil {
		t.Fatal(err)
	}

	// Convert the Arrow records back to OTLP.
	traces, err := tracesotlp.TracesFrom(record)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)})
}
