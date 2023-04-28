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
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type LogRecordIds struct {
	Id                   int
	RecordID             int
	TimeUnixNano         int
	ObservedTimeUnixNano int
	TraceID              int
	SpanID               int
	SeverityNumber       int
	SeverityText         int
	Body                 int
	Attributes           *otlp.AttributeIds
	DropAttributesCount  int
	Flags                int
}

func NewLogRecordIds(scopeLogsDT *arrow.StructType) (*LogRecordIds, error) {
	id, logDT, err := arrowutils.ListOfStructsFieldIDFromStruct(scopeLogsDT, constants.Logs)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	ID, _ := arrowutils.FieldIDFromStruct(logDT, constants.ID)
	timeUnixNano, _ := arrowutils.FieldIDFromStruct(logDT, constants.TimeUnixNano)
	observedTimeUnixNano, _ := arrowutils.FieldIDFromStruct(logDT, constants.ObservedTimeUnixNano)
	traceID, _ := arrowutils.FieldIDFromStruct(logDT, constants.TraceId)
	spanID, _ := arrowutils.FieldIDFromStruct(logDT, constants.SpanId)
	severityNumber, _ := arrowutils.FieldIDFromStruct(logDT, constants.SeverityNumber)
	severityText, _ := arrowutils.FieldIDFromStruct(logDT, constants.SeverityText)
	body, _ := arrowutils.FieldIDFromStruct(logDT, constants.Body)

	attributes, err := otlp.NewAttributeIds(logDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	droppedAttributesCount, _ := arrowutils.FieldIDFromStruct(logDT, constants.DroppedAttributesCount)
	flags, _ := arrowutils.FieldIDFromStruct(logDT, constants.Flags)

	return &LogRecordIds{
		Id:                   id,
		RecordID:             ID,
		TimeUnixNano:         timeUnixNano,
		ObservedTimeUnixNano: observedTimeUnixNano,
		TraceID:              traceID,
		SpanID:               spanID,
		SeverityNumber:       severityNumber,
		SeverityText:         severityText,
		Body:                 body,
		Attributes:           attributes,
		DropAttributesCount:  droppedAttributesCount,
		Flags:                flags,
	}, nil
}

func AppendLogRecordInto(
	logs plog.LogRecordSlice,
	los *arrowutils.ListOfStructs,
	row int,
	ids *LogRecordIds,
	relatedData *RelatedData,
) error {
	logRecord := logs.AppendEmpty()
	deltaID, err := los.U16FieldByID(ids.RecordID, row)
	if err != nil {
		return werror.Wrap(err)
	}
	ID := relatedData.LogRecordIDFromDelta(deltaID)

	timeUnixNano, err := los.TimestampFieldByID(ids.TimeUnixNano, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	observedTimeUnixNano, err := los.TimestampFieldByID(ids.ObservedTimeUnixNano, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}

	traceID, err := los.FixedSizeBinaryFieldByID(ids.TraceID, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	if len(traceID) != 16 {
		return werror.WrapWithContext(common.ErrInvalidTraceIDLength, map[string]interface{}{"row": row, "traceID": traceID})
	}
	spanID, err := los.FixedSizeBinaryFieldByID(ids.SpanID, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	if len(spanID) != 8 {
		return werror.WrapWithContext(common.ErrInvalidSpanIDLength, map[string]interface{}{"row": row, "spanID": spanID})
	}

	severityNumber, err := los.I32FieldByID(ids.SeverityNumber, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	severityText, err := los.StringFieldByID(ids.SeverityText, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}

	body := los.FieldByID(ids.Body)
	if anyValueArr, ok := body.(*array.SparseUnion); ok {
		if err := otlp.UpdateValueFrom(logRecord.Body(), anyValueArr, row); err != nil {
			return werror.Wrap(err)
		}
	} else {
		return werror.WrapWithContext(ErrBodyNotSparseUnion, map[string]interface{}{"row": row})
	}

	logRecordAttrs := logRecord.Attributes()
	attrs := relatedData.LogRecordAttrMapStore.AttributesByID(ID)
	if attrs != nil {
		attrs.CopyTo(logRecordAttrs)
	}

	err = otlp.AppendAttributesInto(logRecord.Attributes(), los.Array(), row, ids.Attributes)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	droppedAttributesCount, err := los.U32FieldByID(ids.DropAttributesCount, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}

	flags, err := los.U32FieldByID(ids.Flags, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}

	var tid pcommon.TraceID
	var sid pcommon.SpanID
	copy(tid[:], traceID)
	copy(sid[:], spanID)

	logRecord.SetTimestamp(pcommon.Timestamp(timeUnixNano))
	logRecord.SetObservedTimestamp(pcommon.Timestamp(observedTimeUnixNano))
	logRecord.SetTraceID(tid)
	logRecord.SetSpanID(sid)
	logRecord.SetSeverityNumber(plog.SeverityNumber(severityNumber))
	logRecord.SetSeverityText(severityText)
	logRecord.SetDroppedAttributesCount(droppedAttributesCount)
	logRecord.SetFlags(plog.LogRecordFlags(flags))
	return nil
}
