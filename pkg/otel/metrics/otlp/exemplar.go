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
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type ExemplarIds struct {
	ID           int
	Attributes   *otlp.AttributeIds
	TimeUnixNano int
	SpanID       int
	TraceID      int
	ValueID      int
}

func NewExemplarIds(schema *arrow.Schema) (*ExemplarIds, error) {
	id, exemplarDT, err := arrowutils.ListOfStructsFieldIDFromSchema(schema, constants.Exemplars)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attributesId, err := otlp.NewAttributeIds(exemplarDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	timeUnixNanoId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.TimeUnixNano)
	spanIdId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.SpanId)
	traceIdId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.TraceId)
	valueId, _ := arrowutils.FieldIDFromStruct(exemplarDT, constants.MetricValue)

	return &ExemplarIds{
		ID:           id,
		Attributes:   attributesId,
		TimeUnixNano: timeUnixNanoId,
		SpanID:       spanIdId,
		TraceID:      traceIdId,
		ValueID:      valueId,
	}, nil
}

func AppendExemplarsInto(exemplarSlice pmetric.ExemplarSlice, record arrow.Record, ndpIdx int, ids *ExemplarIds) error {
	if ids.ID == -1 {
		// No exemplars
		return nil
	}

	exemplars, err := arrowutils.ListOfStructsFromRecord(record, ids.ID, ndpIdx)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"ndpIdx": ndpIdx})
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
			return werror.WrapWithContext(err, map[string]interface{}{"ndpIdx": ndpIdx})
		}

		timeUnixNano, err := exemplars.TimestampFieldByID(ids.TimeUnixNano, exemplarIdx)
		if err != nil {
			return werror.Wrap(err)
		}
		exemplar.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		spanId, err := exemplars.FixedSizeBinaryFieldByID(ids.SpanID, exemplarIdx)
		if err != nil {
			return werror.Wrap(err)
		}

		if len(spanId) == 8 {
			var sid pcommon.SpanID

			copy(sid[:], spanId)
			exemplar.SetSpanID(sid)
		} else {
			return werror.WrapWithContext(common.ErrInvalidSpanIDLength, map[string]interface{}{"spanID": spanId})
		}

		traceID, err := exemplars.FixedSizeBinaryFieldByID(ids.TraceID, exemplarIdx)
		if err != nil {
			return werror.Wrap(err)
		}

		if len(traceID) == 16 {
			var tid pcommon.TraceID

			copy(tid[:], traceID)
			exemplar.SetTraceID(tid)
		} else {
			return werror.WrapWithContext(common.ErrInvalidTraceIDLength, map[string]interface{}{"traceID": traceID})
		}

		value := exemplars.FieldByID(ids.ValueID)
		if valueArr, ok := value.(*array.SparseUnion); ok {
			if err := UpdateValueFromExemplar(exemplar, valueArr, exemplarIdx); err != nil {
				return werror.Wrap(err)
			}
		} else {
			return werror.Wrap(ErrNotArraySparseUnion)
		}
	}

	return nil
}
