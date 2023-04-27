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

import (
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// SpanDT is the Arrow Data Type describing a span.
var (
	SpanDT = arrow.StructOf([]arrow.Field{
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint16, Metadata: schema.Metadata(schema.Optional, schema.DeltaEncoding)},
		{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		{Name: constants.DurationTimeUnixNano, Type: arrow.FixedWidthTypes.Duration_ms, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.TraceId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 16}},
		{Name: constants.SpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}},
		{Name: constants.TraceState, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.ParentSpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Name, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.KIND, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedEventsCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedLinksCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Status, Type: StatusDT, Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// SpanBuilder is a helper to build a span.
type SpanBuilder struct {
	released bool

	builder *builder.StructBuilder

	ib    *builder.Uint16DeltaBuilder     //  id builder
	stunb *builder.TimestampBuilder       // start time unix nano builder
	dtunb *builder.DurationBuilder        // duration time unix nano builder
	tib   *builder.FixedSizeBinaryBuilder // trace id builder
	sib   *builder.FixedSizeBinaryBuilder // span id builder
	tsb   *builder.StringBuilder          // trace state builder
	psib  *builder.FixedSizeBinaryBuilder // parent span id builder
	nb    *builder.StringBuilder          // name builder
	kb    *builder.Int32Builder           // kind builder
	dacb  *builder.Uint32Builder          // dropped attributes count builder
	decb  *builder.Uint32Builder          // dropped events count builder
	dlcb  *builder.Uint32Builder          // dropped links count builder
	sb    *StatusBuilder                  // status builder
}

func SpanBuilderFrom(sb *builder.StructBuilder) *SpanBuilder {
	ib := sb.Uint16DeltaBuilder(constants.ID)
	// As the attributes are sorted before insertion, the delta between two
	// consecutive attributes ID should always be <=1.
	ib.SetMaxDelta(1)

	return &SpanBuilder{
		released: false,
		builder:  sb,
		ib:       ib,
		stunb:    sb.TimestampBuilder(constants.StartTimeUnixNano),
		dtunb:    sb.DurationBuilder(constants.DurationTimeUnixNano),
		tib:      sb.FixedSizeBinaryBuilder(constants.TraceId),
		sib:      sb.FixedSizeBinaryBuilder(constants.SpanId),
		tsb:      sb.StringBuilder(constants.TraceState),
		psib:     sb.FixedSizeBinaryBuilder(constants.ParentSpanId),
		nb:       sb.StringBuilder(constants.Name),
		kb:       sb.Int32Builder(constants.KIND),
		dacb:     sb.Uint32Builder(constants.DroppedAttributesCount),
		decb:     sb.Uint32Builder(constants.DroppedEventsCount),
		dlcb:     sb.Uint32Builder(constants.DroppedLinksCount),
		sb:       StatusBuilderFrom(sb.StructBuilder(constants.Status)),
	}
}

// Build builds the span array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *SpanBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new span to the builder.
func (b *SpanBuilder) Append(span *ptrace.Span, sharedData *SharedData, relatedData *RelatedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(span, func() error {
		ID := relatedData.NextSpanID()

		b.ib.Append(ID)
		b.stunb.Append(arrow.Timestamp(span.StartTimestamp()))
		duration := span.EndTimestamp().AsTime().Sub(span.StartTimestamp().AsTime()).Nanoseconds()
		b.dtunb.Append(arrow.Duration(duration))
		tib := span.TraceID()
		b.tib.Append(tib[:])
		sib := span.SpanID()
		b.sib.Append(sib[:])
		b.tsb.AppendNonEmpty(span.TraceState().AsRaw())
		psib := span.ParentSpanID()
		b.psib.Append(psib[:])
		b.nb.AppendNonEmpty(span.Name())
		b.kb.AppendNonZero(int32(span.Kind()))

		// Span Attributes
		err := relatedData.AttrsBuilders().Span().Accumulator().AppendUniqueAttributesWithID(ID, span.Attributes(), sharedData.sharedAttributes, nil)
		if err != nil {
			return werror.Wrap(err)
		}
		b.dacb.AppendNonZero(span.DroppedAttributesCount())

		// Events
		err = relatedData.EventBuilder().Accumulator().Append(ID, span.Events(), sharedData.sharedEventAttributes)
		if err != nil {
			return werror.Wrap(err)
		}
		b.decb.AppendNonZero(span.DroppedEventsCount())

		// Links
		err = relatedData.LinkBuilder().Accumulator().Append(ID, span.Links(), sharedData.sharedLinkAttributes)
		if err != nil {
			return werror.Wrap(err)
		}
		b.dlcb.AppendNonZero(span.DroppedLinksCount())

		return b.sb.Append(span.Status())
	})
}

// Release releases the memory allocated by the builder.
func (b *SpanBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
