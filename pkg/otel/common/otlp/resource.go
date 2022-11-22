package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ResourceIds struct {
	Id                     int
	Attributes             *AttributeIds
	DroppedAttributesCount int
}

func NewResourceIds(resSpansDT *arrow.StructType) (*ResourceIds, error) {
	resId, resDT, err := arrow_utils.StructFieldIDFromStruct(resSpansDT, constants.RESOURCE)
	if err != nil {
		return nil, err
	}

	attributeIds, err := NewAttributeIds(resDT)
	if err != nil {
		return nil, err
	}

	droppedAttributesCount, _, err := arrow_utils.FieldIDFromStruct(resDT, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}

	return &ResourceIds{
		Id:                     resId,
		Attributes:             attributeIds,
		DroppedAttributesCount: droppedAttributesCount,
	}, nil
}

// UpdateResourceWith updates a resource with the content of an Arrow array.
func UpdateResourceWith(r pcommon.Resource, resList *arrow_utils.ListOfStructs, row int, resIds *ResourceIds) error {
	_, resArr, err := resList.StructByID(resIds.Id, row)
	if err != nil {
		return err
	}

	// Read dropped attributes count
	droppedAttributesCount, err := arrow_utils.U32FromStruct(resArr, row, resIds.DroppedAttributesCount)
	if err != nil {
		return err
	}
	r.SetDroppedAttributesCount(droppedAttributesCount)

	// Read attributes
	err = AppendAttributesInto(r.Attributes(), resArr, row, resIds.Attributes)
	if err != nil {
		return err
	}

	return err
}
