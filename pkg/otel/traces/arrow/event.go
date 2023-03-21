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
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// EventDT is the Arrow Data Type describing a span event.
var (
	EventDT = arrow.StructOf([]arrow.Field{
		{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Name, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
	}...)
)

type EventBuilder struct {
	released bool

	builder *builder.StructBuilder

	tunb *builder.TimestampBuilder  // `time_unix_nano` builder
	nb   *builder.StringBuilder     // `name` builder
	ab   *acommon.AttributesBuilder // `attributes` builder
	dacb *builder.Uint32Builder     // `dropped_attributes_count` builder
}

func EventBuilderFrom(eb *builder.StructBuilder) *EventBuilder {
	return &EventBuilder{
		released: false,
		builder:  eb,
		tunb:     eb.TimestampBuilder(constants.TimeUnixNano),
		nb:       eb.StringBuilder(constants.Name),
		ab:       acommon.AttributesBuilderFrom(eb.MapBuilder(constants.Attributes)),
		dacb:     eb.Uint32Builder(constants.DroppedAttributesCount),
	}
}

// Append appends a new event to the builder.
func (b *EventBuilder) Append(event ptrace.SpanEvent) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(event, func() error {
		b.tunb.Append(arrow.Timestamp(event.Timestamp()))
		b.nb.AppendNonEmpty(event.Name())
		b.dacb.AppendNonZero(event.DroppedAttributesCount())
		return b.ab.Append(event.Attributes())
	})
}

// Build builds the event array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *EventBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
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
