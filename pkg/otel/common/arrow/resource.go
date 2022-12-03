package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ResourceDT is the Arrow Data Type describing a resource.
var (
	ResourceDT = arrow.StructOf([]arrow.Field{
		{Name: constants.ATTRIBUTES, Type: AttributesDT},
		{Name: constants.DROPPED_ATTRIBUTES_COUNT, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

// ResourceBuilder is an Arrow builder for resources.
type ResourceBuilder struct {
	released bool
	builder  *array.StructBuilder
	ab       *AttributesBuilder   // Attributes builder
	dacb     *array.Uint32Builder // Dropped attributes count builder
}

// NewResourceBuilder creates a new resource builder with a given allocator.
func NewResourceBuilder(pool memory.Allocator) *ResourceBuilder {
	return ResourceBuilderFrom(array.NewStructBuilder(pool, ResourceDT))
}

// ResourceBuilderFrom creates a new resource builder from an existing struct builder.
func ResourceBuilderFrom(rb *array.StructBuilder) *ResourceBuilder {
	return &ResourceBuilder{
		released: false,
		builder:  rb,
		ab:       AttributesBuilderFrom(rb.FieldBuilder(0).(*array.MapBuilder)),
		dacb:     rb.FieldBuilder(1).(*array.Uint32Builder),
	}
}

// Append appends a new resource to the builder.
func (b *ResourceBuilder) Append(resource pcommon.Resource) error {
	if b.released {
		return fmt.Errorf("resource builder already released")
	}

	b.builder.Append(true)
	if err := b.ab.Append(resource.Attributes()); err != nil {
		return err
	}
	if resource.DroppedAttributesCount() > 0 {
		b.dacb.Append(resource.DroppedAttributesCount())
	} else {
		b.dacb.AppendNull()
	}
	return nil
}

// Build builds the resource array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ResourceBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("attribute builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the memory allocated by the builder.
func (b *ResourceBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.ab.Release()
		b.dacb.Release()

		b.released = true
	}
}
