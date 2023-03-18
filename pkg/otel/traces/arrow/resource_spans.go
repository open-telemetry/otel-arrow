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

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

var (
	// ResourceSpansDT is the data type for resource spans.
	ResourceSpansDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Resource, Type: acommon.ResourceDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.ScopeSpans, Type: arrow.ListOf(ScopeSpansDT), Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// ResourceSpansBuilder is a helper to build resource spans.
type ResourceSpansBuilder struct {
	released bool

	builder *builder.StructBuilder // builder for the resource spans struct

	rb   *acommon.ResourceBuilder // `resource` builder
	schb *builder.StringBuilder   // `schema_url` builder
	spsb *builder.ListBuilder     // `scope_spans` list builder
	spb  *ScopeSpansBuilder       // `scope_span` builder
}

// ResourceSpansBuilderFrom creates a new ResourceSpansBuilder from an existing builder.
func ResourceSpansBuilderFrom(builder *builder.StructBuilder) *ResourceSpansBuilder {
	spsb := builder.ListBuilder(constants.ScopeSpans)

	return &ResourceSpansBuilder{
		released: false,
		builder:  builder,
		rb:       acommon.ResourceBuilderFrom(builder.StructBuilder(constants.Resource)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
		spsb:     spsb,
		spb:      ScopeSpansBuilderFrom(spsb.StructBuilder()),
	}
}

// Build builds the resource spans array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ResourceSpansBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("resource spans builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new resource spans to the builder.
func (b *ResourceSpansBuilder) Append(ss ptrace.ResourceSpans) error {
	if b.released {
		return fmt.Errorf("resource spans builder already released")
	}

	return b.builder.Append(ss, func() error {
		if err := b.rb.Append(ss.Resource()); err != nil {
			return err
		}
		b.schb.AppendNonEmpty(ss.SchemaUrl())
		sspans := ss.ScopeSpans()
		sc := sspans.Len()
		return b.spsb.Append(sc, func() error {
			for i := 0; i < sc; i++ {
				if err := b.spb.Append(sspans.At(i)); err != nil {
					return err
				}
			}
			return nil
		})
	})
}

// Release releases the memory allocated by the builder.
func (b *ResourceSpansBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
