package arrow_record

import (
	"encoding/json"
	"fmt"
	"math"
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// TestTracesWithNoDictionary
// Initial dictionary index size is 0 ==> no dictionary.
func TestTracesWithNoDictionary(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithNoDictionary(),
	)
	defer producer.Close()
	consumer := NewConsumer()
	defer consumer.Close()

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	schema := producer.TracesAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 0, len(dictWithOverflow))
}

// TestTracesSingleBatchWithDictionaryOverflow
// Initial dictionary size uint8.
// First batch of uint8 + 1 spans ==> dictionary overflow on 3 fields.
// Other consecutive batches should not trigger any other dictionary overflow.
func TestTracesSingleBatchWithDictionaryOverflow(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithUint8InitDictIndex(),
		WithUint32LimitDictIndex(),
	)
	defer producer.Close()
	consumer := NewConsumer()
	defer consumer.Close()

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	schema := producer.TracesAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 3, len(dictWithOverflow))
	require.Equal(t, "uint16", dictWithOverflow["resource_spans.scope_spans.spans.name"])
	require.Equal(t, "uint16", dictWithOverflow["resource_spans.scope_spans.spans.attributes.value.str"])
	require.Equal(t, "uint16", dictWithOverflow["resource_spans.scope_spans.spans.attributes.value.binary"])
}

// TestTracesMultiBatchWithDictionaryOverflow
// Initial dictionary size uint8.
// First and second batch of uint8/2 spans (each) ==> no dictionary overflow.
// Third batch should trigger dictionary overflow.
// All other consecutive batches should not trigger any other dictionary overflow.
func TestTracesMultiBatchWithDictionaryOverflow(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithUint8InitDictIndex(),
		WithUint32LimitDictIndex(),
	)
	defer producer.Close()
	consumer := NewConsumer()
	defer consumer.Close()

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(i*((math.MaxUint8/2)+1), (math.MaxUint8/2)+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	schema := producer.TracesAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 3, len(dictWithOverflow))
	require.Equal(t, "uint16", dictWithOverflow["resource_spans.scope_spans.spans.name"])
	require.Equal(t, "uint16", dictWithOverflow["resource_spans.scope_spans.spans.attributes.value.str"])
	require.Equal(t, "uint16", dictWithOverflow["resource_spans.scope_spans.spans.attributes.value.binary"])
}

// TestTracesSingleBatchWithDictionaryLimit
// Initial dictionary size uint8.
// Limit dictionary index size is uint8.
// First batch of uint8 + 1 spans ==> dictionary index type limit reached so fallback to utf8 or binary.
func TestTracesSingleBatchWithDictionaryLimit(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		WithAllocator(pool),
		WithUint8InitDictIndex(),
		WithUint8LimitDictIndex(),
	)
	defer producer.Close()
	consumer := NewConsumer()
	defer consumer.Close()

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	schema := producer.TracesAdaptiveSchema()
	dictWithOverflow := schema.DictionariesWithOverflow()
	require.Equal(t, 3, len(dictWithOverflow))
	require.Equal(t, "utf8", dictWithOverflow["resource_spans.scope_spans.spans.name"])
	require.Equal(t, "utf8", dictWithOverflow["resource_spans.scope_spans.spans.attributes.value.str"])
	require.Equal(t, "binary", dictWithOverflow["resource_spans.scope_spans.spans.attributes.value.binary"])
}

func GenerateTraces(initValue int, spanCount int) ptrace.Traces {
	trace := ptrace.NewTraces()

	rss := trace.ResourceSpans()
	rss.EnsureCapacity(1)

	rs := rss.AppendEmpty()
	rs.SetSchemaUrl("schema")

	sss := rs.ScopeSpans()
	sss.EnsureCapacity(1)

	spans := sss.AppendEmpty().Spans()
	spans.EnsureCapacity(spanCount)

	for i := 0; i < spanCount; i++ {
		span := spans.AppendEmpty()
		span.SetName(fmt.Sprintf("span_%d", initValue+i))
		attrs := span.Attributes()
		attrs.EnsureCapacity(2)
		attrs.PutStr("attr1", fmt.Sprintf("value_%d", initValue+i))
		attrs.PutEmptyBytes("attr2").Append([]byte(fmt.Sprintf("value_%d", initValue+i))...)
	}

	return trace
}
