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

package trace

import (
	"fmt"
	"strings"

	"github.com/apache/arrow/go/v9/arrow"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/air/rfield"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/ptrace"
)

// OtlpTraceToArrowRecords converts an OTLP trace to one or more Arrow records.
func OtlpTraceToArrowRecords(rr *air.RecordRepository, request ptrace.Traces, cfg *config.Config) ([]arrow.Record, error) {
	switch cfg.TraceEncoding {
	case config.Flat:
		AddFlattenTraces(rr, request, cfg)
	case config.Hierarchical:
		AddHierarchicalTraces(rr, request, cfg)
	case config.Hybrid:
		AddHybridTraces(rr, request, cfg)
	default:
		panic(fmt.Sprintf("unknown trace encoding: %v", cfg.TraceEncoding))
	}

	records, err := rr.BuildRecords()
	if err != nil {
		return nil, err
	}

	return records, nil
}

func AddFlattenTraces(rr *air.RecordRepository, request ptrace.Traces, cfg *config.Config) {
	for i := 0; i < request.ResourceSpans().Len(); i++ {
		resourceSpans := request.ResourceSpans().At(i)

		for j := 0; j < resourceSpans.ScopeSpans().Len(); j++ {
			scopeSpans := resourceSpans.ScopeSpans().At(j)

			for k := 0; k < scopeSpans.Spans().Len(); k++ {
				span := scopeSpans.Spans().At(k)

				record := air.NewRecord()

				if ts := span.StartTimestamp(); ts > 0 {
					record.U64Field(constants.START_TIME_UNIX_NANO, uint64(ts))
				}
				if ts := span.EndTimestamp(); ts > 0 {
					record.U64Field(constants.END_TIME_UNIX_NANO, uint64(ts))
				}
				common.AddResource(record, resourceSpans.Resource(), cfg)
				common.AddScope(record, constants.SCOPE_SPANS, scopeSpans.Scope(), cfg)

				if tid := span.TraceID(); !tid.IsEmpty() {
					record.BinaryField(constants.TRACE_ID, tid[:])
				}

				if sid := span.SpanID(); !sid.IsEmpty() {
					record.BinaryField(constants.SPAN_ID, sid[:])
				}

				if len(span.TraceState()) > 0 {
					record.StringField(constants.TRACE_STATE, string(span.TraceState()))
				}
				if psid := span.ParentSpanID(); !psid.IsEmpty() {
					record.BinaryField(constants.PARENT_SPAN_ID, psid[:])
				}
				if len(span.Name()) > 0 {
					record.StringField(constants.NAME, span.Name())
				}
				record.I32Field(constants.KIND, int32(span.Kind()))
				attributes := common.NewAttributes(span.Attributes(), cfg)
				if attributes != nil {
					record.AddField(attributes)
				}

				if dc := span.DroppedAttributesCount(); dc > 0 {
					record.U32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc))
				}

				// Events
				AddEvents(record, span.Events(), cfg)
				if dc := span.DroppedEventsCount(); dc > 0 {
					record.U32Field(constants.DROPPED_EVENTS_COUNT, uint32(dc))
				}

				// Links
				AddLinksAsListOfStructs(record, span.Links(), cfg)
				if dc := span.DroppedLinksCount(); dc > 0 {
					record.U32Field(constants.DROPPED_LINKS_COUNT, uint32(dc))
				}

				// Status
				if span.Status().Code() != 0 {
					record.I32Field(constants.STATUS, int32(span.Status().Code()))
				}
				if msg := span.Status().Message(); msg != "" {
					record.StringField(constants.STATUS_MESSAGE, span.Status().Message())
				}

				rr.AddRecord(record)
			}
		}
	}
}

