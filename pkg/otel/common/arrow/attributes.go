package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pcommon"
)

// Arrow data types used to build the attribute map.
var (
	// KDT is the Arrow key data type.
	KDT = DictU16String

	// AttributesDT is the Arrow attribute data type.
	AttributesDT = arrow.MapOf(KDT, AnyValueDT)
)

// AttributesBuilder is a helper to build a map of attributes.
type AttributesBuilder struct {
	released bool

	builder *array.MapBuilder              // map builder
	kb      *array.BinaryDictionaryBuilder // key builder
	ib      *AnyValueBuilder               // item any value builder
}

// NewAttributesBuilder creates a new AttributesBuilder with a given allocator.
//
// Once the builder is no longer needed, Build() or Release() must be called to free the
// memory allocated by the builder.
func NewAttributesBuilder(pool memory.Allocator) *AttributesBuilder {
	mb := array.NewMapBuilder(pool, KDT, AnyValueDT, false)
	return AttributesBuilderFrom(mb)
}

// AttributesBuilderFrom creates a new AttributesBuilder from an existing MapBuilder.
func AttributesBuilderFrom(mb *array.MapBuilder) *AttributesBuilder {
	ib := AnyValueBuilderFrom(mb.ItemBuilder().(*array.SparseUnionBuilder))

	return &AttributesBuilder{
		released: false,
		builder:  mb,
		kb:       mb.KeyBuilder().(*array.BinaryDictionaryBuilder),
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

	if attrs.Len() == 0 {
		b.append0Attrs()
		return nil
	}
	b.appendNAttrs(attrs.Len())

	var err error
	attrs.Range(func(key string, v pcommon.Value) bool {
		// Append the key
		err := b.kb.AppendString(key)
		if err != nil {
			return false
		}

		// Append the value
		err = b.ib.Append(v)
		if err != nil {
			return false
		}
		return true
	})
	return err
}

// Release releases the memory allocated by the builder.
func (b *AttributesBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.kb.Release()
		b.ib.Release()

		b.released = true
	}
}

// appendNAttrs appends a new set of key-value pairs to the builder.
func (b *AttributesBuilder) appendNAttrs(count int) {
	b.builder.Append(true)
	b.builder.Reserve(count)
}

// append0Attrs appends an empty set of key-value pairs to the builder.
func (b *AttributesBuilder) append0Attrs() {
	b.builder.AppendNull()
}
