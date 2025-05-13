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

package arrow_record

import (
	"encoding/json"
	"fmt"
	"math"
	"testing"

	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/arrow"
)

// TestTracesWithNoDictionary
// Initial dictionary index size is 0 ==> no dictionary.
func TestTracesWithNoDictionary(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		config.WithAllocator(pool),
		config.WithNoDictionary(),
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

	stdTesting := assert.NewStdUnitTest(t)

	for i := 0; i < 300; i++ {
		traces := GenerateTraces(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			stdTesting,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	builder := producer.TracesRecordBuilderExt()
	require.Equal(t, 0, len(builder.Events().DictionariesWithOverflow))
}

// TestTracesMultiBatchWithDictionaryIndexChanges
// Initial dictionary size uint8.
// First batch of uint8 + 1 spans ==> dictionary overflow on 3 fields.
// Other consecutive batches should not trigger any other dictionary overflow.
func TestTracesMultiBatchWithDictionaryIndexChanges(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		config.WithAllocator(pool),
		config.WithUint8InitDictIndex(),
		config.WithUint32LimitDictIndex(),
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

	stdTesting := assert.NewStdUnitTest(t)

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			stdTesting,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	builder := producer.TracesRecordBuilderExt()
	dictionariesIndexTypeChanged := builder.Events().DictionariesIndexTypeChanged
	require.Equal(t, 1, len(dictionariesIndexTypeChanged))
	require.Equal(t, "uint16", dictionariesIndexTypeChanged["name"])
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
		config.WithAllocator(pool),
		config.WithUint8InitDictIndex(),
		config.WithUint32LimitDictIndex(),
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

	stdTesting := assert.NewStdUnitTest(t)

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(i*((math.MaxUint8/2)+1), (math.MaxUint8/2)+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			stdTesting,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	builder := producer.TracesRecordBuilderExt()
	dictionariesIndexTypeChanged := builder.Events().DictionariesIndexTypeChanged
	require.Equal(t, 1, len(dictionariesIndexTypeChanged))
	require.Equal(t, "uint16", dictionariesIndexTypeChanged["name"])
}

// TestTracesMultiBatchWithDictionaryLimit
// Initial dictionary size uint8.
// Limit dictionary index size is uint8.
// First batch of uint8 + 1 spans ==> dictionary index type limit reached so fallback to utf8 or binary.
func TestTracesMultiBatchWithDictionaryLimit(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(
		config.WithAllocator(pool),
		config.WithUint8InitDictIndex(),
		config.WithUint8LimitDictIndex(),
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

	stdTesting := assert.NewStdUnitTest(t)

	for i := 0; i < 10; i++ {
		traces := GenerateTraces(0, math.MaxUint8+1)
		batch, err := producer.BatchArrowRecordsFromTraces(traces)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.NoError(t, err)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			stdTesting,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	spanBuilder := producer.TracesRecordBuilderExt()
	dictionaryWithOverflow := spanBuilder.Events().DictionariesWithOverflow
	require.Equal(t, 1, len(dictionaryWithOverflow))
	require.True(t, dictionaryWithOverflow["name"])

	spanAttrsBuilder := producer.TracesBuilder().RelatedData().RecordBuilderExt(arrow.PayloadTypes.SpanAttrs)
	dictionaryWithOverflow = spanAttrsBuilder.Events().DictionariesWithOverflow
	require.Equal(t, 2, len(dictionaryWithOverflow))
	require.True(t, dictionaryWithOverflow["str"])
	require.True(t, dictionaryWithOverflow["bytes"])
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
