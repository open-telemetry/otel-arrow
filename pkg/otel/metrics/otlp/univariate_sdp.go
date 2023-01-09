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
	id, sdpDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.DataPoints)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(sdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoId, found := sdpDT.FieldIdx(constants.StartTimeUnixNano)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.StartTimeUnixNano)
	}

	timeUnixNanoId, found := sdpDT.FieldIdx(constants.TimeUnixNano)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.TimeUnixNano)
	}

	countId, found := sdpDT.FieldIdx(constants.SummaryCount)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.SummaryCount)
	}

	sumId, found := sdpDT.FieldIdx(constants.SummarySum)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.SummarySum)
	}

	quantileValues, err := NewQuantileValueIds(sdpDT)
	if err != nil {
		return nil, err
	}

	flagsId, found := sdpDT.FieldIdx(constants.Flags)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.Flags)
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

func AppendUnivariateSdpInto(ndpSlice pmetric.SummaryDataPointSlice, ndp *arrowutils.ListOfStructs, ids *UnivariateSdpIds, smdata *SharedData, mdata *SharedData) error {
	if ndp == nil {
		return nil
	}

	for idx := ndp.Start(); idx < ndp.End(); idx++ {
		sdpValue := ndpSlice.AppendEmpty()

		if ndp.IsNull(idx) {
			continue
		}

		attrs := sdpValue.Attributes()
		if err := otlp.AppendAttributesInto(attrs, ndp.Array(), idx, ids.Attributes); err != nil {
			return err
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
			sdpValue.SetStartTimestamp(*smdata.StartTime)
		} else {
			if mdata.StartTime != nil {
				sdpValue.SetStartTimestamp(*mdata.StartTime)
			} else {
				startTimeUnixNano, err := ndp.TimestampFieldByID(ids.StartTimeUnixNano, idx)
				if err != nil {
					return err
				}
				sdpValue.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
			}
		}

		if smdata.Time != nil {
			sdpValue.SetTimestamp(*smdata.Time)
		} else {
			if mdata.StartTime != nil {
				sdpValue.SetTimestamp(*mdata.Time)
			} else {
				timeUnixNano, err := ndp.TimestampFieldByID(ids.TimeUnixNano, idx)
				if err != nil {
					return err
				}
				sdpValue.SetTimestamp(pcommon.Timestamp(timeUnixNano))
			}
		}

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
