/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

// Attributes record builder for 32-bit Parent IDs.

import (
	"errors"
	"sort"

	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

var (
	// AttrsSchema32 is the Arrow schema used to represent attribute records
	// with 16-bit Parent IDs.
	// This schema doesn't use the Arrow union type to make Parquet conversion
	// more direct.
	AttrsSchema32 = arrow.NewSchema([]arrow.Field{
		{Name: constants.ParentID, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.AttributeKey, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.AttributeType, Type: arrow.PrimitiveTypes.Uint8},
		{Name: constants.AttributeStr, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary16)},
		{Name: constants.AttributeInt, Type: arrow.PrimitiveTypes.Int64, Metadata: schema.Metadata(schema.Optional, schema.Dictionary16)},
		{Name: constants.AttributeDouble, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.AttributeBool, Type: arrow.FixedWidthTypes.Boolean, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.AttributeBytes, Type: arrow.BinaryTypes.Binary, Metadata: schema.Metadata(schema.Optional, schema.Dictionary16)},
		{Name: constants.AttributeSer, Type: arrow.BinaryTypes.Binary, Metadata: schema.Metadata(schema.Optional, schema.Dictionary16)},
	}, nil)

	// DeltaEncodedAttrsSchema32 is the Arrow schema used to represent attribute records
	// with 16-bit Parent IDs that are delta encoded.
	// This schema doesn't use the Arrow union type to make Parquet conversion
	// more direct.
	DeltaEncodedAttrsSchema32 = arrow.NewSchema([]arrow.Field{
		{Name: constants.ParentID, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Dictionary8, schema.DeltaEncoding)},
		{Name: constants.AttributeKey, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.AttributeType, Type: arrow.PrimitiveTypes.Uint8},
		{Name: constants.AttributeStr, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary16)},
		{Name: constants.AttributeInt, Type: arrow.PrimitiveTypes.Int64, Metadata: schema.Metadata(schema.Optional, schema.Dictionary16)},
		{Name: constants.AttributeDouble, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.AttributeBool, Type: arrow.FixedWidthTypes.Boolean, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.AttributeBytes, Type: arrow.BinaryTypes.Binary, Metadata: schema.Metadata(schema.Optional, schema.Dictionary16)},
		{Name: constants.AttributeSer, Type: arrow.BinaryTypes.Binary, Metadata: schema.Metadata(schema.Optional, schema.Dictionary16)},
	}, nil)
)

type (
	Attrs32Builder struct {
		released bool

		builder *builder.RecordBuilderExt // Record builder

		pib   *builder.Uint32Builder
		keyb  *builder.StringBuilder
		typeb *builder.Uint8Builder
		strb  *builder.StringBuilder
		i64b  *builder.Int64Builder
		f64b  *builder.Float64Builder
		boolb *builder.BooleanBuilder
		binb  *builder.BinaryBuilder
		serb  *builder.BinaryBuilder

		accumulator *Attributes32Accumulator
		payloadType *PayloadType

		parentIdEncoding int
	}

	Attrs32ByNothing          struct{}
	Attrs32ByParentIdKeyValue struct{}
	Attrs32ByKeyParentIdValue struct{}
	Attrs32ByKeyValueParentId struct{}
)

func NewAttrs32Builder(rBuilder *builder.RecordBuilderExt, payloadType *PayloadType, sorter Attrs32Sorter) *Attrs32Builder {
	b := &Attrs32Builder{
		released:    false,
		builder:     rBuilder,
		accumulator: NewAttributes32Accumulator(sorter),
		payloadType: payloadType,
	}
	b.init()
	return b
}

func NewDeltaEncodedAttrs32Builder(payloadType *PayloadType, rBuilder *builder.RecordBuilderExt, sorter Attrs32Sorter) *Attrs32Builder {
	b := &Attrs32Builder{
		released:         false,
		builder:          rBuilder,
		accumulator:      NewAttributes32Accumulator(sorter),
		payloadType:      payloadType,
		parentIdEncoding: ParentIdDeltaGroupEncoding,
	}

	b.init()
	return b
}

func NewAttrs32BuilderWithEncoding(rBuilder *builder.RecordBuilderExt, payloadType *PayloadType, conf *Attrs32Config) *Attrs32Builder {
	b := &Attrs32Builder{
		released:         false,
		builder:          rBuilder,
		accumulator:      NewAttributes32Accumulator(conf.Sorter),
		payloadType:      payloadType,
		parentIdEncoding: conf.ParentIdEncoding,
	}

	b.init()
	return b
}

