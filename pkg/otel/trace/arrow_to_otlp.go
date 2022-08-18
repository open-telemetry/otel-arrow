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

	coltrace "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	tracepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/trace/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
)

func ArrowRecordsToOtlpTrace(record arrow.Record) (*coltrace.ExportTraceServiceRequest, error) {
	request := coltrace.ExportTraceServiceRequest{
		ResourceSpans: []*tracepb.ResourceSpans{},
	}

	resourceSpans := map[string]*tracepb.ResourceSpans{}
	scopeSpans := map[string]*tracepb.ScopeSpans{}

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resource, err := common.NewResourceFrom(record, i)
		if err != nil {
			return nil, err
		}
		resId := common.ResourceId(resource)
		if _, ok := resourceSpans[resId]; !ok {
			rs := &tracepb.ResourceSpans{
				Resource:   resource,
				ScopeSpans: []*tracepb.ScopeSpans{},
				SchemaUrl:  "",
			}
			resourceSpans[resId] = rs
		}
		rs := resourceSpans[resId]

		scope, err := common.NewInstrumentationScopeFrom(record, i, constants.SCOPE_SPANS)
		if err != nil {
			return nil, err
		}
		scopeSpanId := resId + "|" + common.ScopeId(scope)
		if _, ok := scopeSpans[scopeSpanId]; !ok {
			ss := &tracepb.ScopeSpans{
				Scope:     scope,
				Spans:     []*tracepb.Span{},
				SchemaUrl: "",
			}
			scopeSpans[scopeSpanId] = ss
			rs.ScopeSpans = append(rs.ScopeSpans, ss)
		}
		ss := scopeSpans[scopeSpanId]

		span, err := NewSpanFrom(record, i)
		if err != nil {
			return nil, err
		}
		ss.Spans = append(ss.Spans, span)
	}

	for _, resourceSpan := range resourceSpans {
		request.ResourceSpans = append(request.ResourceSpans, resourceSpan)
	}

	return &request, nil
}

func NewSpanFrom(record arrow.Record, row int) (*tracepb.Span, error) {
	traceId, err := air.BinaryFromRecord(record, row, constants.TRACE_ID)
	if err != nil {
		return nil, err
	}
	spanId, err := air.BinaryFromRecord(record, row, constants.SPAN_ID)
	if err != nil {
		return nil, err
	}
	traceState, err := air.StringFromRecord(record, row, constants.TRACE_STATE)
	if err != nil {
		return nil, err
	}
	parentSpanId, err := air.BinaryFromRecord(record, row, constants.PARENT_SPAN_ID)
	if err != nil {
		return nil, err
	}
	name, err := air.StringFromRecord(record, row, constants.NAME)
	if err != nil {
		return nil, err
	}
	kind, err := air.I32FromRecord(record, row, constants.KIND)
	if err != nil {
		return nil, err
	}
	startTimeUnixNano, err := air.U64FromRecord(record, row, constants.START_TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}
	endTimeUnixNano, err := air.U64FromRecord(record, row, constants.END_TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}
	droppedAttributesCount, err := air.U32FromRecord(record, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}
	droppedEventsCount, err := air.U32FromRecord(record, row, constants.DROPPED_EVENTS_COUNT)
	if err != nil {
		return nil, err
	}
	droppedLinksCount, err := air.U32FromRecord(record, row, constants.DROPPED_LINKS_COUNT)
	if err != nil {
		return nil, err
	}
	message, err := air.StringFromRecord(record, row, constants.STATUS_MESSAGE)
	if err != nil {
		return nil, err
	}
	code, err := air.I32FromRecord(record, row, constants.STATUS_CODE)
	if err != nil {
		return nil, err
	}
	attrField, attrColumn := air.FieldArray(record, constants.ATTRIBUTES)
	attributes := []*commonpb.KeyValue(nil)
	if attrColumn != nil {
		attributes, err = common.AttributesFrom(attrField.Type, attrColumn, row)
		if err != nil {
			return nil, err
		}
	}
	events, err := EventsFrom(record, row)
	if err != nil {
		return nil, err
	}
	links, err := LinksFrom(record, row)
	if err != nil {
		return nil, err
	}

	return &tracepb.Span{
		TraceId:                traceId,
		SpanId:                 spanId,
		TraceState:             traceState,
		ParentSpanId:           parentSpanId,
		Name:                   name,
		Kind:                   tracepb.Span_SpanKind(kind),
		StartTimeUnixNano:      startTimeUnixNano,
		EndTimeUnixNano:        endTimeUnixNano,
		Attributes:             attributes,
		DroppedAttributesCount: droppedAttributesCount,
		Events:                 events,
		DroppedEventsCount:     droppedEventsCount,
		Links:                  links,
		DroppedLinksCount:      droppedLinksCount,
		Status: &tracepb.Status{
			Message: message,
			Code:    tracepb.Status_StatusCode(code),
		},
	}, nil
}

