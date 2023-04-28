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

// A trace analyzer is a tool designed to generate statistics about the structure
// and content distribution of a stream of OpenTelemetry Protocol (OTLP) traces.
// By using the -stats flag in the benchmark tool, the results of this analysis
// can be conveniently displayed on the console to troubleshoot compression
// ratio issues.

import (
	"encoding/binary"
	"fmt"

	"github.com/HdrHistogram/hdrhistogram-go"
	"github.com/axiomhq/hyperloglog"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
)

type (
	TracesAnalyzer struct {
		TraceCount         int64
		ResourceSpansStats *ResourceSpansStats
	}

	ResourceSpansStats struct {
		TotalCount          int64
		Distribution        *hdrhistogram.Histogram
		ResSpansIDsDistinct *hyperloglog.Sketch
		ResourceStats       *carrow.ResourceStats
		ScopeSpansStats     *ScopeSpansStats
		SchemaUrlStats      *carrow.SchemaUrlStats
	}

	ScopeSpansStats struct {
		Distribution          *hdrhistogram.Histogram
		ScopeSpansIDsDistinct *hyperloglog.Sketch
		ScopeStats            *carrow.ScopeStats
		SchemaUrlStats        *carrow.SchemaUrlStats
		SpanStats             *SpanStats
	}

	SpanStats struct {
		TotalCount        int64
		Distribution      *hdrhistogram.Histogram
		Attributes        *carrow.AttributesStats
		SharedAttributes  *carrow.AttributesStats
		TimeIntervalStats *carrow.TimeIntervalStats
		Name              *carrow.StringStats
		SpanID            *hyperloglog.Sketch
		TraceID           *hyperloglog.Sketch
		ParentSpanID      *hyperloglog.Sketch
		Kind              *hyperloglog.Sketch
		TraceState        *hyperloglog.Sketch
		Events            *EventStats
		DropEventsCount   *hyperloglog.Sketch
		Links             *LinkStats
		DropLinksCount    *hyperloglog.Sketch
		StatusStats       *carrow.StatusStats
	}

	EventStats struct {
		TotalCount       int64
		Missing          int64
		Distribution     *hdrhistogram.Histogram
		Timestamp        *carrow.TimestampStats
		Name             *carrow.StringStats
		Attributes       *carrow.AttributesStats
		SharedAttributes *carrow.AttributesStats
	}

	LinkStats struct {
		TotalCount       int64
		Distribution     *hdrhistogram.Histogram
		TraceID          *hyperloglog.Sketch
		SpanID           *hyperloglog.Sketch
		TraceState       *hyperloglog.Sketch
		Attributes       *carrow.AttributesStats
		SharedAttributes *carrow.AttributesStats
	}
)

func NewTraceAnalyzer() *TracesAnalyzer {
	return &TracesAnalyzer{
		ResourceSpansStats: &ResourceSpansStats{
			Distribution:        hdrhistogram.New(1, 1000000, 2),
			ResSpansIDsDistinct: hyperloglog.New16(),
			ResourceStats: &carrow.ResourceStats{
				AttributesStats: carrow.NewAttributesStats(),
			},
			ScopeSpansStats: &ScopeSpansStats{
				Distribution:          hdrhistogram.New(1, 1000000, 2),
				ScopeSpansIDsDistinct: hyperloglog.New16(),
				ScopeStats: &carrow.ScopeStats{
					AttributesStats: carrow.NewAttributesStats(),
					Name:            carrow.NewStringStats(),
					Version:         carrow.NewStringStats(),
				},
				SpanStats: NewSpanStats(),
				SchemaUrlStats: &carrow.SchemaUrlStats{
					SizeDistribution: hdrhistogram.New(1, 10000, 2),
				},
			},
			SchemaUrlStats: &carrow.SchemaUrlStats{
				SizeDistribution: hdrhistogram.New(1, 10000, 2),
			},
		},
	}
}

func (t *TracesAnalyzer) Analyze(traces *TracesOptimized) {
	t.TraceCount++
	t.ResourceSpansStats.UpdateWith(traces)
}

func (t *TracesAnalyzer) ShowStats(indent string) {
	println()
	print(carrow.Green)
	fmt.Printf("%s%d ExportTraceServiceRequest processed\n", indent, t.TraceCount)
	print(carrow.ColorReset)
	indent += "  "
	t.ResourceSpansStats.ShowStats(indent)
}

