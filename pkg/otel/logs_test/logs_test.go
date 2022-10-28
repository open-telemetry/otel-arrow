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

package logs_test

import (
	"encoding/json"
	"testing"

	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	common_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	common_otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	logs_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	logs_otlp "github.com/f5/otel-arrow-adapter/pkg/otel/logs/otlp"
)

// TestConversionFromSyntheticData tests the conversion of OTLP logs to Arrow and back to OTLP.
// The initial OTLP logs are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP logs compared to the JSON serialization
// of the OTLP logs generated from the Arrow records.
func TestConversionFromSyntheticData(t *testing.T) {
	t.Parallel()

	logsGen := datagen.NewLogsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	// Generate a random OTLP logs request.
	expectedRequest := plogotlp.NewRequestFromLogs(logsGen.Generate(10, 100))

	// Convert the OTLP logs request to Arrow.
	otlpArrowProducer := common_arrow.NewOtlpArrowProducer[plog.ScopeLogs]()
	records, err := otlpArrowProducer.ProduceFrom(logs_arrow.Wrap(expectedRequest.Logs()))
	if err != nil {
		t.Fatal(err)
	}

	// Convert the Arrow records back to OTLP.
	otlpProducer := common_otlp.New[plog.Logs, plog.LogRecord](logs_otlp.LogsProducer{})
	for _, record := range records {
		traces, err := otlpProducer.ProduceFrom(record)
		if err != nil {
			t.Fatal(err)
		}

		actualRequests := make([]json.Marshaler, len(traces))
		for i, t := range traces {
			actualRequests[i] = plogotlp.NewRequestFromLogs(t)
		}

		assert.Equiv(t, []json.Marshaler{expectedRequest}, actualRequests)
	}
}