func (b *Attrs32Builder) init() {
	b.pib = b.builder.Uint32Builder(constants.ParentID)
	b.keyb = b.builder.StringBuilder(constants.AttributeKey)
	b.typeb = b.builder.Uint8Builder(constants.AttributeType)
	b.strb = b.builder.StringBuilder(constants.AttributeStr)
	b.i64b = b.builder.Int64Builder(constants.AttributeInt)
	b.f64b = b.builder.Float64Builder(constants.AttributeDouble)
	b.boolb = b.builder.BooleanBuilder(constants.AttributeBool)
	b.binb = b.builder.BinaryBuilder(constants.AttributeBytes)
	b.serb = b.builder.BinaryBuilder(constants.AttributeSer)
}

func (b *Attrs32Builder) Accumulator() *Attributes32Accumulator {
	return b.accumulator
}

func (b *Attrs32Builder) TryBuild() (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(ErrBuilderAlreadyReleased)
	}

	prevParentID := uint32(0)
	prevKey := ""
	prevValue := pcommon.NewValueEmpty()
	b.accumulator.Sort()

	for _, attr := range b.accumulator.attrs {
		switch b.parentIdEncoding {
		case ParentIdNoEncoding:
			b.pib.Append(attr.ParentID)
		case ParentIdDeltaEncoding:
			delta := attr.ParentID - prevParentID
			prevParentID = attr.ParentID
			b.pib.Append(delta)
		case ParentIdDeltaGroupEncoding:
			if prevKey == attr.Key && Equal(prevValue, attr.Value) {
				delta := attr.ParentID - prevParentID
				prevParentID = attr.ParentID
				b.pib.Append(delta)
			} else {
				prevKey = attr.Key
				prevValue = attr.Value
				prevParentID = attr.ParentID
				b.pib.Append(attr.ParentID)
			}
		}

		b.keyb.Append(attr.Key)
		switch attr.Value.Type() {
		case pcommon.ValueTypeStr:
			b.typeb.Append(uint8(pcommon.ValueTypeStr))
			b.strb.Append(attr.Value.Str())
			b.i64b.AppendNull()
			b.f64b.AppendNull()
			b.boolb.AppendNull()
			b.binb.AppendNull()
			b.serb.AppendNull()
		case pcommon.ValueTypeInt:
			b.typeb.Append(uint8(pcommon.ValueTypeInt))
			b.i64b.Append(attr.Value.Int())
			b.strb.AppendNull()
			b.f64b.AppendNull()
			b.boolb.AppendNull()
			b.binb.AppendNull()
			b.serb.AppendNull()
		case pcommon.ValueTypeDouble:
			b.typeb.Append(uint8(pcommon.ValueTypeDouble))
			b.f64b.Append(attr.Value.Double())
			b.strb.AppendNull()
			b.i64b.AppendNull()
			b.boolb.AppendNull()
			b.binb.AppendNull()
			b.serb.AppendNull()
		case pcommon.ValueTypeBool:
			b.typeb.Append(uint8(pcommon.ValueTypeBool))
			b.boolb.Append(attr.Value.Bool())
			b.strb.AppendNull()
			b.i64b.AppendNull()
			b.f64b.AppendNull()
			b.binb.AppendNull()
			b.serb.AppendNull()
		case pcommon.ValueTypeBytes:
			b.typeb.Append(uint8(pcommon.ValueTypeBytes))
			b.binb.Append(attr.Value.Bytes().AsRaw())
			b.strb.AppendNull()
			b.i64b.AppendNull()
			b.f64b.AppendNull()
			b.boolb.AppendNull()
			b.serb.AppendNull()
		case pcommon.ValueTypeSlice:
			cborData, err := common.Serialize(attr.Value)
			if err != nil {
				break
			}
			b.typeb.Append(uint8(pcommon.ValueTypeSlice))
			b.serb.Append(cborData)
			b.strb.AppendNull()
			b.i64b.AppendNull()
			b.f64b.AppendNull()
			b.boolb.AppendNull()
			b.binb.AppendNull()
		case pcommon.ValueTypeMap:
			cborData, err := common.Serialize(attr.Value)
			if err != nil {
				break
			}
			b.typeb.Append(uint8(pcommon.ValueTypeMap))
			b.serb.Append(cborData)
			b.strb.AppendNull()
			b.i64b.AppendNull()
			b.f64b.AppendNull()
			b.boolb.AppendNull()
			b.binb.AppendNull()
		}
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		b.init()
	}

	return
}

func (b *Attrs32Builder) IsEmpty() bool {
	return b.accumulator.IsEmpty()
}

