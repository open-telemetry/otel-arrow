package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	common_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
)

func UpdateValueFrom(v pcommon.Value, vArr *array.SparseUnion, row int) error {
	tcode := int8(vArr.ChildID(row))
	switch tcode {
	case common_arrow.StrCode:
		val, err := arrow_utils.StringFromArray(vArr.Field(int(tcode)), row)
		if err != nil {
			return err
		}
		v.SetStr(val)
	case common_arrow.I64Code:
		val := vArr.Field(int(tcode)).(*array.Int64).Value(row)
		v.SetInt(val)
	case common_arrow.F64Code:
		val := vArr.Field(int(tcode)).(*array.Float64).Value(row)
		v.SetDouble(val)
	case common_arrow.BoolCode:
		val := vArr.Field(int(tcode)).(*array.Boolean).Value(row)
		v.SetBool(val)
	case common_arrow.BinaryCode:
		val, err := arrow_utils.BinaryFromArray(vArr.Field(int(tcode)), row)
		if err != nil {
			return err
		}
		v.SetEmptyBytes().FromRaw(val)
	default:
		return fmt.Errorf("UpdateValueFrom: unknow type code `%d` in any value union array", tcode)
	}

	return nil
}
