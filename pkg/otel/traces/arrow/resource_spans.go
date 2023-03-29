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
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
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
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new resource spans to the builder.
func (b *ResourceSpansBuilder) Append(rsg *carrow.ResourceSpanGroup) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(rsg, func() error {
		if err := b.rb.Append(rsg.Resource); err != nil {
			return werror.Wrap(err)
		}
		b.schb.AppendNonEmpty(rsg.ResourceSchemaUrl)
		sc := len(rsg.ScopeSpans)
		return b.spsb.Append(sc, func() error {
			for _, spg := range rsg.ScopeSpans {
				if err := b.spb.Append(spg); err != nil {
					return werror.Wrap(err)
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
