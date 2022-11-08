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

// EventDT is the Arrow Data Type describing a span event.
var (
	EventDT = arrow.StructOf([]arrow.Field{
		{Name: constants.TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.NAME, Type: acommon.DictU16String},
		{Name: constants.ATTRIBUTES, Type: acommon.AttributesDT},
		{Name: constants.DROPPED_ATTRIBUTES_COUNT, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

type EventBuilder struct {
	released bool
	builder  *array.StructBuilder
	tunb     *array.Uint64Builder           // time_unix_nano builder
	nb       *array.BinaryDictionaryBuilder // name builder
	ab       *acommon.AttributesBuilder     // attributes builder
	dacb     *array.Uint32Builder           // dropped_attributes_count builder
}

func NewEventBuilder(pool memory.Allocator) *EventBuilder {
	return EventBuilderFrom(array.NewStructBuilder(pool, EventDT))
}

func EventBuilderFrom(eb *array.StructBuilder) *EventBuilder {
	return &EventBuilder{
		released: false,
		builder:  eb,
		tunb:     eb.FieldBuilder(0).(*array.Uint64Builder),
		nb:       eb.FieldBuilder(1).(*array.BinaryDictionaryBuilder),
		ab:       acommon.AttributesBuilderFrom(eb.FieldBuilder(2).(*array.MapBuilder)),
		dacb:     eb.FieldBuilder(3).(*array.Uint32Builder),
	}
}

// Append appends a new event to the builder.
func (b *EventBuilder) Append(event ptrace.SpanEvent) error {
	if b.released {
		return fmt.Errorf("event builder already released")
	}

	b.builder.Append(true)
	b.tunb.Append(uint64(event.Timestamp()))
	name := event.Name()
	if name == "" {
		b.nb.AppendNull()
	} else {
		if err := b.nb.AppendString(name); err != nil {
			return err
		}
	}
	if err := b.ab.Append(event.Attributes()); err != nil {
		return err
	}
	b.dacb.Append(event.DroppedAttributesCount())
	return nil
}

// Build builds the event array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *EventBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("event builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the memory allocated by the builder.
func (b *EventBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.tunb.Release()
		b.nb.Release()
		b.ab.Release()
		b.dacb.Release()

		b.released = true
	}
}