func (r *ResourceSpansStats) UpdateWith(traces *TracesOptimized) {
	resSpan := traces.ResourceSpans

	for ID := range traces.ResourceSpansIdx {
		r.ResSpansIDsDistinct.Insert([]byte(ID))
	}

	r.TotalCount += int64(len(resSpan))
	carrow.RequireNoError(r.Distribution.RecordValue(int64(len(resSpan))))

	for _, rs := range resSpan {
		r.ResourceStats.UpdateWith(rs.Resource)
		r.ScopeSpansStats.UpdateWith(rs.ScopeSpans, rs.ScopeSpansIdx)
		r.SchemaUrlStats.UpdateWith(rs.ResourceSchemaUrl)
	}
}

func (r *ResourceSpansStats) ShowStats(indent string) {
	fmt.Printf("%s                                 |         Distribution per request        |\n", indent)
	print(carrow.Green)
	fmt.Printf("%sResourceSpans%s |    Total|Distinct|   Min|   Max|  Mean| Stdev|   P50|   P99|\n", indent, carrow.ColorReset)
	fmt.Printf("%s              |%9d|%8d|%6d|%6d|%6.1f|%6.1f|%6d|%6d|\n", indent,
		r.TotalCount, r.ResSpansIDsDistinct.Estimate(), r.Distribution.Min(), r.Distribution.Max(), r.Distribution.Mean(), r.Distribution.StdDev(), r.Distribution.ValueAtQuantile(50), r.Distribution.ValueAtQuantile(99),
	)
	indent += "  "
	r.ResourceStats.ShowStats(indent)
	r.ScopeSpansStats.ShowStats(indent)
	r.SchemaUrlStats.ShowStats(indent)
}

func (s *ScopeSpansStats) UpdateWith(scopeSpans []*ScopeSpanGroup, scopeSpansIdx map[string]int) {
	carrow.RequireNoError(s.Distribution.RecordValue(int64(len(scopeSpans))))

	for ID := range scopeSpansIdx {
		s.ScopeSpansIDsDistinct.Insert([]byte(ID))
	}

	for _, ss := range scopeSpans {
		s.ScopeStats.UpdateWith(ss.Scope)
		s.SpanStats.UpdateWith(ss)
		s.SchemaUrlStats.UpdateWith(ss.ScopeSchemaUrl)
	}
}

func (s *ScopeSpansStats) ShowStats(indent string) {
	print(carrow.Green)
	fmt.Printf("%sScopeSpans%s |Distinct|   Min|   Max|  Mean| Stdev|   P50|   P99|\n", indent, carrow.ColorReset)
	fmt.Printf("%s           |%8d|%6d|%6d|%6.1f|%6.1f|%6d|%6d|\n", indent,
		s.ScopeSpansIDsDistinct.Estimate(), s.Distribution.Min(), s.Distribution.Max(), s.Distribution.Mean(), s.Distribution.StdDev(), s.Distribution.ValueAtQuantile(50), s.Distribution.ValueAtQuantile(99),
	)
	s.ScopeStats.ShowStats(indent + "  ")
	s.SpanStats.ShowStats(indent + "  ")
	s.SchemaUrlStats.ShowStats(indent + "  ")
}

func NewSpanStats() *SpanStats {
	return &SpanStats{
		Distribution:      hdrhistogram.New(0, 1000000, 2),
		Attributes:        carrow.NewAttributesStats(),
		SharedAttributes:  carrow.NewAttributesStats(),
		TimeIntervalStats: carrow.NewTimeIntervalStats(),
		SpanID:            hyperloglog.New16(),
		TraceID:           hyperloglog.New16(),
		ParentSpanID:      hyperloglog.New16(),
		Name:              carrow.NewStringStats(),
		Kind:              hyperloglog.New16(),
		TraceState:        hyperloglog.New16(),
		Events:            NewEventStats(),
		DropEventsCount:   hyperloglog.New16(),
		Links:             NewLinkStats(),
		DropLinksCount:    hyperloglog.New16(),
		StatusStats:       carrow.NewStatusStats(),
	}
}

