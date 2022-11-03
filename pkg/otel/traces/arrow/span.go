package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// SpanDT is the Arrow Data Type describing a span.
var (
	SpanDT = arrow.StructOf([]arrow.Field{
		{Name: constants.START_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.END_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.TRACE_ID, Type: acommon.DictU16Fixed16Binary},
		{Name: constants.SPAN_ID, Type: acommon.DictU16Fixed8Binary},
		{Name: constants.TRACE_STATE, Type: acommon.DictU16String},
		{Name: constants.PARENT_SPAN_ID, Type: acommon.DictU16Fixed8Binary},
		{Name: constants.NAME, Type: acommon.DictU16String},
		{Name: constants.KIND, Type: arrow.PrimitiveTypes.Int32},
		{Name: constants.ATTRIBUTES, Type: acommon.AttributesDT},
		{Name: constants.DROPPED_ATTRIBUTES_COUNT, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.SPAN_EVENTS, Type: arrow.ListOf(EventDT)},
		{Name: constants.DROPPED_EVENTS_COUNT, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.SPAN_LINKS, Type: arrow.ListOf(LinkDT)},
		{Name: constants.DROPPED_LINKS_COUNT, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.STATUS, Type: StatusDT},
	}...)
)

// SpanBuilder is a helper to build a span.
type SpanBuilder struct {
	released bool

	builder *array.StructBuilder

	stunb *array.Uint64Builder                    // start time unix nano builder
	etunb *array.Uint64Builder                    // end time unix nano builder
	tib   *array.FixedSizeBinaryDictionaryBuilder // trace id builder
	sib   *array.FixedSizeBinaryDictionaryBuilder // span id builder
	tsb   *array.BinaryDictionaryBuilder          // trace state builder
	psib  *array.FixedSizeBinaryDictionaryBuilder // parent span id builder
	nb    *array.BinaryDictionaryBuilder          // name builder
	kb    *array.Int32Builder                     // kind builder
	ab    *acommon.AttributesBuilder              // attributes builder
	dacb  *array.Uint32Builder                    // dropped attributes count builder
	sesb  *array.ListBuilder                      // span event list builder
	seb   *EventBuilder                           // span event builder
	decb  *array.Uint32Builder                    // dropped events count builder
	slsb  *array.ListBuilder                      // span link list builder
	slb   *LinkBuilder                            // span link builder
	dlcb  *array.Uint32Builder                    // dropped links count builder
	sb    *StatusBuilder                          // status builder
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
		tib:      sb.FieldBuilder(2).(*array.FixedSizeBinaryDictionaryBuilder),
		sib:      sb.FieldBuilder(3).(*array.FixedSizeBinaryDictionaryBuilder),
		tsb:      sb.FieldBuilder(4).(*array.BinaryDictionaryBuilder),
		psib:     sb.FieldBuilder(5).(*array.FixedSizeBinaryDictionaryBuilder),
		nb:       sb.FieldBuilder(6).(*array.BinaryDictionaryBuilder),
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
	if err := b.tib.Append(tib[:]); err != nil {
		return err
	}
	sib := span.SpanID()
	if err := b.sib.Append(sib[:]); err != nil {
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
	if err := b.psib.Append(psib[:]); err != nil {
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
		b.stunb.Release()
		b.etunb.Release()
		b.tib.Release()
		b.sib.Release()
		b.tsb.Release()
		b.psib.Release()
		b.nb.Release()
		b.kb.Release()
		b.ab.Release()
		b.dacb.Release()
		b.sesb.Release()
		b.seb.Release()
		b.decb.Release()
		b.slsb.Release()
		b.slb.Release()
		b.dlcb.Release()
		b.sb.Release()

		b.released = true
	}
}
