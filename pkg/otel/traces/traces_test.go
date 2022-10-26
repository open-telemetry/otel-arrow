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

package traces

import (
	"encoding/json"
	"testing"

	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// TestConversionFromSyntheticData tests the conversion of OTLP traces to Arrow and back to OTLP.
// The initial OTLP traces are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP traces compared to the JSON serialization
// of the OTLP traces generated from the Arrow records.
func TestConversionFromSyntheticData(t *testing.T) {
	t.Parallel()

	tracesGen := datagen.NewTraceGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	// Generate a random OTLP traces request.
	expectedRequest := ptraceotlp.NewRequestFromTraces(tracesGen.Generate(10, 100))

	// Convert the OTLP traces request to Arrow.
	otlpArrowProducer := NewOtlpArrowProducer()
	records, err := otlpArrowProducer.ProduceFrom(expectedRequest.Traces())
	if err != nil {
		t.Fatal(err)
	}

	// Convert the Arrow records back to OTLP.
	otlpProducer := NewOtlpProducer()
	for _, record := range records {
		traces, err := otlpProducer.ProduceFrom(record)
		if err != nil {
			t.Fatal(err)
		}

		actualRequests := make([]json.Marshaler, len(traces))
		for i, t := range traces {
			actualRequests[i] = ptraceotlp.NewRequestFromTraces(t)
		}

		assert.Equiv(t, []json.Marshaler{expectedRequest}, actualRequests)
	}
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

	batch_sizes := []int{1, 10, 100, 1000, 5000, 10000}
	for _, batch_size := range batch_sizes {
		traces := ds.Traces(0, batch_size)
		expectedRequest := ptraceotlp.NewRequestFromTraces(traces[0])

		// Convert the OTLP traces request to Arrow.
		otlpArrowProducer := NewOtlpArrowProducer()
		records, err := otlpArrowProducer.ProduceFrom(expectedRequest.Traces())
		if err != nil {
			t.Fatal(err)
		}

		// Convert the Arrow records back to OTLP.
		otlpProducer := NewOtlpProducer()
		var actualRequests []json.Marshaler
		for _, record := range records {
			traces, err := otlpProducer.ProduceFrom(record)
			if err != nil {
				t.Fatal(err)
			}

			for _, t := range traces {
				actualRequests = append(actualRequests, ptraceotlp.NewRequestFromTraces(t))
			}
		}

		// Check the equivalence of the initial and the generated OTLP traces.
		assert.Equiv(t, []json.Marshaler{expectedRequest}, actualRequests)
	}
}
