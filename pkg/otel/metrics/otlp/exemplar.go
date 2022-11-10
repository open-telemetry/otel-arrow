package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ExemplarIds struct {
	Id           int
	Attributes   *otlp.AttributeIds
	TimeUnixNano int
	SpanId       int
	TraceId      int
	ValueId      int
}

func NewExemplarIds(ndp *arrow.StructType) (*ExemplarIds, error) {
	id, exemplarDT, err := arrow_utils.ListOfStructsFieldIdFromStruct(ndp, constants.EXEMPLARS)
	if err != nil {
		return nil, err
	}

	attributesId, err := otlp.NewAttributeIds(exemplarDT)
	if err != nil {
		return nil, err
	}

	timeUnixNanoId, timeUnixNanoFound := exemplarDT.FieldIdx(constants.TIME_UNIX_NANO)
	if !timeUnixNanoFound {
		return nil, fmt.Errorf("field %s not found", constants.TIME_UNIX_NANO)
	}

	spanIdId, spanIdFound := exemplarDT.FieldIdx(constants.SPAN_ID)
	if !spanIdFound {
		return nil, fmt.Errorf("field %s not found", constants.SPAN_ID)
	}

	traceIdId, traceIdFound := exemplarDT.FieldIdx(constants.TRACE_ID)
	if !traceIdFound {
		return nil, fmt.Errorf("field %s not found", constants.TRACE_ID)
	}

	valueId, valueFound := exemplarDT.FieldIdx(constants.METRIC_VALUE)
	if !valueFound {
		return nil, fmt.Errorf("field %s not found", constants.METRIC_VALUE)
	}

	return &ExemplarIds{
		Id:           id,
		Attributes:   attributesId,
		TimeUnixNano: timeUnixNanoId,
		SpanId:       spanIdId,
		TraceId:      traceIdId,
		ValueId:      valueId,
	}, nil
}

func AppendExemplarsInto(exemplarSlice pmetric.ExemplarSlice, ndp *arrow_utils.ListOfStructs, ndpIdx int, ids *ExemplarIds) error {
	exemplars, err := ndp.ListOfStructsById(ndpIdx, ids.Id)
	if err != nil {
		return err
	}
	if exemplars == nil {
		return nil
	}

	for exemplarIdx := exemplars.Start(); exemplarIdx < exemplars.End(); exemplarIdx++ {
		exemplar := exemplarSlice.AppendEmpty()

		if exemplars.IsNull(exemplarIdx) {
			continue
		}

		if err := otlp.AppendAttributesInto(exemplar.FilteredAttributes(), exemplars.Array(), exemplarIdx, ids.Attributes); err != nil {
			return err
		}
		timeUnixNano, err := exemplars.U64FieldById(ids.TimeUnixNano, exemplarIdx)
		if err != nil {
			return err
		}
		exemplar.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		spanId, err := exemplars.FixedSizeBinaryFieldById(ids.SpanId, exemplarIdx)
		if err != nil {
			return err
		}
		if len(spanId) == 8 {
			var sid pcommon.SpanID
			copy(sid[:], spanId)
			exemplar.SetSpanID(sid)
		} else {
			return fmt.Errorf("invalid span id length %d", len(spanId))
		}

		traceId, err := exemplars.FixedSizeBinaryFieldById(ids.TraceId, exemplarIdx)
		if err != nil {
			return err
		}
		if len(traceId) == 16 {
			var tid pcommon.TraceID
			copy(tid[:], traceId)
			exemplar.SetTraceID(tid)
		} else {
			return fmt.Errorf("invalid trace id length %d", len(traceId))
		}

		value := exemplars.FieldById(ids.ValueId)
		if valueArr, ok := value.(*array.DenseUnion); ok {
			if err := UpdateValueFromExemplar(exemplar, valueArr, exemplarIdx); err != nil {
				return err
			}
		} else {
			return fmt.Errorf("value field shound be a DenseUnion")
		}
	}
	return nil
}
