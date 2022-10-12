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
	"io"
	"strings"

	"github.com/apache/arrow/go/v9/arrow"

	"github.com/lquerel/otel-arrow-adapter/pkg/air"
	"github.com/lquerel/otel-arrow-adapter/pkg/air/config"
	"github.com/lquerel/otel-arrow-adapter/pkg/air/rfield"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/common"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/ptrace"
)

// OtlpArrowProducer produces OTLP Arrow records from OTLP traces.
type OtlpArrowProducer struct {
	cfg *config.Config
	rr  *air.RecordRepository
}

// NewOtlpArrowProducer creates a new OtlpArrowProducer with the default configuration.
// Note: the default attribute encoding is AttributesAsListStructs
func NewOtlpArrowProducer() *OtlpArrowProducer {
	cfg := config.NewUint16DefaultConfig()
	cfg.Attribute.Encoding = config.AttributesAsListStructs

	return &OtlpArrowProducer{
		cfg: cfg,
		rr:  air.NewRecordRepository(cfg),
	}
}

// NewOtlpArrowProducerWith creates a new OtlpArrowProducer with the given configuration.
func NewOtlpArrowProducerWith(cfg *config.Config) *OtlpArrowProducer {
	return &OtlpArrowProducer{
		cfg: cfg,
		rr:  air.NewRecordRepository(cfg),
	}
}

// ProduceFrom produces Arrow records from the given OTLP traces. The generated schemas of the Arrow records follow
// the hierarchical organization of the trace protobuf structure.
//
// Resource signature = resource attributes sig + dropped attributes count sig + schema URL sig
//
// More details can be found in the OTEL 0156 section XYZ.
// TODO: add a reference to the OTEP 0156 section that describes this mapping.
func (p *OtlpArrowProducer) ProduceFrom(traces ptrace.Traces) ([]arrow.Record, error) {
	// Map maintaining groups of scope spans per resource span signature.
	// A resource span signature is based on the resource attributes,
	// the dropped attributes count and the schema URL
	resSpansPerSig := make(map[string]*ScopeSpanGroup)

	for rsIdx := 0; rsIdx < traces.ResourceSpans().Len(); rsIdx++ {
		resourceSpans := traces.ResourceSpans().At(rsIdx)

		// Add resource fields (attributes and dropped attributes count)
		resField, resSig := common.ResourceFieldWithSig(resourceSpans.Resource(), p.cfg)

		// Add schema URL
		var schemaUrl *rfield.Field
		if resourceSpans.SchemaUrl() != "" {
			schemaUrl = rfield.NewStringField(constants.SCHEMA_URL, resourceSpans.SchemaUrl())
			resSig += ",schema_url:" + resourceSpans.SchemaUrl()
		}

		// TODO explore group span per attribute sig to minimize overall schema complexity

		// Group spans per scope span signature
		spansPerScopeSpanSig := GroupSpans(resourceSpans, p.cfg)

		// Create a new entry in the map if the signature is not already present
		resSpanFields := resSpansPerSig[resSig]
		if resSpanFields == nil {
			resSpanFields = NewScopeSpanGroup(resField, schemaUrl)
			resSpansPerSig[resSig] = resSpanFields
		}

		// Merge spans sharing the same scope span signature
		for sig, ss := range spansPerScopeSpanSig {
			scopeSpan := resSpanFields.scopeSpans[sig]
			if scopeSpan == nil {
				resSpanFields.scopeSpans[sig] = ss
			} else {
				scopeSpan.spans = append(scopeSpan.spans, ss.spans...)
			}
		}
	}

	// TODO Other way to explore -> create a single Arrow record from a list of resource spans.
	// All resource spans sharing the same signature are represented as an AIR record.
	for _, resSpanFields := range resSpansPerSig {
		record := air.NewRecord()
		record.ListField(constants.RESOURCE_SPANS, rfield.List{Values: []rfield.Value{
			resSpanFields.ResourceSpan(),
		}})
		p.rr.AddRecord(record)
	}
	//for _, resSpanFields := range resSpansPerSig {
	//	resSpans := resSpanFields.ResourceSpans()
	//	for _, resSpan := range resSpans {
	//		record := air.NewRecord()
	//		record.ListField(constants.RESOURCE_SPANS, rfield.List{Values: []rfield.Value{
	//			resSpan,
	//		}})
	//		p.rr.AddRecord(record)
	//	}
	//}

	// Build all Arrow records from the AIR records
	records, err := p.rr.BuildRecords()
	if err != nil {
		return nil, err
	}

	return records, nil
}

// DumpMetadata dumps the metadata of the underlying AIR record repository.
func (p *OtlpArrowProducer) DumpMetadata(writer io.Writer) {
	p.rr.DumpMetadata(writer)
}

// ScopeSpanGroup groups a set of scope spans for a specific resource and schema url.
type ScopeSpanGroup struct {
	resource   *rfield.Field
	schemaUrl  *rfield.Field
	scopeSpans map[string]*SpanGroup
}

// NewScopeSpanGroup creates a new scope span group for the given resource and schema url.
func NewScopeSpanGroup(resource *rfield.Field, schemaUrl *rfield.Field) *ScopeSpanGroup {
	return &ScopeSpanGroup{
		resource:   resource,
		scopeSpans: make(map[string]*SpanGroup),
		schemaUrl:  schemaUrl,
	}
}

