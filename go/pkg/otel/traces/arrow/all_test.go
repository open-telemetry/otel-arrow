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

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"

	v1 "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/pkg/config"
	jsonassert "github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	carrow "github.com/open-telemetry/otel-arrow/pkg/otel/common/arrow"
	acommon "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema"
	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/builder"
	cfg "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/constants"
	"github.com/open-telemetry/otel-arrow/pkg/otel/internal"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
	"github.com/open-telemetry/otel-arrow/pkg/record_message"
)

var (
	DefaultDictConfig = cfg.NewDictionary(math.MaxUint16, 0.0)
)

func TestStatus(t *testing.T) {
	t.Parallel()

	producerStats := stats.NewProducerStats()
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.Status, Type: StatusDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig, producerStats, nil)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := StatusBuilderFrom(rBuilder.StructBuilder(constants.Status))

		err := sb.Append(Status1())
		require.NoError(t, err)
		err = sb.Append(Status2())
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

	expected := `[{"status":{"code":1,"status_message":"message1"}}
,{"status":{"code":2,"status_message":"message2"}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestEvent(t *testing.T) {
	t.Parallel()

	producerStats := stats.NewProducerStats()
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	eventRBuilder := builder.NewRecordBuilderExt(pool, EventSchema, DefaultDictConfig, producerStats, nil)
	defer eventRBuilder.Release()
	attrsRBuilder := builder.NewRecordBuilderExt(pool, carrow.AttrsSchema32, DefaultDictConfig, producerStats, nil)
	defer attrsRBuilder.Release()

	var eventsRecord, attrsRecord arrow.Record

	for {
		if eventsRecord != nil {
			eventsRecord.Release()
		}
		if attrsRecord != nil {
			attrsRecord.Release()
		}

		ab := carrow.NewAttrs32Builder(attrsRBuilder, carrow.PayloadTypes.EventAttrs, carrow.SortAttrs32ByTypeParentIdKeyValue())
		eb := NewEventBuilder(eventRBuilder, &EventConfig{
			Sorter: SortEventsByNameTimeUnixNano(),
		})
		eb.SetAttributesAccumulator(ab.Accumulator())

		events := ptrace.NewSpanEventSlice()
		Event1().CopyTo(events.AppendEmpty())
		Event2().CopyTo(events.AppendEmpty())

		err := eb.Accumulator().Append(0, events)
		require.NoError(t, err)

		eventsRecord, err = eb.Build()
		if err != nil {
			assert.Error(t, acommon.ErrSchemaNotUpToDate)
			continue
		}

		attrsRecord, err = ab.TryBuild()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	actualEvents, err := eventsRecord.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}
	eventsRecord.Release()

	actualAttrs, err := attrsRecord.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}
	attrsRecord.Release()

	expectedEvents := `[{"dropped_attributes_count":null,"id":0,"name":"event1","parent_id":0,"time_unix_nano":"1970-01-01 00:00:00.000000001Z"}
,{"dropped_attributes_count":1,"id":1,"name":"event2","parent_id":0,"time_unix_nano":"1970-01-01 00:00:00.000000002Z"}
]`

	expectedAttrs := `[{"bool":null,"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":0,"str":null,"type":4}
]`

	require.JSONEq(t, expectedEvents, string(actualEvents))
	require.JSONEq(t, expectedAttrs, string(actualAttrs))
}

func TestLink(t *testing.T) {
	t.Parallel()

	producerStats := stats.NewProducerStats()
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	linkRBuilder := builder.NewRecordBuilderExt(pool, LinkSchema, DefaultDictConfig, producerStats, nil)
	defer linkRBuilder.Release()
	attrsRBuilder := builder.NewRecordBuilderExt(pool, carrow.AttrsSchema32, DefaultDictConfig, producerStats, nil)
	defer attrsRBuilder.Release()

	var linksRecord, attrsRecord arrow.Record
	tracesConf := DefaultConfig()

	for {
		if linksRecord != nil {
			linksRecord.Release()
		}
		if attrsRecord != nil {
			attrsRecord.Release()
		}

		ab := carrow.NewAttrs32Builder(attrsRBuilder, carrow.PayloadTypes.LinkAttrs, carrow.SortAttrs32ByTypeParentIdKeyValue())
		lb := NewLinkBuilder(linkRBuilder, tracesConf.Link)
		lb.SetAttributesAccumulator(ab.Accumulator())

		links := ptrace.NewSpanLinkSlice()
		Link1().CopyTo(links.AppendEmpty())
		Link2().CopyTo(links.AppendEmpty())

		err := lb.Accumulator().Append(0, links)
		require.NoError(t, err)

		linksRecord, err = lb.Build()
		if err != nil {
			assert.Error(t, acommon.ErrSchemaNotUpToDate)
			continue
		}

		attrsRecord, err = ab.TryBuild()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	linksJson, err := linksRecord.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}
	linksRecord.Release()

	attrsJson, err := attrsRecord.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}
	attrsRecord.Release()

	expectedLinks := `[{"dropped_attributes_count":null,"id":0,"parent_id":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"}
,{"dropped_attributes_count":1,"id":1,"parent_id":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}
]`
	expectedAttrs := `[{"bool":null,"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":0,"str":null,"type":4}
,{"bool":false,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
]`

	require.JSONEq(t, expectedLinks, string(linksJson))
	require.JSONEq(t, expectedAttrs, string(attrsJson))
}

func TestTraces(t *testing.T) {
	t.Parallel()

	stdTesting := jsonassert.NewStdUnitTest(t)

	producerStats := stats.NewProducerStats()
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, TracesSchema, DefaultDictConfig, producerStats, nil)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()
	statistics := stats.NewProducerStats()

	for {
		tb, err := NewTracesBuilder(rBuilder, NewConfig(conf), statistics, nil)
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(Traces())
		require.NoError(t, err)

		record, err = tb.Build()
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
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"dropped_attributes_count":null,"dropped_events_count":null,"dropped_links_count":null,"duration_time_unix_nano":"1ns","id":0,"kind":3,"name":"span1","parent_span_id":"qgAAAAAAAAA=","resource":{"dropped_attributes_count":null,"id":0,"schema_url":"schema1"},"schema_url":"schema1","scope":{"dropped_attributes_count":null,"id":0,"name":"scope1","version":"1.0.1"},"span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000001Z","status":{"code":1,"status_message":"message1"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"}
,{"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"duration_time_unix_nano":"1ns","id":1,"kind":3,"name":"span2","parent_span_id":"qgAAAAAAAAA=","resource":{"dropped_attributes_count":null,"id":0,"schema_url":"schema1"},"schema_url":"schema1","scope":{"dropped_attributes_count":null,"id":0,"name":"scope1","version":"1.0.1"},"span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003Z","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}
,{"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"duration_time_unix_nano":"1ns","id":1,"kind":3,"name":"span2","parent_span_id":"qgAAAAAAAAA=","resource":{"dropped_attributes_count":null,"id":0,"schema_url":"schema1"},"schema_url":"schema2","scope":{"dropped_attributes_count":1,"id":1,"name":"scope2","version":"1.0.2"},"span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003Z","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}
,{"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"duration_time_unix_nano":"1ns","id":1,"kind":3,"name":"span2","parent_span_id":"qgAAAAAAAAA=","resource":{"dropped_attributes_count":1,"id":1,"schema_url":"schema2"},"schema_url":"schema2","scope":{"dropped_attributes_count":1,"id":0,"name":"scope2","version":"1.0.2"},"span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003Z","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}
]`

	jsonassert.JSONCanonicalEq(stdTesting, expected, actual)

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

		case v1.ArrowPayloadType_SPAN_ATTRS:
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
		case v1.ArrowPayloadType_SPAN_EVENTS:
			expected = `[{"dropped_attributes_count":null,"id":0,"name":"event1","parent_id":0,"time_unix_nano":"1970-01-01 00:00:00.000000001Z"}
,{"dropped_attributes_count":null,"id":1,"name":"event1","parent_id":1,"time_unix_nano":"1970-01-01 00:00:00.000000001Z"}
,{"dropped_attributes_count":null,"id":1,"name":"event1","parent_id":1,"time_unix_nano":"1970-01-01 00:00:00.000000001Z"}
,{"dropped_attributes_count":null,"id":1,"name":"event1","parent_id":1,"time_unix_nano":"1970-01-01 00:00:00.000000001Z"}
,{"dropped_attributes_count":1,"id":1,"name":"event2","parent_id":0,"time_unix_nano":"1970-01-01 00:00:00.000000002Z"}
]`
		case v1.ArrowPayloadType_SPAN_EVENT_ATTRS:
			expected = `[{"bool":null,"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":4,"str":"string2","type":1}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":4,"str":null,"type":2}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":4,"str":null,"type":3}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":0,"str":null,"type":4}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
]`

		case v1.ArrowPayloadType_SPAN_LINKS:
			expected = `[{"dropped_attributes_count":null,"id":0,"parent_id":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"}
,{"dropped_attributes_count":1,"id":1,"parent_id":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}
,{"dropped_attributes_count":1,"id":1,"parent_id":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}
,{"dropped_attributes_count":1,"id":1,"parent_id":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}
,{"dropped_attributes_count":1,"id":1,"parent_id":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}
]`

		case v1.ArrowPayloadType_SPAN_LINK_ATTRS:
			expected = `[{"bool":null,"double":null,"int":null,"key":"str","parent_id":0,"str":"string1","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"double":null,"int":null,"key":"str","parent_id":1,"str":"string2","type":1}
,{"bool":null,"double":null,"int":1,"key":"int","parent_id":0,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":null,"int":2,"key":"int","parent_id":1,"str":null,"type":2}
,{"bool":null,"double":1,"int":null,"key":"double","parent_id":0,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":null,"double":2,"int":null,"key":"double","parent_id":1,"str":null,"type":3}
,{"bool":false,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
,{"bool":false,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
,{"bool":false,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
,{"bool":false,"double":null,"int":null,"key":"bool","parent_id":1,"str":null,"type":4}
,{"bool":true,"double":null,"int":null,"key":"bool","parent_id":0,"str":null,"type":4}
]`

		default:
			panic(fmt.Sprint("unexpected payload type: ", relatedRecord.PayloadType()))
		}

		actual, err := relatedRecord.Record().MarshalJSON()
		require.NoError(t, err)
		require.JSONEq(t, expected, string(actual))
	}
}

