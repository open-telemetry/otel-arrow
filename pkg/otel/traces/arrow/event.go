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

// EventDT is the Arrow Data Type describing a span event.
var (
	EventDT = arrow.StructOf([]arrow.Field{
		{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		{Name: constants.Name, Type: acommon.DefaultDictString},
		{Name: constants.Attributes, Type: acommon.AttributesDT},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

type EventBuilder struct {
	released bool
	builder  *array.StructBuilder
	tunb     *array.TimestampBuilder            // time_unix_nano builder
	nb       *acommon.AdaptiveDictionaryBuilder // name builder
	ab       *acommon.AttributesBuilder         // attributes builder
	dacb     *array.Uint32Builder               // dropped_attributes_count builder
}

func NewEventBuilder(pool memory.Allocator) *EventBuilder {
	return EventBuilderFrom(array.NewStructBuilder(pool, EventDT))
}

func EventBuilderFrom(eb *array.StructBuilder) *EventBuilder {
	return &EventBuilder{
		released: false,
		builder:  eb,
		tunb:     eb.FieldBuilder(0).(*array.TimestampBuilder),
		nb:       acommon.AdaptiveDictionaryBuilderFrom(eb.FieldBuilder(1)),
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
	b.tunb.Append(arrow.Timestamp(event.Timestamp()))

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

		b.released = true
	}
}
