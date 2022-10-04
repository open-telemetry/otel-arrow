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

package trace

import (
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

func ArrowRecordsToOtlpTrace(record arrow.Record) (ptrace.Traces, error) {
	request := ptrace.NewTraces()

	resourceSpans := map[string]ptrace.ResourceSpans{}
	scopeSpans := map[string]ptrace.ScopeSpans{}

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resource, err := common.NewResourceFrom(record, i)
		if err != nil {
			return request, err
		}
		resId := common.ResourceId(resource)
		rs, ok := resourceSpans[resId]
		if !ok {
			rs = request.ResourceSpans().AppendEmpty()
			resource.CopyTo(rs.Resource())
			// TODO: SchemaURL
			resourceSpans[resId] = rs
		}

		scope, err := common.NewInstrumentationScopeFrom(record, i, constants.SCOPE_SPANS)
		if err != nil {
			return request, err
		}
		scopeSpanId := resId + "|" + common.ScopeId(scope)
		ss, ok := scopeSpans[scopeSpanId]
		if !ok {
			ss = rs.ScopeSpans().AppendEmpty()
			scope.CopyTo(ss.Scope())
			// TODO: SchemaURL
			scopeSpans[scopeSpanId] = ss
		}

		span := ss.Spans().AppendEmpty()
		err = SetSpanFrom(span, record, i)
		if err != nil {
			return request, err
		}
	}

	return request, nil
}

