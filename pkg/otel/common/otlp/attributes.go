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

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type AttributeIds struct {
	Id int
}

func NewAttributeIds(structDT *arrow.StructType) (*AttributeIds, error) {
	id, _ := arrowutils.FieldIDFromStruct(structDT, constants.Attributes)
	return &AttributeIds{Id: id}, nil
}

func NewSharedAttributeIds(structDT *arrow.StructType) *AttributeIds {
	id, found := structDT.FieldIdx(constants.SharedAttributes)
	if !found {
		return nil
	}
	return &AttributeIds{Id: id}
}

func NewSharedEventAttributeIds(structDT *arrow.StructType) *AttributeIds {
	id, found := structDT.FieldIdx(constants.SharedEventAttributes)
	if !found {
		return nil
	}
	return &AttributeIds{Id: id}
}

func NewSharedLinkAttributeIds(structDT *arrow.StructType) *AttributeIds {
	id, found := structDT.FieldIdx(constants.SharedLinkAttributes)
	if !found {
		return nil
	}
	return &AttributeIds{Id: id}
}

func AppendAttributesInto(attrs pcommon.Map, parentArr *array.Struct, row int, attributeIds *AttributeIds) error {
	marr, err := attributesFromStruct(attributeIds.Id, parentArr, row)
	if err != nil {
		return werror.Wrap(err)
	}
	if marr == nil {
		return nil
	}

	return UpdateAttributesFrom(attrs, marr, row)
}

func UpdateAttributesFrom(attrs pcommon.Map, marr *array.Map, row int) error {
	if marr.IsNull(row) {
		return nil
	}

	start := int(marr.Offsets()[row])
	end := int(marr.Offsets()[row+1])

	attrs.EnsureCapacity(end - start)

	keys := marr.Keys()
	values, ok := marr.Items().(*array.SparseUnion)
	if !ok {
		return werror.WrapWithContext(common.ErrNotArraySparseUnion, map[string]interface{}{"row": row})
	}

	for i := start; i < end; i++ {
		key, err := arrowutils.StringFromArray(keys, i)
		if err != nil {
			return werror.Wrap(err)
		}

		if err = UpdateValueFrom(attrs.PutEmpty(key), values, i); err != nil {
			return werror.Wrap(err)
		}
	}
	return nil
}

func attributesFromStruct(fieldID int, parentArr *array.Struct, row int) (marr *array.Map, err error) {
	if fieldID == -1 {
		return nil, nil
	}

	column := parentArr.Field(fieldID)
	switch arr := column.(type) {
	case *array.Map:
		if arr.IsNull(row) {
			return
		}

		marr = arr
	default:
		err = werror.WrapWithContext(common.ErrNotArrayMap, map[string]interface{}{"row": row, "fieldID": fieldID})
	}
	return
}
