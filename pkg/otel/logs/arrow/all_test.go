/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

import (
	"fmt"
	"math"
	"testing"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog"

	v1 "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/config"
	jsonassert "github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)

func TestLogs(t *testing.T) {
	t.Parallel()

	producerStats := stats.NewProducerStats()
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	rBuilder := builder.NewRecordBuilderExt(pool, LogsSchema, DefaultDictConfig, producerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()
	stats := stats.NewProducerStats()

	for {
		tb, err := NewLogsBuilder(rBuilder, NewConfig(conf), stats)
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(Logs())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err != nil {
			assert.Error(t, acommon.ErrSchemaNotUpToDate)
			continue
		}

		relatedRecords, err = tb.RelatedData().BuildRecordMessages()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	actual, err := record.MarshalJSON()
	require.NoError(t, err)

	record.Release()

	expected := `[{"body":{"str":"body1","type":1},"dropped_attributes_count":null,"flags":1,"id":0,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","resource":{"dropped_attributes_count":null,"id":0,"schema_url":"schema1"},"schema_url":"schema1","scope":{"dropped_attributes_count":null,"id":0,"name":"scope1","version":"1.0.1"},"severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}
,{"body":{"str":"body2","type":1},"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","resource":{"dropped_attributes_count":null,"id":0,"schema_url":"schema1"},"schema_url":"schema1","scope":{"dropped_attributes_count":null,"id":0,"name":"scope1","version":"1.0.1"},"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}
,{"body":{"str":"body2","type":1},"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","resource":{"dropped_attributes_count":null,"id":0,"schema_url":"schema1"},"schema_url":"schema2","scope":{"dropped_attributes_count":1,"id":1,"name":"scope2","version":"1.0.2"},"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}
,{"body":{"str":"body2","type":1},"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","resource":{"dropped_attributes_count":1,"id":1,"schema_url":"schema2"},"schema_url":"schema2","scope":{"dropped_attributes_count":1,"id":0,"name":"scope2","version":"1.0.2"},"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}
]`

	jsonassert.JSONCanonicalEq(t, expected, actual)

	for _, relatedRecord := range relatedRecords {
		switch relatedRecord.PayloadType() {
		case v1.ArrowPayloadType_RESOURCE_ATTRS:
			expected = `[{"bool":null,"bytes":null,"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"bool":null,"bytes":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"bytes":null,"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"bool":null,"bytes":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"bytes":null,"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"bool":null,"bytes":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":true,"bytes":null,"double":null,"int":null,"key":"bool","parent_id":0,"str":null,"type":4}
,{"bool":null,"bytes":"Ynl0ZXMx","double":null,"int":null,"key":"bytes","parent_id":0,"str":null,"type":7}
,{"bool":null,"bytes":"Ynl0ZXMy","double":null,"int":null,"key":"bytes","parent_id":1,"str":null,"type":7}
]`

		case v1.ArrowPayloadType_SCOPE_ATTRS:
			expected = `[{"bool":null,"bytes":null,"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"bool":null,"bytes":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"bytes":null,"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"bool":null,"bytes":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"bytes":null,"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"bool":null,"bytes":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":true,"bytes":null,"double":null,"int":null,"key":"bool","parent_id":0,"str":null,"type":4}
,{"bool":null,"bytes":"Ynl0ZXMx","double":null,"int":null,"key":"bytes","parent_id":0,"str":null,"type":7}
,{"bool":null,"bytes":"Ynl0ZXMy","double":null,"int":null,"key":"bytes","parent_id":1,"str":null,"type":7}
]`

		case v1.ArrowPayloadType_LOG_ATTRS:
			expected = `[{"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
]`

		default:
			panic(fmt.Sprint("unexpected payload type: ", relatedRecord.PayloadType()))
		}

		observed, err := relatedRecord.Record().MarshalJSON()
		require.NoError(t, err)
		relatedRecord.Record().Release()

		require.JSONEq(t, expected, string(observed))

		relatedRecord.Record().Release()
	}
}

func LogRecord1() plog.LogRecord {
	log := plog.NewLogRecord()
	log.SetTimestamp(1)
	log.SetObservedTimestamp(2)
	log.SetTraceID([16]byte{0xAA})
	log.SetSpanID([8]byte{0xAA})
	log.SetSeverityNumber(1)
	log.SetSeverityText("severity1")
	log.Body().SetStr("body1")
	attrs := log.Attributes()
	attrs.PutStr("str", "string1")
	attrs.PutInt("int", 1)
	attrs.PutDouble("double", 1)
	log.SetDroppedAttributesCount(0)
	log.SetFlags(1)
	return log
}

func LogRecord2() plog.LogRecord {
	log := plog.NewLogRecord()
	log.SetTimestamp(3)
	log.SetObservedTimestamp(4)
	log.SetTraceID([16]byte{0xAA})
	log.SetSpanID([8]byte{0xAA})
	log.SetSeverityNumber(2)
	log.SetSeverityText("severity2")
	log.Body().SetStr("body2")
	attrs := log.Attributes()
	attrs.PutStr("str", "string2")
	attrs.PutInt("int", 2)
	attrs.PutDouble("double", 2)
	log.SetDroppedAttributesCount(1)
	log.SetFlags(2)
	return log
}

func ScopeLogs1() plog.ScopeLogs {
	scopeLogs := plog.NewScopeLogs()
	scope := scopeLogs.Scope()
	internal.Scope1().CopyTo(scope)
	scopeLogs.SetSchemaUrl("schema1")
	logRecords := scopeLogs.LogRecords()
	logRecord := logRecords.AppendEmpty()
	LogRecord1().CopyTo(logRecord)
	logRecord = logRecords.AppendEmpty()
	LogRecord2().CopyTo(logRecord)
	return scopeLogs
}

func ScopeLogs2() plog.ScopeLogs {
	scopeLogs := plog.NewScopeLogs()
	scope := scopeLogs.Scope()
	internal.Scope2().CopyTo(scope)
	scopeLogs.SetSchemaUrl("schema2")
	logRecords := scopeLogs.LogRecords()
	logRecord := logRecords.AppendEmpty()
	LogRecord2().CopyTo(logRecord)
	return scopeLogs
}

func ResourceLogs1() plog.ResourceLogs {
	rl := plog.NewResourceLogs()
	resource := rl.Resource()
	internal.Resource1().CopyTo(resource)
	scopeLogsSlice := rl.ScopeLogs()
	scopeLogs := scopeLogsSlice.AppendEmpty()
	ScopeLogs1().CopyTo(scopeLogs)
	scopeLogs = scopeLogsSlice.AppendEmpty()
	ScopeLogs2().CopyTo(scopeLogs)
	rl.SetSchemaUrl("schema1")
	return rl
}

func ResourceLogs2() plog.ResourceLogs {
	rl := plog.NewResourceLogs()
	resource := rl.Resource()
	internal.Resource2().CopyTo(resource)
	scopeLogsSlice := rl.ScopeLogs()
	scopeLogs := scopeLogsSlice.AppendEmpty()
	ScopeLogs2().CopyTo(scopeLogs)
	rl.SetSchemaUrl("schema2")
	return rl
}

func Logs() plog.Logs {
	logs := plog.NewLogs()
	resourceLogsSlice := logs.ResourceLogs()
	resourceLogs := resourceLogsSlice.AppendEmpty()
	ResourceLogs1().CopyTo(resourceLogs)
	resourceLogs = resourceLogsSlice.AppendEmpty()
	ResourceLogs2().CopyTo(resourceLogs)
	return logs
}
