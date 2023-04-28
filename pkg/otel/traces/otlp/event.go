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

package otlp

import (
	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	// SpanEventIDs is a struct containing the Arrow field IDs for the Event struct.
	SpanEventIDs struct {
		ID                     int
		TimeUnixNano           int
		Name                   int
		AttrsID                int
		DroppedAttributesCount int
	}

	SpanEventsStore struct {
		nextID     uint16
		eventsByID map[uint16][]*ptrace.SpanEvent
	}
)

// NewSpanEventsStore creates a new SpanEventsStore.
func NewSpanEventsStore() *SpanEventsStore {
	return &SpanEventsStore{
		eventsByID: make(map[uint16][]*ptrace.SpanEvent),
	}
}

// EventsByID returns the events for the given ID.
func (s *SpanEventsStore) EventsByID(ID uint16, sharedAttrs pcommon.Map) []*ptrace.SpanEvent {
	if events, ok := s.eventsByID[ID]; ok {
		if sharedAttrs.Len() > 0 {
			// Add shared attributes to all events.
			for _, event := range events {
				attrs := event.Attributes()
				sharedAttrs.Range(func(k string, v pcommon.Value) bool {
					v.CopyTo(attrs.PutEmpty(k))
					return true
				})
			}
		}
		return events
	}
	return nil
}

// SpanEventsStoreFrom creates an SpanEventsStore from an arrow.Record.
// Note: This function consume the record.
func SpanEventsStoreFrom(record arrow.Record, attrsStore *otlp.Attributes32Store) (*SpanEventsStore, error) {
	defer record.Release()

	store := &SpanEventsStore{
		eventsByID: make(map[uint16][]*ptrace.SpanEvent),
	}

	spanEventIDs, err := SchemaToSpanEventIDs(record.Schema())
	if err != nil {
		return nil, werror.Wrap(err)
	}

	eventsCount := int(record.NumRows())

	// Read all event fields from the record and reconstruct the event lists
	// by ID.
	for row := 0; row < eventsCount; row++ {
		ID, err := arrowutils.U16FromRecord(record, spanEventIDs.ID, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		timeUnixNano, err := arrowutils.TimestampFromRecord(record, spanEventIDs.TimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		name, err := arrowutils.StringFromRecord(record, spanEventIDs.Name, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		attrsID, err := arrowutils.NullableU32FromRecord(record, spanEventIDs.AttrsID, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		dac, err := arrowutils.U32FromRecord(record, spanEventIDs.DroppedAttributesCount, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		event := ptrace.NewSpanEvent()
		event.SetTimestamp(pcommon.Timestamp(timeUnixNano))
		event.SetName(name)

		if attrsID != nil {
			attrs := attrsStore.AttributesByDeltaID(*attrsID)
			if attrs != nil {
				attrs.CopyTo(event.Attributes())
			}
		}

		event.SetDroppedAttributesCount(dac)
		store.eventsByID[ID] = append(store.eventsByID[ID], &event)
	}

	return store, nil
}

// SchemaToSpanEventIDs pre-computes the field IDs for the events record.
func SchemaToSpanEventIDs(schema *arrow.Schema) (*SpanEventIDs, error) {
	ID, err := arrowutils.FieldIDFromSchema(schema, constants.ID)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	timeUnixNano, err := arrowutils.FieldIDFromSchema(schema, constants.TimeUnixNano)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	name, err := arrowutils.FieldIDFromSchema(schema, constants.Name)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attrsID, err := arrowutils.FieldIDFromSchema(schema, constants.AttributesID)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	dac, err := arrowutils.FieldIDFromSchema(schema, constants.DroppedAttributesCount)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &SpanEventIDs{
		ID:                     ID,
		TimeUnixNano:           timeUnixNano,
		Name:                   name,
		AttrsID:                attrsID,
		DroppedAttributesCount: dac,
	}, nil
}
