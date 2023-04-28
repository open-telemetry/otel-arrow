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

// Infrastructure to manage related records.

import (
	"math"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	cfg "github.com/f5/otel-arrow-adapter/pkg/config"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	config "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	// RelatedData is a collection of related/dependent data to span entities.
	RelatedData struct {
		spanCount uint64

		attrsBuilders       *AttrsBuilders
		attrsRecordBuilders *AttrsRecordBuilders

		eventBuilder       *EventBuilder
		eventRecordBuilder *builder.RecordBuilderExt

		linkBuilder       *LinkBuilder
		linkRecordBuilder *builder.RecordBuilderExt
	}

	// AttrsBuilders groups together AttrsBuilder instances used to build related
	// data attributes (i.e. resource attributes, scope attributes, span attributes,
	// event attributes, and link attributes).
	AttrsBuilders struct {
		resource *carrow.Attrs16Builder
		scope    *carrow.Attrs16Builder
		span     *carrow.Attrs16Builder
		event    *carrow.Attrs32Builder
		link     *carrow.Attrs32Builder
	}

	// AttrsRecordBuilders is a collection of RecordBuilderExt instances used
	// to build related data records (i.e. resource attributes, scope attributes,
	// span attributes, event attributes, and link attributes).
	AttrsRecordBuilders struct {
		resource *builder.RecordBuilderExt
		scope    *builder.RecordBuilderExt
		span     *builder.RecordBuilderExt
		event    *builder.RecordBuilderExt
		link     *builder.RecordBuilderExt
	}
)

func NewRelatedData(cfg *cfg.Config, stats *stats.ProducerStats) (*RelatedData, error) {
	attrsResourceRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsScopeRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsSpanRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsEventRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema32, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsLinkRB := builder.NewRecordBuilderExt(cfg.Pool, carrow.AttrsSchema32, config.NewDictionary(cfg.LimitIndexSize), stats)

	attrsResourceBuilder, err := carrow.NewAttrs16Builder(attrsResourceRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsScopeBuilder, err := carrow.NewAttrs16Builder(attrsScopeRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsSpanBuilder, err := carrow.NewAttrs16Builder(attrsSpanRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsEventBuilder, err := carrow.NewAttrs32Builder(attrsEventRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsLinkBuilder, err := carrow.NewAttrs32Builder(attrsLinkRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	eventRB := builder.NewRecordBuilderExt(cfg.Pool, EventSchema, config.NewDictionary(cfg.LimitIndexSize), stats)
	eventBuilder, err := NewEventBuilder(eventRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	linkRB := builder.NewRecordBuilderExt(cfg.Pool, LinkSchema, config.NewDictionary(cfg.LimitIndexSize), stats)
	linkBuilder, err := NewLinkBuilder(linkRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &RelatedData{
		attrsBuilders: &AttrsBuilders{
			resource: attrsResourceBuilder,
			scope:    attrsScopeBuilder,
			span:     attrsSpanBuilder,
			event:    attrsEventBuilder,
			link:     attrsLinkBuilder,
		},
		attrsRecordBuilders: &AttrsRecordBuilders{
			resource: attrsResourceRB,
			scope:    attrsScopeRB,
			span:     attrsSpanRB,
			event:    attrsEventRB,
			link:     attrsLinkRB,
		},
		eventBuilder:       eventBuilder,
		eventRecordBuilder: eventRB,
		linkBuilder:        linkBuilder,
		linkRecordBuilder:  linkRB,
	}, nil
}

func (r *RelatedData) Release() {
	if r.attrsBuilders != nil {
		r.attrsBuilders.Release()
	}
	if r.attrsRecordBuilders != nil {
		r.attrsRecordBuilders.Release()
	}

	r.eventBuilder.Release()
	r.eventRecordBuilder.Release()
	r.linkBuilder.Release()
	r.linkRecordBuilder.Release()
}

func (r *RelatedData) AttrsBuilders() *AttrsBuilders {
	return r.attrsBuilders
}

func (r *RelatedData) EventBuilder() *EventBuilder {
	return r.eventBuilder
}

func (r *RelatedData) LinkBuilder() *LinkBuilder {
	return r.linkBuilder
}

func (r *RelatedData) AttrsRecordBuilders() *AttrsRecordBuilders {
	return r.attrsRecordBuilders
}

func (r *RelatedData) EventRecordBuilder() *builder.RecordBuilderExt {
	return r.eventRecordBuilder
}

func (r *RelatedData) LinkRecordBuilder() *builder.RecordBuilderExt {
	return r.linkRecordBuilder
}

func (r *RelatedData) Reset() {
	r.spanCount = 0
	r.attrsBuilders.Reset()
	r.eventBuilder.Accumulator().Reset()
	r.linkBuilder.Accumulator().Reset()
}

func (r *RelatedData) SpanCount() uint16 {
	return uint16(r.spanCount)
}

func (r *RelatedData) NextSpanID() uint16 {
	sc := r.spanCount

	if sc == math.MaxUint16 {
		panic("maximum number of spans reached per batch, please reduce the batch size to a maximum of 65535 spans")
	}

	r.spanCount++
	return uint16(sc)
}

func (r *RelatedData) BuildRecordMessages() ([]*record_message.RecordMessage, error) {
	recordMessages := make([]*record_message.RecordMessage, 0, 6)

	if !r.attrsBuilders.resource.IsEmpty() {
		attrsResRec, err := r.attrsBuilders.resource.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "resource_attrs:" + r.attrsBuilders.resource.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsResRec, colarspb.OtlpArrowPayloadType_RESOURCE_ATTRS))
	}

	if !r.attrsBuilders.scope.IsEmpty() {
		attrsScopeRec, err := r.attrsBuilders.scope.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "scope_attrs:" + r.attrsBuilders.scope.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsScopeRec, colarspb.OtlpArrowPayloadType_SCOPE_ATTRS))
	}

	if !r.attrsBuilders.span.IsEmpty() {
		attrsSpanRec, err := r.attrsBuilders.span.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "span_attrs:" + r.attrsBuilders.span.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsSpanRec, colarspb.OtlpArrowPayloadType_SPAN_ATTRS))
	}

	if !r.eventBuilder.IsEmpty() {
		eventRec, err := r.eventBuilder.BuildRecord(r.attrsBuilders.event.Accumulator())
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "span_events:" + r.eventBuilder.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, eventRec, colarspb.OtlpArrowPayloadType_SPAN_EVENTS))
	}

	if !r.attrsBuilders.event.IsEmpty() {
		attrsEventRec, err := r.attrsBuilders.event.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "span_event_attrs:" + r.attrsBuilders.event.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsEventRec, colarspb.OtlpArrowPayloadType_SPAN_EVENT_ATTRS))
	}

	if !r.linkBuilder.IsEmpty() {
		linkRec, err := r.linkBuilder.BuildRecord(r.attrsBuilders.link.Accumulator())
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "span_links:" + r.linkBuilder.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, linkRec, colarspb.OtlpArrowPayloadType_SPAN_LINKS))
	}

	if !r.attrsBuilders.link.IsEmpty() {
		attrsLinkRec, err := r.attrsBuilders.link.Build()
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "span_link_attrs:" + r.attrsBuilders.link.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsLinkRec, colarspb.OtlpArrowPayloadType_SPAN_LINK_ATTRS))
	}

	return recordMessages, nil
}

