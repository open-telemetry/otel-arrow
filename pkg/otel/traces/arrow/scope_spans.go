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
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// ScopeSpansDT is the Arrow Data Type describing a scope span.
var (
	ScopeSpansDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Scope, Type: acommon.ScopeDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.Spans, Type: arrow.ListOf(SpanDT), Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// ScopeSpansBuilder is a helper to build a scope spans.
type ScopeSpansBuilder struct {
	released bool

	builder *builder.StructBuilder

	scb  *acommon.ScopeBuilder  // `scope` builder
	schb *builder.StringBuilder // `schema_url` builder
	ssb  *builder.ListBuilder   // `spans` list builder
	sb   *SpanBuilder           // `span` builder
}

func ScopeSpansBuilderFrom(builder *builder.StructBuilder) *ScopeSpansBuilder {
	ssb := builder.ListBuilder(constants.Spans)

	return &ScopeSpansBuilder{
		released: false,
		builder:  builder,
		scb:      acommon.ScopeBuilderFrom(builder.StructBuilder(constants.Scope)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
		ssb:      ssb,
		sb:       SpanBuilderFrom(ssb.StructBuilder()),
	}
}

// Build builds the scope spans array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ScopeSpansBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new scope spans to the builder.
func (b *ScopeSpansBuilder) Append(spg *ScopeSpanGroup) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(spg, func() error {
		if err := b.scb.Append(spg.Scope); err != nil {
			return werror.Wrap(err)
		}
		b.schb.AppendNonEmpty(spg.ScopeSchemaUrl)
		sc := len(spg.Spans)
		return b.ssb.Append(sc, func() error {
			for i := 0; i < sc; i++ {
				if err := b.sb.Append(spg.Spans[i]); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		})
	})
}

// Release releases the memory allocated by the builder.
func (b *ScopeSpansBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