func AddHierarchicalTraces(rr *air.RecordRepository, request ptrace.Traces, cfg *config.Config) {
	record := air.NewRecord()
	resSpansValues := make([]rfield.Value, 0, request.ResourceSpans().Len())

	for i := 0; i < request.ResourceSpans().Len(); i++ {
		resourceSpans := request.ResourceSpans().At(i)
		resFields := make([]*rfield.Field, 0, 3)

		// Resource field
		resFields = append(resFields, common.ResourceField(resourceSpans.Resource(), cfg))

		// Schema URL
		if resourceSpans.SchemaUrl() != "" {
			resFields = append(resFields, rfield.NewStringField(constants.SCHEMA_URL, resourceSpans.SchemaUrl()))
		}

		// Scope spans
		scopeSpansValues := make([]rfield.Value, 0, resourceSpans.ScopeSpans().Len())
		for j := 0; j < resourceSpans.ScopeSpans().Len(); j++ {
			scopeSpans := resourceSpans.ScopeSpans().At(j)

			fields := make([]*rfield.Field, 0, 3)
			fields = append(fields, common.ScopeField(constants.SCOPE_SPANS, scopeSpans.Scope(), cfg))
			if scopeSpans.SchemaUrl() != "" {
				fields = append(fields, rfield.NewStringField(constants.SCHEMA_URL, scopeSpans.SchemaUrl()))
			}

			spanValues := make([]rfield.Value, 0, scopeSpans.Spans().Len())
			for k := 0; k < scopeSpans.Spans().Len(); k++ {
				span := scopeSpans.Spans().At(k)

				fields := make([]*rfield.Field, 0, 10)
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
				if span.TraceState() != "" {
					fields = append(fields, rfield.NewStringField(constants.TRACE_STATE, string(span.TraceState())))
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
				}

				if dc := span.DroppedAttributesCount(); dc > 0 {
					fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
				}

				// Events
				eventsField := EventsField(span.Events(), cfg)
				if eventsField != nil {
					fields = append(fields, eventsField)
				}
				if dc := span.DroppedEventsCount(); dc > 0 {
					fields = append(fields, rfield.NewU32Field(constants.DROPPED_EVENTS_COUNT, uint32(dc)))
				}

				// Links
				linksField := LinksAsListOfStructsField(span.Links(), cfg)
				if linksField != nil {
					fields = append(fields, linksField)
				}
				if dc := span.DroppedLinksCount(); dc > 0 {
					fields = append(fields, rfield.NewU32Field(constants.DROPPED_LINKS_COUNT, uint32(dc)))
				}

				// Status
				if span.Status().Code() != 0 {
					fields = append(fields, rfield.NewI32Field(constants.STATUS, int32(span.Status().Code())))
				}
				if msg := span.Status().Message(); msg != "" {
					fields = append(fields, rfield.NewStringField(constants.STATUS_MESSAGE, msg))
				}

				spanValues = append(spanValues, rfield.NewStruct(fields))
			}
			fields = append(fields, rfield.NewListField(constants.SPANS, rfield.List{
				Values: spanValues,
			}))
			scopeSpansValues = append(scopeSpansValues, rfield.NewStruct(fields))
		}
		resFields = append(resFields, rfield.NewListField(constants.SCOPE_SPANS, rfield.List{Values: scopeSpansValues}))
		resSpansValues = append(resSpansValues, rfield.NewStruct(resFields))
	}
	record.ListField(constants.RESOURCE_SPANS, rfield.List{Values: resSpansValues})
	rr.AddRecord(record)
}

