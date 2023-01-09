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
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type EventIds struct {
	Id                     int
	TimeUnixNano           int
	Name                   int
	Attributes             *otlp.AttributeIds
	DroppedAttributesCount int
}

func NewEventIds(spansDT *arrow.StructType) (*EventIds, error) {
	id, eventDT, err := arrowutils.ListOfStructsFieldIDFromStruct(spansDT, constants.SpanEvents)
	if err != nil {
		return nil, err
	}

	timeUnixNanoID, timeUnixNanoFound := eventDT.FieldIdx(constants.TimeUnixNano)
	if !timeUnixNanoFound {
		return nil, fmt.Errorf("field %s not found", constants.TimeUnixNano)
	}
	nameID, nameFound := eventDT.FieldIdx(constants.Name)
	if !nameFound {
		return nil, fmt.Errorf("field %s not found", constants.Name)
	}
	droppedAttributesCountId, droppedAttributesCountFound := eventDT.FieldIdx(constants.DroppedAttributesCount)
	if !droppedAttributesCountFound {
		return nil, fmt.Errorf("field %s not found", constants.DroppedAttributesCount)
	}
	attributesID, err := otlp.NewAttributeIds(eventDT)
	if err != nil {
		return nil, err
	}

	return &EventIds{
		Id:                     id,
		TimeUnixNano:           timeUnixNanoID,
		Name:                   nameID,
		Attributes:             attributesID,
		DroppedAttributesCount: droppedAttributesCountId,
	}, nil
}

// AppendEventsInto initializes a Span's Events from an Arrow representation.
func AppendEventsInto(spans ptrace.SpanEventSlice, arrowSpans *arrowutils.ListOfStructs, spanIdx int, ids *EventIds) error {
	events, err := arrowSpans.ListOfStructsById(spanIdx, ids.Id)
	if err != nil {
		return err
	}
	if events == nil {
		// No event found
		return nil
	}

	for eventIdx := events.Start(); eventIdx < events.End(); eventIdx++ {
		event := spans.AppendEmpty()

		if events.IsNull(eventIdx) {
			continue
		}

		timeUnixNano, err := events.TimestampFieldByID(ids.TimeUnixNano, eventIdx)
		if err != nil {
			return err
		}

		event.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		name, err := events.StringFieldByID(ids.Name, eventIdx)
		if err != nil {
			return err
		}

		event.SetName(name)

		if err = otlp.AppendAttributesInto(event.Attributes(), events.Array(), eventIdx, ids.Attributes); err != nil {
			return err
		}

		dac, err := events.U32FieldByID(ids.DroppedAttributesCount, eventIdx)
		if err != nil {
			return err
		}

		event.SetDroppedAttributesCount(dac)
	}

	return nil
}
