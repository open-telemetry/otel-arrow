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
	"sort"
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/stats"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
)

type TracesOptimizer struct {
	sort  bool
	stats *stats.TracesStats
}

type TracesOptimized struct {
	ResourceSpansIdx map[string]int // resource span id -> resource span group
	ResourceSpans    []*ResourceSpanGroup
}

type ResourceSpanGroup struct {
	Resource          *pcommon.Resource
	ResourceSchemaUrl string
	ScopeSpansIdx     map[string]int // scope span id -> scope span group
	ScopeSpans        []*ScopeSpanGroup
}

type ScopeSpanGroup struct {
	Scope          *pcommon.InstrumentationScope
	ScopeSchemaUrl string

	Spans []*ptrace.Span
}

func NewTracesOptimizer(cfg ...func(*carrow.Options)) *TracesOptimizer {
	options := carrow.Options{
		Sort:  false,
		Stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	var s *stats.TracesStats
	if options.Stats {
		s = stats.NewTracesStats()
	}

	return &TracesOptimizer{
		sort:  options.Sort,
		stats: s,
	}
}

func (t *TracesOptimizer) Optimize(traces ptrace.Traces) *TracesOptimized {
	tracesOptimized := &TracesOptimized{
		ResourceSpansIdx: make(map[string]int),
		ResourceSpans:    make([]*ResourceSpanGroup, 0),
	}

	resSpans := traces.ResourceSpans()
	for i := 0; i < resSpans.Len(); i++ {
		resSpan := resSpans.At(i)
		tracesOptimized.AddResourceSpan(&resSpan)
	}

	if t.sort {
		for _, resSpanGroup := range tracesOptimized.ResourceSpans {
			resSpanGroup.Sort()
		}
	}

	return tracesOptimized
}

func (t *TracesOptimized) AddResourceSpan(resSpan *ptrace.ResourceSpans) {
	resSpanId := otlp.ResourceID(resSpan.Resource(), resSpan.SchemaUrl())
	resSpanGroupIdx, found := t.ResourceSpansIdx[resSpanId]
	if !found {
		res := resSpan.Resource()
		resSpanGroup := &ResourceSpanGroup{
			Resource:          &res,
			ResourceSchemaUrl: resSpan.SchemaUrl(),
			ScopeSpansIdx:     make(map[string]int),
			ScopeSpans:        make([]*ScopeSpanGroup, 0),
		}
		t.ResourceSpans = append(t.ResourceSpans, resSpanGroup)
		resSpanGroupIdx = len(t.ResourceSpans) - 1
		t.ResourceSpansIdx[resSpanId] = resSpanGroupIdx
	}
	scopeSpans := resSpan.ScopeSpans()
	for i := 0; i < scopeSpans.Len(); i++ {
		scopeSpan := scopeSpans.At(i)
		t.ResourceSpans[resSpanGroupIdx].AddScopeSpan(&scopeSpan)
	}
}

func (r *ResourceSpanGroup) AddScopeSpan(scopeSpan *ptrace.ScopeSpans) {
	scopeSpanId := otlp.ScopeID(scopeSpan.Scope(), scopeSpan.SchemaUrl())
	scopeSpanGroupIdx, found := r.ScopeSpansIdx[scopeSpanId]
	if !found {
		scope := scopeSpan.Scope()
		scopeSpanGroup := &ScopeSpanGroup{
			Scope:          &scope,
			ScopeSchemaUrl: scopeSpan.SchemaUrl(),
			Spans:          make([]*ptrace.Span, 0),
		}
		r.ScopeSpans = append(r.ScopeSpans, scopeSpanGroup)
		scopeSpanGroupIdx = len(r.ScopeSpans) - 1
		r.ScopeSpansIdx[scopeSpanId] = scopeSpanGroupIdx
	}
	spansSlice := scopeSpan.Spans()
	for i := 0; i < spansSlice.Len(); i++ {
		spans := spansSlice.At(i)
		scopeSpans := r.ScopeSpans[scopeSpanGroupIdx]
		scopeSpans.Spans = append(scopeSpans.Spans, &spans)
	}
}

func (r *ResourceSpanGroup) Sort() {
	for _, scopeSpanGroup := range r.ScopeSpans {
		sort.Slice(scopeSpanGroup.Spans, func(i, j int) bool {
			return strings.Compare(
				scopeSpanGroup.Spans[i].TraceID().String(),
				scopeSpanGroup.Spans[j].TraceID().String(),
			) == -1
		})
	}
}