func EventsFrom(record arrow.Record, row int) ([]*tracepb.Span_Event, error) {
	eventsColumn := air.Array(record, constants.SPAN_EVENTS)
	if eventsColumn == nil {
		return nil, nil
	}
	switch eventList := eventsColumn.(type) {
	case *array.List:
		if eventList.IsNull(row) {
			return nil, nil
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
			result := make([]*tracepb.Span_Event, 0, end-start)

			for ; start < end; start++ {
				if events.IsNull(start) {
					result = append(result, nil)
					continue
				}
				event := tracepb.Span_Event{}
				if timeUnixNanoFound {
					column := events.Field(timeUnixNanoId)
					value, err := air.U64FromArray(column, start)
					if err != nil {
						return nil, err
					}
					event.TimeUnixNano = value
				}
				if nameFound {
					column := events.Field(nameId)
					value, err := air.StringFromArray(column, start)
					if err != nil {
						return nil, err
					}
					event.Name = value
				}
				if attributesFound {
					value, err := common.AttributesFrom(attributesField.Type, events.Field(attributesId), start)
					if err != nil {
						return nil, err
					}
					event.Attributes = value
				}
				if droppedAttributesCountFound {
					column := events.Field(droppedAttributesCountId)
					value, err := air.U32FromArray(column, start)
					if err != nil {
						return nil, err
					}
					event.DroppedAttributesCount = value
				}
				result = append(result, &event)
			}
			return result, nil
		default:
			return nil, fmt.Errorf("expecting a struct array for the column events but got %T", events)
		}
	default:
		return nil, fmt.Errorf("expecting a list array for the column events but got %T", eventList)
	}
}

func LinksFrom(record arrow.Record, row int) ([]*tracepb.Span_Link, error) {
	linksColumn := air.Array(record, constants.SPAN_LINKS)
	if linksColumn == nil {
		return nil, nil
	}
	switch linkList := linksColumn.(type) {
	case *array.List:
		if linkList.IsNull(row) {
			return nil, nil
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
			result := make([]*tracepb.Span_Link, 0, end-start)

			for ; start < end; start++ {
				if links.IsNull(start) {
					result = append(result, nil)
					continue
				}

				link := tracepb.Span_Link{}
				if traceIdFound {
					column := links.Field(traceIdId)
					value, err := air.BinaryFromArray(column, start)
					if err != nil {
						return nil, err
					}
					link.TraceId = value
				}
				if spanIdFound {
					column := links.Field(spanIdId)
					value, err := air.BinaryFromArray(column, start)
					if err != nil {
						return nil, err
					}
					link.SpanId = value
				}
				if traceStateFound {
					column := links.Field(traceStateId)
					value, err := air.StringFromArray(column, start)
					if err != nil {
						return nil, err
					}
					link.TraceState = value
				}
				if attributesFound {
					value, err := common.AttributesFrom(attributesField.Type, links.Field(attributesId), start)
					if err != nil {
						return nil, err
					}
					link.Attributes = value
				}
				if droppedAttributesCountFound {
					column := links.Field(droppedAttributesCountId)
					value, err := air.U32FromArray(column, start)
					if err != nil {
						return nil, err
					}
					link.DroppedAttributesCount = value
				}
				result = append(result, &link)
			}
			return result, nil
		default:
			return nil, fmt.Errorf("expecting a struct array for the column links but got %T", links)
		}
	default:
		return nil, fmt.Errorf("expecting a list array for the column links but got %T", linkList)
	}
}
