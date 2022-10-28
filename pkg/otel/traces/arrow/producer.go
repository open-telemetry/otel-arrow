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
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/traces"

	"go.opentelemetry.io/collector/pdata/ptrace"
)

// A TopLevelWrapper wraps a [ptrace.Traces] to expose methods of the [common.TopLevelEntities] interface.
type TopLevelWrapper struct {
	traces.Constants
	traces ptrace.Traces
}

// A ResourceSpansSliceWrapper wraps a [ptrace.ResourceSpansSlice] to expose methods of the [common.TopLevelEntitiesSlice]
// interface.
type ResourceSpansSliceWrapper struct {
	rss ptrace.ResourceSpansSlice
}

// A ResourceSpansWrapper wraps a [ptrace.ResourceSpans] to expose methods of the [common.ResourceEntities] interface.
type ResourceSpansWrapper struct {
	rs ptrace.ResourceSpans
}

// Wrap wraps a [ptrace.Traces] to expose methods of the [common.TopLevelEntities] interface.
func Wrap(traces ptrace.Traces) TopLevelWrapper {
	return TopLevelWrapper{traces: traces}
}

// ResourceSlice returns a [ptrace.ResourceSpansSlice].
func (t TopLevelWrapper) ResourceSlice() arrow.TopLevelEntitiesSlice[ptrace.ScopeSpans] {
	return ResourceSpansSliceWrapper{rss: t.traces.ResourceSpans()}
}

// EntityGrouper converts [ptrace.Span]s of a [ptrace.ScopeSpans] into their AIR representation and groups them based
// on a given configuration. A span signature is based on the span, span events, span links attributes when the
// attribute encoding configuration is AttributesAsStructs, otherwise it is an empty string.
func (t TopLevelWrapper) EntityGrouper(scopeSpans ptrace.ScopeSpans, cfg *config.Config) map[string][]rfield.Value {
	scopeSpanList := scopeSpans.Spans()
	spans := make(map[string][]rfield.Value, scopeSpanList.Len())
	for k := 0; k < scopeSpanList.Len(); k++ {
		var spanSig strings.Builder
		span := scopeSpanList.At(k)

		fields := make([]*rfield.Field, 0, 15)
		if ts := span.StartTimestamp(); ts > 0 {
			fields = append(fields, rfield.NewU64Field(constants.START_TIME_UNIX_NANO, uint64(ts)))
		}
		if ts := span.EndTimestamp(); ts > 0 {
			fields = append(fields, rfield.NewU64Field(constants.END_TIME_UNIX_NANO, uint64(ts)))
		}

		if tid := span.TraceID(); !tid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.TRACE_ID, tid[:]))
		}
		if sid := span.SpanID(); !sid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.SPAN_ID, sid[:]))
		}
		if ts := span.TraceState().AsRaw(); ts != "" {
			fields = append(fields, rfield.NewStringField(constants.TRACE_STATE, ts))
		}
		if psid := span.ParentSpanID(); !psid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.PARENT_SPAN_ID, psid[:]))
		}
		if span.Name() != "" {
			fields = append(fields, rfield.NewStringField(constants.NAME, span.Name()))
		}
		fields = append(fields, rfield.NewI32Field(constants.KIND, int32(span.Kind())))
		attributes := common.NewAttributes(span.Attributes(), cfg)
		if attributes != nil {
			fields = append(fields, attributes)
			if cfg.Attribute.Encoding == config.AttributesAsStructs {
				attributes.Normalize()
				attributes.WriteSigType(&spanSig)
			}
		}

		if dc := span.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}

		// Events
		eventsSig, eventsField := events(span.Events(), cfg)
		if eventsField != nil {
			fields = append(fields, eventsField)
			if len(eventsSig) > 0 && cfg.Attribute.Encoding == config.AttributesAsStructs {
				spanSig.WriteString(eventsSig)
			}
		}
		if dc := span.DroppedEventsCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_EVENTS_COUNT, uint32(dc)))
		}

		// Links
		linksSig, linksField := links(span.Links(), cfg)
		if linksField != nil {
			fields = append(fields, linksField)
			if len(linksSig) > 0 && cfg.Attribute.Encoding == config.AttributesAsStructs {
				spanSig.WriteString(linksSig)
			}
		}
		if dc := span.DroppedLinksCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_LINKS_COUNT, uint32(dc)))
		}

		// Status
		statusField := status(span)
		if statusField != nil {
			fields = append(fields, rfield.NewStructField(constants.STATUS, *statusField))
		}

		spanValue := rfield.NewStruct(fields)

		if len(eventsSig) > 0 {
			spanSig.WriteByte(',')
			spanSig.WriteString(eventsSig)
		}
		if len(linksSig) > 0 {
			spanSig.WriteByte(',')
			spanSig.WriteString(linksSig)
		}
		ssig := spanSig.String()
		spans[ssig] = append(spans[ssig], spanValue)
	}
	return spans
}

