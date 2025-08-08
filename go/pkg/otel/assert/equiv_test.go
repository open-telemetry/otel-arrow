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

package assert

import (
	"encoding/hex"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

func TestEquiv(t *testing.T) {
	t.Parallel()

	stdTesting := NewStdUnitTest(t)
	traces := ptrace.NewTraces()
	rs := traces.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("foo1", "bar")
	rs.Resource().Attributes().PutInt("foo2", 123)
	rs.Resource().Attributes().PutDouble("foo3", 123.0)
	rs.Resource().Attributes().PutBool("foo4", true)
	rs.SetSchemaUrl("https://foo.bar")

	expectedTraces := []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(traces),
	}

	actualTraces := []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(traces),
		ptraceotlp.NewExportRequestFromTraces(traces),
	}
	Equiv(stdTesting, expectedTraces, actualTraces)

	traces = ptrace.NewTraces()
	rs = traces.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("foo", "bar")
	rs.Resource().Attributes().PutStr("baz", "qux")
	rs.SetSchemaUrl("https://foo.bar")
	actualTraces = []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(traces),
	}
	NotEquiv(stdTesting, expectedTraces, actualTraces)
}

func TestEquivSortAndMerge(t *testing.T) {
	t.Parallel()

	stdTesting := NewStdUnitTest(t)
	split_res_and_scope := ptrace.NewTraces()
	rs := split_res_and_scope.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("k2", "v2")
	rs.Resource().Attributes().PutStr("k1", "v1")
	rs.Resource().Attributes().PutStr("k3", "v3")
	ss := rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2")
	ss.Scope().Attributes().PutStr("k1", "v1")
	span := ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123)
	span.SetEndTimestamp(456)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(1234)
	span.SetEndTimestamp(4567)
	span.Attributes().PutStr("k3", "v3")
	span.Attributes().PutStr("k2", "v2")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k1", "v1") // scope with same attributes as above
	ss.Scope().Attributes().PutStr("k2", "v2") // so that spans are merged and scopes are deduplicated
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(12345)
	span.SetEndTimestamp(45678)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")
	rs = split_res_and_scope.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("k3", "v3") // resource with same attributes as above
	rs.Resource().Attributes().PutStr("k2", "v2") // so that spans are merged and resources are deduplicated
	rs.Resource().Attributes().PutStr("k1", "v1")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2") // scope with same attributes as above
	ss.Scope().Attributes().PutStr("k1", "v1") // so that spans are merged and scopes are deduplicated
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123456)
	span.SetEndTimestamp(456789)
	span.Attributes().PutStr("k1", "v1")
	span.Attributes().PutStr("k2", "v2")
	link := span.Links().AppendEmpty()
	link.Attributes().PutStr("k2", "lv2")
	link.Attributes().PutStr("k1", "lv1")
	link.SetTraceID([16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16})
	link.SetSpanID([8]byte{1, 2, 3, 4, 5, 6, 7, 8})
	link = span.Links().AppendEmpty()
	link.Attributes().PutStr("k3", "lv3")
	link.Attributes().PutStr("k1", "lv1")
	link.SetTraceID([16]byte{11, 12, 13, 14, 15, 16, 17, 18, 19, 110, 111, 112, 113, 114, 115, 116})
	link.SetSpanID([8]byte{11, 12, 13, 14, 15, 16, 17, 18})
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2") // scope with different attributes
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123456)
	span.SetEndTimestamp(456789)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	event := span.Events().AppendEmpty()
	event.SetTimestamp(1234567)
	event.SetName("event1")
	event.Attributes().PutStr("k2.1", "ev2")
	event.Attributes().PutStr("k1.2", "ev1")
	event = span.Events().AppendEmpty()
	event.SetTimestamp(12345678)
	event.SetName("event2")
	event.Attributes().PutStr("k3.1", "ev2")
	event.Attributes().PutStr("k1.2", "ev1")

	merged_res_and_scope := ptrace.NewTraces()
	rs = merged_res_and_scope.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("k2", "v2")
	rs.Resource().Attributes().PutStr("k1", "v1")
	rs.Resource().Attributes().PutStr("k3", "v3")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2")
	ss.Scope().Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123)
	span.SetEndTimestamp(456)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(1234)
	span.SetEndTimestamp(4567)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123456)
	span.SetEndTimestamp(456789)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	link = span.Links().AppendEmpty()
	link.Attributes().PutStr("k3", "lv3")
	link.Attributes().PutStr("k1", "lv1")
	link.SetTraceID([16]byte{11, 12, 13, 14, 15, 16, 17, 18, 19, 110, 111, 112, 113, 114, 115, 116})
	link.SetSpanID([8]byte{11, 12, 13, 14, 15, 16, 17, 18})
	link = span.Links().AppendEmpty()
	link.Attributes().PutStr("k2", "lv2")
	link.Attributes().PutStr("k1", "lv1")
	link.SetTraceID([16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16})
	link.SetSpanID([8]byte{1, 2, 3, 4, 5, 6, 7, 8})
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(12345)
	span.SetEndTimestamp(45678)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123456)
	span.SetEndTimestamp(456789)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	event = span.Events().AppendEmpty()
	event.SetTimestamp(12345678)
	event.SetName("event2")
	event.Attributes().PutStr("k3.1", "ev2")
	event.Attributes().PutStr("k1.2", "ev1")
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1234567)
	event.SetName("event1")
	event.Attributes().PutStr("k2.1", "ev2")
	event.Attributes().PutStr("k1.2", "ev1")

	expectedTraces := []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(merged_res_and_scope),
	}
	actualTraces := []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(split_res_and_scope),
	}
	Equiv(stdTesting, expectedTraces, actualTraces)
}

