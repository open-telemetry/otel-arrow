package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type UnivariateSdpIds struct {
	Id                int
	Attributes        *otlp.AttributeIds
	StartTimeUnixNano int
	TimeUnixNano      int
	Count             int
	Sum               int
	QuantileValues    *QuantileValueIds
	Flags             int
}

func NewUnivariateSdpIds(parentDT *arrow.StructType) (*UnivariateSdpIds, error) {
	id, sdpDT, err := arrow_utils.ListOfStructsFieldIDFromStruct(parentDT, constants.DATA_POINTS)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(sdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoId, found := sdpDT.FieldIdx(constants.START_TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.START_TIME_UNIX_NANO)
	}

	timeUnixNanoId, found := sdpDT.FieldIdx(constants.TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.TIME_UNIX_NANO)
	}

	countId, found := sdpDT.FieldIdx(constants.SUMMARY_COUNT)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.SUMMARY_COUNT)
	}

	sumId, found := sdpDT.FieldIdx(constants.SUMMARY_SUM)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.SUMMARY_SUM)
	}

	quantileValues, err := NewQuantileValueIds(sdpDT)
	if err != nil {
		return nil, err
	}

	flagsId, found := sdpDT.FieldIdx(constants.FLAGS)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.FLAGS)
	}

	return &UnivariateSdpIds{
		Id:                id,
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNanoId,
		TimeUnixNano:      timeUnixNanoId,
		Count:             countId,
		Sum:               sumId,
		QuantileValues:    quantileValues,
		Flags:             flagsId,
	}, nil
}

func AppendUnivariateSdpInto(ndpSlice pmetric.SummaryDataPointSlice, ndp *arrow_utils.ListOfStructs, ids *UnivariateSdpIds) error {
	if ndp == nil {
		return nil
	}

	for idx := ndp.Start(); idx < ndp.End(); idx++ {
		sdpValue := ndpSlice.AppendEmpty()

		if ndp.IsNull(idx) {
			continue
		}

		if err := otlp.AppendAttributesInto(sdpValue.Attributes(), ndp.Array(), idx, ids.Attributes); err != nil {
			return err
		}

		startTimeUnixNano, err := ndp.U64FieldByID(ids.StartTimeUnixNano, idx)
		if err != nil {
			return err
		}
		sdpValue.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := ndp.U64FieldByID(ids.TimeUnixNano, idx)
		if err != nil {
			return err
		}
		sdpValue.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		count, err := ndp.U64FieldByID(ids.Count, idx)
		if err != nil {
			return err
		}
		sdpValue.SetCount(count)

		sum, err := ndp.F64FieldByID(ids.Sum, idx)
		if err != nil {
			return err
		}
		sdpValue.SetSum(sum)

		qValues, err := ndp.ListOfStructsById(idx, ids.QuantileValues.Id)
		if err != nil {
			return err
		}
		err = AppendQuantileValuesInto(sdpValue.QuantileValues(), qValues, idx, ids.QuantileValues)
		if err != nil {
			return err
		}

		flags, err := ndp.U32FieldByID(ids.Flags, idx)
		if err != nil {
			return err
		}
		sdpValue.SetFlags(pmetric.DataPointFlags(flags))
	}

	return nil
}