func AddHybridTraces(rr *air.RecordRepository, request ptrace.Traces, cfg *config.Config) {
	similarResSpans := make(map[string][]rfield.Value)

	for i := 0; i < request.ResourceSpans().Len(); i++ {
		resourceSpans := request.ResourceSpans().At(i)
		resFields := make([]*rfield.Field, 0, 3)

		// Resource field
		resFields = append(resFields, common.ResourceField(resourceSpans.Resource(), cfg))

		// Schema URL
		if resourceSpans.SchemaUrl() != "" {
			resFields = append(resFields, rfield.NewStringField(constants.SCHEMA_URL, resourceSpans.SchemaUrl()))
		}

		// Scope spans
		similarScopeSpans := make(map[string][]rfield.Value)
		for j := 0; j < resourceSpans.ScopeSpans().Len(); j++ {
			scopeSpans := resourceSpans.ScopeSpans().At(j)
			fields := make([]*rfield.Field, 0, 3)
			fields = append(fields, common.ScopeField(constants.SCOPE_SPANS, scopeSpans.Scope(), cfg))
			if scopeSpans.SchemaUrl() != "" {
				fields = append(fields, rfield.NewStringField(constants.SCHEMA_URL, scopeSpans.SchemaUrl()))
			}

			similarSpans := make(map[string][]rfield.Value)
			for k := 0; k < scopeSpans.Spans().Len(); k++ {
				span := scopeSpans.Spans().At(k)

				fields := make([]*rfield.Field, 0, 10)
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
				if span.TraceState() != "" {
					fields = append(fields, rfield.NewStringField(constants.TRACE_STATE, string(span.TraceState())))
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
				}

				if dc := span.DroppedAttributesCount(); dc > 0 {
					fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
				}

				// Events
				eventsField := EventsField(span.Events(), cfg)
				if eventsField != nil {
					fields = append(fields, eventsField)
				}
				if dc := span.DroppedEventsCount(); dc > 0 {
					fields = append(fields, rfield.NewU32Field(constants.DROPPED_EVENTS_COUNT, uint32(dc)))
				}

				// Links
				linksField := LinksAsListOfStructsField(span.Links(), cfg)
				if linksField != nil {
					fields = append(fields, linksField)
				}
				if dc := span.DroppedLinksCount(); dc > 0 {
					fields = append(fields, rfield.NewU32Field(constants.DROPPED_LINKS_COUNT, uint32(dc)))
				}

				// Status
				if span.Status().Code() != 0 {
					fields = append(fields, rfield.NewI32Field(constants.STATUS, int32(span.Status().Code())))
				}
				if span.Status().Message() != "" {
					fields = append(fields, rfield.NewStringField(constants.STATUS_MESSAGE, span.Status().Message()))
				}

				spanValue := rfield.NewStruct(fields)

				// Compute signature for the span value
				var sig strings.Builder
				attributes.Normalize()
				attributes.WriteSignature(&sig)
				//spanValue.Normalize()
				//spanValue.WriteSignature(&sig)
				similarSpans[sig.String()] = append(similarSpans[sig.String()], spanValue)
			}
			for sig, spans := range similarSpans {
				scopeSpansFields := make([]*rfield.Field, 0, 3)
				copy(scopeSpansFields, fields)
				scopeSpansFields = append(scopeSpansFields, rfield.NewListField(constants.SPANS, rfield.List{
					Values: spans,
				}))
				similarScopeSpans[sig] = append(similarScopeSpans[sig], rfield.NewStruct(scopeSpansFields))
			}
		}
		for sig, scopeSpans := range similarScopeSpans {
			resFieldsCopy := make([]*rfield.Field, 0, 3)
			copy(resFieldsCopy, resFields)
			resFieldsCopy = append(resFieldsCopy, rfield.NewListField(constants.SCOPE_SPANS, rfield.List{Values: scopeSpans}))
			similarResSpans[sig] = append(similarResSpans[sig], rfield.NewStruct(resFieldsCopy))
		}
	}
	for _, resSpans := range similarResSpans {
		record := air.NewRecord()
		record.ListField(constants.RESOURCE_SPANS, rfield.List{Values: resSpans})
		rr.AddRecord(record)
	}
}

func AddEvents(record *air.Record, events ptrace.SpanEventSlice, cfg *config.Config) {
	eventsField := EventsField(events, cfg)
	if eventsField != nil {
		record.AddField(eventsField)
	}
}

func EventsField(events ptrace.SpanEventSlice, cfg *config.Config) *rfield.Field {
	if events.Len() == 0 {
		return nil
	}

	convertedEvents := make([]rfield.Value, 0, events.Len())

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
		}
		if dc := event.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}
		convertedEvents = append(convertedEvents, &rfield.Struct{
			Fields: fields,
		})
	}
	return rfield.NewListField(constants.SPAN_EVENTS, rfield.List{
		Values: convertedEvents,
	})
}

func AddLinksAsListOfStructs(record *air.Record, links ptrace.SpanLinkSlice, cfg *config.Config) {
	linksField := LinksAsListOfStructsField(links, cfg)
	if linksField != nil {
		record.AddField(linksField)
	}
}

