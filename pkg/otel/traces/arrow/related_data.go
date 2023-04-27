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
	"errors"
	"math"

	"github.com/apache/arrow/go/v12/arrow"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	config2 "github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	config "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// RelatedData is a collection of related data constructs used by traces.
type RelatedData struct {
	spanCount uint16

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
type AttrsBuilders struct {
	resource *Attrs16Builder
	scope    *Attrs16Builder
	span     *Attrs16Builder
	event    *Attrs32Builder
	link     *Attrs32Builder
}

// AttrsRecordBuilders is a collection of RecordBuilderExt instances used
// to build related data records (i.e. resource attributes, scope attributes,
// span attributes, event attributes, and link attributes).
type AttrsRecordBuilders struct {
	resource *builder.RecordBuilderExt
	scope    *builder.RecordBuilderExt
	span     *builder.RecordBuilderExt
	event    *builder.RecordBuilderExt
	link     *builder.RecordBuilderExt
}

func NewRelatedData(cfg *config2.Config, stats *stats.ProducerStats) (*RelatedData, error) {
	attrsResourceRB := builder.NewRecordBuilderExt(cfg.Pool, AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsScopeRB := builder.NewRecordBuilderExt(cfg.Pool, AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsSpanRB := builder.NewRecordBuilderExt(cfg.Pool, AttrsSchema16, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsEventRB := builder.NewRecordBuilderExt(cfg.Pool, AttrsSchema32, config.NewDictionary(cfg.LimitIndexSize), stats)
	attrsLinkRB := builder.NewRecordBuilderExt(cfg.Pool, AttrsSchema32, config.NewDictionary(cfg.LimitIndexSize), stats)

	attrsResourceBuilder, err := NewAttrs16Builder(attrsResourceRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsScopeBuilder, err := NewAttrs16Builder(attrsScopeRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsSpanBuilder, err := NewAttrs16Builder(attrsSpanRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsEventBuilder, err := NewAttrs32Builder(attrsEventRB)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsLinkBuilder, err := NewAttrs32Builder(attrsLinkRB)
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
	return r.spanCount
}

func (r *RelatedData) NextSpanID() uint16 {
	sc := r.spanCount

	if sc == math.MaxUint16 {
		panic("maximum number of spans reached per batch, please reduce the batch size to a maximum of 65535 spans")
	}

	r.spanCount++
	return sc
}

func (r *RelatedData) BuildRecordMessages() ([]*record_message.RecordMessage, error) {
	recordMessages := make([]*record_message.RecordMessage, 0, 6)

	if !r.attrsBuilders.resource.IsEmpty() {
		attrsResRec, err := r.attrsBuilders.BuildAttrs16Record(r.attrsBuilders.resource)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "resource_attrs:" + r.attrsBuilders.resource.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsResRec, colarspb.OtlpArrowPayloadType_RESOURCE_ATTRS))
	}

	if !r.attrsBuilders.scope.IsEmpty() {
		attrsScopeRec, err := r.attrsBuilders.BuildAttrs16Record(r.attrsBuilders.scope)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		schemaID := "scope_attrs:" + r.attrsBuilders.scope.SchemaID()
		recordMessages = append(recordMessages, record_message.NewRelatedDataMessage(schemaID, attrsScopeRec, colarspb.OtlpArrowPayloadType_SCOPE_ATTRS))
	}

	if !r.attrsBuilders.span.IsEmpty() {
		attrsSpanRec, err := r.attrsBuilders.BuildAttrs16Record(r.attrsBuilders.span)
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
		attrsEventRec, err := r.attrsBuilders.BuildAttrs32Record(r.attrsBuilders.event)
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
		attrsLinkRec, err := r.attrsBuilders.BuildAttrs32Record(r.attrsBuilders.link)
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

func (ab *AttrsBuilders) Resource() *Attrs16Builder {
	return ab.resource
}

func (ab *AttrsBuilders) Scope() *Attrs16Builder {
	return ab.scope
}

func (ab *AttrsBuilders) Span() *Attrs16Builder {
	return ab.span
}

func (ab *AttrsBuilders) Event() *Attrs32Builder {
	return ab.event
}

func (ab *AttrsBuilders) Link() *Attrs32Builder {
	return ab.link
}

func (ab *AttrsBuilders) Reset() {
	ab.resource.Accumulator().Reset()
	ab.scope.Accumulator().Reset()
	ab.span.Accumulator().Reset()
	ab.event.Accumulator().Reset()
	ab.link.Accumulator().Reset()
}

func (ab *AttrsBuilders) BuildAttrs16Record(attrsBuilder *Attrs16Builder) (arrow.Record, error) {
	schemaNotUpToDateCount := 0

	var record arrow.Record
	var err error

	// Loop until the record is built successfully.
	// Intermediaries steps may be required to update the schema.
	for {
		record, err = attrsBuilder.Build()
		if err != nil {
			if record != nil {
				record.Release()
			}

			switch {
			case errors.Is(err, schema.ErrSchemaNotUpToDate):
				schemaNotUpToDateCount++
				if schemaNotUpToDateCount > 5 {
					panic("Too many consecutive schema updates. This shouldn't happen.")
				}
			default:
				return nil, werror.Wrap(err)
			}
		} else {
			break
		}
	}
	return record, werror.Wrap(err)
}

func (ab *AttrsBuilders) BuildAttrs32Record(attrsBuilder *Attrs32Builder) (arrow.Record, error) {
	schemaNotUpToDateCount := 0

	var record arrow.Record
	var err error

	// Loop until the record is built successfully.
	// Intermediaries steps may be required to update the schema.
	for {
		record, err = attrsBuilder.Build()
		if err != nil {
			if record != nil {
				record.Release()
			}

			switch {
			case errors.Is(err, schema.ErrSchemaNotUpToDate):
				schemaNotUpToDateCount++
				if schemaNotUpToDateCount > 5 {
					panic("Too many consecutive schema updates. This shouldn't happen.")
				}
			default:
				return nil, werror.Wrap(err)
			}
		} else {
			break
		}
	}
	return record, werror.Wrap(err)
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
