package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type LinkIds struct {
	Id                     int
	TraceId                int
	SpanId                 int
	TraceState             int
	Attributes             *otlp.AttributeIds
	DroppedAttributesCount int
}

func NewLinkIds(spanDT *arrow.StructType) (*LinkIds, error) {
	id, linkDT, err := arrow_utils.ListOfStructsFieldIdFromStruct(spanDT, constants.SPAN_LINKS)
	if err != nil {
		return nil, err
	}

	traceId, _, err := arrow_utils.FieldIdFromStruct(linkDT, constants.TRACE_ID)
	if err != nil {
		return nil, err
	}

	spanId, _, err := arrow_utils.FieldIdFromStruct(linkDT, constants.SPAN_ID)
	if err != nil {
		return nil, err
	}

	traceState, _, err := arrow_utils.FieldIdFromStruct(linkDT, constants.TRACE_STATE)
	if err != nil {
		return nil, err
	}

	attributeIds, err := otlp.NewAttributeIds(linkDT)
	if err != nil {
		return nil, err
	}

	droppedAttributesCount, _, err := arrow_utils.FieldIdFromStruct(linkDT, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}

	return &LinkIds{
		Id:                     id,
		TraceId:                traceId,
		SpanId:                 spanId,
		TraceState:             traceState,
		Attributes:             attributeIds,
		DroppedAttributesCount: droppedAttributesCount,
	}, nil
}

// AppendLinksInto initializes a Span's Links from an Arrow representation.
func AppendLinksInto(result ptrace.SpanLinkSlice, los *arrow_utils.ListOfStructs, row int, ids *LinkIds) error {
	linkLos, err := los.ListOfStructsById(row, ids.Id)

	if err != nil {
		return err
	}
	if linkLos == nil {
		// No links found
		return nil
	}

	for linkIdx := linkLos.Start(); linkIdx < linkLos.End(); linkIdx++ {
		link := result.AppendEmpty()

		if linkLos.IsNull(linkIdx) {
			continue
		}

		traceId, err := linkLos.FixedSizeBinaryFieldById(ids.TraceId, linkIdx)
		if err != nil {
			return err
		}
		if len(traceId) == 16 {
			var tid pcommon.TraceID
			copy(tid[:], traceId)
			link.SetTraceID(tid)
		} else {
			return fmt.Errorf("invalid TraceID len")
		}

		spanId, err := linkLos.FixedSizeBinaryFieldById(ids.SpanId, linkIdx)
		if err != nil {
			return err
		}
		if len(spanId) == 8 {
			var sid pcommon.SpanID
			copy(sid[:], spanId)
			link.SetSpanID(sid)
		} else {
			return fmt.Errorf("invalid SpanID len")
		}

		traceState, err := linkLos.StringFieldById(ids.TraceState, linkIdx)
		if err != nil {
			return err
		}
		link.TraceState().FromRaw(traceState)

		if err = otlp.AppendAttributesInto(link.Attributes(), linkLos.Array(), linkIdx, ids.Attributes); err != nil {
			return err
		}
		dac, err := linkLos.U32FieldById(ids.DroppedAttributesCount, linkIdx)
		if err != nil {
			return err
		}
		link.SetDroppedAttributesCount(dac)
	}
	return nil
}