func Status1() ptrace.Status {
	status := ptrace.NewStatus()
	status.SetCode(ptrace.StatusCodeOk)
	status.SetMessage("message1")
	return status
}

func Status2() ptrace.Status {
	status := ptrace.NewStatus()
	status.SetCode(ptrace.StatusCodeError)
	status.SetMessage("message2")
	return status
}

func Event1() ptrace.SpanEvent {
	event := ptrace.NewSpanEvent()
	event.SetName("event1")
	event.SetTimestamp(1)
	attrs := event.Attributes()
	attrs.PutStr("str", "string1")
	attrs.PutInt("int", 1)
	attrs.PutDouble("double", 1)
	attrs.PutBool("bool", true)
	event.SetDroppedAttributesCount(0)
	return event
}

func Event2() ptrace.SpanEvent {
	event := ptrace.NewSpanEvent()
	event.SetName("event2")
	event.SetTimestamp(2)
	attrs := event.Attributes()
	attrs.PutStr("str", "string2")
	attrs.PutInt("int", 2)
	attrs.PutDouble("double", 2)
	event.SetDroppedAttributesCount(1)
	return event
}

func Link1() ptrace.SpanLink {
	link := ptrace.NewSpanLink()
	link.SetTraceID([16]byte{0xAA})
	link.SetSpanID([8]byte{0xAA})
	link.TraceState().FromRaw("key1=value1")
	attrs := link.Attributes()
	attrs.PutStr("str", "string1")
	attrs.PutInt("int", 1)
	attrs.PutDouble("double", 1)
	attrs.PutBool("bool", true)
	link.SetDroppedAttributesCount(0)
	return link
}

