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

package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// SpanDT is the Arrow Data Type describing a span.
var (
	SpanDT = arrow.StructOf([]arrow.Field{
		{Name: constants.StartTimeUnixNano, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.EndTimeUnixNano, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.TraceId, Type: acommon.DefaultDictFixed16Binary},
		{Name: constants.SpanId, Type: acommon.DefaultDictFixed8Binary},
		{Name: constants.TraceState, Type: acommon.DefaultDictString},
		{Name: constants.ParentSpanId, Type: acommon.DefaultDictFixed8Binary},
		{Name: constants.Name, Type: acommon.DefaultDictString},
		{Name: constants.KIND, Type: arrow.PrimitiveTypes.Int32},
		{Name: constants.Attributes, Type: acommon.AttributesDT},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.SpanEvents, Type: arrow.ListOf(EventDT)},
		{Name: constants.DroppedEventsCount, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.SpanLinks, Type: arrow.ListOf(LinkDT)},
		{Name: constants.DroppedLinksCount, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.Status, Type: StatusDT},
	}...)
)

// SpanBuilder is a helper to build a span.
type SpanBuilder struct {
	released bool

	builder *array.StructBuilder

	stunb *array.Uint64Builder               // start time unix nano builder
	etunb *array.Uint64Builder               // end time unix nano builder
	tib   *acommon.AdaptiveDictionaryBuilder // trace id builder
	sib   *acommon.AdaptiveDictionaryBuilder // span id builder
	tsb   *acommon.AdaptiveDictionaryBuilder // trace state builder
	psib  *acommon.AdaptiveDictionaryBuilder // parent span id builder
	nb    *acommon.AdaptiveDictionaryBuilder // name builder
	kb    *array.Int32Builder                // kind builder
	ab    *acommon.AttributesBuilder         // attributes builder
	dacb  *array.Uint32Builder               // dropped attributes count builder
	sesb  *array.ListBuilder                 // span event list builder
	seb   *EventBuilder                      // span event builder
	decb  *array.Uint32Builder               // dropped events count builder
	slsb  *array.ListBuilder                 // span link list builder
	slb   *LinkBuilder                       // span link builder
	dlcb  *array.Uint32Builder               // dropped links count builder
	sb    *StatusBuilder                     // status builder
}

// NewSpanBuilder creates a new SpansBuilder with a given allocator.
//
// Once the builder is no longer needed, Release() must be called to free the
// memory allocated by the builder.
func NewSpanBuilder(pool memory.Allocator) *SpanBuilder {
	sb := array.NewStructBuilder(pool, SpanDT)
	return SpanBuilderFrom(sb)
}

func SpanBuilderFrom(sb *array.StructBuilder) *SpanBuilder {
	return &SpanBuilder{
		released: false,
		builder:  sb,
		stunb:    sb.FieldBuilder(0).(*array.Uint64Builder),
		etunb:    sb.FieldBuilder(1).(*array.Uint64Builder),
		tib:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(2)),
		sib:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(3)),
		tsb:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(4)),
		psib:     acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(5)),
		nb:       acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(6)),
		kb:       sb.FieldBuilder(7).(*array.Int32Builder),
		ab:       acommon.AttributesBuilderFrom(sb.FieldBuilder(8).(*array.MapBuilder)),
		dacb:     sb.FieldBuilder(9).(*array.Uint32Builder),
		sesb:     sb.FieldBuilder(10).(*array.ListBuilder),
		seb:      EventBuilderFrom(sb.FieldBuilder(10).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		decb:     sb.FieldBuilder(11).(*array.Uint32Builder),
		slsb:     sb.FieldBuilder(12).(*array.ListBuilder),
		slb:      LinkBuilderFrom(sb.FieldBuilder(12).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		dlcb:     sb.FieldBuilder(13).(*array.Uint32Builder),
		sb:       StatusBuilderFrom(sb.FieldBuilder(14).(*array.StructBuilder)),
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

	b.builder.Append(true)
	b.stunb.Append(uint64(span.StartTimestamp()))
	b.etunb.Append(uint64(span.EndTimestamp()))
	tib := span.TraceID()
	if err := b.tib.AppendBinary(tib[:]); err != nil {
		return err
	}
	sib := span.SpanID()
	if err := b.sib.AppendBinary(sib[:]); err != nil {
		return err
	}
	traceState := span.TraceState().AsRaw()
	if traceState == "" {
		b.tsb.AppendNull()
	} else {
		if err := b.tsb.AppendString(traceState); err != nil {
			return err
		}
	}
	psib := span.ParentSpanID()
	if err := b.psib.AppendBinary(psib[:]); err != nil {
		return err
	}
	name := span.Name()
	if name == "" {
		b.nb.AppendNull()
	} else {
		if err := b.nb.AppendString(name); err != nil {
			return err
		}
	}
	b.kb.Append(int32(span.Kind()))
	if err := b.ab.Append(span.Attributes()); err != nil {
		return err
	}
	b.dacb.Append(span.DroppedAttributesCount())
	evts := span.Events()
	sc := evts.Len()
	if sc > 0 {
		b.sesb.Append(true)
		b.sesb.Reserve(sc)
		for i := 0; i < sc; i++ {
			if err := b.seb.Append(evts.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.sesb.Append(false)
	}
	b.decb.Append(span.DroppedEventsCount())
	lks := span.Links()
	lc := lks.Len()
	if lc > 0 {
		b.slsb.Append(true)
		b.slsb.Reserve(lc)
		for i := 0; i < lc; i++ {
			if err := b.slb.Append(lks.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.slsb.Append(false)
	}
	b.dlcb.Append(span.DroppedLinksCount())
	if err := b.sb.Append(span.Status()); err != nil {
		return err
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *SpanBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
