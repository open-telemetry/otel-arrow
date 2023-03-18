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
	"errors"
	"fmt"

	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	commonarrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
)

var errInvalidFieldId = errors.New("Invalid field id")

func UpdateValueFrom(v pcommon.Value, vArr *array.SparseUnion, row int) error {
	tcode := vArr.TypeCode(row)
	fieldId := vArr.ChildID(row)

	switch tcode {
	case commonarrow.StrCode:
		strArr := vArr.Field(fieldId)
		if strArr == nil {
			return fmt.Errorf("UpdateValueFrom(fieldID=%d, strArr): %w", fieldId, errInvalidFieldId)
		}
		val, err := arrowutils.StringFromArray(strArr, row)
		if err != nil {
			return err
		}
		v.SetStr(val)
	case commonarrow.I64Code:
		i64Arr := vArr.Field(fieldId)
		if i64Arr == nil {
			return fmt.Errorf("UpdateValueFrom(fieldID=%d, i64Arr): %w", fieldId, errInvalidFieldId)
		}
		val := i64Arr.(*array.Int64).Value(row)
		v.SetInt(val)
	case commonarrow.F64Code:
		f64Arr := vArr.Field(fieldId)
		if f64Arr == nil {
			return fmt.Errorf("UpdateValueFrom(fieldID=%d, f64Arr): %w", fieldId, errInvalidFieldId)
		}
		val := f64Arr.(*array.Float64).Value(row)
		v.SetDouble(val)
	case commonarrow.BoolCode:
		boolArr := vArr.Field(fieldId)
		if boolArr == nil {
			return fmt.Errorf("UpdateValueFrom(fieldID=%d, boolArr): %w", fieldId, errInvalidFieldId)
		}
		val := boolArr.(*array.Boolean).Value(row)
		v.SetBool(val)
	case commonarrow.BinaryCode:
		binArr := vArr.Field(fieldId)
		if binArr == nil {
			return fmt.Errorf("UpdateValueFrom(fieldID=%d, binArr): %w", fieldId, errInvalidFieldId)
		}
		val, err := arrowutils.BinaryFromArray(binArr, row)
		if err != nil {
			return err
		}
		v.SetEmptyBytes().FromRaw(val)
	case commonarrow.CborCode:
		cborArr := vArr.Field(fieldId)
		if cborArr == nil {
			return fmt.Errorf("UpdateValueFrom(fieldID=%d, cborArr): %w", fieldId, errInvalidFieldId)
		}
		val, err := arrowutils.BinaryFromArray(cborArr, row)
		if err != nil {
			return err
		}
		if err = common.Deserialize(val, v); err != nil {
			return err
		}
	default:
		return fmt.Errorf("UpdateValueFrom: unknow type code `%d` in any value union array", tcode)
	}

	return nil
}
