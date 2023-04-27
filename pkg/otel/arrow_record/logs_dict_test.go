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

	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// TestLogsWithNoDictionary
// Initial dictionary index size is 0 ==> no dictionary.
func TestLogsWithNoDictionary(t *testing.T) {
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

	builder := producer.LogsRecordBuilderExt()
	require.Equal(t, 0, len(builder.Events().DictionariesWithOverflow))
}

// TestLogsMultiBatchWithDictionaryIndexChanges
// Initial dictionary size uint8.
// First batch of uint8 + 1 spans ==> dictionary overflow on 3 fields.
// Other consecutive batches should not trigger any other dictionary overflow.
func TestLogsMultiBatchWithDictionaryIndexChanges(t *testing.T) {
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

	builder := producer.LogsRecordBuilderExt()
	dictionariesIndexTypeChanged := builder.Events().DictionariesIndexTypeChanged
	require.Equal(t, 1, len(dictionariesIndexTypeChanged))
	require.Equal(t, "uint16", dictionariesIndexTypeChanged["resource_logs.item.scope_logs.item.logs.item.severity_text"])
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

	builder := producer.LogsRecordBuilderExt()
	dictionariesIndexTypeChanged := builder.Events().DictionariesIndexTypeChanged
	require.Equal(t, 1, len(dictionariesIndexTypeChanged))
	require.Equal(t, "uint16", dictionariesIndexTypeChanged["resource_logs.item.scope_logs.item.logs.item.severity_text"])
}

// TestLogsSingleBatchWithDictionaryLimit
// Initial dictionary size uint8.
// Limit dictionary index size is uint8.
// First batch of uint8 + 1 spans ==> dictionary index type limit reached so fallback to utf8 or binary.
func TestLogsMultiBatchWithDictionaryLimit(t *testing.T) {
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

	builder := producer.LogsRecordBuilderExt()
	dictionaryWithOverflow := builder.Events().DictionariesWithOverflow
	require.Equal(t, 4, len(dictionaryWithOverflow))
	require.True(t, dictionaryWithOverflow["resource_logs.item.scope_logs.item.logs.item.severity_text"])
	require.True(t, dictionaryWithOverflow["resource_logs.item.scope_logs.item.logs.item.body.str"])
	require.True(t, dictionaryWithOverflow["resource_logs.item.scope_logs.item.logs.item.attributes.value.str"])
	require.True(t, dictionaryWithOverflow["resource_logs.item.scope_logs.item.logs.item.attributes.value.binary"])
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
