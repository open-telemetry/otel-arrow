/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package arrow

import (
	"fmt"
	"io"
	"strings"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/air"
	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
)

type ScopeEntities interface {
	ptrace.ScopeSpans | plog.ScopeLogs

	Scope() pcommon.InstrumentationScope
	SchemaUrl() string
}

type TopLevelEntities[SE ScopeEntities] interface {
	ResourceSlice() TopLevelEntitiesSlice[SE]
	EntityGrouper(SE, *config.Config) map[string][]rfield.Value
	ResourceEntitiesLabel() string
	ScopeEntitiesLabel() string
	EntitiesLabel() string
}

type TopLevelEntitiesSlice[SE ScopeEntities] interface {
	Len() int
	At(i int) ResourceEntities[SE]
}

type ResourceEntities[SE ScopeEntities] interface {
	Resource() pcommon.Resource
	SchemaUrl() string
	ScopeEntities() ScopeEntitiesSlice[SE]
}

type ScopeEntitiesSlice[SE ScopeEntities] interface {
	Len() int
	At(i int) SE
}

// OtlpArrowProducer produces OTLP Arrow records from OTLP entities.
type OtlpArrowProducer[SE ScopeEntities] struct {
	cfg *config.Config
	rr  *air.RecordRepository
}

// NewOtlpArrowProducer creates a new OtlpArrowProducer with the default configuration.
// Note: the default attribute encoding is AttributesAsListStructs
func NewOtlpArrowProducer[SE ScopeEntities]() *OtlpArrowProducer[SE] {
	cfg := config.NewUint16DefaultConfig()
	cfg.Attribute.Encoding = config.AttributesAsListStructs

	return &OtlpArrowProducer[SE]{
		cfg: cfg,
		rr:  air.NewRecordRepository(cfg),
	}
}

func NewOtlpArrowProducerWithConfig[SE ScopeEntities](cfg *config.Config) *OtlpArrowProducer[SE] {
	return &OtlpArrowProducer[SE]{
		cfg: cfg,
		rr:  air.NewRecordRepository(cfg),
	}
}

// ProduceFrom produces Arrow records from the given OTLP entities. The generated schemas of the Arrow records follow
// the hierarchical organization of the entity protobuf structure (e.g. Traces, Logs protobuf message).
//
// Resource signature = resource attributes sig + dropped attributes count sig + schema URL sig
//
// More details can be found in the OTEL 0156 section XYZ.
// TODO add a reference to the OTEP 0156 section that describes this mapping.
func (p *OtlpArrowProducer[T]) ProduceFrom(tle TopLevelEntities[T]) ([]arrow.Record, error) {
	resLogList := tle.ResourceSlice()
	// Resource logs grouped per signature. The resource log signature is based on the resource attributes, the dropped
	// attributes count, the schema URL, and the scope logs signature.
	resLogsPerSig := make(map[string]*ResourceEntity)

	for rsIdx := 0; rsIdx < resLogList.Len(); rsIdx++ {
		resLogs := resLogList.At(rsIdx)

		// Add resource fields (attributes and dropped attributes count)
		resField, resSig := common.ResourceFieldWithSig(resLogs.Resource(), p.cfg)

		// Add schema URL
		var schemaUrl *rfield.Field
		if resLogs.SchemaUrl() != "" {
			schemaUrl = rfield.NewStringField(constants.SCHEMA_URL, resLogs.SchemaUrl())
			resSig += ",schema_url:" + resLogs.SchemaUrl()
		}

		// Group logs per scope span signature
		//logsPerScopeLogSig := GroupScopeLogs(resLogs.ScopeLogs(), p.cfg)
		logsPerScopeLogSig := GroupScopeEntities[T](resLogs.ScopeEntities(), tle.EntityGrouper, p.cfg)

		// Create a new entry in the map if the signature is not already present
		resLogFields := resLogsPerSig[resSig]
		if resLogFields == nil {
			resLogFields = NewResourceEntity(resField, schemaUrl)
			resLogsPerSig[resSig] = resLogFields
		}

		// Merge logs sharing the same scope log signature
		for sig, sl := range logsPerScopeLogSig {
			scopeLog := resLogFields.ScopeEntities[sig]
			if scopeLog == nil {
				resLogFields.ScopeEntities[sig] = sl
			} else {
				scopeLog.Entities = append(scopeLog.Entities, sl.Entities...)
			}
		}
	}

	// All resource logs sharing the same signature are represented as an AIR record.
	for _, resLogFields := range resLogsPerSig {
		record := air.NewRecord()
		record.ListField(tle.ResourceEntitiesLabel(), rfield.List{Values: []rfield.Value{
			resLogFields.AirValue(tle.ScopeEntitiesLabel(), tle.EntitiesLabel()),
		}})
		p.rr.AddRecord(record)
	}

	// Build all Arrow records from the AIR records
	records, err := p.rr.BuildRecords()
	if err != nil {
		return nil, err
	}

	return records, nil
}

