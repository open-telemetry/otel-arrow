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

package stats

import (
	"fmt"

	"go.opentelemetry.io/collector/pdata/ptrace"
)

type TracesStats struct {
	resourceSpans      *Metric
	resourceAttributes *Metric
	scopeSpans         *Metric
	scopeAttributes    *Metric
	spans              *Metric
	spansAttributes    *Metric
	links              *Metric
	linksAttributes    *Metric
	events             *Metric
	eventsAttributes   *Metric
}

func NewTracesStats() *TracesStats {
	return &TracesStats{
		resourceSpans:      NewMetric(),
		resourceAttributes: NewMetric(),
		scopeSpans:         NewMetric(),
		scopeAttributes:    NewMetric(),
		spans:              NewMetric(),
		spansAttributes:    NewMetric(),
		links:              NewMetric(),
		linksAttributes:    NewMetric(),
		events:             NewMetric(),
		eventsAttributes:   NewMetric(),
	}
}

func (ts *TracesStats) Analyze(traces ptrace.Traces) {
	resSpansSlice := traces.ResourceSpans()
	ts.resourceSpans.Record(float64(resSpansSlice.Len()))

	for i := 0; i < resSpansSlice.Len(); i++ {
		resSpans := resSpansSlice.At(i)
		scopeSpansSlice := resSpans.ScopeSpans()

		ts.scopeSpans.Record(float64(scopeSpansSlice.Len()))
		ts.resourceAttributes.Record(float64(resSpans.Resource().Attributes().Len()))

		for j := 0; j < scopeSpansSlice.Len(); j++ {
			scopeSpans := scopeSpansSlice.At(j)
			spansSlice := scopeSpans.Spans()

			ts.spans.Record(float64(spansSlice.Len()))
			ts.scopeAttributes.Record(float64(scopeSpans.Scope().Attributes().Len()))

			for k := 0; k < spansSlice.Len(); k++ {
				span := spansSlice.At(k)
				attrs := span.Attributes()

				ts.spansAttributes.Record(float64(attrs.Len()))

				linksSlice := span.Links()
				ts.links.Record(float64(linksSlice.Len()))
				for l := 0; l < linksSlice.Len(); l++ {
					link := linksSlice.At(l)
					linkAttrs := link.Attributes()
					ts.linksAttributes.Record(float64(linkAttrs.Len()))
				}

				eventsSlice := span.Events()
				ts.events.Record(float64(eventsSlice.Len()))
				for l := 0; l < eventsSlice.Len(); l++ {
					event := eventsSlice.At(l)
					eventAttrs := event.Attributes()
					ts.eventsAttributes.Record(float64(eventAttrs.Len()))
				}
			}
		}
	}
}

func (ts *TracesStats) ShowStats() {
	fmt.Printf("\t- ResourceSpans           => %s\n", ts.resourceSpans.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Resource     => %s\n", ts.resourceAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- ScopeSpans/ResourceSpan => %s\n", ts.scopeSpans.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Scope        => %s\n", ts.scopeAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- Spans/ScopeSpan         => %s\n", ts.spans.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Span         => %s\n", ts.spansAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- Links/Span              => %s\n", ts.links.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Link         => %s\n", ts.linksAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- Events/Span             => %s\n", ts.events.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Event        => %s\n", ts.eventsAttributes.ComputeSummary().ToString())
}
