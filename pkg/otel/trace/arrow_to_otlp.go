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
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
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
	columns := Columns(record)

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resId := ResourceId(record, i)
		if _, ok := resourceSpans[resId]; !ok {
			rs := &tracepb.ResourceSpans{
				Resource:   NewResourceFrom(record, i),
				ScopeSpans: []*tracepb.ScopeSpans{},
				SchemaUrl:  MakeResourceSchemaUrlFrom(record, i),
			}
			resourceSpans[resId] = rs
		}
		rs := resourceSpans[resId]

		scopeSpanId := resId + "|" + ScopeSpanId(record, i)
		if _, ok := scopeSpans[scopeSpanId]; !ok {
			ss := &tracepb.ScopeSpans{
				Scope:     NewInstrumentationScopeFrom(record, i),
				Spans:     []*tracepb.Span{},
				SchemaUrl: MakeScopeSchemaUrlFrom(record, i),
			}
			scopeSpans[scopeSpanId] = ss
			rs.ScopeSpans = append(rs.ScopeSpans, ss)
		}
		ss := scopeSpans[scopeSpanId]

		span, err := NewSpanFrom(columns, record, i)
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

func Columns(record arrow.Record) map[string]int {
	columns := map[string]int{}
	for i := int64(0); i < record.NumCols(); i++ {
		columns[record.ColumnName(int(i))] = int(i)
	}
	return columns
}

func ResourceId(record arrow.Record, row int) string {
	return ""
}

func ScopeSpanId(record arrow.Record, row int) string {
	return ""
}

func NewResourceFrom(record arrow.Record, row int) *resourcepb.Resource {
	return nil
}

func MakeResourceSchemaUrlFrom(record arrow.Record, row int) string {
	return ""
}

func NewInstrumentationScopeFrom(record arrow.Record, row int) *v1.InstrumentationScope {
	return &v1.InstrumentationScope{
		Name:                   "",  // ToDo
		Version:                "",  // ToDo
		Attributes:             nil, // ToDo
		DroppedAttributesCount: 0,
	}
}

func MakeScopeSchemaUrlFrom(record arrow.Record, row int) string {
	return ""
}

func NewSpanFrom(columns map[string]int, record arrow.Record, row int) (*tracepb.Span, error) {
	traceId, err := air.ReadBinary(record, row, constants.TRACE_ID, columns)
	if err != nil {
		return nil, err
	}
	spanId, err := air.ReadBinary(record, row, constants.SPAN_ID, columns)
	if err != nil {
		return nil, err
	}
	traceState, err := air.ReadString(record, row, constants.TRACE_STATE, columns)
	if err != nil {
		return nil, err
	}
	parentSpanId, err := air.ReadBinary(record, row, constants.PARENT_SPAN_ID, columns)
	if err != nil {
		return nil, err
	}
	name, err := air.ReadString(record, row, constants.NAME, columns)
	if err != nil {
		return nil, err
	}
	kind, err := air.ReadInt32(record, row, constants.KIND, columns)
	if err != nil {
		return nil, err
	}
	startTimeUnixNano, err := air.ReadUint64(record, row, constants.START_TIME_UNIX_NANO, columns)
	if err != nil {
		return nil, err
	}
	endTimeUnixNano, err := air.ReadUint64(record, row, constants.END_TIME_UNIX_NANO, columns)
	if err != nil {
		return nil, err
	}
	droppedAttributesCount, err := air.ReadUint32(record, row, constants.DROPPED_ATTRIBUTES_COUNT, columns)
	if err != nil {
		return nil, err
	}
	droppedEventsCount, err := air.ReadUint32(record, row, constants.DROPPED_EVENTS_COUNT, columns)
	if err != nil {
		return nil, err
	}
	droppedLinksCount, err := air.ReadUint32(record, row, constants.DROPPED_LINKS_COUNT, columns)
	if err != nil {
		return nil, err
	}
	message, err := air.ReadString(record, row, constants.STATUS_MESSAGE, columns)
	if err != nil {
		return nil, err
	}
	code, err := air.ReadInt32(record, row, constants.STATUS_CODE, columns)
	if err != nil {
		return nil, err
	}
	attrColumn := air.Column(record, constants.ATTRIBUTES, columns)
	attributes := []*v1.KeyValue(nil)
	if attrColumn != nil {
		attributes, err = common.AttributesFrom(attrColumn)
		if err != nil {
			return nil, err
		}
	}
	events, err := EventsFrom(record, row, columns)
	if err != nil {
		return nil, err
	}
	links, err := LinksFrom(record, row, columns)
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

func EventsFrom(record arrow.Record, row int, columns map[string]int) ([]*tracepb.Span_Event, error) {
	eventsColumn := air.Column(record, constants.SPAN_EVENTS, columns)
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
			for i := 0; i < events.Len(); i++ {
				// ToDo continue here (LQ)
				//events.Field()
				//eventList.Offsets()[row]
			}
			return nil, nil
		default:
			return nil, fmt.Errorf("expecting a struct array for the column events but got %T", events)
		}
	default:
		return nil, fmt.Errorf("expecting a list array for the column events but got %T", eventList)
	}
}

func EventFrom(array *array.Struct, row int) (*tracepb.Span_Event, error) {
	return nil, nil
}

func LinksFrom(record arrow.Record, row int, columns map[string]int) ([]*tracepb.Span_Link, error) {
	return nil, nil
}