func LinksAsListOfStructsField(links ptrace.SpanLinkSlice, cfg *config.Config) *rfield.Field {
	if links.Len() == 0 {
		return nil
	}

	convertedLinks := make([]rfield.Value, 0, links.Len())

	for i := 0; i < links.Len(); i++ {
		link := links.At(i)
		fields := make([]*rfield.Field, 0, 4)

		if tid := link.TraceID(); !tid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.TRACE_ID, tid[:]))
		}
		if sid := link.SpanID(); !sid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.SPAN_ID, sid[:]))
		}
		if link.TraceState() != "" {
			fields = append(fields, rfield.NewStringField(constants.TRACE_STATE, string(link.TraceState())))
		}
		attributes := common.NewAttributes(link.Attributes(), cfg)
		if attributes != nil {
			fields = append(fields, attributes)
		}
		if dc := link.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}
		convertedLinks = append(convertedLinks, &rfield.Struct{
			Fields: fields,
		})
	}
	return rfield.NewListField(constants.SPAN_LINKS, rfield.List{
		Values: convertedLinks,
	})
}

func AddLinksAsStructOfLists(record *air.Record, links []ptrace.SpanLink) {
	if links == nil {
		return
	}

	fields := make([]*rfield.Field, 5)

	for pos, link := range links {
		if tid := link.TraceID(); !tid.IsEmpty() {
			if fields[0] == nil {
				fields[0] = rfield.NewListField(constants.TRACE_ID, rfield.List{Values: make([]rfield.Value, len(links))})
				for i := 0; i < pos; i++ {
					fields[0].Value.(*rfield.List).Values[pos] = nil
				}
			}
			fields[0].Value.(*rfield.List).Values[pos] = &rfield.Binary{Value: tid[:]}
		}
		if sid := link.SpanID(); !sid.IsEmpty() {
			if fields[1] == nil {
				fields[1] = rfield.NewListField(constants.SPAN_ID, rfield.List{Values: make([]rfield.Value, len(links))})
				for i := 0; i < pos; i++ {
					fields[1].Value.(*rfield.List).Values[pos] = nil
				}
			}
			fields[1].Value.(*rfield.List).Values[pos] = &rfield.Binary{Value: sid[:]}
		}
		if link.TraceState() != "" {
			if fields[2] == nil {
				fields[2] = rfield.NewListField(constants.TRACE_STATE, rfield.List{Values: make([]rfield.Value, len(links))})
				for i := 0; i < pos; i++ {
					fields[2].Value.(*rfield.List).Values[pos] = nil
				}
			}
			fields[2].Value.(*rfield.List).Values[pos] = rfield.NewString(string(link.TraceState()))
		}
		attributes := common.AttributesValue(link.Attributes())
		if attributes != nil {
			if fields[3] == nil {
				fields[3] = rfield.NewListField(constants.ATTRIBUTES, rfield.List{Values: make([]rfield.Value, len(links))})
				for i := 0; i < pos; i++ {
					fields[3].Value.(*rfield.List).Values[pos] = nil
				}
			}
			fields[3].Value.(*rfield.List).Values[pos] = attributes
		}
		if dc := link.DroppedAttributesCount(); dc > 0 {
			if fields[4] == nil {
				fields[4] = rfield.NewListField(constants.DROPPED_ATTRIBUTES_COUNT, rfield.List{Values: make([]rfield.Value, len(links))})
				for i := 0; i < pos; i++ {
					fields[4].Value.(*rfield.List).Values[pos] = nil
				}
			}
			fields[4].Value.(*rfield.List).Values[pos] = rfield.NewU32(dc)
		}
	}
	nonEmptyFields := make([]*rfield.Field, 0, len(fields))
	for _, field := range fields {
		if field != nil {
			nonEmptyFields = append(nonEmptyFields, field)
		}
	}
	if len(nonEmptyFields) > 0 {
		record.StructField(constants.SPAN_LINKS, rfield.Struct{
			Fields: nonEmptyFields,
		})
	}
}
