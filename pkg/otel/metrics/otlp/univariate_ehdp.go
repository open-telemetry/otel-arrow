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
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
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
	id, ehdpDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.DataPoints)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(ehdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.StartTimeUnixNano)
	timeUnixNanoID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.TimeUnixNano)
	countID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.HistogramCount)
	sumID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.HistogramSum)
	scaleID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.ExpHistogramScale)
	zeroCountID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.ExpHistogramZeroCount)

	positiveID, positiveDT, err := arrowutils.StructFieldIDFromStruct(ehdpDT, constants.ExpHistogramPositive)
	if err != nil {
		return nil, err
	}
	positive, err := NewEHistogramDataPointBucketsIds(positiveID, positiveDT)
	if err != nil {
		return nil, err
	}

	negativeID, negativeDT, err := arrowutils.StructFieldIDFromStruct(ehdpDT, constants.ExpHistogramNegative)
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

	flagsID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.Flags)
	minID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.HistogramMin)
	maxID, _ := arrowutils.FieldIDFromStruct(ehdpDT, constants.HistogramMax)

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
func AppendUnivariateEHistogramDataPointInto(ehdpSlice pmetric.ExponentialHistogramDataPointSlice, ehdp *arrowutils.ListOfStructs, ids *UnivariateEHistogramDataPointIds, smdata *SharedData, mdata *SharedData) error {
	if ehdp == nil {
		return nil
	}

	for ehdpIdx := ehdp.Start(); ehdpIdx < ehdp.End(); ehdpIdx++ {
		ehdpVal := ehdpSlice.AppendEmpty()

		if ehdp.IsNull(ehdpIdx) {
			continue
		}

		attrs := ehdpVal.Attributes()
		if err := otlp.AppendAttributesInto(attrs, ehdp.Array(), ehdpIdx, ids.Attributes); err != nil {
			return fmt.Errorf("AppendUnivariateEHistogramDataPointInto->%w", err)
		}
		smdata.Attributes.Range(func(k string, v pcommon.Value) bool {
			v.CopyTo(attrs.PutEmpty(k))
			return true
		})
		mdata.Attributes.Range(func(k string, v pcommon.Value) bool {
			v.CopyTo(attrs.PutEmpty(k))
			return true
		})

		if smdata.StartTime != nil {
			ehdpVal.SetStartTimestamp(*smdata.StartTime)
		} else {
			if mdata.StartTime != nil {
				ehdpVal.SetStartTimestamp(*mdata.StartTime)
			} else {
				startTimeUnixNano, err := ehdp.TimestampFieldByID(ids.StartTimeUnixNano, ehdpIdx)
				if err != nil {
					return err
				}
				ehdpVal.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
			}
		}

		if smdata.Time != nil {
			ehdpVal.SetTimestamp(*smdata.Time)
		} else {
			if mdata.Time != nil {
				ehdpVal.SetTimestamp(*mdata.Time)
			} else {
				timeUnixNano, err := ehdp.TimestampFieldByID(ids.TimeUnixNano, ehdpIdx)
				if err != nil {
					return err
				}
				ehdpVal.SetTimestamp(pcommon.Timestamp(timeUnixNano))
			}
		}

		err := AppendCountSumInto(ehdp, ids, ehdpIdx, ehdpVal)
		if err != nil {
			return err
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
			return fmt.Errorf("AppendUnivariateEHistogramDataPointInto(field='exemplars')->%w", err)
		}

		flags, err := ehdp.U32FieldByID(ids.Flags, ehdpIdx)
		if err != nil {
			return err
		}
		ehdpVal.SetFlags(pmetric.DataPointFlags(flags))

		err = AppendMinMaxInto(ehdp, ids, ehdpIdx, ehdpVal)
		if err != nil {
			return err
		}
	}

	return nil
}

func AppendMinMaxInto(ehdp *arrowutils.ListOfStructs, ids *UnivariateEHistogramDataPointIds, ehdpIdx int, ehdpVal pmetric.ExponentialHistogramDataPoint) error {
	min, err := ehdp.F64OrNilFieldByID(ids.Min, ehdpIdx)
	if err != nil {
		return fmt.Errorf("AppendMinMaxInto(field='min')->%w", err)
	}
	if min != nil {
		ehdpVal.SetMin(*min)
	}

	max, err := ehdp.F64OrNilFieldByID(ids.Max, ehdpIdx)
	if err != nil {
		return fmt.Errorf("AppendMinMaxInto(field='max')->%w", err)
	}
	if max != nil {
		ehdpVal.SetMax(*max)
	}
	return nil
}

func AppendCountSumInto(ehdp *arrowutils.ListOfStructs, ids *UnivariateEHistogramDataPointIds, ehdpIdx int, ehdpVal pmetric.ExponentialHistogramDataPoint) error {
	count, err := ehdp.U64FieldByID(ids.Count, ehdpIdx)
	if err != nil {
		return err
	}
	ehdpVal.SetCount(count)

	sum, err := ehdp.F64OrNilFieldByID(ids.Sum, ehdpIdx)
	if err != nil {
		return fmt.Errorf("AppendCountSumInto(field='sum')->%w", err)
	}
	if sum != nil {
		ehdpVal.SetSum(*sum)
	}
	return err
}
