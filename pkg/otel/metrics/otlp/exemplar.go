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
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ExemplarIds struct {
	Id           int
	Attributes   *otlp.AttributeIds
	TimeUnixNano int
	SpanID       int
	TraceID      int
	ValueID      int
}

func NewExemplarIds(ndp *arrow.StructType) (*ExemplarIds, error) {
	id, exemplarDT, err := arrowutils.ListOfStructsFieldIDFromStruct(ndp, constants.Exemplars)
	if err != nil {
		return nil, err
	}

	attributesId, err := otlp.NewAttributeIds(exemplarDT)
	if err != nil {
		return nil, err
	}

	timeUnixNanoId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.TimeUnixNano)
	spanIdId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.SpanId)
	traceIdId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.TraceId)
	valueId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.MetricValue)

	return &ExemplarIds{
		Id:           id,
		Attributes:   attributesId,
		TimeUnixNano: timeUnixNanoId,
		SpanID:       spanIdId,
		TraceID:      traceIdId,
		ValueID:      valueId,
	}, nil
}

func AppendExemplarsInto(exemplarSlice pmetric.ExemplarSlice, ndp *arrowutils.ListOfStructs, ndpIdx int, ids *ExemplarIds) error {
	exemplars, err := ndp.ListOfStructsById(ndpIdx, ids.Id)
	if err != nil {
		return fmt.Errorf("AppendExemplarsInto(field='examplars')->%w", err)
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
			return fmt.Errorf("AppendExemplarsInto->%w", err)
		}

		timeUnixNano, err := exemplars.TimestampFieldByID(ids.TimeUnixNano, exemplarIdx)
		if err != nil {
			return err
		}
		exemplar.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		spanId, err := exemplars.FixedSizeBinaryFieldByID(ids.SpanID, exemplarIdx)
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

		traceId, err := exemplars.FixedSizeBinaryFieldByID(ids.TraceID, exemplarIdx)
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

		value := exemplars.FieldByID(ids.ValueID)
		if valueArr, ok := value.(*array.SparseUnion); ok {
			if err := UpdateValueFromExemplar(exemplar, valueArr, exemplarIdx); err != nil {
				return err
			}
		} else {
			return fmt.Errorf("value field shound be a SparseUnion")
		}
	}

	return nil
}
