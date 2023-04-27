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
	"time"

	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type SpansIds struct {
	SpansID              int
	ID                   int
	TraceID              int
	SpanID               int
	TraceState           int
	ParentSpanID         int
	Name                 int
	Kind                 int
	StartTimeUnixNano    int
	DurationTimeUnixNano int
	DropAttributesCount  int
	DropEventsCount      int
	DropLinksCount       int
	Status               *StatusIds
}

type StatusIds struct {
	Id      int
	Code    int
	Message int
}

func NewSpansIds(scopeSpansDT *arrow.StructType) (*SpansIds, error) {
	spansID, spanDT, err := arrowutils.ListOfStructsFieldIDFromStruct(scopeSpansDT, constants.Spans)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	ID, _ := arrowutils.FieldIDFromStruct(spanDT, constants.ID)
	traceId, _ := arrowutils.FieldIDFromStruct(spanDT, constants.TraceId)
	spanId, _ := arrowutils.FieldIDFromStruct(spanDT, constants.SpanId)
	traceState, _ := arrowutils.FieldIDFromStruct(spanDT, constants.TraceState)
	parentSpanId, _ := arrowutils.FieldIDFromStruct(spanDT, constants.ParentSpanId)
	name, _ := arrowutils.FieldIDFromStruct(spanDT, constants.Name)
	kind, _ := arrowutils.FieldIDFromStruct(spanDT, constants.KIND)
	startTimeUnixNano, _ := arrowutils.FieldIDFromStruct(spanDT, constants.StartTimeUnixNano)
	durationTimeUnixNano, _ := arrowutils.FieldIDFromStruct(spanDT, constants.DurationTimeUnixNano)
	droppedAttributesCount, _ := arrowutils.FieldIDFromStruct(spanDT, constants.DroppedAttributesCount)
	droppedEventsCount, _ := arrowutils.FieldIDFromStruct(spanDT, constants.DroppedEventsCount)
	droppedLinksCount, _ := arrowutils.FieldIDFromStruct(spanDT, constants.DroppedLinksCount)

	status, err := NewStatusIds(spanDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &SpansIds{
		SpansID:              spansID,
		ID:                   ID,
		TraceID:              traceId,
		SpanID:               spanId,
		TraceState:           traceState,
		ParentSpanID:         parentSpanId,
		Name:                 name,
		Kind:                 kind,
		StartTimeUnixNano:    startTimeUnixNano,
		DurationTimeUnixNano: durationTimeUnixNano,
		DropAttributesCount:  droppedAttributesCount,
		DropEventsCount:      droppedEventsCount,
		DropLinksCount:       droppedLinksCount,
		Status:               status,
	}, nil
}

func NewStatusIds(spansDT *arrow.StructType) (*StatusIds, error) {
	statusId, statusDT, err := arrowutils.StructFieldIDFromStruct(spansDT, constants.Status)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	code, _ := arrowutils.FieldIDFromStruct(statusDT, constants.StatusCode)
	message, _ := arrowutils.FieldIDFromStruct(statusDT, constants.StatusMessage)

	return &StatusIds{
		Id:      statusId,
		Code:    code,
		Message: message,
	}, nil
}

func AppendSpanInto(
	spans ptrace.SpanSlice,
	los *arrowutils.ListOfStructs,
	row int,
	ids *SpansIds,
	sharedAttrs pcommon.Map,
	sharedEventAttrs pcommon.Map,
	sharedLinkAttrs pcommon.Map,
	relatedData *RelatedData,
) error {
	span := spans.AppendEmpty()
	deltaID, err := los.U16FieldByID(ids.ID, row)
	if err != nil {
		return werror.Wrap(err)
	}
	ID := relatedData.SpanIDFromDelta(deltaID)

	traceID, err := los.FixedSizeBinaryFieldByID(ids.TraceID, row)
	if err != nil {
		return werror.Wrap(err)
	}
	if len(traceID) != 16 {
		return werror.WrapWithContext(common.ErrInvalidTraceIDLength, map[string]interface{}{"traceID": traceID})
	}
	spanID, err := los.FixedSizeBinaryFieldByID(ids.SpanID, row)
	if err != nil {
		return werror.Wrap(err)
	}
	if len(spanID) != 8 {
		return werror.WrapWithContext(common.ErrInvalidSpanIDLength, map[string]interface{}{"spanID": spanID})
	}
	traceState, err := los.StringFieldByID(ids.TraceState, row)
	if err != nil {
		return werror.Wrap(err)
	}
	parentSpanID, err := los.FixedSizeBinaryFieldByID(ids.ParentSpanID, row)
	if err != nil {
		return werror.Wrap(err)
	}
	if parentSpanID != nil && len(parentSpanID) != 8 {
		return werror.WrapWithContext(common.ErrInvalidSpanIDLength, map[string]interface{}{"parentSpanID": parentSpanID})
	}
	name, err := los.StringFieldByID(ids.Name, row)
	if err != nil {
		return werror.Wrap(err)
	}
	kind, err := los.I32FieldByID(ids.Kind, row)
	if err != nil {
		return werror.Wrap(err)
	}
	startTimeUnixNano, err := los.TimestampFieldByID(ids.StartTimeUnixNano, row)
	if err != nil {
		return werror.Wrap(err)
	}
	durationNano, err := los.DurationFieldByID(ids.DurationTimeUnixNano, row)
	if err != nil {
		return werror.Wrap(err)
	}
	endTimeUnixNano := startTimeUnixNano.ToTime(arrow.Nanosecond).Add(time.Duration(durationNano))
	droppedAttributesCount, err := los.U32FieldByID(ids.DropAttributesCount, row)
	if err != nil {
		return werror.Wrap(err)
	}
	droppedEventsCount, err := los.U32FieldByID(ids.DropEventsCount, row)
	if err != nil {
		return werror.Wrap(err)
	}
	droppedLinksCount, err := los.U32FieldByID(ids.DropLinksCount, row)
	if err != nil {
		return werror.Wrap(err)
	}
	statusDt, statusArr, err := los.StructByID(ids.Status.Id, row)
	if err != nil {
		return werror.Wrap(err)
	}
	if statusDt != nil {
		// Status exists
		message, err := arrowutils.StringFromStruct(statusArr, row, ids.Status.Message)
		if err != nil {
			return werror.Wrap(err)
		}
		span.Status().SetMessage(message)

		code, err := arrowutils.I32FromStruct(statusArr, row, ids.Status.Code)
		if err != nil {
			return werror.Wrap(err)
		}
		span.Status().SetCode(ptrace.StatusCode(code))
	}
	spanAttrs := span.Attributes()
	attrs := relatedData.SpanAttrMapStore.AttributesByID(ID)
	if attrs != nil {
		attrs.CopyTo(spanAttrs)
	}
	if sharedAttrs.Len() > 0 {
		sharedAttrs.Range(func(k string, v pcommon.Value) bool {
			v.CopyTo(spanAttrs.PutEmpty(k))
			return true
		})
	}

	events := relatedData.SpanEventsStore.EventsByID(ID, sharedEventAttrs)
	eventSlice := span.Events()
	for _, event := range events {
		event.MoveTo(eventSlice.AppendEmpty())
	}

	links := relatedData.SpanLinksStore.LinksByID(ID, sharedLinkAttrs)
	linkSlice := span.Links()
	for _, link := range links {
		link.MoveTo(linkSlice.AppendEmpty())
	}

	var tid pcommon.TraceID
	var sid pcommon.SpanID
	var psid pcommon.SpanID
	copy(tid[:], traceID)
	copy(sid[:], spanID)
	copy(psid[:], parentSpanID)

	span.SetTraceID(tid)
	span.SetSpanID(sid)
	span.TraceState().FromRaw(traceState)
	span.SetParentSpanID(psid)
	span.SetName(name)
	span.SetKind(ptrace.SpanKind(kind))
	span.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
	span.SetEndTimestamp(pcommon.Timestamp(endTimeUnixNano.UnixNano()))
	span.SetDroppedAttributesCount(droppedAttributesCount)
	span.SetDroppedEventsCount(droppedEventsCount)
	span.SetDroppedLinksCount(droppedLinksCount)

	return nil
}