// Len returns the number of elements in the slice.
func (t ResourceSpansSliceWrapper) Len() int {
	return t.rss.Len()
}

// At returns the element at the given index.
func (t ResourceSpansSliceWrapper) At(i int) arrow.ResourceEntities[ptrace.ScopeSpans] {
	return ResourceSpansWrapper{rs: t.rss.At(i)}
}

// Resource returns the resource associated with the resource spans.
func (t ResourceSpansWrapper) Resource() pcommon.Resource {
	return t.rs.Resource()
}

// SchemaUrl returns the schema URL associated with the resource spans.
func (t ResourceSpansWrapper) SchemaUrl() string {
	return t.rs.SchemaUrl()
}

// ScopeEntities returns the scope spans associated with the resource spans.
func (t ResourceSpansWrapper) ScopeEntities() arrow.ScopeEntitiesSlice[ptrace.ScopeSpans] {
	return t.rs.ScopeSpans()
}

// status converts OTLP span status to their AIR representation or returns nil when the status has no field.
func status(span ptrace.Span) *rfield.Struct {
	fields := make([]*rfield.Field, 0, 2)

	if span.Status().Code() != 0 {
		fields = append(fields, rfield.NewI32Field(constants.STATUS_CODE, int32(span.Status().Code())))
	}
	if span.Status().Message() != "" {
		fields = append(fields, rfield.NewStringField(constants.STATUS_MESSAGE, span.Status().Message()))
	}

	if len(fields) > 0 {
		return rfield.NewStruct(fields)
	} else {
		return nil
	}
}

// events converts OTLP span events into their AIR representation or returns nil when there is no events.
func events(events ptrace.SpanEventSlice, cfg *config.Config) (string, *rfield.Field) {
	if events.Len() == 0 {
		return "", nil
	}

	var sig strings.Builder
	airEvents := make([]rfield.Value, 0, events.Len())

	for i := 0; i < events.Len(); i++ {
		event := events.At(i)
		fields := make([]*rfield.Field, 0, 4)

		if ts := event.Timestamp(); ts > 0 {
			fields = append(fields, rfield.NewU64Field(constants.TIME_UNIX_NANO, uint64(ts)))
		}
		if event.Name() != "" {
			fields = append(fields, rfield.NewStringField(constants.NAME, event.Name()))
		}
		attributes := common.NewAttributes(event.Attributes(), cfg)
		if attributes != nil {
			fields = append(fields, attributes)
			if cfg.Attribute.Encoding == config.AttributesAsStructs {
				attributes.Normalize()
				attributes.WriteSigType(&sig)
			}
		}
		if dc := event.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}
		airEvents = append(airEvents, &rfield.Struct{
			Fields: fields,
		})
	}
	return sig.String(), rfield.NewListField(constants.SPAN_EVENTS, rfield.List{
		Values: airEvents,
	})
}

// links converts OTLP span links into their AIR representation or returns nil when there is no links.
func links(links ptrace.SpanLinkSlice, cfg *config.Config) (string, *rfield.Field) {
	if links.Len() == 0 {
		return "", nil
	}

	var sig strings.Builder
	airLinks := make([]rfield.Value, 0, links.Len())

	for i := 0; i < links.Len(); i++ {
		link := links.At(i)
		fields := make([]*rfield.Field, 0, 5)

		if tid := link.TraceID(); !tid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.TRACE_ID, tid[:]))
		}
		if sid := link.SpanID(); !sid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.SPAN_ID, sid[:]))
		}
		if ts := link.TraceState().AsRaw(); ts != "" {
			fields = append(fields, rfield.NewStringField(constants.TRACE_STATE, ts))
		}
		attributes := common.NewAttributes(link.Attributes(), cfg)
		if attributes != nil {
			fields = append(fields, attributes)
			if cfg.Attribute.Encoding == config.AttributesAsStructs {
				attributes.Normalize()
				attributes.WriteSigType(&sig)
			}
		}
		if dc := link.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}
		airLinks = append(airLinks, &rfield.Struct{
			Fields: fields,
		})
	}
	return sig.String(), rfield.NewListField(constants.SPAN_LINKS, rfield.List{
		Values: airLinks,
	})
}
