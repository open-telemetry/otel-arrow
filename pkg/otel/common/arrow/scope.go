package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ScopeDT is the Arrow Data Type describing a scope.
var (
	ScopeDT = arrow.StructOf([]arrow.Field{
		{Name: constants.NAME, Type: DictU16String},
		{Name: constants.VERSION, Type: DictU16String},
		{Name: constants.ATTRIBUTES, Type: AttributesDT},
		{Name: constants.DROPPED_ATTRIBUTES_COUNT, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

type ScopeBuilder struct {
	released bool
	builder  *array.StructBuilder
	nb       *array.BinaryDictionaryBuilder // Name builder
	vb       *array.BinaryDictionaryBuilder // Version builder
	ab       *AttributesBuilder             // Attributes builder
	dacb     *array.Uint32Builder           // Dropped attributes count builder
}

// NewScopeBuilder creates a new instrumentation scope array builder with a given allocator.
func NewScopeBuilder(pool memory.Allocator) *ScopeBuilder {
	return ScopeBuilderFrom(array.NewStructBuilder(pool, ScopeDT))
}

// ScopeBuilderFrom creates a new instrumentation scope array builder from an existing struct builder.
func ScopeBuilderFrom(sb *array.StructBuilder) *ScopeBuilder {
	return &ScopeBuilder{
		released: false,
		builder:  sb,
		nb:       sb.FieldBuilder(0).(*array.BinaryDictionaryBuilder),
		vb:       sb.FieldBuilder(1).(*array.BinaryDictionaryBuilder),
		ab:       AttributesBuilderFrom(sb.FieldBuilder(2).(*array.MapBuilder)),
		dacb:     sb.FieldBuilder(3).(*array.Uint32Builder),
	}
}

// Append appends a new instrumentation scope to the builder.
func (b *ScopeBuilder) Append(resource pcommon.InstrumentationScope) error {
	if b.released {
		return fmt.Errorf("scope builder already released")
	}

	b.builder.Append(true)
	name := resource.Name()
	if name == "" {
		b.nb.AppendNull()
	} else {
		if err := b.nb.AppendString(name); err != nil {
			return err
		}
	}
	version := resource.Version()
	if version == "" {
		b.vb.AppendNull()
	} else {
		if err := b.vb.AppendString(version); err != nil {
			return err
		}
	}
	if err := b.ab.Append(resource.Attributes()); err != nil {
		return err
	}
	b.dacb.Append(resource.DroppedAttributesCount())
	return nil
}

// Build builds the instrumentation scope array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ScopeBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("scope builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the memory allocated by the builder.
func (b *ScopeBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.nb.Release()
		b.vb.Release()
		b.ab.Release()
		b.dacb.Release()

		b.released = true
	}
}
