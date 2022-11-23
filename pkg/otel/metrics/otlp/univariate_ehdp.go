package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
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
	id, ehdpDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.DATA_POINTS)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(ehdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoID, found := ehdpDT.FieldIdx(constants.START_TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.START_TIME_UNIX_NANO)
	}

	timeUnixNanoID, found := ehdpDT.FieldIdx(constants.TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.TIME_UNIX_NANO)
	}

	countID, found := ehdpDT.FieldIdx(constants.HISTOGRAM_COUNT)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_COUNT)
	}

	sumID, found := ehdpDT.FieldIdx(constants.HISTOGRAM_SUM)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_SUM)
	}

	scaleID, found := ehdpDT.FieldIdx(constants.EXP_HISTOGRAM_SCALE)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.EXP_HISTOGRAM_SCALE)
	}

	zeroCountID, found := ehdpDT.FieldIdx(constants.EXP_HISTOGRAM_ZERO_COUNT)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.EXP_HISTOGRAM_ZERO_COUNT)
	}

	positiveID, positiveDT, err := arrowutils.StructFieldIDFromStruct(ehdpDT, constants.EXP_HISTOGRAM_POSITIVE)
	if err != nil {
		return nil, err
	}
	positive, err := NewEHistogramDataPointBucketsIds(positiveID, positiveDT)
	if err != nil {
		return nil, err
	}

	negativeID, negativeDT, err := arrowutils.StructFieldIDFromStruct(ehdpDT, constants.EXP_HISTOGRAM_NEGATIVE)
	if err != nil {
		return nil, err
	}
	negative, err := NewEHistogramDataPointBucketsIds(negativeID, negativeDT)
	if err != nil {
		return nil, err
	}

	exemplars, err := NewExemplarIds(ehdpDT)
	if err != nil {
		return nil, err
	}

	flagsID, found := ehdpDT.FieldIdx(constants.FLAGS)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.FLAGS)
	}

	minID, found := ehdpDT.FieldIdx(constants.HISTOGRAM_MIN)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_MIN)
	}

	maxID, found := ehdpDT.FieldIdx(constants.HISTOGRAM_MAX)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.HISTOGRAM_MAX)
	}

	return &UnivariateEHistogramDataPointIds{
		Id:                id,
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNanoID,
		TimeUnixNano:      timeUnixNanoID,
		Count:             countID,
		Sum:               sumID,
		Scale:             scaleID,
		ZeroCount:         zeroCountID,
		Positive:          positive,
		Negative:          negative,
		Exemplars:         exemplars,
		Flags:             flagsID,
		Min:               minID,
		Max:               maxID,
	}, nil
}

// AppendUnivariateEHistogramDataPointInto appends exponential histogram data points into the
// given slice of ExponentialHistogramDataPoint decoded from the ehdp array.
func AppendUnivariateEHistogramDataPointInto(ehdpSlice pmetric.ExponentialHistogramDataPointSlice, ehdp *arrowutils.ListOfStructs, ids *UnivariateEHistogramDataPointIds) error {
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

		startTimeUnixNano, err := ehdp.U64FieldByID(ids.StartTimeUnixNano, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := ehdp.U64FieldByID(ids.TimeUnixNano, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		count, err := ehdp.U64FieldByID(ids.Count, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetCount(count)

		sum, err := ehdp.F64OrNilFieldByID(ids.Sum, ehdpIdx)
		if err != nil {
			return err
		}
		if sum != nil {
			ehdpVal.SetSum(*sum)
		}

		scale, err := ehdp.I32FieldByID(ids.Scale, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetScale(scale)

		zeroCount, err := ehdp.U64FieldByID(ids.ZeroCount, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetZeroCount(zeroCount)

		_, positive, err := ehdp.StructByID(ids.Positive.Id, ehdpIdx)
		if err != nil {
			return err
		}
		if positive != nil {
			if err := AppendUnivariateEHistogramDataPointBucketsInto(ehdpVal.Positive(), positive, ids.Positive, ehdpIdx); err != nil {
				return err
			}
		}

		_, negative, err := ehdp.StructByID(ids.Negative.Id, ehdpIdx)
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

		flags, err := ehdp.U32FieldByID(ids.Flags, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetFlags(pmetric.DataPointFlags(flags))

		min, err := ehdp.F64OrNilFieldByID(ids.Min, ehdpIdx)
		if err != nil {
			return err
		}
		if min != nil {
			ehdpVal.SetMin(*min)
		}

		max, err := ehdp.F64OrNilFieldByID(ids.Max, ehdpIdx)
		if err != nil {
			return err
		}
		if max != nil {
			ehdpVal.SetMax(*max)
		}
	}

	return nil
}