func TestNotEquivSortAndMerge(t *testing.T) {
	t.Parallel()

	stdTesting := NewStdUnitTest(t)
	traces_1 := ptrace.NewTraces()
	rs := traces_1.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("k2", "v2")
	rs.Resource().Attributes().PutStr("k1", "v1")
	rs.Resource().Attributes().PutStr("k3", "v3")
	ss := rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2")
	ss.Scope().Attributes().PutStr("k1", "v1")
	span := ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123)
	span.SetEndTimestamp(456)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(1234)
	span.SetEndTimestamp(4567)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2.1") // scope with different attributes
	ss.Scope().Attributes().PutStr("k1", "v1.1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(12345)
	span.SetEndTimestamp(45678)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")
	rs = traces_1.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("k2", "v2")
	rs.Resource().Attributes().PutStr("k1", "v1")
	rs.Resource().Attributes().PutStr("k3", "v3")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2")
	ss.Scope().Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123456)
	span.SetEndTimestamp(456789)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")

	traces_2 := ptrace.NewTraces()
	rs = traces_2.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("k2", "v2")
	rs.Resource().Attributes().PutStr("k1", "v1")
	rs.Resource().Attributes().PutStr("k3", "v3")
	ss = rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("k2", "v2")
	ss.Scope().Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123)
	span.SetEndTimestamp(456)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(1234)
	span.SetEndTimestamp(4567)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(123456)
	span.SetEndTimestamp(456789)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k1", "v1")
	span = ss.Spans().AppendEmpty()
	span.SetStartTimestamp(12345)
	span.SetEndTimestamp(45678)
	span.Attributes().PutStr("k2", "v2")
	span.Attributes().PutStr("k3", "v3")

	expectedTraces := []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(traces_2),
	}
	actualTraces := []json.Marshaler{
		ptraceotlp.NewExportRequestFromTraces(traces_1),
	}
	NotEquiv(stdTesting, expectedTraces, actualTraces)
}