func (ab *AttrsBuilders) Release() {
	ab.resource.Release()
	ab.scope.Release()
	ab.span.Release()
	ab.event.Release()
	ab.link.Release()
}

func (ab *AttrsBuilders) Resource() *carrow.Attrs16Builder {
	return ab.resource
}

func (ab *AttrsBuilders) Scope() *carrow.Attrs16Builder {
	return ab.scope
}

func (ab *AttrsBuilders) Span() *carrow.Attrs16Builder {
	return ab.span
}

func (ab *AttrsBuilders) Event() *carrow.Attrs32Builder {
	return ab.event
}

func (ab *AttrsBuilders) Link() *carrow.Attrs32Builder {
	return ab.link
}

func (ab *AttrsBuilders) Reset() {
	ab.resource.Accumulator().Reset()
	ab.scope.Accumulator().Reset()
	ab.span.Accumulator().Reset()
	ab.event.Accumulator().Reset()
	ab.link.Accumulator().Reset()
}

func (arb *AttrsRecordBuilders) Release() {
	arb.resource.Release()
	arb.scope.Release()
	arb.span.Release()
	arb.event.Release()
	arb.link.Release()
}

func (arb *AttrsRecordBuilders) Resource() *builder.RecordBuilderExt {
	return arb.resource
}

func (arb *AttrsRecordBuilders) Scope() *builder.RecordBuilderExt {
	return arb.scope
}

func (arb *AttrsRecordBuilders) Span() *builder.RecordBuilderExt {
	return arb.span
}

func (arb *AttrsRecordBuilders) Event() *builder.RecordBuilderExt {
	return arb.event
}

func (arb *AttrsRecordBuilders) Link() *builder.RecordBuilderExt {
	return arb.link
}
