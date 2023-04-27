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

// Events are represented as Arrow records.
//
// An event accumulator is used to collect of the events across all spans, and
// once the entire trace is processed, the events are being globally sorted and
// written to the Arrow record batch. This process improves the compression
// ratio of the Arrow record batch.

import (
	"errors"
	"math"
	"sort"

	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// EventSchema is the Arrow Data Type describing an event (as a related record
// to the main trace record).
var (
	EventSchema = arrow.NewSchema([]arrow.Field{
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint16},
		{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Name, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.AttributesID, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional, schema.DeltaEncoding)},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
	}, nil)
)

type (
	// EventBuilder is an Arrow builder for events.
	EventBuilder struct {
		released bool

		builder *builder.RecordBuilderExt

		ib   *builder.Uint16Builder      // `id` builder
		tunb *builder.TimestampBuilder   // `time_unix_nano` builder
		nb   *builder.StringBuilder      // `name` builder
		aib  *builder.Uint32DeltaBuilder // attributes id builder
		dacb *builder.Uint32Builder      // `dropped_attributes_count` builder

		accumulator *EventAccumulator
	}

	// Event is an internal representation of an event used by the
	// EventAccumulator.
	Event struct {
		ID                     uint16
		TimeUnixNano           pcommon.Timestamp
		Name                   string
		Attributes             pcommon.Map
		SharedAttributes       *common.SharedAttributes
		DroppedAttributesCount uint32
	}

	// EventAccumulator is an accumulator for events that is used to sort events
	// globally in order to improve compression.
	EventAccumulator struct {
		groupCount uint16
		events     []Event
	}
)

func NewEventBuilder(rBuilder *builder.RecordBuilderExt) (*EventBuilder, error) {
	b := &EventBuilder{
		released:    false,
		builder:     rBuilder,
		accumulator: NewEventAccumulator(),
	}
	if err := b.init(); err != nil {
		return nil, werror.Wrap(err)
	}
	return b, nil
}

func (b *EventBuilder) init() error {
	b.ib = b.builder.Uint16Builder(constants.ID)
	b.tunb = b.builder.TimestampBuilder(constants.TimeUnixNano)
	b.nb = b.builder.StringBuilder(constants.Name)
	b.aib = b.builder.Uint32DeltaBuilder(constants.AttributesID)
	// As the attributes are sorted before insertion, the delta between two
	// consecutive attributes ID should always be <=1.
	b.aib.SetMaxDelta(1)
	b.dacb = b.builder.Uint32Builder(constants.DroppedAttributesCount)
	return nil
}

func (b *EventBuilder) SchemaID() string {
	return b.builder.SchemaID()
}

func (b *EventBuilder) IsEmpty() bool {
	return b.accumulator.IsEmpty()
}

func (b *EventBuilder) Accumulator() *EventAccumulator {
	return b.accumulator
}

func (b *EventBuilder) BuildRecord(attrsAccu *acommon.Attributes32Accumulator) (record arrow.Record, err error) {
	schemaNotUpToDateCount := 0

	// Loop until the record is built successfully.
	// Intermediaries steps may be required to update the schema.
	for {
		attrsAccu.Reset()
		record, err = b.TryBuild(attrsAccu)
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

func (b *EventBuilder) TryBuild(attrsAccu *acommon.Attributes32Accumulator) (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	b.accumulator.Sort()

	for _, event := range b.accumulator.events {
		b.ib.Append(event.ID)
		b.tunb.Append(arrow.Timestamp(event.TimeUnixNano.AsTime().UnixNano()))
		b.nb.AppendNonEmpty(event.Name)

		// Attributes
		var ID int64
		ID, err = attrsAccu.AppendUniqueAttributes(event.Attributes, event.SharedAttributes, nil)
		if err != nil {
			return
		}
		if ID >= 0 {
			b.aib.Append(uint32(ID))
		} else {
			b.aib.AppendNull()
		}

		b.dacb.AppendNonZero(event.DroppedAttributesCount)
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		initErr := b.init()
		if initErr != nil {
			return nil, werror.Wrap(initErr)
		}
	}
	return
}

// Release releases the memory allocated by the builder.
func (b *EventBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}

// NewEventAccumulator creates a new EventAccumulator.
func NewEventAccumulator() *EventAccumulator {
	return &EventAccumulator{
		groupCount: 0,
		events:     make([]Event, 0),
	}
}

func (a *EventAccumulator) IsEmpty() bool {
	return len(a.events) == 0
}

// Append appends a slice of events to the accumulator.
func (a *EventAccumulator) Append(spanID uint16, events ptrace.SpanEventSlice, sharedAttrs *common.SharedAttributes) error {
	if a.groupCount == math.MaxUint16 {
		panic("The maximum number of group of events has been reached (max is uint16).")
	}

	if events.Len() == 0 {
		return nil
	}

	for i := 0; i < events.Len(); i++ {
		evt := events.At(i)
		a.events = append(a.events, Event{
			ID:                     spanID,
			TimeUnixNano:           evt.Timestamp(),
			Name:                   evt.Name(),
			Attributes:             evt.Attributes(),
			SharedAttributes:       sharedAttrs,
			DroppedAttributesCount: evt.DroppedAttributesCount(),
		})
	}

	a.groupCount++

	return nil
}

func (a *EventAccumulator) Sort() {
	sort.Slice(a.events, func(i, j int) bool {
		if a.events[i].Name == a.events[j].Name {
			return a.events[i].TimeUnixNano < a.events[j].TimeUnixNano
		} else {
			return a.events[i].Name < a.events[j].Name
		}
	})
}

func (a *EventAccumulator) Reset() {
	a.groupCount = 0
	a.events = a.events[:0]
}
