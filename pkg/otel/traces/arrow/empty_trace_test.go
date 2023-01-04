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
)

// An empty trace should not cause an error.
func TestEmptyTrace(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A resource span without resource or scope spans should not cause an error.
func TestEmptyResource(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	require.NotNil(t, rs)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A resource without attributes should not cause an error.
func TestEmptyResourceAttribute(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	r := rs.Resource()
	require.NotNil(t, r)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A resource span with an empty scope spans should not cause an error.
func TestEmptyScopeSpan(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	require.NotNil(t, ss)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A scope without attributes should not cause an error.
func TestEmptyScope(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	s := ss.Scope()
	require.NotNil(t, s)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A scope with empty `attributes` should not cause an error.
func TestEmptyScopeAttribute(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	s := ss.Scope()
	a := s.Attributes()
	require.NotNil(t, a)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A scope spans with no span should not cause an error.
func TestEmptySpans(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	require.NotNil(t, ss)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A span without attributes should not cause an error.
func TestEmptySpanAttribute(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	sp := ss.Spans().AppendEmpty()
	a := sp.Attributes()
	require.NotNil(t, a)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A span without status should not cause an error.
func TestEmptySpanStatus(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	sp := ss.Spans().AppendEmpty()
	s := sp.Status()
	require.NotNil(t, s)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A span without link should not cause an error.
func TestEmptySpanLink(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	sp := ss.Spans().AppendEmpty()
	l := sp.Links().AppendEmpty()
	require.NotNil(t, l)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}

// A span without event should not cause an error.
func TestEmptySpanEvent(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := acommon.NewAdaptiveSchema(Schema)
	defer schema.Release()
	builder, err := NewTracesBuilder(pool, schema)
	require.NoError(t, err)
	defer builder.Release()

	trace := ptrace.NewTraces()
	rs := trace.ResourceSpans().AppendEmpty()
	ss := rs.ScopeSpans().AppendEmpty()
	sp := ss.Spans().AppendEmpty()
	e := sp.Events().AppendEmpty()
	require.NotNil(t, e)

	err = builder.Append(trace)
	require.NoError(t, err)

	record, err := builder.Build()
	defer record.Release()
	require.NoError(t, err)
	require.NotNil(t, record)
}