func TestNonPositionalIndex(t *testing.T) {
	t.Parallel()

	// Resource Metrics
	resMetrics := map[string]interface{}{
		"resource": map[string]interface{}{
			"attributes": map[string]interface{}{
				"k2": "v2",
				"k1": "v1",
				"k3": "v3",
			},
			"schema_url": "https://foo.bar",
		},
		"scopeMetrics": []map[string]interface{}{
			{
				"scope": map[string]interface{}{
					"attributes": map[string]interface{}{
						"k2": "v2",
						"k1": "v1",
					},
					"name":    "foo",
					"version": "1.0.0",
				},
				"schema_url": "https://foo.bar",
				"metrics":    []interface{}{},
			},
		},
	}
	assert.Equal(t, nonPositionalIndex("resourceMetrics", resMetrics), "{attributes={k1=v1,k2=v2,k3=v3},schema_url=https://foo.bar}")
	resMetrics = map[string]interface{}{}
	assert.Equal(t, nonPositionalIndex("resourceMetrics", resMetrics), "_")

	// Resource Logs
	resLogs := map[string]interface{}{
		"resource": map[string]interface{}{
			"attributes": map[string]interface{}{
				"k2": "v2",
				"k1": "v1",
				"k3": "v3",
			},
			"schema_url": "https://foo.bar",
		},
		"scopeLogs": []map[string]interface{}{
			{
				"scope": map[string]interface{}{
					"attributes": map[string]interface{}{
						"k2": "v2",
						"k1": "v1",
					},
					"name":    "foo",
					"version": "1.0.0",
				},
				"schema_url": "https://foo.bar",
				"logs":       []interface{}{},
			},
		},
	}
	assert.Equal(t, nonPositionalIndex("resourceLogs", resLogs), "{attributes={k1=v1,k2=v2,k3=v3},schema_url=https://foo.bar}")

	// Resource Spans
	resSpans := map[string]interface{}{
		"resource": map[string]interface{}{
			"attributes": map[string]interface{}{
				"k2": "v2",
				"k1": "v1",
				"k3": "v3",
			},
			"schema_url": "https://foo.bar",
		},
		"scopeSpans": []map[string]interface{}{
			{
				"scope": map[string]interface{}{
					"attributes": map[string]interface{}{
						"k2": "v2",
						"k1": "v1",
					},
					"name":    "foo",
					"version": "1.0.0",
				},
				"schema_url": "https://foo.bar",
				"spans":      []interface{}{},
			},
		},
	}
	assert.Equal(t, nonPositionalIndex("resourceSpans", resSpans), "{attributes={k1=v1,k2=v2,k3=v3},schema_url=https://foo.bar}")

	// Scope Metrics
	scopeMetrics := map[string]interface{}{
		"scope": map[string]interface{}{
			"attributes": map[string]interface{}{
				"k2": "v2",
				"k1": "v1",
			},
			"name":    "foo",
			"version": "1.0.0",
		},
		"metrics": []interface{}{},
	}
	assert.Equal(t, nonPositionalIndex("scopeMetrics", scopeMetrics), "{attributes={k1=v1,k2=v2},name=foo,version=1.0.0}")

	// Scope Logs
	scopeLogs := map[string]interface{}{
		"scope": map[string]interface{}{
			"attributes": map[string]interface{}{
				"k2": "v2",
				"k1": "v1",
			},
			"name":    "foo",
			"version": "1.0.0",
		},
		"logs": []interface{}{},
	}
	assert.Equal(t, nonPositionalIndex("scopeLogs", scopeLogs), "{attributes={k1=v1,k2=v2},name=foo,version=1.0.0}")

	// Scope Spans
	scopeSpans := map[string]interface{}{
		"scope": map[string]interface{}{
			"attributes": map[string]interface{}{
				"k2": "v2",
				"k1": "v1",
			},
			"name":    "foo",
			"version": "1.0.0",
		},
		"spans": []interface{}{},
	}
	assert.Equal(t, nonPositionalIndex("scopeSpans", scopeSpans), "{attributes={k1=v1,k2=v2},name=foo,version=1.0.0}")

	// Other
	other := map[string]interface{}{
		"foo": "bar",
	}
	assert.Equal(t, nonPositionalIndex("other", other), "_")
}

