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

package traces

import (
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

// OtlpProducer produces OTLP traces from OTLP Arrow traces.
type OtlpProducer struct {
}

// NewOtlpProducer creates a new OtlpProducer.
func NewOtlpProducer() *OtlpProducer {
	return &OtlpProducer{}
}

// ProduceFrom produces OTLP Traces from an Arrow Record.
func (p *OtlpProducer) ProduceFrom(record arrow.Record) ([]ptrace.Traces, error) {
	return ArrowRecordToOtlpTraces(record)
}

// ArrowRecordToOtlpTraces converts an Arrow Record to an OTLP Trace.
// TODO: add a reference to the OTEP 0156 section that describes this mapping.
func ArrowRecordToOtlpTraces(record arrow.Record) ([]ptrace.Traces, error) {
	// Each first level row in the Arrow Record represents a Traces entity.
	tracesCount := int(record.NumRows())
	allTraces := make([]ptrace.Traces, 0, tracesCount)

	for traceIdx := 0; traceIdx < tracesCount; traceIdx++ {
		traces := ptrace.NewTraces()

		arrowResSpans, err := air.ListOfStructsFromRecord(record, constants.RESOURCE_SPANS, traceIdx)
		if err != nil {
			return allTraces, err
		}
		traces.ResourceSpans().EnsureCapacity(arrowResSpans.End() - arrowResSpans.Start())

		for resSpanIdx := arrowResSpans.Start(); resSpanIdx < arrowResSpans.End(); resSpanIdx++ {
			resSpan := traces.ResourceSpans().AppendEmpty()

			resource, err := common.NewResourceFrom(arrowResSpans, resSpanIdx)
			if err != nil {
				return allTraces, err
			}
			resource.CopyTo(resSpan.Resource())

			schemaUrl, err := arrowResSpans.StringFieldByName(constants.SCHEMA_URL, resSpanIdx)
			if err != nil {
				return allTraces, err
			}
			resSpan.SetSchemaUrl(schemaUrl)

			arrowScopeSpans, err := arrowResSpans.ListOfStructsByName(constants.SCOPE_SPANS, resSpanIdx)
			if err != nil {
				return allTraces, err
			}
			for scopeSpanIdx := arrowScopeSpans.Start(); scopeSpanIdx < arrowScopeSpans.End(); scopeSpanIdx++ {
				scopeSpan := resSpan.ScopeSpans().AppendEmpty()

				scope, err := common.NewScopeFrom(arrowScopeSpans, scopeSpanIdx)
				if err != nil {
					return allTraces, err
				}
				scope.CopyTo(scopeSpan.Scope())

				schemaUrl, err := arrowScopeSpans.StringFieldByName(constants.SCHEMA_URL, scopeSpanIdx)
				if err != nil {
					return allTraces, err
				}
				scopeSpan.SetSchemaUrl(schemaUrl)

				arrowSpans, err := arrowScopeSpans.ListOfStructsByName(constants.SPANS, scopeSpanIdx)
				if err != nil {
					return allTraces, err
				}
				for spanIdx := arrowSpans.Start(); spanIdx < arrowSpans.End(); spanIdx++ {
					span := scopeSpan.Spans().AppendEmpty()
					err = SetSpanFrom(span, arrowSpans, spanIdx)
					if err != nil {
						return allTraces, err
					}
				}
			}
		}

		allTraces = append(allTraces, traces)
	}

	return allTraces, nil
}

// SetSpanFrom initializes a Span from an Arrow representation.
func SetSpanFrom(span ptrace.Span, los *air.ListOfStructs, row int) error {
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
	message, err := air.StringFromStruct(statusDt, statusArr, row, constants.STATUS_MESSAGE)
	if err != nil {
		return err
	}
	code, err := air.I32FromStruct(statusDt, statusArr, row, constants.STATUS_CODE)
	if err != nil {
		return err
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
	span.TraceStateStruct().FromRaw(traceState)
	span.SetParentSpanID(psid)
	span.SetName(name)
	span.SetKind(ptrace.SpanKind(kind))
	span.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
	span.SetEndTimestamp(pcommon.Timestamp(endTimeUnixNano))
	span.SetDroppedAttributesCount(droppedAttributesCount)
	span.SetDroppedEventsCount(droppedEventsCount)
	span.SetDroppedLinksCount(droppedLinksCount)
	span.Status().SetCode(ptrace.StatusCode(code))
	span.Status().SetMessage(message)
	return nil
}

// CopyEventsFrom initializes a Span's Events from an Arrow representation.
func CopyEventsFrom(result ptrace.SpanEventSlice, los *air.ListOfStructs, row int) error {
	eventLos, err := los.ListOfStructsByName(constants.SPAN_EVENTS, row)

	if err != nil {
		return err
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
			link.SetTraceState(ptrace.TraceState(value))
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