func (b *Attrs32Builder) Build() (arrow.Record, error) {
	schemaNotUpToDateCount := 0

	var record arrow.Record
	var err error

	// Loop until the record is built successfully.
	// Intermediaries steps may be required to update the schema.
	for {
		record, err = b.TryBuild()
		if err != nil {
			if record != nil {
				record.Release()
			}

			switch {
			case errors.Is(err, schema.ErrSchemaNotUpToDate):
				schemaNotUpToDateCount++
				if schemaNotUpToDateCount > 5 {
					panic("Too many consecutive schema updates. This shouldn't happen.")
				}
			default:
				return nil, werror.Wrap(err)
			}
		} else {
			break
		}
	}

	// ToDo Keep this code for debugging purposes.
	//if err == nil && attrs32Counters[b.payloadType.PayloadType().String()] == 0 {
	//	println(b.payloadType.PayloadType().String())
	//	arrow2.PrintRecord(record)
	//	attrs32Counters[b.payloadType.PayloadType().String()] += 1
	//}

	return record, werror.Wrap(err)
}

// ToDo Keep this code for debugging purposes.
//var attrs32Counters = make(map[string]int)

func (b *Attrs32Builder) SchemaID() string {
	return b.builder.SchemaID()
}

func (b *Attrs32Builder) Schema() *arrow.Schema {
	return b.builder.Schema()
}

func (b *Attrs32Builder) PayloadType() *PayloadType {
	return b.payloadType
}

func (b *Attrs32Builder) Reset() {
	b.accumulator.Reset()
}

// Release releases the memory allocated by the builder.
func (b *Attrs32Builder) Release() {
	if !b.released {
		b.builder.Release()
		b.released = true
	}
}

func (b *Attrs32Builder) ShowSchema() {
	b.builder.ShowSchema()
}

// No sorting
// ==========

func UnsortedAttrs32() *Attrs32ByNothing {
	return &Attrs32ByNothing{}
}

func (s Attrs32ByNothing) Sort(_ []Attr32) {
	// Do nothing
}

// Sorts the attributes by parentID, key, and value
// ================================================

func SortAttrs32ByParentIdKeyValue() *Attrs32ByParentIdKeyValue {
	return &Attrs32ByParentIdKeyValue{}
}

func (s Attrs32ByParentIdKeyValue) Sort(attrs []Attr32) {
	sort.Slice(attrs, func(i, j int) bool {
		attrsI := attrs[i]
		attrsJ := attrs[j]
		if attrsI.Value.Type() == attrsJ.Value.Type() {
			if attrsI.ParentID == attrsJ.ParentID {
				if attrsI.Key == attrsJ.Key {
					return IsLess(attrsI.Value, attrsJ.Value)
				} else {
					return attrsI.Key < attrsJ.Key
				}
			} else {
				return attrsI.ParentID < attrsJ.ParentID
			}
		} else {
			return attrsI.Value.Type() < attrsJ.Value.Type()
		}
	})
}

// Sorts the attributes by key, parentID, and value
// ================================================

func SortAttrs32ByKeyParentIdValue() *Attrs32ByKeyParentIdValue {
	return &Attrs32ByKeyParentIdValue{}
}

func (s Attrs32ByKeyParentIdValue) Sort(attrs []Attr32) {
	sort.Slice(attrs, func(i, j int) bool {
		attrsI := attrs[i]
		attrsJ := attrs[j]
		if attrsI.Value.Type() == attrsJ.Value.Type() {
			if attrsI.Key == attrsJ.Key {
				if attrsI.ParentID == attrsJ.ParentID {
					return IsLess(attrsI.Value, attrsJ.Value)
				} else {
					return attrsI.ParentID < attrsJ.ParentID
				}
			} else {
				return attrsI.Key < attrsJ.Key
			}
		} else {
			return attrsI.Value.Type() < attrsJ.Value.Type()
		}
	})
}

// Sorts the attributes by key, value, and parentID
// ================================================

func SortAttrs32ByKeyValueParentId() *Attrs32ByKeyValueParentId {
	return &Attrs32ByKeyValueParentId{}
}

func (s Attrs32ByKeyValueParentId) Sort(attrs []Attr32) {
	sort.Slice(attrs, func(i, j int) bool {
		attrsI := attrs[i]
		attrsJ := attrs[j]
		if attrsI.Value.Type() == attrsJ.Value.Type() {
			if attrsI.Key == attrsJ.Key {
				cmp := Compare(attrsI.Value, attrsJ.Value)
				if cmp == 0 {
					return attrsI.ParentID < attrsJ.ParentID
				} else {
					return cmp < 0
				}
			} else {
				return attrsI.Key < attrsJ.Key
			}
		} else {
			return attrsI.Value.Type() < attrsJ.Value.Type()
		}
	})
}
