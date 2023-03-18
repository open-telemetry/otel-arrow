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
	"fmt"
	"sort"
	"strings"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
)

// Arrow data types used to build the attribute map.
var (
	// KDT is the Arrow key data type.
	KDT = arrow.BinaryTypes.String

	// AttributesDT is the Arrow attribute data type.
	// ToDo support dictionary on keys.
	AttributesDT = arrow.MapOf(KDT, AnyValueDT)
)

// AttributesBuilder is a helper to build a map of attributes.
type AttributesBuilder struct {
	released bool

	builder *builder.MapBuilder    // map builder
	kb      *builder.StringBuilder // key builder
	ib      *AnyValueBuilder       // item any value builder
}

type (
	mapEntry struct {
		key   string
		value pcommon.Value
	}

	mapEntries []mapEntry
)

var _ sort.Interface = mapEntries{}

func (me mapEntries) Len() int {
	return len(me)
}

func (me mapEntries) Swap(i, j int) {
	me[i], me[j] = me[j], me[i]
}

func (me mapEntries) Less(i, j int) bool {
	return me[i].key < me[j].key
}

func AttributesId(attrs pcommon.Map) string {
	tmp := make(mapEntries, 0, attrs.Len())
	attrs.Range(func(k string, v pcommon.Value) bool {
		tmp = append(tmp, mapEntry{
			key:   k,
			value: v,
		})
		return true
	})
	sort.Stable(tmp)

	var attrsId strings.Builder
	attrsId.WriteString("{")
	for _, e := range tmp {
		if attrsId.Len() > 1 {
			attrsId.WriteString(",")
		}
		attrsId.WriteString(e.key)
		attrsId.WriteString(":")
		attrsId.WriteString(ValueID(e.value))
	}
	attrsId.WriteString("}")
	return attrsId.String()
}

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
		return nil, fmt.Errorf("attribute builder already released")
	}

	defer b.Release()
	return b.builder.NewMapArray(), nil
}

// Append appends a new set of attributes to the builder.
// Note: empty keys are skipped.
func (b *AttributesBuilder) Append(attrs pcommon.Map) error {
	if b.released {
		return fmt.Errorf("attribute builder already released")
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
		return err
	})
}

func (b *AttributesBuilder) AppendUniqueAttributes(attrs pcommon.Map, smattrs *common.SharedAttributes, mattrs *common.SharedAttributes) error {
	if b.released {
		return fmt.Errorf("attribute builder already released")
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
			err = b.ib.Append(v)

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