// DumpMetadata dumps the metadata of the produced Arrow records.
func (p *OtlpArrowProducer[T]) DumpMetadata(f io.Writer) {
	p.rr.DumpMetadata(f)
}

// GroupScopeEntities groups OTLP entities per signature scope entities signature.
// A scope OTLP entity signature is based on the combination of scope attributes, dropped attributes count, the schema URL, and
// entity signatures.
// entityGrouper converts OTLP scope entities into their AIR representation and groups them based on a given
// configuration. An OTLP entity signature is based on the attributes when the attribute encoding configuration is
// AttributesAsStructs, otherwise it is an empty string.
func GroupScopeEntities[SE ScopeEntities](scopeEntityList ScopeEntitiesSlice[SE], entityGrouper func(SE, *config.Config) map[string][]rfield.Value, cfg *config.Config) (scopeEntitiesPerSig map[string]*EntityGroup) {
	scopeEntitiesPerSig = make(map[string]*EntityGroup, scopeEntityList.Len())

	for j := 0; j < scopeEntityList.Len(); j++ {
		scopeEntities := scopeEntityList.At(j)

		var sig strings.Builder

		scopeField := common.ScopeField(constants.SCOPE, scopeEntities.Scope(), cfg)
		scopeField.Normalize()
		scopeField.WriteSigType(&sig)

		var schemaField *rfield.Field
		if scopeEntities.SchemaUrl() != "" {
			schemaField = rfield.NewStringField(constants.SCHEMA_URL, scopeEntities.SchemaUrl())
			sig.WriteString(",")
			schemaField.WriteSig(&sig)
		}

		// Group entities per signature
		entities := entityGrouper(scopeEntities, cfg)

		for entitySig, entityGroup := range entities {
			sig.WriteByte(',')
			sig.WriteString(entitySig)

			// Create a new entry in the map if the signature is not already present
			seSig := sig.String()
			seFields := scopeEntitiesPerSig[seSig]
			if seFields == nil {
				seFields = &EntityGroup{
					Scope:     scopeField,
					Entities:  make([]rfield.Value, 0, 16),
					SchemaUrl: schemaField,
				}
				scopeEntitiesPerSig[seSig] = seFields
			}

			seFields.Entities = append(seFields.Entities, entityGroup...)
		}
	}
	return
}

// ResourceEntity groups a set of scope OTLP entities sharing the same resource, schema url, and entity signature.
type ResourceEntity struct {
	resource      *rfield.Field
	schemaUrl     *rfield.Field
	ScopeEntities map[string]*EntityGroup
}

// NewResourceEntity creates a new resource OTLP entity for the given resource and schema url.
func NewResourceEntity(resource *rfield.Field, schemaUrl *rfield.Field) *ResourceEntity {
	return &ResourceEntity{
		resource:      resource,
		ScopeEntities: make(map[string]*EntityGroup),
		schemaUrl:     schemaUrl,
	}
}

// AirValue builds an AIR representation of the current resource OTLP entity for this group of scope entities.
// Resource entity = resource fields + schema URL + scope entities.
// Values for scopeEntitiesField: constants.SCOPE_LOGS, constants.SCOPE_SPANS
// Values for entityField: constants.LOGS, constants.SPANS
func (slg *ResourceEntity) AirValue(scopeEntitiesField string, entityField string) rfield.Value {
	fields := make([]*rfield.Field, 0, 3)
	if slg.resource != nil {
		fields = append(fields, slg.resource)
	}
	if slg.schemaUrl != nil {
		fields = append(fields, slg.schemaUrl)
	}
	if len(slg.ScopeEntities) > 0 {
		scopeEntities := make([]rfield.Value, 0, len(slg.ScopeEntities))
		for _, se := range slg.ScopeEntities {
			scopeEntities = append(scopeEntities, se.ScopeEntity(entityField))
		}
		fields = append(fields, rfield.NewListField(scopeEntitiesField, rfield.List{Values: scopeEntities}))
	}
	return rfield.NewStruct(fields)
}

// EntityGroup groups a set of OTLP entities sharing the same signature.
type EntityGroup struct {
	Scope     *rfield.Field
	SchemaUrl *rfield.Field
	Entities  []rfield.Value
}

