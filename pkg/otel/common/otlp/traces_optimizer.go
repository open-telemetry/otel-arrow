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

package otlp

import (
	"sort"
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/stats"
)

type TracesOptimizer struct {
	sort  bool
	stats *stats.TracesStats
}

type TracesOptimized struct {
	ResourceSpans map[string]*ResourceSpanGroup // resource span id -> resource span group
}

type ResourceSpanGroup struct {
	Resource          *pcommon.Resource
	ResourceSchemaUrl string
	ScopeSpans        map[string]*ScopeSpanGroup // scope span id -> scope span group
}

type ScopeSpanGroup struct {
	Scope          *pcommon.InstrumentationScope
	ScopeSchemaUrl string

	Spans []*ptrace.Span
}

type Options struct {
	sort  bool
	stats bool
}

func WithSort() func(*Options) {
	return func(o *Options) {
		o.sort = true
	}
}

func WithStats() func(*Options) {
	return func(o *Options) {
		o.stats = true
	}
}

func NewTracesOptimizer(cfg ...func(*Options)) *TracesOptimizer {
	options := Options{
		sort:  false,
		stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	var s *stats.TracesStats
	if options.stats {
		s = stats.NewTracesStats()
	}

	return &TracesOptimizer{
		sort:  options.sort,
		stats: s,
	}
}

func (t *TracesOptimizer) Optimize(traces ptrace.Traces) *TracesOptimized {
	tracesOptimized := &TracesOptimized{
		ResourceSpans: make(map[string]*ResourceSpanGroup),
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
	resSpanId := ResourceID(resSpan.Resource(), resSpan.SchemaUrl())
	resSpanGroup, found := t.ResourceSpans[resSpanId]
	if !found {
		res := resSpan.Resource()
		resSpanGroup = &ResourceSpanGroup{
			Resource:          &res,
			ResourceSchemaUrl: resSpan.SchemaUrl(),
			ScopeSpans:        make(map[string]*ScopeSpanGroup),
		}
		t.ResourceSpans[resSpanId] = resSpanGroup
	}
	scopeSpans := resSpan.ScopeSpans()
	for i := 0; i < scopeSpans.Len(); i++ {
		scopeSpan := scopeSpans.At(i)
		resSpanGroup.AddScopeSpan(&scopeSpan)
	}
}

func (r *ResourceSpanGroup) AddScopeSpan(scopeSpan *ptrace.ScopeSpans) {
	scopeSpanId := ScopeID(scopeSpan.Scope(), scopeSpan.SchemaUrl())
	scopeSpanGroup, found := r.ScopeSpans[scopeSpanId]
	if !found {
		scope := scopeSpan.Scope()
		scopeSpanGroup = &ScopeSpanGroup{
			Scope:          &scope,
			ScopeSchemaUrl: scopeSpan.SchemaUrl(),
			Spans:          make([]*ptrace.Span, 0),
		}
		r.ScopeSpans[scopeSpanId] = scopeSpanGroup
	}
	spansSlice := scopeSpan.Spans()
	for i := 0; i < spansSlice.Len(); i++ {
		spans := spansSlice.At(i)
		scopeSpanGroup.Spans = append(scopeSpanGroup.Spans, &spans)
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
