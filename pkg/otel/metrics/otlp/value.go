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
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	arrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

func UpdateValueFromExemplar(v pmetric.Exemplar, vArr *array.SparseUnion, row int) error {
	tcode := vArr.TypeCode(row)
	fieldId := vArr.ChildID(row)

	switch tcode {
	case arrow.I64Code:
		val, err := arrowutils.I64FromArray(vArr.Field(fieldId), row)
		if err != nil {
			return werror.Wrap(err)
		}
		v.SetIntValue(val)
	case arrow.F64Code:
		val, err := arrowutils.F64FromArray(vArr.Field(fieldId), row)
		if err != nil {
			return werror.Wrap(err)
		}
		v.SetDoubleValue(val)
	default:
		return werror.WrapWithContext(ErrUnknownTypeCode, map[string]interface{}{"typeCode": tcode, "row": row})
	}
	return nil
}

func UpdateValueFromNumberDataPoint(v pmetric.NumberDataPoint, vArr *array.SparseUnion, row int) error {
	tcode := vArr.TypeCode(row)
	fieldId := vArr.ChildID(row)

	switch tcode {
	case arrow.I64Code:
		val, err := arrowutils.I64FromArray(vArr.Field(fieldId), row)
		if err != nil {
			return werror.Wrap(err)
		}
		v.SetIntValue(val)
	case arrow.F64Code:
		val, err := arrowutils.F64FromArray(vArr.Field(fieldId), row)
		if err != nil {
			return werror.Wrap(err)
		}
		v.SetDoubleValue(val)
	default:
		return werror.WrapWithContext(ErrUnknownTypeCode, map[string]interface{}{"typeCode": tcode, "row": row})
	}
	return nil
}