func (s *SpanStats) UpdateWith(ss *ScopeSpanGroup) {
	spans := ss.Spans
	carrow.RequireNoError(s.Distribution.RecordValue(int64(len(spans))))

	sharedAttrs := pcommon.NewMap()
	ss.SharedData.sharedAttributes.CopyTo(sharedAttrs)
	s.SharedAttributes.UpdateWith(sharedAttrs, 0)

	s.TimeIntervalStats.UpdateWithSpans(spans)

	for _, span := range spans {
		s.Attributes.UpdateWith(span.Attributes(), span.DroppedAttributesCount())
		s.Name.UpdateWith(span.Name())
		s.SpanID.Insert([]byte(span.SpanID().String()))
		s.TraceID.Insert([]byte(span.TraceID().String()))
		s.ParentSpanID.Insert([]byte(span.ParentSpanID().String()))
		s.Kind.Insert([]byte(span.Kind().String()))
		s.TraceState.Insert([]byte(span.TraceState().AsRaw()))
		s.Events.UpdateWith(span.Events(), span.DroppedEventsCount(), ss.SharedData.sharedEventAttributes)
		s.Links.UpdateWith(span.Links(), span.DroppedLinksCount(), ss.SharedData.sharedLinkAttributes)

		b := make([]byte, 8)
		binary.LittleEndian.PutUint64(b, uint64(span.DroppedEventsCount()))
		s.DropEventsCount.Insert(b)
		binary.LittleEndian.PutUint64(b, uint64(span.DroppedLinksCount()))
		s.DropLinksCount.Insert(b)
		s.StatusStats.UpdateWith(span.Status())
	}

	s.TotalCount += int64(len(spans))
}

func (s *SpanStats) ShowStats(indent string) {
	print(carrow.Green)
	fmt.Printf("%sSpans%s |   Total|   Min|   Max|  Mean|  Stdev|   P50|   P99|\n", indent, carrow.ColorReset)
	fmt.Printf("%s      |%8d|%6d|%6d|%6.1f|%7.1f|%6d|%6d|\n", indent,
		s.TotalCount, s.Distribution.Min(), s.Distribution.Max(), s.Distribution.Mean(), s.Distribution.StdDev(), s.Distribution.ValueAtQuantile(50), s.Distribution.ValueAtQuantile(99),
	)
	indent += "  "
	s.TimeIntervalStats.ShowStats(indent)
	s.Name.ShowStats("Name", indent)
	fmt.Printf("%s             |Distinct|   Total|%%Distinct|\n", indent)
	fmt.Printf("%s%sSpanID%s       |%8d|%8d|%8.1f%%|\n", indent, carrow.Green, carrow.ColorReset, s.SpanID.Estimate(), s.TotalCount, 100.0*float64(s.SpanID.Estimate())/float64(s.TotalCount))
	fmt.Printf("%s%sTraceID%s      |%8d|%8d|%8.1f%%|\n", indent, carrow.Green, carrow.ColorReset, s.TraceID.Estimate(), s.TotalCount, 100.0*float64(s.TraceID.Estimate())/float64(s.TotalCount))
	fmt.Printf("%s%sParentSpanID%s |%8d|%8d|%8.1f%%|\n", indent, carrow.Green, carrow.ColorReset, s.ParentSpanID.Estimate(), s.TotalCount, 100.0*float64(s.ParentSpanID.Estimate())/float64(s.TotalCount))
	fmt.Printf("%s%sKind%s (Distinct=%d)\n", indent, carrow.Green, carrow.ColorReset, s.Kind.Estimate())
	fmt.Printf("%s%sTraceState%s (Distinct=%d)\n", indent, carrow.Green, carrow.ColorReset, s.TraceState.Estimate())

	s.Attributes.ShowStats(indent, "Attributes", carrow.Green)
	s.SharedAttributes.ShowStats(indent, "SharedAttributes", carrow.Cyan)
	s.Events.ShowStats(indent)
	fmt.Printf("%s%sDroppedEventsCount%s (Distinct=%d)\n", indent, carrow.Green, carrow.ColorReset, s.DropEventsCount.Estimate())
	s.Links.ShowStats(indent)
	fmt.Printf("%s%sDroppedLinksCount%s (Distinct=%d)\n", indent, carrow.Green, carrow.ColorReset, s.DropLinksCount.Estimate())
	s.StatusStats.ShowStats(indent)
}

func NewEventStats() *EventStats {
	return &EventStats{
		Distribution:     hdrhistogram.New(0, 10000, 2),
		Timestamp:        carrow.NewTimestampStats(),
		Name:             carrow.NewStringStats(),
		Attributes:       carrow.NewAttributesStats(),
		SharedAttributes: carrow.NewAttributesStats(),
	}
}

