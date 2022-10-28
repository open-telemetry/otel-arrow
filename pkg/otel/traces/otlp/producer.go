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

package otlp

import (
	"fmt"

	"github.com/f5/otel-arrow-adapter/pkg/air"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/traces"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

type TopLevelWrapper struct {
	traces ptrace.Traces
}

func (w TopLevelWrapper) ResourceEntities() otlp.ResourceEntitiesSlice[ptrace.Span] {
	return ResourceSpansSliceWrapper{slice: w.traces.ResourceSpans()}
}

func (w TopLevelWrapper) Unwrap() ptrace.Traces {
	return w.traces
}

type ResourceSpansSliceWrapper struct {
	slice ptrace.ResourceSpansSlice
}

func (s ResourceSpansSliceWrapper) EnsureCapacity(newCap int) {
	s.slice.EnsureCapacity(newCap)
}

func (s ResourceSpansSliceWrapper) AppendEmpty() otlp.ResourceEntities[ptrace.Span] {
	return ResourceSpansWrapper{resourceSpans: s.slice.AppendEmpty()}
}

type ResourceSpansWrapper struct {
	resourceSpans ptrace.ResourceSpans
}

func (w ResourceSpansWrapper) Resource() pcommon.Resource {
	return w.resourceSpans.Resource()
}
func (w ResourceSpansWrapper) SetSchemaUrl(schemaUrl string) {
	w.resourceSpans.SetSchemaUrl(schemaUrl)
}
func (w ResourceSpansWrapper) ScopeEntities() otlp.ScopeEntities[ptrace.Span] {
	return ScopeSpansWrapper{scopeSpans: w.resourceSpans.ScopeSpans().AppendEmpty()}
}

type ScopeSpansWrapper struct {
	scopeSpans ptrace.ScopeSpans
}

func (w ScopeSpansWrapper) Scope() pcommon.InstrumentationScope {
	return w.scopeSpans.Scope()
}
func (w ScopeSpansWrapper) SetSchemaUrl(schemaUrl string) {
	w.scopeSpans.SetSchemaUrl(schemaUrl)
}
func (w ScopeSpansWrapper) Entity() ptrace.Span {
	return w.scopeSpans.Spans().AppendEmpty()
}

type SpansProducer struct {
	traces.Constants
}

func (p SpansProducer) NewTopLevelEntities() otlp.TopLevelEntities[ptrace.Traces, ptrace.Span] {
	return TopLevelWrapper{ptrace.NewTraces()}
}
func (p SpansProducer) EntityProducer(scopeSpan otlp.ScopeEntities[ptrace.Span], los *air.ListOfStructs, row int) error {
	span := scopeSpan.Entity()
	traceId, err := los.BinaryFieldByName(constants.TRACE_ID, row)
	if err != nil {
		return err
	}
	if len(traceId) != 16 {
		return fmt.Errorf("trace_id field should be 16 bytes")
	}
	spanId, err := los.BinaryFieldByName(constants.SPAN_ID, row)
	if err != nil {
		return err
	}
	if len(spanId) != 8 {
		return fmt.Errorf("span_id field should be 8 bytes")
	}
	traceState, err := los.StringFieldByName(constants.TRACE_STATE, row)
	if err != nil {
		return err
	}
	parentSpanId, err := los.BinaryFieldByName(constants.PARENT_SPAN_ID, row)
	if err != nil {
		return err
	}
	if parentSpanId != nil && len(parentSpanId) != 8 {
		return fmt.Errorf("parent_span_id field should be 8 bytes")
	}
	name, err := los.StringFieldByName(constants.NAME, row)
	if err != nil {
		return err
	}
	kind, err := los.I32FieldByName(constants.KIND, row)
	if err != nil {
		return err
	}
	startTimeUnixNano, err := los.U64FieldByName(constants.START_TIME_UNIX_NANO, row)
	if err != nil {
		return err
	}
	endTimeUnixNano, err := los.U64FieldByName(constants.END_TIME_UNIX_NANO, row)
	if err != nil {
		return err
	}
	droppedAttributesCount, err := los.U32FieldByName(constants.DROPPED_ATTRIBUTES_COUNT, row)
	if err != nil {
		return err
	}
	droppedEventsCount, err := los.U32FieldByName(constants.DROPPED_EVENTS_COUNT, row)
	if err != nil {
		return err
	}
	droppedLinksCount, err := los.U32FieldByName(constants.DROPPED_LINKS_COUNT, row)
	if err != nil {
		return err
	}
	statusDt, statusArr, err := los.StructArray(constants.STATUS, row)
	if err != nil {
		return err
	}
	if statusDt != nil {
		// Status exists
		message, err := air.StringFromStruct(statusDt, statusArr, row, constants.STATUS_MESSAGE)
		if err != nil {
			return err
		}
		span.Status().SetMessage(message)

		code, err := air.I32FromStruct(statusDt, statusArr, row, constants.STATUS_CODE)
		if err != nil {
			return err
		}
		span.Status().SetCode(ptrace.StatusCode(code))
	}
	attrs, err := los.ListOfStructsByName(constants.ATTRIBUTES, row)
	if err != nil {
		return err
	}
	if attrs != nil {
		err = attrs.CopyAttributesFrom(span.Attributes())
	}
	if err := CopyEventsFrom(span.Events(), los, row); err != nil {
		return err
	}
	if err := CopyLinksFrom(span.Links(), los, row); err != nil {
		return err
	}
	var tid pcommon.TraceID
	var sid pcommon.SpanID
	var psid pcommon.SpanID
	copy(tid[:], traceId)
	copy(sid[:], spanId)
	copy(psid[:], parentSpanId)

	span.SetTraceID(tid)
	span.SetSpanID(sid)
	span.TraceState().FromRaw(traceState)
	span.SetParentSpanID(psid)
	span.SetName(name)
	span.SetKind(ptrace.SpanKind(kind))
	span.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
	span.SetEndTimestamp(pcommon.Timestamp(endTimeUnixNano))
	span.SetDroppedAttributesCount(droppedAttributesCount)
	span.SetDroppedEventsCount(droppedEventsCount)
	span.SetDroppedLinksCount(droppedLinksCount)
	return nil
}

// CopyEventsFrom initializes a Span's Events from an Arrow representation.
func CopyEventsFrom(result ptrace.SpanEventSlice, los *air.ListOfStructs, row int) error {
	eventLos, err := los.ListOfStructsByName(constants.SPAN_EVENTS, row)

	if err != nil {
		return err
	}
	if eventLos == nil {
		// No events found
		return nil
	}

	timeUnixNanoId, timeUnixNanoFound := eventLos.FieldIdx(constants.TIME_UNIX_NANO)
	nameId, nameFound := eventLos.FieldIdx(constants.NAME)
	attributesId, attributesFound := eventLos.FieldIdx(constants.ATTRIBUTES)
	droppedAttributesCountId, droppedAttributesCountFound := eventLos.FieldIdx(constants.DROPPED_ATTRIBUTES_COUNT)

	for eventIdx := eventLos.Start(); eventIdx < eventLos.End(); eventIdx++ {
		event := result.AppendEmpty()

		if eventLos.IsNull(eventIdx) {
			continue
		}

		if timeUnixNanoFound {
			value, err := eventLos.U64FieldById(timeUnixNanoId, eventIdx)
			if err != nil {
				return err
			}
			event.SetTimestamp(pcommon.Timestamp(value))
		}
		if nameFound {
			value, err := eventLos.StringFieldById(nameId, eventIdx)
			if err != nil {
				return err
			}
			event.SetName(value)
		}
		if attributesFound {
			attrs, err := eventLos.ListOfStructsById(eventIdx, attributesId, constants.ATTRIBUTES)
			if err != nil {
				return err
			}
			if attrs != nil {
				err = attrs.CopyAttributesFrom(event.Attributes())
			}
		}
		if droppedAttributesCountFound {
			value, err := eventLos.U32FieldById(droppedAttributesCountId, eventIdx)
			if err != nil {
				return err
			}
			event.SetDroppedAttributesCount(value)
		}
	}
	return nil
}

// CopyLinksFrom initializes a Span's Links from an Arrow representation.
func CopyLinksFrom(result ptrace.SpanLinkSlice, los *air.ListOfStructs, row int) error {
	linkLos, err := los.ListOfStructsByName(constants.SPAN_LINKS, row)

	if err != nil {
		return err
	}
	if linkLos == nil {
		// No links found
		return nil
	}

	traceIdId, traceIdFound := linkLos.FieldIdx(constants.TRACE_ID)
	spanIdId, spanIdFound := linkLos.FieldIdx(constants.SPAN_ID)
	traceStateId, traceStateFound := linkLos.FieldIdx(constants.TRACE_STATE)
	attributesId, attributesFound := linkLos.FieldIdx(constants.ATTRIBUTES)
	droppedAttributesCountId, droppedAttributesCountFound := linkLos.FieldIdx(constants.DROPPED_ATTRIBUTES_COUNT)

	for linkIdx := linkLos.Start(); linkIdx < linkLos.End(); linkIdx++ {
		link := result.AppendEmpty()

		if linkLos.IsNull(linkIdx) {
			continue
		}

		if traceIdFound {
			value, err := linkLos.BinaryFieldById(traceIdId, linkIdx)
			if err != nil {
				return err
			}
			if len(value) == 16 {
				var tid pcommon.TraceID
				copy(tid[:], value)
				link.SetTraceID(tid)
			} else {
				return fmt.Errorf("invalid TraceID len")
			}
		}
		if spanIdFound {
			value, err := linkLos.BinaryFieldById(spanIdId, linkIdx)
			if err != nil {
				return err
			}
			if len(value) == 8 {
				var sid pcommon.SpanID
				copy(sid[:], value)
				link.SetSpanID(sid)
			} else {
				return fmt.Errorf("invalid SpanID len")
			}
		}
		if traceStateFound {
			value, err := linkLos.StringFieldById(traceStateId, linkIdx)
			if err != nil {
				return err
			}
			link.TraceState().FromRaw(value)
		}
		if attributesFound {
			attrs, err := linkLos.ListOfStructsById(linkIdx, attributesId, constants.ATTRIBUTES)
			if err != nil {
				return err
			}
			if attrs != nil {
				err = attrs.CopyAttributesFrom(link.Attributes())
			}
		}
		if droppedAttributesCountFound {
			value, err := linkLos.U32FieldById(droppedAttributesCountId, linkIdx)
			if err != nil {
				return err
			}
			link.SetDroppedAttributesCount(value)
		}
	}
	return nil
}
