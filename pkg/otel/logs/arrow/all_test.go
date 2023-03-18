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
	"math"
	"testing"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

var DefaultDictConfig = &cfg.Dictionary{
	MaxCard: math.MaxUint16,
}

func TestLogRecord(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.Logs, Type: LogRecordDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := LogRecordBuilderFrom(rBuilder.StructBuilder(constants.Logs))

		err := sb.Append(LogRecord1())
		require.NoError(t, err)

		err = sb.Append(LogRecord2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"logs":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}}
,{"logs":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestScopeLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ScopeLogs, Type: ScopeLogsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		ssb := ScopeLogsBuilderFrom(rBuilder.StructBuilder(constants.ScopeLogs))

		err := ssb.Append(ScopeLogs1())
		require.NoError(t, err)

		err = ssb.Append(ScopeLogs2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"scope_logs":{"logs":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}}}
,{"scope_logs":{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestResourceLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceLogs, Type: ResourceLogsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		rsb := ResourceLogsBuilderFrom(rBuilder.StructBuilder(constants.ResourceLogs))

		err := rsb.Append(ResourceLogs1())
		require.NoError(t, err)

		err = rsb.Append(ResourceLogs2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"resource_logs":{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema1","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}},{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}}
,{"resource_logs":{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema2","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	rBuilder := builder.NewRecordBuilderExt(pool, Schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		tb, err := NewLogsBuilder(rBuilder)
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(Logs())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	require.NoError(t, err)

	record.Release()

	expected := `[{"resource_logs":[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema1","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}},{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]},{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema2","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}]}
]`
	require.JSONEq(t, expected, string(json))
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