// ResourceSpan builds an AIR representation of the current resource span for this group of scope spans.
// Resource span = resource fields + schema URL + scope spans
func (r *ScopeSpanGroup) ResourceSpan() rfield.Value {
	fields := make([]*rfield.Field, 0, 3)
	if r.resource != nil {
		fields = append(fields, r.resource)
	}
	if r.schemaUrl != nil {
		fields = append(fields, r.schemaUrl)
	}
	if len(r.scopeSpans) > 0 {
		scopeSpans := make([]rfield.Value, 0, len(r.scopeSpans))
		for _, ss := range r.scopeSpans {
			scopeSpans = append(scopeSpans, ss.ScopeSpan())
		}
		fields = append(fields, rfield.NewListField(constants.SCOPE_SPANS, rfield.List{Values: scopeSpans}))
	}
	return rfield.NewStruct(fields)
}

func (r *ScopeSpanGroup) ResourceSpans() []rfield.Value {
	resSpans := make([]rfield.Value, 0, len(r.scopeSpans))

	for _, ss := range r.scopeSpans {
		fields := make([]*rfield.Field, 0, 3)
		if r.resource != nil {
			fields = append(fields, r.resource)
		}
		if r.schemaUrl != nil {
			fields = append(fields, r.schemaUrl)
		}
		fields = append(fields, rfield.NewListField(constants.SCOPE_SPANS, rfield.List{Values: []rfield.Value{ss.ScopeSpan()}}))
		resSpans = append(resSpans, rfield.NewStruct(fields))
	}
	return resSpans
}

// SpanGroup groups a set of spans for a specific scope span and schema url.
type SpanGroup struct {
	scope     *rfield.Field
	schemaUrl *rfield.Field
	spans     []rfield.Value
}

// ScopeSpan builds an AIR representation of the current scope span for this group of spans.
// Scope span = scope fields + schema URL + spans
func (s *SpanGroup) ScopeSpan() rfield.Value {
	fields := make([]*rfield.Field, 0, 3)
	if s.scope != nil {
		fields = append(fields, s.scope)
	}
	if s.schemaUrl != nil {
		fields = append(fields, s.schemaUrl)
	}
	if len(s.spans) > 0 {
		fields = append(fields, rfield.NewListField(constants.SPANS, rfield.List{Values: s.spans}))
	}

	return rfield.NewStruct(fields)
}

// GroupSpans groups spans per signature.
// A scope span signature is based on the scope attributes, the dropped attributes count and the schema URL.
func GroupSpans(resourceSpans ptrace.ResourceSpans, cfg *config.Config) (scopeSpansPerSig map[string]*SpanGroup) {
	scopeSpansPerSig = make(map[string]*SpanGroup)
	for j := 0; j < resourceSpans.ScopeSpans().Len(); j++ {
		scopeSpans := resourceSpans.ScopeSpans().At(j)

		var sig strings.Builder

		scopeField := common.ScopeField(constants.SCOPE, scopeSpans.Scope(), cfg)
		scopeField.Normalize()
		scopeField.WriteSignature(&sig)

		var schemaField *rfield.Field
		if scopeSpans.SchemaUrl() != "" {
			schemaField = rfield.NewStringField(constants.SCHEMA_URL, scopeSpans.SchemaUrl())
			sig.WriteString(",")
			schemaField.WriteSignature(&sig)
		}

		// Create a new entry in the map if the signature is not already present
		ssSig := sig.String()
		ssFields := scopeSpansPerSig[ssSig]
		if ssFields == nil {
			ssFields = &SpanGroup{
				scope:     scopeField,
				spans:     make([]rfield.Value, 0, 16),
				schemaUrl: schemaField,
			}
			scopeSpansPerSig[ssSig] = ssFields
		}

		spans := spans(scopeSpans, cfg)
		ssFields.spans = append(ssFields.spans, spans...)
	}
	return
}

// spans converts ptrace.ScopeSpans in their AIR representation.
func spans(scopeSpans ptrace.ScopeSpans, cfg *config.Config) (spans []rfield.Value) {
	spans = make([]rfield.Value, 0, scopeSpans.Spans().Len())
	for k := 0; k < scopeSpans.Spans().Len(); k++ {
		span := scopeSpans.Spans().At(k)

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
		}

		if dc := span.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}

		// Events
		eventsField := events(span.Events(), cfg)
		if eventsField != nil {
			fields = append(fields, eventsField)
		}
		if dc := span.DroppedEventsCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_EVENTS_COUNT, uint32(dc)))
		}

		// Links
		linksField := links(span.Links(), cfg)
		if linksField != nil {
			fields = append(fields, linksField)
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

		spans = append(spans, spanValue)
	}
	return
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
func events(events ptrace.SpanEventSlice, cfg *config.Config) *rfield.Field {
	if events.Len() == 0 {
		return nil
	}

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
		}
		if dc := event.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}
		airEvents = append(airEvents, &rfield.Struct{
			Fields: fields,
		})
	}
	return rfield.NewListField(constants.SPAN_EVENTS, rfield.List{
		Values: airEvents,
	})
}

// links converts OTLP span links into their AIR representation or returns nil when there is no links.
func links(links ptrace.SpanLinkSlice, cfg *config.Config) *rfield.Field {
	if links.Len() == 0 {
		return nil
	}

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
		}
		if dc := link.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}
		airLinks = append(airLinks, &rfield.Struct{
			Fields: fields,
		})
	}
	return rfield.NewListField(constants.SPAN_LINKS, rfield.List{
		Values: airLinks,
	})
}