// ScopeEntity builds an AIR representation of the current scope OTLP entity for this group of entities.
// Scope OTLP entity = scope fields + schema URL + entities
func (lg *EntityGroup) ScopeEntity(entityField string) rfield.Value {
	fields := make([]*rfield.Field, 0, 3)
	if lg.Scope != nil {
		fields = append(fields, lg.Scope)
	}
	if lg.SchemaUrl != nil {
		fields = append(fields, lg.SchemaUrl)
	}
	if len(lg.Entities) > 0 {
		fields = append(fields, rfield.NewListField(entityField, rfield.List{Values: lg.Entities}))
	}

	return rfield.NewStruct(fields)
}

func AttributesId(attrs pcommon.Map) string {
	var attrsId strings.Builder
	attrs.Sort()
	attrsId.WriteString("{")
	attrs.Range(func(k string, v pcommon.Value) bool {
		if attrsId.Len() > 1 {
			attrsId.WriteString(",")
		}
		attrsId.WriteString(k)
		attrsId.WriteString(":")
		attrsId.WriteString(ValueId(v))
		return true
	})
	attrsId.WriteString("}")
	return attrsId.String()
}

// TODO replace this implementation with the one used for traces.
func NewResourceFromOld(record arrow.Record, row int) (pcommon.Resource, error) {
	r := pcommon.NewResource()
	resourceField, resourceArray, err := air.StructFromRecord(record, constants.RESOURCE)
	if err != nil {
		return r, err
	}
	droppedAttributesCount, err := air.U32FromStruct(resourceField, resourceArray, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return r, err
	}
	attrField, attrArray, err := air.FieldArrayOfStruct(resourceField, resourceArray, constants.ATTRIBUTES)
	if err != nil {
		return r, err
	}
	if attrField != nil {
		if err = CopyAttributesFrom(r.Attributes(), attrField.Type, attrArray, row); err != nil {
			return r, err
		}
	}
	r.SetDroppedAttributesCount(droppedAttributesCount)
	return r, nil
}

// NewResourceFrom creates a new Resource from the given array and row.
func NewResourceFrom(resList *air.ListOfStructs, row int) (pcommon.Resource, error) {
	r := pcommon.NewResource()
	resDt, resArr, err := resList.StructArray(constants.RESOURCE, row)
	if err != nil {
		return r, err
	}

	// Read dropped attributes count
	droppedAttributesCount, err := air.U32FromStruct(resDt, resArr, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return r, err
	}
	r.SetDroppedAttributesCount(droppedAttributesCount)

	// Read attributes
	attrs, err := air.ListOfStructsFromStruct(resDt, resArr, row, constants.ATTRIBUTES)
	if err != nil {
		return r, err
	}
	if attrs != nil {
		err = attrs.CopyAttributesFrom(r.Attributes())
	}

	return r, err
}

func NewScopeFrom(listOfStructs *air.ListOfStructs, row int) (pcommon.InstrumentationScope, error) {
	s := pcommon.NewInstrumentationScope()
	scopeField, scopeArray, err := listOfStructs.StructArray(constants.SCOPE, row)
	if err != nil {
		return s, err
	}
	name, err := air.StringFromStruct(scopeField, scopeArray, row, constants.NAME)
	if err != nil {
		return s, err
	}
	version, err := air.StringFromStruct(scopeField, scopeArray, row, constants.VERSION)
	if err != nil {
		return s, err
	}
	droppedAttributesCount, err := air.U32FromStruct(scopeField, scopeArray, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return s, err
	}

	attrs, err := air.ListOfStructsFromStruct(scopeField, scopeArray, row, constants.ATTRIBUTES)
	if err != nil {
		return s, err
	}
	if attrs != nil {
		err = attrs.CopyAttributesFrom(s.Attributes())
	}
	s.SetName(name)
	s.SetVersion(version)
	s.SetDroppedAttributesCount(droppedAttributesCount)
	return s, nil
}

func NewInstrumentationScopeFrom(record arrow.Record, row int, scope string) (pcommon.InstrumentationScope, error) {
	s := pcommon.NewInstrumentationScope()
	scopeField, scopeArray, err := air.StructFromRecord(record, scope)
	if err != nil {
		return s, err
	}
	name, err := air.StringFromStruct(scopeField, scopeArray, row, constants.NAME)
	if err != nil {
		return s, err
	}
	version, err := air.StringFromStruct(scopeField, scopeArray, row, constants.VERSION)
	if err != nil {
		return s, err
	}
	droppedAttributesCount, err := air.U32FromStruct(scopeField, scopeArray, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return s, err
	}
	attrField, attrArray, err := air.FieldArrayOfStruct(scopeField, scopeArray, constants.ATTRIBUTES)
	if err != nil {
		return s, err
	}
	if attrField != nil {
		if err = CopyAttributesFrom(s.Attributes(), attrField.Type, attrArray, row); err != nil {
			return s, err
		}
	}
	s.SetName(name)
	s.SetVersion(version)
	s.SetDroppedAttributesCount(droppedAttributesCount)
	return s, nil
}

