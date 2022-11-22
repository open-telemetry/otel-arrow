package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ScopeIds struct {
	Id                     int
	Name                   int
	Version                int
	Attributes             *AttributeIds
	DroppedAttributesCount int
}

func NewScopeIds(resSpansDT *arrow.StructType) (*ScopeIds, error) {
	scopeID, scopeDT, err := arrow_utils.StructFieldIDFromStruct(resSpansDT, constants.SCOPE)
	if err != nil {
		return nil, err
	}
	nameID, _, err := arrow_utils.FieldIDFromStruct(scopeDT, constants.NAME)
	if err != nil {
		return nil, err
	}
	versionID, _, err := arrow_utils.FieldIDFromStruct(scopeDT, constants.VERSION)
	if err != nil {
		return nil, err
	}
	droppedAttributesCountID, _, err := arrow_utils.FieldIDFromStruct(scopeDT, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}
	attributeIds, err := NewAttributeIds(scopeDT)
	if err != nil {
		return nil, err
	}
	return &ScopeIds{
		Id:                     scopeID,
		Name:                   nameID,
		Version:                versionID,
		DroppedAttributesCount: droppedAttributesCountID,
		Attributes:             attributeIds,
	}, nil
}

// UpdateScopeWith appends a scope into a given scope spans from an Arrow list of structs.
func UpdateScopeWith(s pcommon.InstrumentationScope, listOfStructs *arrow_utils.ListOfStructs, row int, ids *ScopeIds) error {
	_, scopeArray, err := listOfStructs.StructByID(ids.Id, row)
	if err != nil {
		return err
	}
	name, err := arrow_utils.StringFromStruct(scopeArray, row, ids.Name)
	if err != nil {
		return err
	}
	version, err := arrow_utils.StringFromStruct(scopeArray, row, ids.Version)
	if err != nil {
		return err
	}
	droppedAttributesCount, err := arrow_utils.U32FromStruct(scopeArray, row, ids.DroppedAttributesCount)
	if err != nil {
		return err
	}

	err = AppendAttributesInto(s.Attributes(), scopeArray, row, ids.Attributes)
	if err != nil {
		return err
	}
	s.SetName(name)
	s.SetVersion(version)
	s.SetDroppedAttributesCount(droppedAttributesCount)
	return nil
}

func NewScopeFromRecord(record arrow.Record, row int, scope string) (pcommon.InstrumentationScope, error) {
	s := pcommon.NewInstrumentationScope()
	scopeField, scopeArray, err := arrow_utils.StructFromRecord(record, scope)
	if err != nil {
		return s, err
	}
	name, err := arrow_utils.OldStringFromStruct(scopeField, scopeArray, row, constants.NAME)
	if err != nil {
		return s, err
	}
	version, err := arrow_utils.OldStringFromStruct(scopeField, scopeArray, row, constants.VERSION)
	if err != nil {
		return s, err
	}
	droppedAttributesCount, err := arrow_utils.U32FromStructOld(scopeField, scopeArray, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return s, err
	}
	attrField, attrArray, err := arrow_utils.FieldArrayOfStruct(scopeField, scopeArray, constants.ATTRIBUTES)
	if err != nil {
		return s, err
	}
	if attrField != nil {
		if err = carrow.CopyAttributesFrom(s.Attributes(), attrField.Type, attrArray, row); err != nil {
			return s, err
		}
	}
	s.SetName(name)
	s.SetVersion(version)
	s.SetDroppedAttributesCount(droppedAttributesCount)
	return s, nil
}
