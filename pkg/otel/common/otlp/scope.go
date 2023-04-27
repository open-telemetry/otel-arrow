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

type ScopeIds struct {
	Id                     int
	Name                   int
	Version                int
	AttrsID                int
	DroppedAttributesCount int
}

func NewScopeIds(resSpansDT *arrow.StructType) (*ScopeIds, error) {
	scopeID, scopeDT, err := arrowutils.StructFieldIDFromStruct(resSpansDT, constants.Scope)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	nameID, _ := arrowutils.FieldIDFromStruct(scopeDT, constants.Name)
	versionID, _ := arrowutils.FieldIDFromStruct(scopeDT, constants.Version)
	droppedAttributesCountID, _ := arrowutils.FieldIDFromStruct(scopeDT, constants.DroppedAttributesCount)
	attrsID, _ := arrowutils.FieldIDFromStruct(scopeDT, constants.AttributesID)
	return &ScopeIds{
		Id:                     scopeID,
		Name:                   nameID,
		Version:                versionID,
		DroppedAttributesCount: droppedAttributesCountID,
		AttrsID:                attrsID,
	}, nil
}

// UpdateScopeWith appends a scope into a given scope spans from an Arrow list of structs.
func UpdateScopeWith(
	s pcommon.InstrumentationScope,
	listOfStructs *arrowutils.ListOfStructs,
	row int,
	ids *ScopeIds,
	attrsStore *Attributes16Store,
) error {
	_, scopeArray, err := listOfStructs.StructByID(ids.Id, row)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	name, err := arrowutils.StringFromStruct(scopeArray, row, ids.Name)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	version, err := arrowutils.StringFromStruct(scopeArray, row, ids.Version)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	droppedAttributesCount, err := arrowutils.U32FromStruct(scopeArray, row, ids.DroppedAttributesCount)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}

	attrsID, err := arrowutils.NullableU16FromStruct(scopeArray, row, ids.AttrsID)
	if err != nil {
		return werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	if attrsID != nil {
		attrs := attrsStore.AttributesByDeltaID(*attrsID)
		if attrs != nil {
			attrs.CopyTo(s.Attributes())
		}
	}
	s.SetName(name)
	s.SetVersion(version)
	s.SetDroppedAttributesCount(droppedAttributesCount)
	return nil
}
