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

// UnivariateEHistogramDataPointIds is a struct containing the field ids for the
// fields of ExponentialHistogramDataPoint.
type UnivariateEHistogramDataPointIds struct {
	Id                int
	Attributes        *otlp.AttributeIds
	StartTimeUnixNano int
	TimeUnixNano      int
	Count             int
	Sum               int
	Scale             int
	ZeroCount         int
	Positive          *EHistogramDataPointBucketsIds
	Negative          *EHistogramDataPointBucketsIds
	Exemplars         *ExemplarIds
	Flags             int
	Min               int
	Max               int
}

// NewUnivariateEHistogramDataPointIds returns a new UnivariateEHistogramDataPointIds struct.
func NewUnivariateEHistogramDataPointIds(parentDT *arrow.StructType) (*UnivariateEHistogramDataPointIds, error) {
	id, ehdpDT, err := arrow_utils.ListOfStructsFieldIdFromStruct(parentDT, constants.DATA_POINTS)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(ehdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoId, found := ehdpDT.FieldIdx(constants.START_TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.START_TIME_UNIX_NANO)
	}

	timeUnixNanoId, found := ehdpDT.FieldIdx(constants.TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.TIME_UNIX_NANO)
	}

	countId, found := ehdpDT.FieldIdx(constants.HISTOGRAM_COUNT)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_COUNT)
	}

	sumId, found := ehdpDT.FieldIdx(constants.HISTOGRAM_SUM)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_SUM)
	}

	scaleId, found := ehdpDT.FieldIdx(constants.EXP_HISTOGRAM_SCALE)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.EXP_HISTOGRAM_SCALE)
	}

	zeroCountId, found := ehdpDT.FieldIdx(constants.EXP_HISTOGRAM_ZERO_COUNT)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.EXP_HISTOGRAM_ZERO_COUNT)
	}

	positiveId, positiveDT, err := arrow_utils.StructFieldIdFromStruct(ehdpDT, constants.EXP_HISTOGRAM_POSITIVE)
	if err != nil {
		return nil, err
	}
	positive, err := NewEHistogramDataPointBucketsIds(positiveId, positiveDT)
	if err != nil {
		return nil, err
	}

	negativeId, negativeDT, err := arrow_utils.StructFieldIdFromStruct(ehdpDT, constants.EXP_HISTOGRAM_NEGATIVE)
	if err != nil {
		return nil, err
	}
	negative, err := NewEHistogramDataPointBucketsIds(negativeId, negativeDT)
	if err != nil {
		return nil, err
	}

	exemplars, err := NewExemplarIds(ehdpDT)
	if err != nil {
		return nil, err
	}

	flagsId, found := ehdpDT.FieldIdx(constants.FLAGS)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.FLAGS)
	}

	minId, found := ehdpDT.FieldIdx(constants.HISTOGRAM_MIN)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_MIN)
	}

	maxId, found := ehdpDT.FieldIdx(constants.HISTOGRAM_MAX)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_MAX)
	}

	return &UnivariateEHistogramDataPointIds{
		Id:                id,
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNanoId,
		TimeUnixNano:      timeUnixNanoId,
		Count:             countId,
		Sum:               sumId,
		Scale:             scaleId,
		ZeroCount:         zeroCountId,
		Positive:          positive,
		Negative:          negative,
		Exemplars:         exemplars,
		Flags:             flagsId,
		Min:               minId,
		Max:               maxId,
	}, nil
}

// AppendUnivariateEHistogramDataPointInto appends exponential histogram data points into the
// given slice of ExponentialHistogramDataPoint decoded from the ehdp array.
func AppendUnivariateEHistogramDataPointInto(ehdpSlice pmetric.ExponentialHistogramDataPointSlice, ehdp *arrow_utils.ListOfStructs, ids *UnivariateEHistogramDataPointIds) error {
	if ehdp == nil {
		return nil
	}

	for ehdpIdx := ehdp.Start(); ehdpIdx < ehdp.End(); ehdpIdx++ {
		ehdpVal := ehdpSlice.AppendEmpty()

		if ehdp.IsNull(ehdpIdx) {
			continue
		}

		if err := otlp.AppendAttributesInto(ehdpVal.Attributes(), ehdp.Array(), ehdpIdx, ids.Attributes); err != nil {
			return err
		}

		startTimeUnixNano, err := ehdp.U64FieldById(ids.StartTimeUnixNano, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := ehdp.U64FieldById(ids.TimeUnixNano, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		count, err := ehdp.U64FieldById(ids.Count, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetCount(count)

		sum, err := ehdp.F64FieldById(ids.Sum, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetSum(sum)

		scale, err := ehdp.I32FieldById(ids.Scale, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetScale(scale)

		zeroCount, err := ehdp.U64FieldById(ids.ZeroCount, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetZeroCount(zeroCount)

		_, positive, err := ehdp.StructById(ids.Positive.Id, ehdpIdx)
		if err != nil {
			return err
		}
		if positive != nil {
			if err := AppendUnivariateEHistogramDataPointBucketsInto(ehdpVal.Positive(), positive, ids.Positive, ehdpIdx); err != nil {
				return err
			}
		}

		_, negative, err := ehdp.StructById(ids.Negative.Id, ehdpIdx)
		if err != nil {
			return err
		}
		if negative != nil {
			if err := AppendUnivariateEHistogramDataPointBucketsInto(ehdpVal.Negative(), negative, ids.Negative, ehdpIdx); err != nil {
				return err
			}
		}

		exemplars, err := ehdp.ListOfStructsById(ehdpIdx, ids.Exemplars.Id)
		if exemplars != nil && err == nil {
			if err := AppendExemplarsInto(ehdpVal.Exemplars(), exemplars, ehdpIdx, ids.Exemplars); err != nil {
				return err
			}
		} else if err != nil {
			return err
		}

		flags, err := ehdp.U32FieldById(ids.Flags, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetFlags(pmetric.DataPointFlags(flags))

		min, err := ehdp.F64FieldById(ids.Min, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetMin(min)

		max, err := ehdp.F64FieldById(ids.Max, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetMax(max)
	}

	return nil
}