func ResourceId(r pcommon.Resource) string {
	return AttributesId(r.Attributes()) + "|" + fmt.Sprintf("dac:%d", r.DroppedAttributesCount())
}

func ScopeId(is pcommon.InstrumentationScope) string {
	return "name:" + is.Name() + "|version:" + is.Version() + "|" + AttributesId(is.Attributes()) + "|" + fmt.Sprintf("dac:%d", is.DroppedAttributesCount())
}

func ValueId(v pcommon.Value) string {
	switch v.Type() {
	case pcommon.ValueTypeStr:
		return v.Str()
	case pcommon.ValueTypeInt:
		return fmt.Sprintf("%d", v.Int())
	case pcommon.ValueTypeDouble:
		return fmt.Sprintf("%f", v.Double())
	case pcommon.ValueTypeBool:
		return fmt.Sprintf("%t", v.Bool())
	case pcommon.ValueTypeMap:
		return AttributesId(v.Map())
	case pcommon.ValueTypeBytes:
		return fmt.Sprintf("%x", v.Bytes().AsRaw())
	case pcommon.ValueTypeSlice:
		values := v.Slice()
		valueId := "["
		for i := 0; i < values.Len(); i++ {
			if len(valueId) > 1 {
				valueId += ","
			}
			valueId += ValueId(values.At(i))
		}
		valueId += "]"
		return valueId
	default:
		// includes pcommon.ValueTypeEmpty
		panic("unsupported value type")
	}
}

func CopyAttributesFrom(a pcommon.Map, dt arrow.DataType, arr arrow.Array, row int) error {
	structType, ok := dt.(*arrow.StructType)
	if !ok {
		return fmt.Errorf("attributes is not a struct")
	}
	attrArray := arr.(*array.Struct)
	a.EnsureCapacity(attrArray.NumField())
	for i := 0; i < attrArray.NumField(); i++ {
		valueField := structType.Field(i)

		newV := a.PutEmpty(valueField.Name)

		if err := CopyValueFrom(newV, valueField.Type, attrArray.Field(i), row); err != nil {
			return err
		}
	}
	return nil
}

func CopyValueFrom(dest pcommon.Value, dt arrow.DataType, arr arrow.Array, row int) error {
	switch t := dt.(type) {
	case *arrow.BooleanType:
		v, err := air.BoolFromArray(arr, row)
		if err != nil {
			return err
		}
		dest.SetBool(v)
		return nil
	case *arrow.Float64Type:
		v, err := air.F64FromArray(arr, row)
		if err != nil {
			return err
		}
		dest.SetDouble(v)
		return nil
	case *arrow.Int64Type:
		v, err := air.I64FromArray(arr, row)
		if err != nil {
			return err
		}
		dest.SetInt(v)
		return nil
	case *arrow.StringType:
		v, err := air.StringFromArray(arr, row)
		if err != nil {
			return err
		}
		dest.SetStr(v)
		return nil
	case *arrow.BinaryType:
		v, err := air.BinaryFromArray(arr, row)
		if err != nil {
			return err
		}
		dest.SetEmptyBytes().FromRaw(v)
		return nil
	case *arrow.StructType:
		if err := CopyAttributesFrom(dest.SetEmptyMap(), dt, arr, row); err != nil {
			return err
		}
		return nil
	case *arrow.ListType:
		arrList, ok := arr.(*array.List)
		if !ok {
			return fmt.Errorf("array is not a list")
		}
		if err := SetArrayValue(dest.SetEmptySlice(), arrList, row); err != nil {
			return err
		}
		return nil
	case *arrow.DictionaryType:
		switch t.ValueType.(type) {
		case *arrow.StringType:
			v, err := air.StringFromArray(arr, row)
			if err != nil {
				return err
			}
			dest.SetStr(v)
			return nil
		case *arrow.BinaryType:
			v, err := air.BinaryFromArray(arr, row)
			if err != nil {
				return err
			}
			dest.SetEmptyBytes().FromRaw(v)
			return nil
		default:
			return fmt.Errorf("unsupported dictionary value type %T", t.ValueType)
		}
	default:
		return fmt.Errorf("%T is not a supported value type", t)
	}
}

func SetArrayValue(result pcommon.Slice, arrList *array.List, row int) error {
	start := int(arrList.Offsets()[row])
	end := int(arrList.Offsets()[row+1])
	result.EnsureCapacity(end - start)

	arrItems := arrList.ListValues()
	for ; start < end; start++ {
		v := result.AppendEmpty()
		if arrList.IsNull(start) {
			continue
		}
		if err := CopyValueFrom(v, arrList.DataType(), arrItems, start); err != nil {
			return err
		}
	}

	return nil
}