func Link2() ptrace.SpanLink {
	link := ptrace.NewSpanLink()
	link.SetTraceID([16]byte{0xAA})
	link.SetSpanID([8]byte{0xAA})
	link.TraceState().FromRaw("key2=value2")
	attrs := link.Attributes()
	attrs.PutStr("str", "string2")
	attrs.PutInt("int", 2)
	attrs.PutDouble("double", 2)
	attrs.PutBool("bool", false)
	link.SetDroppedAttributesCount(1)
	return link
}

func Span1() ptrace.Span {
	span := ptrace.NewSpan()
	span.SetStartTimestamp(1)
	span.SetEndTimestamp(2)
	span.SetTraceID([16]byte{0xAA})
	span.SetSpanID([8]byte{0xAA})
	span.TraceState().FromRaw("key1=value1")
	span.SetParentSpanID([8]byte{0xAA})
	span.SetName("span1")
	span.SetKind(ptrace.SpanKindClient)
	attrs := span.Attributes()
	attrs.PutStr("str", "string1")
	attrs.PutInt("int", 1)
	attrs.PutDouble("double", 1)
	span.SetDroppedAttributesCount(0)
	events := span.Events()
	evt := events.AppendEmpty()
	Event1().CopyTo(evt)
	evt = events.AppendEmpty()
	Event2().CopyTo(evt)
	span.SetDroppedEventsCount(0)
	links := span.Links()
	lnk := links.AppendEmpty()
	Link1().CopyTo(lnk)
	lnk = links.AppendEmpty()
	Link2().CopyTo(lnk)
	span.SetDroppedLinksCount(0)
	status := span.Status()
	Status1().CopyTo(status)
	return span
}

