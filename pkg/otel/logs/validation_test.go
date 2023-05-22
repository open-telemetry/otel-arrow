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
	"math"
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	logsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	logsotlp "github.com/f5/otel-arrow-adapter/pkg/otel/logs/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
)

var (
	DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)
	producerStats     = stats.NewProducerStats()
)

// TestConversionFromSyntheticData tests the conversion of OTLP logs to Arrow and back to OTLP.
// The initial OTLP logs are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP logs compared to the JSON serialization
// of the OTLP logs generated from the Arrow records.
func TestConversionFromSyntheticData(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec	// only used for testing
	logsGen := datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	// Generate a random OTLP logs request.
	expectedRequest := plogotlp.NewExportRequestFromLogs(logsGen.Generate(1, 100))

	// Convert the OTLP logs request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, logsarrow.LogsSchema, DefaultDictConfig, producerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		lb, err := logsarrow.NewLogsBuilder(rBuilder, logsarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer lb.Release()

		err = lb.Append(expectedRequest.Logs())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = lb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	relatedData, _, err := logsotlp.RelatedDataFrom(relatedRecords)
	require.NoError(t, err)

	// Convert the Arrow records back to OTLP.
	logs, err := logsotlp.LogsFrom(record, relatedData)
	require.NoError(t, err)

	record.Release()

	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)})
}
