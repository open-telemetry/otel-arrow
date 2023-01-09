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
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type UnivariateNdpIds struct {
	Id                int
	Attributes        *otlp.AttributeIds
	StartTimeUnixNano int
	TimeUnixNano      int
	MetricValue       int
	Exemplars         *ExemplarIds
	Flags             int
}

func NewUnivariateNdpIds(parentDT *arrow.StructType) (*UnivariateNdpIds, error) {
	id, univariateNdpDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.DataPoints)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(univariateNdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoId, found := univariateNdpDT.FieldIdx(constants.StartTimeUnixNano)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.StartTimeUnixNano)
	}

	timeUnixNanoId, found := univariateNdpDT.FieldIdx(constants.TimeUnixNano)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.TimeUnixNano)
	}

	metricValueId, found := univariateNdpDT.FieldIdx(constants.MetricValue)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.MetricValue)
	}

	exemplars, err := NewExemplarIds(univariateNdpDT)
	if err != nil {
		return nil, err
	}

	flagsId, found := univariateNdpDT.FieldIdx(constants.Flags)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.Flags)
	}

	return &UnivariateNdpIds{
		Id:                id,
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNanoId,
		TimeUnixNano:      timeUnixNanoId,
		MetricValue:       metricValueId,
		Exemplars:         exemplars,
		Flags:             flagsId,
	}, nil
}

func AppendUnivariateNdpInto(ndpSlice pmetric.NumberDataPointSlice, ndp *arrowutils.ListOfStructs, ids *UnivariateNdpIds, smdata *SharedData, mdata *SharedData) error {
	if ndp == nil {
		return nil
	}

	for ndpIdx := ndp.Start(); ndpIdx < ndp.End(); ndpIdx++ {
		ndpValue := ndpSlice.AppendEmpty()

		if ndp.IsNull(ndpIdx) {
			continue
		}

		attrs := ndpValue.Attributes()
		if err := otlp.AppendAttributesInto(attrs, ndp.Array(), ndpIdx, ids.Attributes); err != nil {
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
			ndpValue.SetStartTimestamp(*smdata.StartTime)
		} else {
			if mdata.StartTime != nil {
				ndpValue.SetStartTimestamp(*mdata.StartTime)
			} else {
				startTimeUnixNano, err := ndp.TimestampFieldByID(ids.StartTimeUnixNano, ndpIdx)
				if err != nil {
					return err
				}
				ndpValue.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
			}
		}
		if smdata.Time != nil {
			ndpValue.SetTimestamp(*smdata.Time)
		} else {
			if mdata.Time != nil {
				ndpValue.SetTimestamp(*mdata.Time)
			} else {
				timeUnixNano, err := ndp.TimestampFieldByID(ids.TimeUnixNano, ndpIdx)
				if err != nil {
					return err
				}
				ndpValue.SetTimestamp(pcommon.Timestamp(timeUnixNano))
			}
		}

		value := ndp.FieldByID(ids.MetricValue)
		if valueArr, ok := value.(*array.SparseUnion); ok {
			if err := UpdateValueFromNumberDataPoint(ndpValue, valueArr, ndpIdx); err != nil {
				return err
			}
		} else {
			return fmt.Errorf("value field shound be a SparseUnion")
		}

		flags, err := ndp.U32FieldByID(ids.Flags, ndpIdx)
		if err != nil {
			return err
		}
		ndpValue.SetFlags(pmetric.DataPointFlags(flags))

		exemplars, err := ndp.ListOfStructsById(ndpIdx, ids.Exemplars.Id)
		if exemplars != nil && err == nil {
			if err := AppendExemplarsInto(ndpValue.Exemplars(), exemplars, ndpIdx, ids.Exemplars); err != nil {
				return err
			}
		} else if err != nil {
			return err
		}
	}

	return nil
}