func Span2() ptrace.Span {
	span := ptrace.NewSpan()
	span.SetStartTimestamp(3)
	span.SetEndTimestamp(4)
	span.SetTraceID([16]byte{0xAA})
	span.SetSpanID([8]byte{0xAA})
	span.TraceState().FromRaw("key1=value2")
	span.SetParentSpanID([8]byte{0xAA})
	span.SetName("span2")
	span.SetKind(ptrace.SpanKindClient)
	attrs := span.Attributes()
	attrs.PutStr("str", "string2")
	attrs.PutInt("int", 2)
	attrs.PutDouble("double", 2)
	span.SetDroppedAttributesCount(1)
	events := span.Events()
	evt := events.AppendEmpty()
	Event1().CopyTo(evt)
	span.SetDroppedEventsCount(1)
	links := span.Links()
	lnk := links.AppendEmpty()
	Link2().CopyTo(lnk)
	span.SetDroppedLinksCount(1)
	status := span.Status()
	Status2().CopyTo(status)
	return span
}

func ScopeSpans1() ptrace.ScopeSpans {
	scopeSpans := ptrace.NewScopeSpans()
	scope := scopeSpans.Scope()
	internal.Scope1().CopyTo(scope)
	scopeSpans.SetSchemaUrl("schema1")
	spans := scopeSpans.Spans()
	span := spans.AppendEmpty()
	Span1().CopyTo(span)
	span = spans.AppendEmpty()
	Span2().CopyTo(span)
	return scopeSpans
}

func ScopeSpans2() ptrace.ScopeSpans {
	scopeSpans := ptrace.NewScopeSpans()
	scope := scopeSpans.Scope()
	internal.Scope2().CopyTo(scope)
	scopeSpans.SetSchemaUrl("schema2")
	spans := scopeSpans.Spans()
	span := spans.AppendEmpty()
	Span2().CopyTo(span)
	return scopeSpans
}

func ResourceSpans1() ptrace.ResourceSpans {
	rs := ptrace.NewResourceSpans()
	resource := rs.Resource()
	internal.Resource1().CopyTo(resource)
	scopeSpansSlice := rs.ScopeSpans()
	scopeSpans := scopeSpansSlice.AppendEmpty()
	ScopeSpans1().CopyTo(scopeSpans)
	scopeSpans = scopeSpansSlice.AppendEmpty()
	ScopeSpans2().CopyTo(scopeSpans)
	rs.SetSchemaUrl("schema1")
	return rs
}

func ResourceSpans2() ptrace.ResourceSpans {
	rs := ptrace.NewResourceSpans()
	resource := rs.Resource()
	internal.Resource2().CopyTo(resource)
	scopeSpansSlice := rs.ScopeSpans()
	scopeSpans := scopeSpansSlice.AppendEmpty()
	ScopeSpans2().CopyTo(scopeSpans)
	rs.SetSchemaUrl("schema2")
	return rs
}

func Traces() ptrace.Traces {
	traces := ptrace.NewTraces()
	resourceSpansSlice := traces.ResourceSpans()
	resourceSpans := resourceSpansSlice.AppendEmpty()
	ResourceSpans1().CopyTo(resourceSpans)
	resourceSpans = resourceSpansSlice.AppendEmpty()
	ResourceSpans2().CopyTo(resourceSpans)
	return traces
}
