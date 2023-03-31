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
	"sort"
	"strings"

	"github.com/HdrHistogram/hdrhistogram-go"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
)

type TracesOptimizer struct {
	sort  bool
	stats *TracesStats
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

type TracesStats struct {
	TracesCount            int
	ResourceSpansHistogram *hdrhistogram.Histogram
	ResourceAttrsHistogram *carrow.AttributesStats
	ScopeSpansHistogram    *hdrhistogram.Histogram
	ScopeAttrsHistogram    *carrow.AttributesStats
	SpansHistogram         *hdrhistogram.Histogram
	SpanAttrsHistogram     *carrow.AttributesStats
	EventsHistogram        *hdrhistogram.Histogram
	EventAttrsHistogram    *carrow.AttributesStats
	LinksHistogram         *hdrhistogram.Histogram
	LinkAttrsHistogram     *carrow.AttributesStats
}

func NewTracesOptimizer(cfg ...func(*carrow.Options)) *TracesOptimizer {
	options := carrow.Options{
		Sort:  false,
		Stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	var s *TracesStats
	if options.Stats {
		s = &TracesStats{
			TracesCount:            0,
			ResourceSpansHistogram: hdrhistogram.New(1, 1000000, 1),
			ResourceAttrsHistogram: carrow.NewAttributesStats(),
			ScopeSpansHistogram:    hdrhistogram.New(1, 1000000, 1),
			ScopeAttrsHistogram:    carrow.NewAttributesStats(),
			SpansHistogram:         hdrhistogram.New(1, 1000000, 1),
			SpanAttrsHistogram:     carrow.NewAttributesStats(),
			EventsHistogram:        hdrhistogram.New(1, 1000000, 1),
			EventAttrsHistogram:    carrow.NewAttributesStats(),
			LinksHistogram:         hdrhistogram.New(1, 1000000, 1),
			LinkAttrsHistogram:     carrow.NewAttributesStats(),
		}
	}

	return &TracesOptimizer{
		sort:  options.Sort,
		stats: s,
	}
}

func (t *TracesOptimizer) Stats() *TracesStats {
	return t.stats
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

	if t.stats != nil {
		tracesOptimized.RecordStats(t.stats)
	}
	return tracesOptimized
}

func (t *TracesOptimized) RecordStats(stats *TracesStats) {
	stats.TracesCount++
	if err := stats.ResourceSpansHistogram.RecordValue(int64(len(t.ResourceSpans))); err != nil {
		panic(fmt.Sprintf("number of resource spans is out of range: %v", err))
	}
	for _, resSpanGroup := range t.ResourceSpans {
		attrs := resSpanGroup.Resource.Attributes()
		stats.ResourceAttrsHistogram.UpdateStats(attrs)
		resSpanGroup.RecordStats(stats)
	}
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

func (t *ResourceSpanGroup) RecordStats(stats *TracesStats) {
	if err := stats.ScopeSpansHistogram.RecordValue(int64(len(t.ScopeSpans))); err != nil {
		panic(fmt.Sprintf("number of scope spans is out of range: %v", err))
	}
	for _, scopeSpansGroup := range t.ScopeSpans {
		attrs := scopeSpansGroup.Scope.Attributes()
		stats.ScopeAttrsHistogram.UpdateStats(attrs)
		scopeSpansGroup.RecordStats(stats)
	}
}

func (t *ScopeSpanGroup) RecordStats(stats *TracesStats) {
	if err := stats.SpansHistogram.RecordValue(int64(len(t.Spans))); err != nil {
		panic(fmt.Sprintf("number of spans is out of range: %v", err))
	}
	for _, span := range t.Spans {
		attrs := span.Attributes()
		stats.SpanAttrsHistogram.UpdateStats(attrs)
		spanEventsSlice := span.Events()
		if spanEventsSlice.Len() > 0 {
			if err := stats.EventsHistogram.RecordValue(int64(spanEventsSlice.Len())); err != nil {
				panic(fmt.Sprintf("number of events is out of range: %v", err))
			}
			for i := 0; i < spanEventsSlice.Len(); i++ {
				event := spanEventsSlice.At(i)
				attrs = event.Attributes()
				stats.EventAttrsHistogram.UpdateStats(attrs)
			}
		}
		spanLinksSlice := span.Links()
		if spanLinksSlice.Len() > 0 {
			if err := stats.LinksHistogram.RecordValue(int64(spanLinksSlice.Len())); err != nil {
				panic(fmt.Sprintf("number of links is out of range: %v", err))
			}
			for i := 0; i < spanLinksSlice.Len(); i++ {
				link := spanLinksSlice.At(i)
				attrs = link.Attributes()
				stats.LinkAttrsHistogram.UpdateStats(attrs)
			}
		}
	}
}

func (t *TracesStats) Show() {
	fmt.Printf("\n== Traces structure distribution ============================================================\n")
	fmt.Printf("Traces           -> %d\n", t.TracesCount)
	fmt.Printf("  ResourceSpans  -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ResourceSpansHistogram.Mean(),
		t.ResourceSpansHistogram.Min(),
		t.ResourceSpansHistogram.Max(),
		t.ResourceSpansHistogram.StdDev(),
		t.ResourceSpansHistogram.ValueAtQuantile(50),
		t.ResourceSpansHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("    Resource\n")
	t.ResourceAttrsHistogram.Show("      ")
	fmt.Printf("    ScopeSpans     -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ScopeSpansHistogram.Mean(),
		t.ScopeSpansHistogram.Min(),
		t.ScopeSpansHistogram.Max(),
		t.ScopeSpansHistogram.StdDev(),
		t.ScopeSpansHistogram.ValueAtQuantile(50),
		t.ScopeSpansHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("      Scope\n")
	t.ScopeAttrsHistogram.Show("        ")
	fmt.Printf("      Spans        -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.SpansHistogram.Mean(),
		t.SpansHistogram.Min(),
		t.SpansHistogram.Max(),
		t.SpansHistogram.StdDev(),
		t.SpansHistogram.ValueAtQuantile(50),
		t.SpansHistogram.ValueAtQuantile(99),
	)
	t.SpanAttrsHistogram.Show("        ")
	if t.EventsHistogram.Mean() > 0 {
		fmt.Printf("        Events       -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			t.EventsHistogram.Mean(),
			t.EventsHistogram.Min(),
			t.EventsHistogram.Max(),
			t.EventsHistogram.StdDev(),
			t.EventsHistogram.ValueAtQuantile(50),
			t.EventsHistogram.ValueAtQuantile(99),
		)
		t.EventAttrsHistogram.Show("          ")
	}
	if t.LinksHistogram.Mean() > 0 {
		fmt.Printf("        Links        -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			t.LinksHistogram.Mean(),
			t.LinksHistogram.Min(),
			t.LinksHistogram.Max(),
			t.LinksHistogram.StdDev(),
			t.LinksHistogram.ValueAtQuantile(50),
			t.LinksHistogram.ValueAtQuantile(99),
		)
		t.LinkAttrsHistogram.Show("          ")
	}
}
