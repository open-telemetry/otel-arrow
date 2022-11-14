package arrow_record

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// TestDictionaryOverflow tests the manage of dictionary overflow both side producer and consumer.
// Dictionary keys are configured as uint16 in the schema, so the maximum number of values is 65536.
func TestDictionaryOverflow(t *testing.T) {
	producer := NewProducer()
	consumer := NewConsumer()

	for i := 0; i < 70000; i++ {
		trace := ptrace.NewTraces()
		rss := trace.ResourceSpans()
		rss.EnsureCapacity(1)
		rs := rss.AppendEmpty()
		rs.SetSchemaUrl(fmt.Sprintf("schema_%d", i))

		batch, err := producer.BatchArrowRecordsFromTraces(trace)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(trace)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	for i := 0; i < 10000; i++ {
		trace := ptrace.NewTraces()
		rss := trace.ResourceSpans()
		rss.EnsureCapacity(1)
		rs := rss.AppendEmpty()
		rs.SetSchemaUrl(fmt.Sprintf("schema_%d", i))

		batch, err := producer.BatchArrowRecordsFromTraces(trace)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(trace)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}
}
