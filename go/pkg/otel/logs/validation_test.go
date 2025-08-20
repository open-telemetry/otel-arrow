/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package logs_test

import (
	"encoding/json"
	"math"
	"math/rand"
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/array"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/open-telemetry/otel-arrow/go/pkg/config"
	"github.com/open-telemetry/otel-arrow/go/pkg/datagen"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/assert"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/common"
	acommon "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/builder"
	cfg "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/schema/config"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/constants"
	logsarrow "github.com/open-telemetry/otel-arrow/go/pkg/otel/logs/arrow"
	logsotlp "github.com/open-telemetry/otel-arrow/go/pkg/otel/logs/otlp"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/stats"
	"github.com/open-telemetry/otel-arrow/go/pkg/record_message"
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

func TestDecodingWithMissingOptionalTraceSpanIdFields(t *testing.T) {
	entropy := datagen.NewTestEntropy()
	logsGen := datagen.NewLogsGenerator(entropy, entropy.NewSingleResourceAttributes(), entropy.NewSingleInstrumentationScopes())

	// generate a record batch of OTLP Logs
	data := plogotlp.NewExportRequestFromLogs(logsGen.Generate(5, 100))

	// convert the OTLP logs to an OTAP record
	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	recordBuilder := builder.NewRecordBuilderExt(pool, logsarrow.LogsSchema, DefaultDictConfig, producerStats, nil)
	conf := config.DefaultConfig()
	for {
		lb, err := logsarrow.NewLogsBuilder(recordBuilder, logsarrow.NewConfig(conf), stats.NewProducerStats(), nil)
		require.NoError(t, err)
		defer lb.Release()

		err = lb.Append(data.Logs())
		require.NoError(t, err)

		record, err = recordBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = lb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	// find the column index of trace_id and span_id columns
	schema := record.Schema()
	columnIndices := schema.FieldIndices(constants.TraceId)
	require.Equal(t, 1, len(columnIndices))
	traceIdColumnIndex := columnIndices[0]

	columnIndices = schema.FieldIndices(constants.SpanId)
	require.Equal(t, 1, len(columnIndices))
	spanIdColumnIndex := columnIndices[0]

	// for column we're going to remove because traceID will get removed first below
	if spanIdColumnIndex > traceIdColumnIndex {
		spanIdColumnIndex -= 1
	}

	// remove the trace_id and span_id columns
	columns := record.Columns()
	columns = append(columns[:traceIdColumnIndex], columns[traceIdColumnIndex+1:]...)
	columns = append(columns[:spanIdColumnIndex], columns[spanIdColumnIndex+1:]...)

	// update schema
	fields := schema.Fields()
	fields = append(fields[:traceIdColumnIndex], fields[traceIdColumnIndex+1:]...)
	fields = append(fields[:spanIdColumnIndex], fields[spanIdColumnIndex+1:]...)
	schema = arrow.NewSchema(fields, nil)

	// create new record with these columns removed
	record = array.NewRecord(schema, columns, record.NumRows())

	// Try converting the arrow records back to OTLP
	relatedData, _, err := logsotlp.RelatedDataFrom(relatedRecords)
	require.NoError(t, err)

	otlpLogs, err := logsotlp.LogsFrom(record, relatedData)
	defer record.Release()
	require.NoError(t, err)

	// check that the trace ID and span ID are what we expect, which is an empty array all 0s
	for resourceIdx := range otlpLogs.ResourceLogs().Len() {
		resource := otlpLogs.ResourceLogs().At(resourceIdx)
		for scopeIdx := range resource.ScopeLogs().Len() {
			scope := resource.ScopeLogs().At(scopeIdx)
			for logIdx := range scope.LogRecords().Len() {
				log := scope.LogRecords().At(logIdx)
				spanId := log.SpanID()
				traceId := log.TraceID()
				expectedSpanId := make([]byte, 8)
				expectedTraceId := make([]byte, 16)
				require.Equal(t, expectedSpanId, spanId[:])
				require.Equal(t, expectedTraceId, traceId[:])
			}
		}
	}
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
