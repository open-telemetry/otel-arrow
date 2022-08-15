/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package dataset

import (
	"fmt"
	"log"
	"os"
	"sort"
	"strings"

	"golang.org/x/exp/rand"
	"google.golang.org/protobuf/proto"

	coltrace "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	otelcommon "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	oteltrace "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/trace/v1"
)

// ===== Real trace dataset =====

type RealTraceDataset struct {
	spans []SpanAPI
	s2r   map[*oteltrace.Span]*oteltrace.ResourceSpans
	s2s   map[*oteltrace.Span]*oteltrace.ScopeSpans
}

type SpanAPI struct {
	*oteltrace.Span
}

type spanSorter struct {
	*RealTraceDataset
	field string
}

var _ sort.Interface = spanSorter{}

func NewRealTraceDataset(path string, sortOrder []string) *RealTraceDataset {
	data, err := os.ReadFile(path)
	if err != nil {
		log.Fatal("read file:", err)
	}
	var otlp coltrace.ExportTraceServiceRequest
	if err := proto.Unmarshal(data, &otlp); err != nil {
		log.Fatal("unmarshal:", err)
	}

	ds := &RealTraceDataset{
		s2r: map[*oteltrace.Span]*oteltrace.ResourceSpans{},
		s2s: map[*oteltrace.Span]*oteltrace.ScopeSpans{},
	}

	for _, rs := range otlp.ResourceSpans {
		for _, ss := range rs.ScopeSpans {
			for _, s := range ss.Spans {
				ds.spans = append(ds.spans, SpanAPI{Span: s})
				ds.s2r[s] = rs
				ds.s2s[s] = ss
			}
		}
	}

	rand.Shuffle(len(ds.spans), spanSorter{RealTraceDataset: ds}.Swap)

	for i := len(sortOrder) - 1; i >= 0; i-- {
		sort.Stable(spanSorter{
			RealTraceDataset: ds,
			field:            sortOrder[i],
		})
	}

	return ds
}

func (d *RealTraceDataset) Len() int {
	return len(d.spans)
}

func (d *RealTraceDataset) Traces(offset, size int) []*coltrace.ExportTraceServiceRequest {
	var otlp coltrace.ExportTraceServiceRequest

	ssm := map[*oteltrace.ScopeSpans]*oteltrace.ScopeSpans{}
	rsm := map[*oteltrace.ResourceSpans]*oteltrace.ResourceSpans{}

	for _, span := range d.spans[offset : offset+size] {
		inscope := d.s2s[span.Span]
		outscope := ssm[inscope]

		if outscope == nil {
			outscope = &oteltrace.ScopeSpans{}
			ssm[inscope] = outscope

			inres := d.s2r[span.Span]
			outres := rsm[inres]

			if outres == nil {
				outres = &oteltrace.ResourceSpans{}
				otlp.ResourceSpans = append(otlp.ResourceSpans, outres)
				outres.Resource = inres.Resource
			}

			outres.ScopeSpans = append(outres.ScopeSpans, outscope)
			outscope.Scope = inscope.Scope
		}

		outscope.Spans = append(outscope.Spans, span.Span)
	}

	return []*coltrace.ExportTraceServiceRequest{&otlp}
}

func v2s(v *otelcommon.AnyValue) string {
	switch t := v.Value.(type) {
	case *otelcommon.AnyValue_StringValue:
		return t.StringValue
	case *otelcommon.AnyValue_BoolValue:
		return fmt.Sprint(t.BoolValue)
	case *otelcommon.AnyValue_IntValue:
		return fmt.Sprint(t.IntValue)
	case *otelcommon.AnyValue_DoubleValue:
		return fmt.Sprint(t.DoubleValue)
	case *otelcommon.AnyValue_ArrayValue:
		return fmt.Sprint(t.ArrayValue)
	case *otelcommon.AnyValue_KvlistValue:
		return fmt.Sprint(t.KvlistValue)
	case *otelcommon.AnyValue_BytesValue:
		return string(t.BytesValue)
	}
	panic("unknown type")
}

func (d *RealTraceDataset) Get(field string, span SpanAPI) string {
	switch field {
	case "trace_id":
		return string(span.TraceId)
	case "span_id":
		return string(span.SpanId)
	default:
		// scan attributes next
	}
	for _, attr := range span.Attributes {
		if attr.Key == field {
			return v2s(attr.Value)
		}
	}
	for _, attr := range d.s2r[span.Span].Resource.Attributes {
		if attr.Key == field {
			return v2s(attr.Value)
		}
	}
	panic(fmt.Sprintf("missing Get lookup: %v %v", field, span))
}

func (ss spanSorter) Len() int {
	return len(ss.spans)
}

func (ss spanSorter) Swap(i, j int) {
	ss.spans[i], ss.spans[j] = ss.spans[j], ss.spans[i]
}

func (ss spanSorter) Less(i, j int) bool {
	return strings.Compare(ss.Get(ss.field, ss.spans[i]), ss.Get(ss.field, ss.spans[j])) < 0
}
