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
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type LogRecordIds struct {
	Id                   int
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
		return nil, err
	}

	timeUnixNano, _, err := arrowutils.FieldIDFromStruct(logDT, constants.TimeUnixNano)
	if err != nil {
		return nil, err
	}

	observedTimeUnixNano, _, err := arrowutils.FieldIDFromStruct(logDT, constants.ObservedTimeUnixNano)
	if err != nil {
		return nil, err
	}

	traceID, _, err := arrowutils.FieldIDFromStruct(logDT, constants.TraceId)
	if err != nil {
		return nil, err
	}

	spanID, _, err := arrowutils.FieldIDFromStruct(logDT, constants.SpanId)
	if err != nil {
		return nil, err
	}

	severityNumber, _, err := arrowutils.FieldIDFromStruct(logDT, constants.SeverityNumber)
	if err != nil {
		return nil, err
	}

	severityText, _, err := arrowutils.FieldIDFromStruct(logDT, constants.SeverityText)
	if err != nil {
		return nil, err
	}

	body, _, err := arrowutils.FieldIDFromStruct(logDT, constants.Body)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(logDT)
	if err != nil {
		return nil, err
	}

	droppedAttributesCount, _, err := arrowutils.FieldIDFromStruct(logDT, constants.DroppedAttributesCount)
	if err != nil {
		return nil, err
	}

	flags, _, err := arrowutils.FieldIDFromStruct(logDT, constants.Flags)
	if err != nil {
		return nil, err
	}

	return &LogRecordIds{
		Id:                   id,
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

func AppendLogRecordInto(logs plog.LogRecordSlice, los *arrowutils.ListOfStructs, row int, ids *LogRecordIds) error {
	logRecord := logs.AppendEmpty()

	timeUnixNano, err := los.TimestampFieldByID(ids.TimeUnixNano, row)
	if err != nil {
		return err
	}
	observedTimeUnixNano, err := los.TimestampFieldByID(ids.ObservedTimeUnixNano, row)
	if err != nil {
		return err
	}

	traceID, err := los.FixedSizeBinaryFieldByID(ids.TraceID, row)
	if err != nil {
		return err
	}
	if len(traceID) != 16 {
		return fmt.Errorf("trace_id field should be 16 bytes")
	}
	spanID, err := los.FixedSizeBinaryFieldByID(ids.SpanID, row)
	if err != nil {
		return err
	}
	if len(spanID) != 8 {
		return fmt.Errorf("span_id field should be 8 bytes")
	}

	severityNumber, err := los.I32FieldByID(ids.SeverityNumber, row)
	if err != nil {
		return err
	}
	severityText, err := los.StringFieldByID(ids.SeverityText, row)
	if err != nil {
		return err
	}

	body := los.FieldByID(ids.Body)
	if anyValueArr, ok := body.(*array.SparseUnion); ok {
		if err := otlp.UpdateValueFrom(logRecord.Body(), anyValueArr, row); err != nil {
			return err
		}
	} else {
		return fmt.Errorf("body field should be a sparse union")
	}

	err = otlp.AppendAttributesInto(logRecord.Attributes(), los.Array(), row, ids.Attributes)
	if err != nil {
		return err
	}
	droppedAttributesCount, err := los.U32FieldByID(ids.DropAttributesCount, row)
	if err != nil {
		return err
	}

	flags, err := los.U32FieldByID(ids.Flags, row)
	if err != nil {
		return err
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