func SetSpanFrom(span ptrace.Span, record arrow.Record, row int) error {
	traceId, err := air.BinaryFromRecord(record, row, constants.TRACE_ID)
	if err != nil {
		return err
	}
	if len(traceId) != 16 {
		return fmt.Errorf("TraceID field should be 16 bytes")
	}
	spanId, err := air.BinaryFromRecord(record, row, constants.SPAN_ID)
	if err != nil {
		return err
	}
	if len(spanId) != 8 {
		return fmt.Errorf("SpanID field should be 8 bytes")
	}
	traceState, err := air.StringFromRecord(record, row, constants.TRACE_STATE)
	if err != nil {
		return err
	}
	parentSpanId, err := air.BinaryFromRecord(record, row, constants.PARENT_SPAN_ID)
	if err != nil {
		return err
	}
	if len(parentSpanId) != 8 {
		return fmt.Errorf("SpanID field should be 8 bytes")
	}
	name, err := air.StringFromRecord(record, row, constants.NAME)
	if err != nil {
		return err
	}
	kind, err := air.I32FromRecord(record, row, constants.KIND)
	if err != nil {
		return err
	}
	startTimeUnixNano, err := air.U64FromRecord(record, row, constants.START_TIME_UNIX_NANO)
	if err != nil {
		return err
	}
	endTimeUnixNano, err := air.U64FromRecord(record, row, constants.END_TIME_UNIX_NANO)
	if err != nil {
		return err
	}
	droppedAttributesCount, err := air.U32FromRecord(record, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return err
	}
	droppedEventsCount, err := air.U32FromRecord(record, row, constants.DROPPED_EVENTS_COUNT)
	if err != nil {
		return err
	}
	droppedLinksCount, err := air.U32FromRecord(record, row, constants.DROPPED_LINKS_COUNT)
	if err != nil {
		return err
	}
	message, err := air.StringFromRecord(record, row, constants.STATUS_MESSAGE)
	if err != nil {
		return err
	}
	code, err := air.I32FromRecord(record, row, constants.STATUS_CODE)
	if err != nil {
		return err
	}
	if attrField, attrColumn := air.FieldArray(record, constants.ATTRIBUTES); attrColumn != nil {
		if err = common.CopyAttributesFrom(span.Attributes(), attrField.Type, attrColumn, row); err != nil {
			return err
		}
	}
	if err := CopyEventsFrom(span.Events(), record, row); err != nil {
		return err
	}
	if err := CopyLinksFrom(span.Links(), record, row); err != nil {
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

func CopyEventsFrom(result ptrace.SpanEventSlice, record arrow.Record, row int) error {
	eventsColumn := air.Array(record, constants.SPAN_EVENTS)
	if eventsColumn == nil {
		return nil
	}
	switch eventList := eventsColumn.(type) {
	case *array.List:
		if eventList.IsNull(row) {
			return nil
		}
		switch events := eventList.ListValues().(type) {
		case *array.Struct:
			dt := events.DataType().(*arrow.StructType)
			start := int(eventList.Offsets()[row])
			end := int(eventList.Offsets()[row+1])
			timeUnixNanoId, timeUnixNanoFound := dt.FieldIdx(constants.TIME_UNIX_NANO)
			nameId, nameFound := dt.FieldIdx(constants.NAME)
			attributesField, attributesId, attributesFound := air.FieldOfStruct(dt, constants.ATTRIBUTES)
			droppedAttributesCountId, droppedAttributesCountFound := dt.FieldIdx(constants.DROPPED_ATTRIBUTES_COUNT)

			for ; start < end; start++ {
				event := result.AppendEmpty()
				if events.IsNull(start) {
					continue
				}
				if timeUnixNanoFound {
					column := events.Field(timeUnixNanoId)
					value, err := air.U64FromArray(column, start)
					if err != nil {
						return err
					}
					event.SetTimestamp(pcommon.Timestamp(value))
				}
				if nameFound {
					column := events.Field(nameId)
					value, err := air.StringFromArray(column, start)
					if err != nil {
						return err
					}
					event.SetName(value)
				}
				if attributesFound {
					if err := common.CopyAttributesFrom(event.Attributes(), attributesField.Type, events.Field(attributesId), start); err != nil {
						return err
					}
				}
				if droppedAttributesCountFound {
					column := events.Field(droppedAttributesCountId)
					value, err := air.U32FromArray(column, start)
					if err != nil {
						return err
					}
					event.SetDroppedAttributesCount(value)
				}
			}
			return nil
		default:
			return fmt.Errorf("expecting a struct array for the column events but got %T", events)
		}
	default:
		return fmt.Errorf("expecting a list array for the column events but got %T", eventList)
	}
}

func CopyLinksFrom(result ptrace.SpanLinkSlice, record arrow.Record, row int) error {
	linksColumn := air.Array(record, constants.SPAN_LINKS)
	if linksColumn == nil {
		return nil
	}
	switch linkList := linksColumn.(type) {
	case *array.List:
		if linkList.IsNull(row) {
			return nil
		}
		switch links := linkList.ListValues().(type) {
		case *array.Struct:
			dt := links.DataType().(*arrow.StructType)
			start := int(linkList.Offsets()[row])
			end := int(linkList.Offsets()[row+1])
			traceIdId, traceIdFound := dt.FieldIdx(constants.TRACE_ID)
			spanIdId, spanIdFound := dt.FieldIdx(constants.SPAN_ID)
			traceStateId, traceStateFound := dt.FieldIdx(constants.TRACE_STATE)
			attributesField, attributesId, attributesFound := air.FieldOfStruct(dt, constants.ATTRIBUTES)
			droppedAttributesCountId, droppedAttributesCountFound := dt.FieldIdx(constants.DROPPED_ATTRIBUTES_COUNT)

			for ; start < end; start++ {
				link := result.AppendEmpty()

				if links.IsNull(start) {
					continue
				}

				if traceIdFound {
					column := links.Field(traceIdId)
					value, err := air.BinaryFromArray(column, start)
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
					column := links.Field(spanIdId)
					value, err := air.BinaryFromArray(column, start)
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
					column := links.Field(traceStateId)
					value, err := air.StringFromArray(column, start)
					if err != nil {
						return err
					}
					link.SetTraceState(ptrace.TraceState(value))
				}
				if attributesFound {
					if err := common.CopyAttributesFrom(link.Attributes(), attributesField.Type, links.Field(attributesId), start); err != nil {
						return err
					}
				}
				if droppedAttributesCountFound {
					column := links.Field(droppedAttributesCountId)
					value, err := air.U32FromArray(column, start)
					if err != nil {
						return err
					}
					link.SetDroppedAttributesCount(value)
				}
			}
			return nil
		default:
			return fmt.Errorf("expecting a struct array for the column links but got %T", links)
		}
	default:
		return fmt.Errorf("expecting a list array for the column links but got %T", linkList)
	}
}
