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
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// SpanDT is the Arrow Data Type describing a span.
var (
	SpanDT = arrow.StructOf([]arrow.Field{
		{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.EndTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.TraceId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 16}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.SpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.TraceState, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.ParentSpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.Name, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.KIND, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SpanEvents, Type: arrow.ListOf(EventDT), Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedEventsCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SpanLinks, Type: arrow.ListOf(LinkDT), Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedLinksCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Status, Type: StatusDT, Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// SpanBuilder is a helper to build a span.
type SpanBuilder struct {
	released bool

	builder *builder.StructBuilder

	stunb *builder.TimestampBuilder       // start time unix nano builder
	etunb *builder.TimestampBuilder       // end time unix nano builder
	tib   *builder.FixedSizeBinaryBuilder // trace id builder
	sib   *builder.FixedSizeBinaryBuilder // span id builder
	tsb   *builder.StringBuilder          // trace state builder
	psib  *builder.FixedSizeBinaryBuilder // parent span id builder
	nb    *builder.StringBuilder          // name builder
	kb    *builder.Int32Builder           // kind builder
	ab    *acommon.AttributesBuilder      // attributes builder
	dacb  *builder.Uint32Builder          // dropped attributes count builder
	sesb  *builder.ListBuilder            // span event list builder
	seb   *EventBuilder                   // span event builder
	decb  *builder.Uint32Builder          // dropped events count builder
	slsb  *builder.ListBuilder            // span link list builder
	slb   *LinkBuilder                    // span link builder
	dlcb  *builder.Uint32Builder          // dropped links count builder
	sb    *StatusBuilder                  // status builder
}

func SpanBuilderFrom(sb *builder.StructBuilder) *SpanBuilder {
	sesb := sb.ListBuilder(constants.SpanEvents)
	slsb := sb.ListBuilder(constants.SpanLinks)

	return &SpanBuilder{
		released: false,
		builder:  sb,
		stunb:    sb.TimestampBuilder(constants.StartTimeUnixNano),
		etunb:    sb.TimestampBuilder(constants.EndTimeUnixNano),
		tib:      sb.FixedSizeBinaryBuilder(constants.TraceId),
		sib:      sb.FixedSizeBinaryBuilder(constants.SpanId),
		tsb:      sb.StringBuilder(constants.TraceState),
		psib:     sb.FixedSizeBinaryBuilder(constants.ParentSpanId),
		nb:       sb.StringBuilder(constants.Name),
		kb:       sb.Int32Builder(constants.KIND),
		ab:       acommon.AttributesBuilderFrom(sb.MapBuilder(constants.Attributes)),
		dacb:     sb.Uint32Builder(constants.DroppedAttributesCount),
		sesb:     sesb,
		seb:      EventBuilderFrom(sesb.StructBuilder()),
		decb:     sb.Uint32Builder(constants.DroppedEventsCount),
		slsb:     slsb,
		slb:      LinkBuilderFrom(slsb.StructBuilder()),
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
		return nil, fmt.Errorf("span builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new span to the builder.
func (b *SpanBuilder) Append(span ptrace.Span) error {
	if b.released {
		return fmt.Errorf("span builder already released")
	}

	return b.builder.Append(span, func() error {
		b.stunb.Append(arrow.Timestamp(span.StartTimestamp()))
		b.etunb.Append(arrow.Timestamp(span.EndTimestamp()))
		tib := span.TraceID()
		b.tib.Append(tib[:])
		sib := span.SpanID()
		b.sib.Append(sib[:])
		b.tsb.AppendNonEmpty(span.TraceState().AsRaw())
		psib := span.ParentSpanID()
		b.psib.Append(psib[:])
		b.nb.AppendNonEmpty(span.Name())
		b.kb.AppendNonZero(int32(span.Kind()))
		if err := b.ab.Append(span.Attributes()); err != nil {
			return err
		}
		b.dacb.AppendNonZero(span.DroppedAttributesCount())
		evts := span.Events()
		sc := evts.Len()
		if err := b.sesb.Append(sc, func() error {
			for i := 0; i < sc; i++ {
				if err := b.seb.Append(evts.At(i)); err != nil {
					return err
				}
			}
			return nil
		}); err != nil {
			return err
		}
		b.decb.AppendNonZero(span.DroppedEventsCount())
		lks := span.Links()
		lc := lks.Len()
		if err := b.slsb.Append(lc, func() error {
			for i := 0; i < lc; i++ {
				if err := b.slb.Append(lks.At(i)); err != nil {
					return err
				}
			}
			return nil
		}); err != nil {
			return err
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
