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

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/datagen"
	"github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common"
	acommon "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/builder"
	cfg "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/config"
	logsarrow "github.com/open-telemetry/otel-arrow/pkg/otel/logs/arrow"
	logsotlp "github.com/open-telemetry/otel-arrow/pkg/otel/logs/otlp"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
	"github.com/open-telemetry/otel-arrow/pkg/record_message"
)

var (
	DefaultDictConfig = cfg.NewDictionary(math.MaxUint16, 0.0)
	producerStats     = stats.NewProducerStats()
)

// TestLogsEncodingDecoding tests the conversion of OTLP logs to OTel Arrow logs
// and back to OTLP. The initial OTLP logs are generated from a synthetic
// dataset.
//
// The validation process is based on the JSON comparison the OTLP logs generated
// and the OTLP logs decoded from the OTel Arrow logs. This comparison is strict
// and accept differences in the order of the fields.
func TestLogsEncodingDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy()
	logsGen := datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	expectedRequest := plogotlp.NewExportRequestFromLogs(logsGen.Generate(5000, 100))

	CheckEncodeDecode(t, expectedRequest)
}

// TestInvalidLogsDecoding is similar to TestLogsEncodingDecoding but introduces
// some random modification of the Arrow Records used to represent OTel logs.
// These modifications should be handled gracefully by the decoding process and
// generate an error but should never panic.
func TestInvalidLogsDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy()
	logsGen := datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	expectedRequest := plogotlp.NewExportRequestFromLogs(logsGen.Generate(100, 100))

	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func CheckEncodeDecode(
	t *testing.T,
	expectedRequest plogotlp.ExportRequest,
) {
	stdTesting := assert.NewStdUnitTest(t)

	// Convert the OTLP logs request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, logsarrow.LogsSchema, DefaultDictConfig, producerStats, nil)
	defer rBuilder.Release()

	conf := config.DefaultConfig()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	for {
		lb, err := logsarrow.NewLogsBuilder(rBuilder, logsarrow.NewConfig(conf), stats.NewProducerStats(), nil)
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
	record.Release()
	require.NoError(t, err)

	assert.Equiv(stdTesting, []json.Marshaler{expectedRequest}, []json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)})
}

func MultiRoundOfCheckEncodeMessUpDecode(
	t *testing.T,
	expectedRequest plogotlp.ExportRequest,
) {
	rng := rand.New(rand.NewSource(42))

	for i := 0; i < 100; i++ {
		CheckEncodeMessUpDecode(t, expectedRequest, rng)
	}
}

func CheckEncodeMessUpDecode(
	t *testing.T,
	expectedRequest plogotlp.ExportRequest,
	rng *rand.Rand,
) {
	// Convert the OTLP logs request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, logsarrow.LogsSchema, DefaultDictConfig, producerStats, nil)
	defer rBuilder.Release()

	conf := config.DefaultConfig()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	for {
		lb, err := logsarrow.NewLogsBuilder(rBuilder, logsarrow.NewConfig(conf), stats.NewProducerStats(), nil)
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

	// Mix up the Arrow records in such a way as to make decoding impossible.
	mainRecordChanged, record, relatedRecords := common.MixUpArrowRecords(rng, record, relatedRecords)

	relatedData, _, err := logsotlp.RelatedDataFrom(relatedRecords)

	// Convert the Arrow records back to OTLP.
	_, err = logsotlp.LogsFrom(record, relatedData)
	record.Release()

	if mainRecordChanged || relatedData == nil {
		require.Error(t, err)
	} else {
		require.NoError(t, err)
	}
}
