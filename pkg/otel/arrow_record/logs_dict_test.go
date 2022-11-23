package arrow_record

import (
	"encoding/json"
	"fmt"
	"math"
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// TestLogsWithNoDictionary
// Initial dictionary index size is 0 ==> no dictionary.
func TestLogsWithNoDictionary(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithNoDictionary(),
	)
	defer func() {
		if err := producer.Close(); err != nil {
			t.Fatal(err)
		}
	}()
	consumer := NewConsumer()
	defer func() {
		if err := consumer.Close(); err != nil {
			t.Fatal(err)
		}
	}()

	for i := 0; i < 300; i++ {
		logs := GenerateLogs(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromLogs(logs)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.LogsFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)},
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(received[0])},
		)
	}

	schema := producer.LogsAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 0, len(dictWithOverflow))
}

// TestLogsSingleBatchWithDictionaryOverflow
// Initial dictionary size uint8.
// First batch of uint8 + 1 spans ==> dictionary overflow on 3 fields.
// Other consecutive batches should not trigger any other dictionary overflow.
func TestLogsSingleBatchWithDictionaryOverflow(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithUint8InitDictIndex(),
		WithUint32LimitDictIndex(),
	)
	defer func() {
		if err := producer.Close(); err != nil {
			t.Fatal(err)
		}
	}()
	consumer := NewConsumer()
	defer func() {
		if err := consumer.Close(); err != nil {
			t.Fatal(err)
		}
	}()

	for i := 0; i < 10; i++ {
		logs := GenerateLogs(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromLogs(logs)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.LogsFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)},
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(received[0])},
		)
	}

	schema := producer.LogsAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 4, len(dictWithOverflow))
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.severity_text"])
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.body.str"])
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.attributes.value.str"])
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.attributes.value.binary"])
}

// TestLogsMultiBatchWithDictionaryOverflow
// Initial dictionary size uint8.
// First and second batch of uint8/2 spans (each) ==> no dictionary overflow.
// Third batch should trigger dictionary overflow.
// All other consecutive batches should not trigger any other dictionary overflow.
func TestLogsMultiBatchWithDictionaryOverflow(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithUint8InitDictIndex(),
		WithUint32LimitDictIndex(),
	)
	defer func() {
		if err := producer.Close(); err != nil {
			t.Fatal(err)
		}
	}()
	consumer := NewConsumer()
	defer func() {
		if err := consumer.Close(); err != nil {
			t.Fatal(err)
		}
	}()

	for i := 0; i < 10; i++ {
		logs := GenerateLogs(i*((math.MaxUint8/2)+1), (math.MaxUint8/2)+1)
		batch, err := producer.BatchArrowRecordsFromLogs(logs)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.LogsFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)},
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(received[0])},
		)
	}

	schema := producer.LogsAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 4, len(dictWithOverflow))
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.severity_text"])
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.body.str"])
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.attributes.value.str"])
	require.Equal(t, "uint16", dictWithOverflow["resource_logs.scope_logs.logs.attributes.value.binary"])
}

// TestLogsSingleBatchWithDictionaryLimit
// Initial dictionary size uint8.
// Limit dictionary index size is uint8.
// First batch of uint8 + 1 spans ==> dictionary index type limit reached so fallback to utf8 or binary.
func TestLogsSingleBatchWithDictionaryLimit(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithUint8InitDictIndex(),
		WithUint8LimitDictIndex(),
	)
	defer func() {
		if err := producer.Close(); err != nil {
			t.Fatal(err)
		}
	}()
	consumer := NewConsumer()
	defer func() {
		if err := consumer.Close(); err != nil {
			t.Fatal(err)
		}
	}()

	for i := 0; i < 10; i++ {
		logs := GenerateLogs(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromLogs(logs)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.LogsFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)},
			[]json.Marshaler{plogotlp.NewExportRequestFromLogs(received[0])},
		)
	}

	schema := producer.LogsAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 4, len(dictWithOverflow))
	require.Equal(t, "utf8", dictWithOverflow["resource_logs.scope_logs.logs.severity_text"])
	require.Equal(t, "utf8", dictWithOverflow["resource_logs.scope_logs.logs.body.str"])
	require.Equal(t, "utf8", dictWithOverflow["resource_logs.scope_logs.logs.attributes.value.str"])
	require.Equal(t, "binary", dictWithOverflow["resource_logs.scope_logs.logs.attributes.value.binary"])
}

func GenerateLogs(initValue int, logCount int) plog.Logs {
	logs := plog.NewLogs()

	rls := logs.ResourceLogs()
	rls.EnsureCapacity(1)

	rl := rls.AppendEmpty()
	rl.SetSchemaUrl("schema")

	sls := rl.ScopeLogs()
	sls.EnsureCapacity(1)

	logRecords := sls.AppendEmpty().LogRecords()
	logRecords.EnsureCapacity(logCount)

	for i := 0; i < logCount; i++ {
		log := logRecords.AppendEmpty()
		log.Body().SetStr(fmt.Sprintf("body-%d", initValue+i))
		log.SetSeverityText(fmt.Sprintf("severity_%d", initValue+i))
		attrs := log.Attributes()
		attrs.EnsureCapacity(2)
		attrs.PutStr("attr1", fmt.Sprintf("value_%d", initValue+i))
		attrs.PutEmptyBytes("attr2").Append([]byte(fmt.Sprintf("value_%d", initValue+i))...)
	}

	return logs
}
