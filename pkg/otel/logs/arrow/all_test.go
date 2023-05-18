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
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)
var ProducerStats = stats.NewProducerStats()

func TestLogRecord(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.Logs, Type: LogRecordDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage
	relatedData, err := NewRelatedData(config.DefaultConfig(), stats.NewProducerStats())
	require.NoError(t, err)

	for {
		relatedData.Reset()
		if relatedRecords != nil {
			for _, r := range relatedRecords {
				r.Record().Release()
			}
		}

		sb := LogRecordBuilderFrom(rBuilder.StructBuilder(constants.Logs))

		logRecord := LogRecord1()
		err := sb.Append(&logRecord, relatedData)
		require.NoError(t, err)

		logRecord = LogRecord2()
		err = sb.Append(&logRecord, relatedData)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err != nil {
			assert.Error(t, acommon.ErrSchemaNotUpToDate)
			continue
		}

		relatedRecords, err = relatedData.BuildRecordMessages()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	actual, err := record.MarshalJSON()
	require.NoError(t, err)

	record.Release()

	expected := `[{"logs":{"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"id":0,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}}
,{"logs":{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}}
]`
	require.JSONEq(t, expected, string(actual))

	for _, relatedRecord := range relatedRecords {
		switch relatedRecord.PayloadType() {
		case v1.OtlpArrowPayloadType_LOG_ATTRS:
			expected = `[{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
]`

		default:
			panic("unexpected payload type")
		}

		actual, err := relatedRecord.Record().MarshalJSON()
		require.NoError(t, err)
		relatedRecord.Record().Release()

		require.JSONEq(t, expected, string(actual))

		relatedRecord.Record().Release()
	}
}

func TestScopeLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ScopeLogs, Type: ScopeLogsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage
	relatedData, err := NewRelatedData(config.DefaultConfig(), stats.NewProducerStats())
	require.NoError(t, err)

	for {
		relatedData.Reset()
		if relatedRecords != nil {
			for _, r := range relatedRecords {
				r.Record().Release()
			}
		}

		ssb := ScopeLogsBuilderFrom(rBuilder.StructBuilder(constants.ScopeLogs))

		err := ssb.Append(ToScopeLogGroup(ScopeLogs1()), relatedData)
		require.NoError(t, err)

		err = ssb.Append(ToScopeLogGroup(ScopeLogs2()), relatedData)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err != nil {
			assert.Error(t, acommon.ErrSchemaNotUpToDate)
			continue
		}

		relatedRecords, err = relatedData.BuildRecordMessages()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	actual, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"scope_logs":{"logs":[{"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"id":0,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"id":0,"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}}}
,{"scope_logs":{"logs":[{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"id":1,"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}}
]`

	require.JSONEq(t, expected, string(actual))

	for _, relatedRecord := range relatedRecords {
		switch relatedRecord.PayloadType() {
		case v1.OtlpArrowPayloadType_SCOPE_ATTRS:
			expected = `[{"parent_id":0,"key":"bool","value":[3,true]}
,{"parent_id":0,"key":"bytes","value":[4,"Ynl0ZXMx"]}
,{"parent_id":1,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
]`

		case v1.OtlpArrowPayloadType_LOG_ATTRS:
			expected = `[{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":2,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":2,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
,{"parent_id":2,"key":"str","value":[0,"string2"]}
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

func ToScopeLogGroup(scopeLogs plog.ScopeLogs) *ScopeLogGroup {
	logs := make([]*plog.LogRecord, 0, scopeLogs.LogRecords().Len())
	scope := scopeLogs.Scope()

	logRecords := scopeLogs.LogRecords()
	for i := 0; i < logRecords.Len(); i++ {
		log := logRecords.At(i)
		logs = append(logs, &log)
	}
	return &ScopeLogGroup{
		Scope:          &scope,
		ScopeSchemaUrl: scopeLogs.SchemaUrl(),
		Logs:           logs,
	}
}

func TestResourceLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceLogs, Type: ResourceLogsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage
	relatedData, err := NewRelatedData(config.DefaultConfig(), stats.NewProducerStats())
	require.NoError(t, err)

	for {
		relatedData.Reset()
		if relatedRecords != nil {
			for _, r := range relatedRecords {
				r.Record().Release()
			}
		}

		rsb := ResourceLogsBuilderFrom(rBuilder.StructBuilder(constants.ResourceLogs))

		err := rsb.Append(ToResourceLogGroup(ResourceLogs1()), relatedData)
		require.NoError(t, err)

		err = rsb.Append(ToResourceLogGroup(ResourceLogs2()), relatedData)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err != nil {
			assert.Error(t, acommon.ErrSchemaNotUpToDate)
			continue
		}

		relatedRecords, err = relatedData.BuildRecordMessages()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	actual, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"resource_logs":{"resource":{"id":0,"dropped_attributes_count":null},"schema_url":"schema1","scope_logs":[{"logs":[{"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"id":0,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"id":0,"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}},{"logs":[{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"id":1,"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}}
,{"resource_logs":{"resource":{"id":1,"dropped_attributes_count":1},"schema_url":"schema2","scope_logs":[{"logs":[{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"id":1,"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}}
]`
	jsonassert.JSONCanonicalEq(t, expected, actual)

	for _, relatedRecord := range relatedRecords {
		switch relatedRecord.PayloadType() {
		case v1.OtlpArrowPayloadType_RESOURCE_ATTRS:
			expected = `[{"parent_id":0,"key":"bool","value":[3,true]}
,{"parent_id":0,"key":"bytes","value":[4,"Ynl0ZXMx"]}
,{"parent_id":1,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
]`

		case v1.OtlpArrowPayloadType_SCOPE_ATTRS:
			expected = `[{"parent_id":0,"key":"bool","value":[3,true]}
,{"parent_id":0,"key":"bytes","value":[4,"Ynl0ZXMx"]}
,{"parent_id":1,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":2,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":2,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":2,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
,{"parent_id":2,"key":"str","value":[0,"string2"]}
]`

		case v1.OtlpArrowPayloadType_LOG_ATTRS:
			expected = `[{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":2,"key":"double","value":[2,2]}
,{"parent_id":3,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":2,"key":"int","value":[1,2]}
,{"parent_id":3,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
,{"parent_id":2,"key":"str","value":[0,"string2"]}
,{"parent_id":3,"key":"str","value":[0,"string2"]}
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

func ToResourceLogGroup(resLogs plog.ResourceLogs) *ResourceLogGroup {
	resource := resLogs.Resource()
	resSpanGroup := ResourceLogGroup{
		Resource:          &resource,
		ResourceSchemaUrl: resLogs.SchemaUrl(),
		ScopeLogsIdx:      make(map[string]int),
		ScopeLogs:         make([]*ScopeLogGroup, 0),
	}
	scopeLogsSlice := resLogs.ScopeLogs()
	for i := 0; i < scopeLogsSlice.Len(); i++ {
		scopeLogs := scopeLogsSlice.At(i)
		resSpanGroup.AddScopeLogs(&scopeLogs)
	}
	return &resSpanGroup
}

func TestLogs(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	rBuilder := builder.NewRecordBuilderExt(pool, Schema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()
	stats := stats.NewProducerStats()

	for {
		tb, err := NewLogsBuilder(rBuilder, conf, stats)
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

	expected := `[{"resource_logs":[{"resource":{"id":0,"dropped_attributes_count":null},"schema_url":"schema1","scope_logs":[{"logs":[{"body":[0,"body1"],"dropped_attributes_count":null,"flags":1,"id":0,"observed_time_unix_nano":"1970-01-01 00:00:00.000000002","severity_number":1,"severity_text":"severity1","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="},{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema1","scope":{"id":0,"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"}},{"logs":[{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"id":1,"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]},{"resource":{"id":1,"dropped_attributes_count":1},"schema_url":"schema2","scope_logs":[{"logs":[{"body":[0,"body2"],"dropped_attributes_count":1,"flags":2,"id":1,"observed_time_unix_nano":"1970-01-01 00:00:00.000000004","severity_number":2,"severity_text":"severity2","span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000003","trace_id":"qgAAAAAAAAAAAAAAAAAAAA=="}],"schema_url":"schema2","scope":{"id":1,"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"}}]}]}
]`

	jsonassert.JSONCanonicalEq(t, expected, actual)

	for _, relatedRecord := range relatedRecords {
		switch relatedRecord.PayloadType() {
		case v1.OtlpArrowPayloadType_RESOURCE_ATTRS:
			expected = `[{"parent_id":0,"key":"bool","value":[3,true]}
,{"parent_id":0,"key":"bytes","value":[4,"Ynl0ZXMx"]}
,{"parent_id":1,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
]`

		case v1.OtlpArrowPayloadType_SCOPE_ATTRS:
			expected = `[{"parent_id":0,"key":"bool","value":[3,true]}
,{"parent_id":0,"key":"bytes","value":[4,"Ynl0ZXMx"]}
,{"parent_id":1,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":2,"key":"bytes","value":[4,"Ynl0ZXMy"]}
,{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":2,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":2,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
,{"parent_id":2,"key":"str","value":[0,"string2"]}
]`

		case v1.OtlpArrowPayloadType_LOG_ATTRS:
			expected = `[{"parent_id":0,"key":"double","value":[2,1]}
,{"parent_id":1,"key":"double","value":[2,2]}
,{"parent_id":2,"key":"double","value":[2,2]}
,{"parent_id":3,"key":"double","value":[2,2]}
,{"parent_id":0,"key":"int","value":[1,1]}
,{"parent_id":1,"key":"int","value":[1,2]}
,{"parent_id":2,"key":"int","value":[1,2]}
,{"parent_id":3,"key":"int","value":[1,2]}
,{"parent_id":0,"key":"str","value":[0,"string1"]}
,{"parent_id":1,"key":"str","value":[0,"string2"]}
,{"parent_id":2,"key":"str","value":[0,"string2"]}
,{"parent_id":3,"key":"str","value":[0,"string2"]}
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
