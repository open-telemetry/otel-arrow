package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type AttributeIds struct {
	Id int
}

func NewAttributeIds(structDT *arrow.StructType) (*AttributeIds, error) {
	id, found := structDT.FieldIdx(constants.ATTRIBUTES)
	if !found {
		return nil, fmt.Errorf("`attributes` field not found in Arrow struct")
	}
	return &AttributeIds{Id: id}, nil
}

func NewSharedAttributeIds(structDT *arrow.StructType) *AttributeIds {
	id, found := structDT.FieldIdx(constants.SHARED_ATTRIBUTES)
	if !found {
		return nil
	}
	return &AttributeIds{Id: id}
}

func AppendAttributesInto(attrs pcommon.Map, parentArr *array.Struct, row int, attributeIds *AttributeIds) error {
	marr, start, end, err := attributesFromStruct(attributeIds.Id, parentArr, row)
	if err != nil {
		return err
	}
	if marr == nil {
		return nil
	}
	attrs.EnsureCapacity(end - start)

	keys := marr.Keys()
	values, ok := marr.Items().(*array.SparseUnion)
	if !ok {
		return fmt.Errorf("`attributes` is not an Arrow sparse union")
	}

	for i := start; i < end; i++ {
		key, err := arrowutils.StringFromArray(keys, i)
		if err != nil {
			return err
		}

		if err := UpdateValueFrom(attrs.PutEmpty(key), values, i); err != nil {
			return err
		}
	}
	return nil
}

func attributesFromStruct(fieldId int, parentArr *array.Struct, row int) (marr *array.Map, start int, end int, err error) {
	start = 0
	end = 0

	column := parentArr.Field(fieldId)
	switch arr := column.(type) {
	case *array.Map:
		if arr.IsNull(row) {
			return
		}

		start = int(arr.Offsets()[row])
		end = int(arr.Offsets()[row+1])
		marr = arr
	default:
		err = fmt.Errorf("`attributes` is not an Arrow map")
	}
	return
}
