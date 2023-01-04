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

package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ResourceDT is the Arrow Data Type describing a resource.
var (
	ResourceDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Attributes, Type: AttributesDT},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32},
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

		b.released = true
	}
}
