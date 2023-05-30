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

import (
	"bytes"
	"math"
	"strings"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// ToDo Standard attributes (i.e. AttributesDT) will be removed in the future. They are still used as shared attributes in the metrics.
// ToDo this file must be redistributed into `attributes_16.go` and `attributes_32.go` files.

// ParentID encodings
const (
	// ParentIdNoEncoding stores the parent ID as is.
	ParentIdNoEncoding = iota
	// ParentIdDeltaEncoding stores the parent ID as a delta from the previous
	// parent ID.
	ParentIdDeltaEncoding
	// ParentIdDeltaGroupEncoding stores the parent ID as a delta from the
	// previous parent ID in the same group. A group is defined by the
	// combination Key and Value.
	ParentIdDeltaGroupEncoding
)

// Arrow data types used to build the attribute map.
var (
	// KDT is the Arrow key data type.
	KDT = arrow.BinaryTypes.String

	// AttributesDT is the Arrow attribute data type.
	AttributesDT = arrow.MapOfWithMetadata(
		KDT, schema.Metadata(schema.Dictionary8),
		AnyValueDT, schema.Metadata(),
	)
)

type (
	// AttributesBuilder is a helper to build a map of attributes.
	AttributesBuilder struct {
		released bool

		builder *builder.MapBuilder    // map builder
		kb      *builder.StringBuilder // key builder
		ib      *AnyValueBuilder       // item any value builder
	}

	// Attr16 is an attribute with a 16-bit ParentID.
	Attr16 struct {
		ParentID uint16
		Key      string
		Value    pcommon.Value
	}

	// Attrs16Sorter is used to sort attributes with 16-bit ParentIDs.
	Attrs16Sorter interface {
		Sort(attrs []Attr16)
		Encode(parentID uint16, key string, value pcommon.Value) uint16
		Reset()
	}

	// Attr32 is an attribute with a 32-bit ParentID.
	Attr32 struct {
		ParentID uint32
		Key      string
		Value    pcommon.Value
	}

	// Attrs32Sorter is used to sort attributes with 32-bit ParentIDs.
	Attrs32Sorter interface {
		Sort(attrs []Attr32)
		Encode(parentID uint32, key string, value pcommon.Value) uint32
		Reset()
	}

	// Attributes16Accumulator accumulates attributes for the scope of an entire
	// batch. It is used to sort globally all attributes and optimize the
	// compression ratio. Attribute IDs are 16-bit.
	Attributes16Accumulator struct {
		attrsMapCount uint16
		attrs         []Attr16
		sorter        Attrs16Sorter
	}

	// Attributes32Accumulator accumulates attributes for the scope of an entire
	// batch. It is used to sort globally all attributes and optimize the
	// compression ratio. Attribute IDs are 32-bit.
	Attributes32Accumulator struct {
		attrsMapCount uint32
		attrs         []Attr32
		sorter        Attrs32Sorter
	}
)

// NewAttributesBuilder creates a new AttributesBuilder with a given allocator.
//
// Once the builder is no longer needed, Build() or Release() must be called to free the
// memory allocated by the builder.
func NewAttributesBuilder(builder *builder.MapBuilder) *AttributesBuilder {
	return AttributesBuilderFrom(builder)
}

// AttributesBuilderFrom creates a new AttributesBuilder from an existing MapBuilder.
func AttributesBuilderFrom(mb *builder.MapBuilder) *AttributesBuilder {
	ib := AnyValueBuilderFrom(mb.ItemSparseUnionBuilder())

	return &AttributesBuilder{
		released: false,
		builder:  mb,
		kb:       mb.KeyStringBuilder(),
		ib:       ib,
	}
}

// Build builds the attribute array map.
//
// Once the returned array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *AttributesBuilder) Build() (*array.Map, error) {
	if b.released {
		return nil, werror.Wrap(ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewMapArray(), nil
}

func (b *AttributesBuilder) AppendNull() error {
	if b.released {
		return werror.Wrap(ErrBuilderAlreadyReleased)
	}

	b.builder.AppendNull()
	return nil
}

// Append appends a new set of attributes to the builder.
// Note: empty keys are skipped.
func (b *AttributesBuilder) Append(attrs pcommon.Map) error {
	if b.released {
		return werror.Wrap(ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(attrs.Len(), func() error {
		var err error
		attrs.Range(func(key string, v pcommon.Value) bool {
			if key == "" {
				// Skip entries with empty keys
				return true
			}
			b.kb.AppendNonEmpty(key)
			return b.ib.Append(v) == nil
		})
		return werror.Wrap(err)
	})
}

func (b *AttributesBuilder) AppendUniqueAttributes(attrs pcommon.Map, smattrs *common.SharedAttributes, mattrs *common.SharedAttributes) error {
	if b.released {
		return werror.Wrap(ErrBuilderAlreadyReleased)
	}

	uniqueAttrsCount := attrs.Len()
	if smattrs != nil {
		uniqueAttrsCount -= smattrs.Len()
	}
	if mattrs != nil {
		uniqueAttrsCount -= mattrs.Len()
	}

	return b.builder.Append(uniqueAttrsCount, func() error {
		var err error

		attrs.Range(func(key string, v pcommon.Value) bool {
			if key == "" {
				// Skip entries with empty keys
				return true
			}

			// Skip the current attribute if it is a scope metric shared attribute
			// or a metric shared attribute
			smattrsFound := false
			mattrsFound := false
			if smattrs != nil {
				_, smattrsFound = smattrs.Attributes[key]
			}
			if mattrs != nil {
				_, mattrsFound = mattrs.Attributes[key]
			}
			if smattrsFound || mattrsFound {
				return true
			}

			b.kb.AppendNonEmpty(key)
			err = werror.WrapWithContext(b.ib.Append(v), map[string]interface{}{"key": key, "value": v})

			uniqueAttrsCount--
			return err == nil && uniqueAttrsCount > 0
		})

		return err
	})
}

// Release releases the memory allocated by the builder.
func (b *AttributesBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}

func NewAttributes16Accumulator(sorter Attrs16Sorter) *Attributes16Accumulator {
	return &Attributes16Accumulator{
		attrs:  make([]Attr16, 0),
		sorter: sorter,
	}
}

func (c *Attributes16Accumulator) IsEmpty() bool {
	return len(c.attrs) == 0
}

// ToDo Remove this method once `resource` and `scope` are migrated to use the new AppendWithID method.

func (c *Attributes16Accumulator) Append(attrs pcommon.Map) (int64, error) {
	ID := c.attrsMapCount

	if attrs.Len() == 0 {
		return -1, nil
	}

	if c.attrsMapCount == math.MaxUint16 {
		panic("The maximum number of group of attributes has been reached (max is uint16).")
	}

	attrs.Range(func(k string, v pcommon.Value) bool {
		c.attrs = append(c.attrs, Attr16{
			ParentID: ID,
			Key:      k,
			Value:    v,
		})
		return true
	})

	c.attrsMapCount++

	return int64(ID), nil
}

func (c *Attributes16Accumulator) AppendWithID(parentID uint16, attrs pcommon.Map) error {
	if attrs.Len() == 0 {
		return nil
	}

	if c.attrsMapCount == math.MaxUint16 {
		panic("The maximum number of group of attributes has been reached (max is uint16).")
	}

	attrs.Range(func(key string, v pcommon.Value) bool {
		if key == "" {
			// Skip entries with empty keys
			return true
		}

		c.attrs = append(c.attrs, Attr16{
			ParentID: parentID,
			Key:      key,
			Value:    v,
		})

		return true
	})

	c.attrsMapCount++

	return nil
}

// ToDo Remove this method once shared attributes are removed logs and traces.

func (c *Attributes16Accumulator) AppendUniqueAttributesWithID(parentID uint16, attrs pcommon.Map, smattrs *common.SharedAttributes, mattrs *common.SharedAttributes) error {
	uniqueAttrsCount := attrs.Len()
	if smattrs != nil {
		uniqueAttrsCount -= smattrs.Len()
	}
	if mattrs != nil {
		uniqueAttrsCount -= mattrs.Len()
	}

	if uniqueAttrsCount == 0 {
		return nil
	}

	if c.attrsMapCount == math.MaxUint16 {
		panic("The maximum number of group of attributes has been reached (max is uint16).")
	}

	attrs.Range(func(key string, v pcommon.Value) bool {
		if key == "" {
			// Skip entries with empty keys
			return true
		}

		// Skip the current attribute if it is a scope metric shared attribute
		// or a metric shared attribute
		smattrsFound := false
		mattrsFound := false
		if smattrs != nil {
			_, smattrsFound = smattrs.Attributes[key]
		}
		if mattrs != nil {
			_, mattrsFound = mattrs.Attributes[key]
		}
		if smattrsFound || mattrsFound {
			return true
		}

		c.attrs = append(c.attrs, Attr16{
			ParentID: parentID,
			Key:      key,
			Value:    v,
		})

		uniqueAttrsCount--
		return uniqueAttrsCount > 0
	})

	c.attrsMapCount++

	return nil
}

// Sort sorts the attributes based on the provided sorter.
// The sorter is part of the global configuration and can be different for
// different payload types.
func (c *Attributes16Accumulator) Sort() {
	c.sorter.Sort(c.attrs)
	c.sorter.Reset()
}

func (c *Attributes16Accumulator) Reset() {
	c.attrsMapCount = 0
	c.attrs = c.attrs[:0]
}

func NewAttributes32Accumulator(sorter Attrs32Sorter) *Attributes32Accumulator {
	return &Attributes32Accumulator{
		attrs:  make([]Attr32, 0),
		sorter: sorter,
	}
}

func (c *Attributes32Accumulator) IsEmpty() bool {
	return len(c.attrs) == 0
}

func (c *Attributes32Accumulator) Append(ID uint32, attrs pcommon.Map) error {
	if attrs.Len() == 0 {
		return nil
	}

	if c.attrsMapCount == math.MaxUint32 {
		panic("The maximum number of group of attributes has been reached (max is uint32).")
	}

	attrs.Range(func(key string, v pcommon.Value) bool {
		if key == "" {
			// Skip entries with empty keys
			return true
		}

		c.attrs = append(c.attrs, Attr32{
			ParentID: ID,
			Key:      key,
			Value:    v,
		})

		return true
	})

	c.attrsMapCount++

	return nil
}

func (c *Attributes32Accumulator) AppendUniqueAttributesWithID(ID uint32, attrs pcommon.Map, smattrs *common.SharedAttributes, mattrs *common.SharedAttributes) error {
	uniqueAttrsCount := attrs.Len()
	if smattrs != nil {
		uniqueAttrsCount -= smattrs.Len()
	}
	if mattrs != nil {
		uniqueAttrsCount -= mattrs.Len()
	}

	if uniqueAttrsCount == 0 {
		return nil
	}

	if c.attrsMapCount == math.MaxUint32 {
		panic("The maximum number of group of attributes has been reached (max is uint32).")
	}

	attrs.Range(func(key string, v pcommon.Value) bool {
		if key == "" {
			// Skip entries with empty keys
			return true
		}

		// Skip the current attribute if it is a scope metric shared attribute
		// or a metric shared attribute
		smattrsFound := false
		mattrsFound := false
		if smattrs != nil {
			_, smattrsFound = smattrs.Attributes[key]
		}
		if mattrs != nil {
			_, mattrsFound = mattrs.Attributes[key]
		}
		if smattrsFound || mattrsFound {
			return true
		}

		c.attrs = append(c.attrs, Attr32{
			ParentID: ID,
			Key:      key,
			Value:    v,
		})

		uniqueAttrsCount--
		return uniqueAttrsCount > 0
	})

	c.attrsMapCount++

	return nil
}

// Sort sorts the attributes based on the provided sorter.
// The sorter is part of the global configuration and can be different for
// different payload types.
func (c *Attributes32Accumulator) Sort() {
	c.sorter.Sort(c.attrs)
	c.sorter.Reset()
}

func (c *Attributes32Accumulator) Reset() {
	c.attrsMapCount = 0
	c.attrs = c.attrs[:0]
}

func Equal(a, b pcommon.Value) bool {
	switch a.Type() {
	case pcommon.ValueTypeInt:
		if b.Type() == pcommon.ValueTypeInt {
			return a.Int() == b.Int()
		} else {
			return false
		}
	case pcommon.ValueTypeDouble:
		if b.Type() == pcommon.ValueTypeDouble {
			return a.Double() == b.Double()
		} else {
			return false
		}
	case pcommon.ValueTypeBool:
		if b.Type() == pcommon.ValueTypeBool {
			return a.Bool() == b.Bool()
		} else {
			return false
		}
	case pcommon.ValueTypeStr:
		if b.Type() == pcommon.ValueTypeStr {
			return a.Str() == b.Str()
		} else {
			return false
		}
	case pcommon.ValueTypeBytes:
		if a.Type() == pcommon.ValueTypeBytes && b.Type() == pcommon.ValueTypeBytes {
			return bytes.Equal(a.Bytes().AsRaw(), b.Bytes().AsRaw())
		} else {
			return false
		}
	case pcommon.ValueTypeMap:
		return false
	case pcommon.ValueTypeSlice:
		return false
	case pcommon.ValueTypeEmpty:
		return false
	default:
		return false
	}
}

func IsLess(a, b pcommon.Value) bool {
	switch a.Type() {
	case pcommon.ValueTypeInt:
		if b.Type() == pcommon.ValueTypeInt {
			return a.Int() < b.Int()
		} else {
			return false
		}
	case pcommon.ValueTypeDouble:
		if b.Type() == pcommon.ValueTypeDouble {
			return a.Double() < b.Double()
		} else {
			return false
		}
	case pcommon.ValueTypeBool:
		return a.Bool() == true && b.Bool() == false
	case pcommon.ValueTypeStr:
		if b.Type() == pcommon.ValueTypeStr {
			return a.Str() < b.Str()
		} else {
			return false
		}
	case pcommon.ValueTypeBytes:
		if a.Type() == pcommon.ValueTypeBytes && b.Type() == pcommon.ValueTypeBytes {
			return bytes.Compare(a.Bytes().AsRaw(), b.Bytes().AsRaw()) < 0
		} else {
			return false
		}
	case pcommon.ValueTypeMap:
		return false
	case pcommon.ValueTypeSlice:
		return false
	case pcommon.ValueTypeEmpty:
		return false
	default:
		return false
	}
}

func Compare(a, b pcommon.Value) int {
	switch a.Type() {
	case pcommon.ValueTypeInt:
		aI := a.Int()
		if b.Type() == pcommon.ValueTypeInt {
			bI := b.Int()
			if aI == bI {
				return 0
			} else if aI < bI {
				return -1
			} else {
				return 1
			}
		} else {
			return 1
		}
	case pcommon.ValueTypeDouble:
		aD := a.Double()
		if b.Type() == pcommon.ValueTypeDouble {
			bD := b.Double()
			if aD == bD {
				return 0
			} else if aD < bD {
				return -1
			} else {
				return 1
			}
		} else {
			return 1
		}
	case pcommon.ValueTypeBool:
		aB := a.Bool()
		bB := b.Bool()
		if aB == bB {
			return 0
		} else if aB == true && bB == false {
			return 1
		} else {
			return -1
		}
	case pcommon.ValueTypeStr:
		if b.Type() == pcommon.ValueTypeStr {
			return strings.Compare(a.Str(), b.Str())
		} else {
			return 1
		}
	case pcommon.ValueTypeBytes:
		if a.Type() == pcommon.ValueTypeBytes && b.Type() == pcommon.ValueTypeBytes {
			return bytes.Compare(a.Bytes().AsRaw(), b.Bytes().AsRaw())
		} else {
			return 1
		}
	case pcommon.ValueTypeMap:
		return 1
	case pcommon.ValueTypeSlice:
		return 1
	case pcommon.ValueTypeEmpty:
		return 1
	default:
		return 1
	}
}