func TestSig(t *testing.T) {
	t.Parallel()

	// Simple values
	assert.Equal(t, sig(int64(10)), "10")
	assert.Equal(t, sig(3.1415), "3.1415")
	assert.Equal(t, sig(true), "true")
	assert.Equal(t, sig(false), "false")
	assert.Equal(t, sig("foo"), "foo")

	// Array of simple values
	assert.Equal(t, sig([]int64{1, 2, 3}), "[1,2,3]")
	assert.Equal(t, sig([]float64{1.1, 2.1, 3.1}), "[1.1,2.1,3.1]")
	assert.Equal(t, sig([]bool{true, false}), "[true,false]")
	assert.Equal(t, sig([]string{"foo", "bar"}), "[foo,bar]")
	assert.Equal(t, sig([]interface{}{int64(1), "one", true, 1.23, false}), "[1,one,true,1.23,false]")

	// Map of simple values
	assert.Equal(t, sig(map[string]interface{}{"key2": 2, "key1": 1}), "{key1=1,key2=2}")
	assert.Equal(t, sig(map[string]interface{}{"key2": int64(2), "key1": int64(1)}), "{key1=1,key2=2}")
	assert.Equal(t, sig(map[string]interface{}{"key2": 2.1, "key1": 1.1}), "{key1=1.1,key2=2.1}")
	assert.Equal(t, sig(map[string]interface{}{"key2": true, "key1": false}), "{key1=false,key2=true}")
	assert.Equal(t, sig(map[string]interface{}{"key2": "two", "key1": "one"}), "{key1=one,key2=two}")

	// Map containing OTel attributes
	attrs := make([]interface{}, 0)
	attrs = append(attrs, attribute("key2", "value2"))
	attrs = append(attrs, attribute("key1", "value1"))
	attrs = append(attrs, attribute("key3", "value3"))
	assert.Equal(t, sig(map[string]interface{}{
		"name":       "my-service",
		"attributes": attrs,
		"version":    "1.0.0",
	}), "{attributes={key1=value1,key2=value2,key3=value3},name=my-service,version=1.0.0}")
}

func TestTryAttributesSig(t *testing.T) {
	t.Parallel()

	// Valid case
	attrs := make([]interface{}, 0)
	attrs = append(attrs, attribute("key2", "value2"))
	attrs = append(attrs, attribute("key1", "value1"))
	attrs = append(attrs, attribute("key3", "value3"))
	attrsSig, done := tryAttributesSig(attrs)
	// attrs is a valid slice of attributes.
	assert.True(t, done)
	// All key/value pairs are sorted by key.
	assert.Equal(t, attrsSig, "{key1=value1,key2=value2,key3=value3}")

	// Empty attributes
	attrs = make([]interface{}, 0)
	attrsSig, done = tryAttributesSig(attrs)
	// attrs is a valid slice of attributes.
	assert.True(t, done)
	// All key/value pairs are sorted by key.
	assert.Equal(t, attrsSig, "{}")

	// Complex attributes
	attrs = make([]interface{}, 0)
	attrs = append(attrs, attribute("key2", "value2"))
	attrs = append(attrs, attribute("key1", "value1"))
	attrs = append(attrs, map[string]interface{}{
		"key": "key3",
		"value": map[string]interface{}{
			"service.name":    "my-service",
			"service.version": "1.0.0",
			"host.name":       "my-host",
		},
	})
	attrs = append(attrs, map[string]interface{}{
		"key":   "key0",
		"value": []interface{}{int64(1), "one", true, 1.23, false},
	})
	attrs = append(attrs, map[string]interface{}{
		"key":   "key4",
		"value": []bool{true, false, true},
	})
	attrsSig, done = tryAttributesSig(attrs)
	// attrs is a valid slice of attributes.
	assert.True(t, done)
	// All key/value pairs are sorted by key.
	assert.Equal(t, attrsSig, "{key0=[1,one,true,1.23,false],key1=value1,key2=value2,key3={host.name=my-host,service.name=my-service,service.version=1.0.0},key4=[true,false,true]}")

	// Invalid case 1
	attrs = make([]interface{}, 0)
	attrs = append(attrs, attribute("key2", "value2"))
	attrs = append(attrs, attribute("key1", "value1"))
	attrs = append(attrs, map[string]interface{}{
		"value": 2,
	})
	_, done = tryAttributesSig(attrs)
	// attrs is not a valid slice of attributes.
	assert.False(t, done)

	// Invalid case 2
	attrs = make([]interface{}, 0)
	attrs = append(attrs, attribute("key2", "value2"))
	attrs = append(attrs, attribute("key1", "value1"))
	attrs = append(attrs, "bla")
	_, done = tryAttributesSig(attrs)
	// attrs is not a valid slice of attributes.
	assert.False(t, done)
}

func attribute(key string, value interface{}) interface{} {
	return map[string]interface{}{
		"key":   key,
		"value": value,
	}
}

func traceID(id string) [16]byte {
	data, err := hex.DecodeString(id)
	if err != nil {
		panic(err)
	}
	var traceID [16]byte
	copy(traceID[:], data[:16])
	return traceID
}

func spanID(id string) [8]byte {
	data, err := hex.DecodeString(id)
	if err != nil {
		panic(err)
	}
	var spanID [8]byte
	copy(spanID[:], data[:8])
	return spanID
}
