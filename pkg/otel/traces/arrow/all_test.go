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

package arrow

import (
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

func TestStatus(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewStatusBuilder(pool)

	if err := sb.Append(Status1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Status2()); err != nil {
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

	expected := `[{"code":1,"status_message":"message1"}
,{"code":2,"status_message":"message2"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestEvent(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	eb := NewEventBuilder(pool)

	if err := eb.Append(Event1()); err != nil {
		t.Fatal(err)
	}
	if err := eb.Append(Event2()); err != nil {
		t.Fatal(err)
	}
	arr, err := eb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"name":"event2","time_unix_nano":"1970-01-01 00:00:00.000000002"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestLink(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	lb := NewLinkBuilder(pool)

	if err := lb.Append(Link1()); err != nil {
		t.Fatal(err)
	}
	if err := lb.Append(Link2()); err != nil {
		t.Fatal(err)
	}
	arr, err := lb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestSpan(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewSpanBuilder(pool)

	if err := sb.Append(Span1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Span2()); err != nil {
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

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"dropped_attributes_count":0,"dropped_events_count":0,"dropped_links_count":0,"end_time_unix_nano":"1970-01-01 00:00:00.000000002","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"name":"event2","time_unix_nano":"1970-01-01 00:00:00.000000002"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span1","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000001","status":{"code":1,"status_message":"message1"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestScopeSpans(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	ssb := NewScopeSpansBuilder(pool)

	if err := ssb.Append(ScopeSpans1()); err != nil {
		t.Fatal(err)
	}
	if err := ssb.Append(ScopeSpans2()); err != nil {
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

	expected := `[{"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"spans":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"dropped_attributes_count":0,"dropped_events_count":0,"dropped_links_count":0,"end_time_unix_nano":"1970-01-01 00:00:00.000000002","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"name":"event2","time_unix_nano":"1970-01-01 00:00:00.000000002"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span1","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000001","status":{"code":1,"status_message":"message1"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]}
,{"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"spans":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestResourceSpans(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	rsb := NewResourceSpansBuilder(pool)

	if err := rsb.Append(ResourceSpans1()); err != nil {
		t.Fatal(err)
	}
	if err := rsb.Append(ResourceSpans2()); err != nil {
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

	expected := `[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema1","scope_spans":[{"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"spans":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"dropped_attributes_count":0,"dropped_events_count":0,"dropped_links_count":0,"end_time_unix_nano":"1970-01-01 00:00:00.000000002","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"name":"event2","time_unix_nano":"1970-01-01 00:00:00.000000002"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span1","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000001","status":{"code":1,"status_message":"message1"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]},{"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"spans":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]}]}
,{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema2","scope_spans":[{"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"spans":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestTraces(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	traceSchema := acommon.NewAdaptiveSchema(pool, Schema)
	defer traceSchema.Release()
	tb, err := NewTracesBuilder(traceSchema)
	require.NoError(t, err)
	defer tb.Release()

	if err := tb.Append(Traces()); err != nil {
		t.Fatal(err)
	}
	record, err := tb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer record.Release()

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"resource_spans":[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema1","scope_spans":[{"schema_url":"schema1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"spans":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]}],"dropped_attributes_count":0,"dropped_events_count":0,"dropped_links_count":0,"end_time_unix_nano":"1970-01-01 00:00:00.000000002","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"name":"event2","time_unix_nano":"1970-01-01 00:00:00.000000002"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span1","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000001","status":{"code":1,"status_message":"message1"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value1"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]},{"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"spans":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]}]},{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema2","scope_spans":[{"schema_url":"schema2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"spans":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]}],"dropped_attributes_count":1,"dropped_events_count":1,"dropped_links_count":1,"end_time_unix_nano":"1970-01-01 00:00:00.000000004","events":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]}],"dropped_attributes_count":0,"name":"event1","time_unix_nano":"1970-01-01 00:00:00.000000001"}],"kind":3,"links":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bool","value":[3,false]}],"dropped_attributes_count":1,"span_id":"qgAAAAAAAAA=","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key2=value2"}],"name":"span2","parent_span_id":"qgAAAAAAAAA=","span_id":"qgAAAAAAAAA=","start_time_unix_nano":"1970-01-01 00:00:00.000000003","status":{"code":2,"status_message":"message2"},"trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","trace_state":"key1=value2"}]}]}]}
]`

	require.JSONEq(t, expected, string(json))
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
