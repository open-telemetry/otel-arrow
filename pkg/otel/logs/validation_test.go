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
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	logs_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	logs_otlp "github.com/f5/otel-arrow-adapter/pkg/otel/logs/otlp"
)

// TestConversionFromSyntheticData tests the conversion of OTLP logs to Arrow and back to OTLP.
// The initial OTLP logs are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP logs compared to the JSON serialization
// of the OTLP logs generated from the Arrow records.
func TestConversionFromSyntheticData(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64()))
	logsGen := datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	// Generate a random OTLP logs request.
	expectedRequest := plogotlp.NewExportRequestFromLogs(logsGen.Generate(10, 100))

	// Convert the OTLP logs request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	logsSchema := acommon.NewAdaptiveSchema(logs_arrow.Schema)
	defer logsSchema.Release()
	lb, err := logs_arrow.NewLogsBuilder(pool, logsSchema)
	require.NoError(t, err)
	err = lb.Append(expectedRequest.Logs())
	require.NoError(t, err)
	record, err := lb.Build()
	require.NoError(t, err)
	defer record.Release()

	// Convert the Arrow records back to OTLP.
	logs, err := logs_otlp.LogsFrom(record)
	require.NoError(t, err)
	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)})
}
