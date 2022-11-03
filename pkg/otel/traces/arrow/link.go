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

// LinkDT is the Arrow Data Type describing a link event.
var (
	LinkDT = arrow.StructOf([]arrow.Field{
		{Name: constants.TRACE_ID, Type: acommon.DictU16Fixed16Binary},
		// TODO: Not sure a dictionary if needed here
		{Name: constants.SPAN_ID, Type: acommon.DictU16Fixed8Binary},
		{Name: constants.TRACE_STATE, Type: acommon.DictU16String},
		{Name: constants.ATTRIBUTES, Type: acommon.AttributesDT},
		{Name: constants.DROPPED_ATTRIBUTES_COUNT, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

type LinkBuilder struct {
	released bool
	builder  *array.StructBuilder
	tib      *array.FixedSizeBinaryDictionaryBuilder // trace id builder
	sib      *array.FixedSizeBinaryDictionaryBuilder // span id builder
	tsb      *array.BinaryDictionaryBuilder          // trace state builder
	ab       *acommon.AttributesBuilder              // attributes builder
	dacb     *array.Uint32Builder                    // dropped attributes count builder
}

func NewLinkBuilder(pool memory.Allocator) *LinkBuilder {
	return LinkBuilderFrom(array.NewStructBuilder(pool, LinkDT))
}

func LinkBuilderFrom(lb *array.StructBuilder) *LinkBuilder {
	return &LinkBuilder{
		released: false,
		builder:  lb,
		tib:      lb.FieldBuilder(0).(*array.FixedSizeBinaryDictionaryBuilder),
		sib:      lb.FieldBuilder(1).(*array.FixedSizeBinaryDictionaryBuilder),
		tsb:      lb.FieldBuilder(2).(*array.BinaryDictionaryBuilder),
		ab:       acommon.AttributesBuilderFrom(lb.FieldBuilder(3).(*array.MapBuilder)),
		dacb:     lb.FieldBuilder(4).(*array.Uint32Builder),
	}
}

// Append appends a new link to the builder.
func (b *LinkBuilder) Append(link ptrace.SpanLink) error {
	if b.released {
		return fmt.Errorf("link builder already released")
	}

	b.builder.Append(true)
	tid := link.TraceID()
	if err := b.tib.Append(tid[:]); err != nil {
		return err
	}
	sid := link.SpanID()
	if err := b.sib.Append(sid[:]); err != nil {
		return err
	}
	traceState := link.TraceState().AsRaw()
	if traceState == "" {
		b.tsb.AppendNull()
	} else {
		if err := b.tsb.AppendString(traceState); err != nil {
			return err
		}
	}
	if err := b.ab.Append(link.Attributes()); err != nil {
		return err
	}
	b.dacb.Append(link.DroppedAttributesCount())
	return nil
}

// Build builds the link array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *LinkBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("link builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the memory allocated by the builder.
func (b *LinkBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.tib.Release()
		b.sib.Release()
		b.tsb.Release()
		b.ab.Release()
		b.dacb.Release()

		b.released = true
	}
}