func (e *EventStats) UpdateWith(events ptrace.SpanEventSlice, dac uint32, sharedAttributes *common.SharedAttributes) {
	ec := events.Len()

	if ec == 0 {
		e.Missing++
		return
	}

	e.TotalCount += int64(ec)

	carrow.RequireNoError(e.Distribution.RecordValue(int64(ec)))

	for i := 0; i < ec; i++ {
		event := events.At(i)
		e.Timestamp.UpdateWith(event.Timestamp())
		e.Name.UpdateWith(event.Name())
		e.Attributes.UpdateWith(event.Attributes(), dac)
	}

	sharedAttrs := pcommon.NewMap()
	sharedAttributes.CopyTo(sharedAttrs)
	e.SharedAttributes.UpdateWith(sharedAttrs, 0)
}

func (e *EventStats) ShowStats(indent string) {
	print(carrow.Green)
	fmt.Printf("%sEvents%s |  Count|Missing|    Min|    Max|   Mean|  Stdev|    P50|    P99|\n", indent, carrow.ColorReset)
	fmt.Printf("%s       |%7d|%7d|%7d|%7d|%7.1f|%7.1f|%7d|%7d|\n", indent,
		e.TotalCount, e.Missing, e.Distribution.Min(), e.Distribution.Max(), e.Distribution.Mean(), e.Distribution.StdDev(), e.Distribution.ValueAtQuantile(50), e.Distribution.ValueAtQuantile(99),
	)

	indent += "  "

	e.Timestamp.ShowStats("Timestamp", indent)
	e.Name.ShowStats("Name", indent)
	e.Attributes.ShowStats(indent, "Attributes", carrow.Green)
	e.SharedAttributes.ShowStats(indent, "SharedAttributes", carrow.Cyan)
}

func NewLinkStats() *LinkStats {
	return &LinkStats{
		Distribution:     hdrhistogram.New(0, 10000, 2),
		TraceID:          hyperloglog.New16(),
		SpanID:           hyperloglog.New16(),
		TraceState:       hyperloglog.New16(),
		Attributes:       carrow.NewAttributesStats(),
		SharedAttributes: carrow.NewAttributesStats(),
	}
}

func (l *LinkStats) UpdateWith(links ptrace.SpanLinkSlice, dac uint32, sharedAttributes *common.SharedAttributes) {
	l.TotalCount += int64(links.Len())
	carrow.RequireNoError(l.Distribution.RecordValue(int64(links.Len())))
	for i := 0; i < links.Len(); i++ {
		link := links.At(i)
		l.TraceID.Insert([]byte(link.TraceID().String()))
		l.SpanID.Insert([]byte(link.SpanID().String()))
		l.TraceState.Insert([]byte(link.TraceState().AsRaw()))
		l.Attributes.UpdateWith(link.Attributes(), dac)
	}

	sharedAttrs := pcommon.NewMap()
	sharedAttributes.CopyTo(sharedAttrs)
	l.SharedAttributes.UpdateWith(sharedAttrs, 0)
}

func (l *LinkStats) ShowStats(indent string) {
	print(carrow.Green)
	fmt.Printf("%sLinks%s |  Count|    Min|    Max|   Mean|  Stdev|    P50|    P99|\n", indent, carrow.ColorReset)
	fmt.Printf("%s      |%7d|%7d|%7d|%7.1f|%7.1f|%7d|%7d|\n", indent,
		l.TotalCount, l.Distribution.Min(), l.Distribution.Max(), l.Distribution.Mean(), l.Distribution.StdDev(), l.Distribution.ValueAtQuantile(50), l.Distribution.ValueAtQuantile(99),
	)

	indent += "  "

	fmt.Printf("%s           |Distinct|   Total|%%Distinct|\n", indent)
	print(carrow.Green)
	fmt.Printf("%sTraceID%s    |%8d|%8d|%8.1f%%|\n", indent, carrow.ColorReset, l.TraceID.Estimate(), l.TotalCount, 100.0*float64(l.TraceID.Estimate())/float64(l.TotalCount))
	print(carrow.Green)
	fmt.Printf("%sSpanID%s     |%8d|%8d|%8.1f%%|\n", indent, carrow.ColorReset, l.SpanID.Estimate(), l.TotalCount, 100.0*float64(l.SpanID.Estimate())/float64(l.TotalCount))
	print(carrow.Green)
	fmt.Printf("%sTraceState%s |%8d|%8d|%8.1f%%|\n", indent, carrow.ColorReset, l.TraceState.Estimate(), l.TotalCount, 100.0*float64(l.TraceState.Estimate())/float64(l.TotalCount))

	l.Attributes.ShowStats(indent, "Attributes", carrow.Green)
	l.SharedAttributes.ShowStats(indent, "SharedAttributes", carrow.Cyan)
}
