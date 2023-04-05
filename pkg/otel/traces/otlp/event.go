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
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
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
		return nil, werror.Wrap(err)
	}

	timeUnixNanoID, _ := arrowutils.FieldIDFromStruct(eventDT, constants.TimeUnixNano)
	nameID, _ := arrowutils.FieldIDFromStruct(eventDT, constants.Name)
	droppedAttributesCountId, _ := arrowutils.FieldIDFromStruct(eventDT, constants.DroppedAttributesCount)
	attributesID, err := otlp.NewAttributeIds(eventDT)
	if err != nil {
		return nil, werror.Wrap(err)
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
		return werror.Wrap(err)
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
			return werror.Wrap(err)
		}

		event.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		name, err := events.StringFieldByID(ids.Name, eventIdx)
		if err != nil {
			return werror.Wrap(err)
		}

		event.SetName(name)

		if err = otlp.AppendAttributesInto(event.Attributes(), events.Array(), eventIdx, ids.Attributes); err != nil {
			return werror.Wrap(err)
		}

		dac, err := events.U32FieldByID(ids.DroppedAttributesCount, eventIdx)
		if err != nil {
			return werror.Wrap(err)
		}

		event.SetDroppedAttributesCount(dac)
	}

	return nil
}
