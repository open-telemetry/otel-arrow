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

// A trace optimizer used to regroup spans by resource and scope.

import (
	"bytes"
	"sort"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"golang.org/x/exp/maps"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/pdata"
)

type (
	TracesOptimizer struct {
		sort bool
	}

	TracesOptimized struct {
		ResourceSpansIdx map[string]int // resource span id -> resource span group
		ResourceSpans    []*ResourceSpanGroup
	}

	ResourceSpanGroup struct {
		Resource          *pcommon.Resource
		ResourceSchemaUrl string
		ScopeSpansIdx     map[string]int // scope span id -> scope span group
		ScopeSpans        []*ScopeSpanGroup
	}

	ScopeSpanGroup struct {
		Scope          *pcommon.InstrumentationScope
		ScopeSchemaUrl string

		SharedData *SharedData
		Spans      []*ptrace.Span
	}

	// SharedData contains all the shared attributes between spans, events, and links.
	SharedData struct {
		sharedAttributes      *common.SharedAttributes
		sharedEventAttributes *common.SharedAttributes
		sharedLinkAttributes  *common.SharedAttributes
	}
)

func NewTracesOptimizer(cfg ...func(*carrow.Options)) *TracesOptimizer {
	options := carrow.Options{
		Sort:  false,
		Stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	return &TracesOptimizer{
		sort: options.Sort,
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

	for _, resSpanGroup := range tracesOptimized.ResourceSpans {
		// Compute shared attributes for all spans in the resource span group.
		for _, spg := range resSpanGroup.ScopeSpans {
			spg.SharedData = collectAllSharedAttributes(spg.Spans)
		}

		resSpanGroup.Sort()
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
			spanI := scopeSpanGroup.Spans[i]
			spanJ := scopeSpanGroup.Spans[j]

			var traceI [16]byte
			var traceJ [16]byte

			traceI = spanI.TraceID()
			traceJ = spanJ.TraceID()
			cmp := bytes.Compare(traceI[:], traceJ[:])
			if cmp == 0 {
				return spanI.StartTimestamp() < spanJ.StartTimestamp()
			} else {
				return cmp == -1
			}
		})
	}
}

func collectAllSharedAttributes(spans []*ptrace.Span) *SharedData {
	sharedAttrs := make(map[string]pcommon.Value)
	firstSpan := true

	sharedEventAttrs := make(map[string]pcommon.Value)
	firstEvent := true

	sharedLinkAttrs := make(map[string]pcommon.Value)
	firstLink := true

	for i := 0; i < len(spans); i++ {
		span := spans[i]
		attrs := span.Attributes()

		firstSpan = collectSharedAttributes(&attrs, firstSpan, sharedAttrs)

		// Collect shared event attributes
		eventSlice := span.Events()
		if eventSlice.Len() > 1 {
			for j := 0; j < eventSlice.Len(); j++ {
				event := eventSlice.At(j)
				evtAttrs := event.Attributes()

				firstEvent = collectSharedAttributes(&evtAttrs, firstEvent, sharedEventAttrs)
				if len(sharedEventAttrs) == 0 {
					break
				}
			}
		}

		// Collect shared link attributes
		linkSlice := span.Links()
		if linkSlice.Len() > 1 {
			for j := 0; j < linkSlice.Len(); j++ {
				link := linkSlice.At(j)
				linkAttrs := link.Attributes()

				firstLink = collectSharedAttributes(&linkAttrs, firstLink, sharedLinkAttrs)
				if len(sharedLinkAttrs) == 0 {
					break
				}
			}
		}

		if len(sharedAttrs) == 0 && len(sharedEventAttrs) == 0 && len(sharedLinkAttrs) == 0 {
			break
		}
	}

	if len(spans) == 1 {
		sharedAttrs = make(map[string]pcommon.Value)
	}

	return &SharedData{
		sharedAttributes: &common.SharedAttributes{
			Attributes: sharedAttrs,
		},
		sharedEventAttributes: &common.SharedAttributes{
			Attributes: sharedEventAttrs,
		},
		sharedLinkAttributes: &common.SharedAttributes{
			Attributes: sharedLinkAttrs,
		},
	}
}

func collectSharedAttributes(attrs *pcommon.Map, first bool, sharedAttrs map[string]pcommon.Value) bool {
	if first {
		attrs.Range(func(k string, v pcommon.Value) bool {
			sharedAttrs[k] = v
			return true
		})
		return false
	} else {
		if len(sharedAttrs) > 0 {
			if attrs.Len() == 0 {
				maps.Clear(sharedAttrs)
				return first
			}
			for k, v := range sharedAttrs {
				if otherV, ok := attrs.Get(k); ok {
					if !pdata.ValuesEqual(v, otherV) {
						delete(sharedAttrs, k)
					}
				} else {
					delete(sharedAttrs, k)
				}
			}
		}
	}
	return first
}
