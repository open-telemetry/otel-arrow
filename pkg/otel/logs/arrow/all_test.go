package arrow

import (
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

func TestLogRecord(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewLogRecordBuilder(pool)

	if err := sb.Append(LogRecord1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(LogRecord2()); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":0,"flags":1,"observed_time_unix_nano":2,"severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":1,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}
]`

	require.JSONEq(t, expected, string(json))
}

func TestScopeLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	ssb := NewScopeLogsBuilder(pool)

	if err := ssb.Append(ScopeLogs1()); err != nil {
		t.Fatal(err)
	}
	if err := ssb.Append(ScopeLogs2()); err != nil {
		t.Fatal(err)
	}
	arr, err := ssb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"logs":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":0,"flags":1,"observed_time_unix_nano":2,"severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":1,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":0,"name":"scope1","version":"1.0.1"}}
,{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestResourceLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	rsb := NewResourceLogsBuilder(pool)

	if err := rsb.Append(ResourceLogs1()); err != nil {
		t.Fatal(err)
	}
	if err := rsb.Append(ResourceLogs2()); err != nil {
		t.Fatal(err)
	}
	arr, err := rsb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":0},"schema_url":"schema1","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":0,"flags":1,"observed_time_unix_nano":2,"severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":1,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":0,"name":"scope1","version":"1.0.1"}},{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}
,{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema2","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	logsSchema := acommon.NewAdaptiveSchema(Schema)
	defer logsSchema.Release()
	tb, err := NewLogsBuilder(pool, logsSchema)
	require.NoError(t, err)

	err = tb.Append(Logs())
	require.NoError(t, err)

	record, err := tb.Build()
	require.NoError(t, err)
	defer record.Release()

	json, err := record.MarshalJSON()
	require.NoError(t, err)

	expected := `[{"resource_logs":[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":0},"schema_url":"schema1","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"body":[0,"body1"],"dropped_attributes_count":0,"flags":1,"observed_time_unix_nano":2,"severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":1,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":0,"name":"scope1","version":"1.0.1"}},{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]},{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema2","scope_logs":[{"logs":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"observed_time_unix_nano":4,"severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":3,"trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}]}
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
