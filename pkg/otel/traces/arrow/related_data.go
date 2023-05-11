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

// Infrastructure to manage span related records.

import (
	"math"

	cfg "github.com/f5/otel-arrow-adapter/pkg/config"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
)

type (
	// RelatedData is a collection of related/dependent data to span entities.
	RelatedData struct {
		spanCount uint64

		relatedRecordsManager *carrow.RelatedRecordsManager

		attrsBuilders *AttrsBuilders
		eventBuilder  *EventBuilder
		linkBuilder   *LinkBuilder
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
)

func NewRelatedData(cfg *cfg.Config, stats *stats.ProducerStats) (*RelatedData, error) {
	rrManager := carrow.NewRelatedRecordsManager(cfg, stats)

	attrsResourceBuilder := rrManager.Declare(carrow.PayloadTypes.ResourceAttrs, carrow.AttrsSchema16, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		return carrow.NewAttrs16Builder(b, carrow.PayloadTypes.ResourceAttrs)
	})

	attrsScopeBuilder := rrManager.Declare(carrow.PayloadTypes.ScopeAttrs, carrow.AttrsSchema16, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		return carrow.NewAttrs16Builder(b, carrow.PayloadTypes.ScopeAttrs)
	})

	attrsSpanBuilder := rrManager.Declare(carrow.PayloadTypes.SpanAttrs, carrow.AttrsSchema16, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		return carrow.NewAttrs16Builder(b, carrow.PayloadTypes.SpanAttrs)
	})

	eventBuilder := rrManager.Declare(carrow.PayloadTypes.Event, EventSchema, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		return NewEventBuilder(b)
	})

	linkBuilder := rrManager.Declare(carrow.PayloadTypes.Link, LinkSchema, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		return NewLinkBuilder(b)
	})

	attrsEventBuilder := rrManager.Declare(carrow.PayloadTypes.EventAttrs, carrow.AttrsSchema32, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		ab := carrow.NewAttrs32Builder(b, carrow.PayloadTypes.EventAttrs)
		eventBuilder.(*EventBuilder).SetAttributesAccumulator(ab.Accumulator())
		return ab
	})

	attrsLinkBuilder := rrManager.Declare(carrow.PayloadTypes.LinkAttrs, carrow.AttrsSchema32, func(b *builder.RecordBuilderExt) carrow.RelatedRecordBuilder {
		ab := carrow.NewAttrs32Builder(b, carrow.PayloadTypes.LinkAttrs)
		linkBuilder.(*LinkBuilder).SetAttributesAccumulator(ab.Accumulator())
		return ab
	})

	return &RelatedData{
		relatedRecordsManager: rrManager,
		attrsBuilders: &AttrsBuilders{
			resource: attrsResourceBuilder.(*carrow.Attrs16Builder),
			scope:    attrsScopeBuilder.(*carrow.Attrs16Builder),
			span:     attrsSpanBuilder.(*carrow.Attrs16Builder),
			event:    attrsEventBuilder.(*carrow.Attrs32Builder),
			link:     attrsLinkBuilder.(*carrow.Attrs32Builder),
		},
		eventBuilder: eventBuilder.(*EventBuilder),
		linkBuilder:  linkBuilder.(*LinkBuilder),
	}, nil
}

func (r *RelatedData) Release() {
	r.relatedRecordsManager.Release()
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

func (r *RelatedData) RecordBuilderExt(payloadType *carrow.PayloadType) *builder.RecordBuilderExt {
	return r.relatedRecordsManager.RecordBuilderExt(payloadType)
}

func (r *RelatedData) Reset() {
	r.spanCount = 0
	r.relatedRecordsManager.Reset()
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
	return r.relatedRecordsManager.BuildRecordMessages()
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
