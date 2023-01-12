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

	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	commonarrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
)

func UpdateValueFrom(v pcommon.Value, vArr *array.SparseUnion, row int) error {
	tcode := int8(vArr.ChildID(row))
	switch tcode {
	case commonarrow.StrCode:
		val, err := arrowutils.StringFromArray(vArr.Field(int(tcode)), row)
		if err != nil {
			return err
		}
		v.SetStr(val)
	case commonarrow.I64Code:
		val := vArr.Field(int(tcode)).(*array.Int64).Value(row)
		v.SetInt(val)
	case commonarrow.F64Code:
		val := vArr.Field(int(tcode)).(*array.Float64).Value(row)
		v.SetDouble(val)
	case commonarrow.BoolCode:
		val := vArr.Field(int(tcode)).(*array.Boolean).Value(row)
		v.SetBool(val)
	case commonarrow.BinaryCode:
		val, err := arrowutils.BinaryFromArray(vArr.Field(int(tcode)), row)
		if err != nil {
			return err
		}
		v.SetEmptyBytes().FromRaw(val)
	case commonarrow.CborCode:
		val, err := arrowutils.BinaryFromArray(vArr.Field(int(tcode)), row)
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
