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
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type ResourceIds struct {
	Id                     int
	AttrsID                int
	DroppedAttributesCount int
}

func NewResourceIds(resSpansDT *arrow.StructType) (*ResourceIds, error) {
	resId, resDT, err := arrowutils.StructFieldIDFromStruct(resSpansDT, constants.Resource)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	attributeIds, _ := arrowutils.FieldIDFromStruct(resDT, constants.ID)
	droppedAttributesCount, _ := arrowutils.FieldIDFromStruct(resDT, constants.DroppedAttributesCount)

	return &ResourceIds{
		Id:                     resId,
		AttrsID:                attributeIds,
		DroppedAttributesCount: droppedAttributesCount,
	}, nil
}

// UpdateResourceWith updates a resource with the content of an Arrow array.
func UpdateResourceWith(r pcommon.Resource, resList *arrowutils.ListOfStructs, row int, resIds *ResourceIds, attrsStore *Attributes16Store) error {
	_, resArr, err := resList.StructByID(resIds.Id, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}

	// Read dropped attributes count
	droppedAttributesCount, err := arrowutils.U32FromStruct(resArr, row, resIds.DroppedAttributesCount)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	r.SetDroppedAttributesCount(droppedAttributesCount)

	// Read attributes
	attrsId, err := arrowutils.NullableU16FromStruct(resArr, row, resIds.AttrsID)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	if attrsId != nil {
		attrs := attrsStore.AttributesByDeltaID(*attrsId)
		if attrs != nil {
			attrs.CopyTo(r.Attributes())
		}
	}

	return err
}
