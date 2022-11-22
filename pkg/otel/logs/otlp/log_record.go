package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type LogRecordIds struct {
	Id                   int
	TimeUnixNano         int
	ObservedTimeUnixNano int
	TraceId              int
	SpanId               int
	SeverityNumber       int
	SeverityText         int
	Body                 int
	Attributes           *otlp.AttributeIds
	DropAttributesCount  int
	Flags                int
}

func NewLogRecordIds(scopeLogsDT *arrow.StructType) (*LogRecordIds, error) {
	id, logDT, err := arrow_utils.ListOfStructsFieldIDFromStruct(scopeLogsDT, constants.LOGS)
	if err != nil {
		return nil, err
	}

	timeUnixNano, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}

	observedTimeUnixNano, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.OBSERVED_TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}

	traceId, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.TRACE_ID)
	if err != nil {
		return nil, err
	}

	spanId, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.SPAN_ID)
	if err != nil {
		return nil, err
	}

	severityNumber, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.SEVERITY_NUMBER)
	if err != nil {
		return nil, err
	}

	severityText, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.SEVERITY_TEXT)
	if err != nil {
		return nil, err
	}

	body, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.BODY)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(logDT)
	if err != nil {
		return nil, err
	}

	droppedAttributesCount, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}

	flags, _, err := arrow_utils.FieldIDFromStruct(logDT, constants.FLAGS)
	if err != nil {
		return nil, err
	}

	return &LogRecordIds{
		Id:                   id,
		TimeUnixNano:         timeUnixNano,
		ObservedTimeUnixNano: observedTimeUnixNano,
		TraceId:              traceId,
		SpanId:               spanId,
		SeverityNumber:       severityNumber,
		SeverityText:         severityText,
		Body:                 body,
		Attributes:           attributes,
		DropAttributesCount:  droppedAttributesCount,
		Flags:                flags,
	}, nil
}

func AppendLogRecordInto(logs plog.LogRecordSlice, los *arrow_utils.ListOfStructs, row int, ids *LogRecordIds) error {
	logRecord := logs.AppendEmpty()

	timeUnixNano, err := los.U64FieldByID(ids.TimeUnixNano, row)
	if err != nil {
		return err
	}
	observedTimeUnixNano, err := los.U64FieldByID(ids.ObservedTimeUnixNano, row)
	if err != nil {
		return err
	}

	traceId, err := los.FixedSizeBinaryFieldByID(ids.TraceId, row)
	if err != nil {
		return err
	}
	if len(traceId) != 16 {
		return fmt.Errorf("trace_id field should be 16 bytes")
	}
	spanId, err := los.FixedSizeBinaryFieldByID(ids.SpanId, row)
	if err != nil {
		return err
	}
	if len(spanId) != 8 {
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
	copy(tid[:], traceId)
	copy(sid[:], spanId)

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
