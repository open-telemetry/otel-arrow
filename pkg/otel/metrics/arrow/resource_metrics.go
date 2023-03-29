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
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

var (
	ResourceMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Resource, Type: acommon.ResourceDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.ScopeMetrics, Type: arrow.ListOf(ScopeMetricsDT), Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// ResourceMetricsBuilder is a helper to build resource metrics.
type ResourceMetricsBuilder struct {
	released bool

	builder *builder.StructBuilder // builder for the resource metrics struct

	rb   *acommon.ResourceBuilder // resource builder
	schb *builder.StringBuilder   // schema url builder
	spsb *builder.ListBuilder     // scope metrics list builder
	smb  *ScopeMetricsBuilder     // scope metrics builder
}

// ResourceMetricsBuilderFrom creates a new ResourceMetricsBuilder from an existing builder.
func ResourceMetricsBuilderFrom(builder *builder.StructBuilder) *ResourceMetricsBuilder {
	spsb := builder.ListBuilder(constants.ScopeMetrics)
	return &ResourceMetricsBuilder{
		released: false,
		builder:  builder,
		rb:       acommon.ResourceBuilderFrom(builder.StructBuilder(constants.Resource)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
		spsb:     spsb,
		smb:      ScopeMetricsBuilderFrom(spsb.StructBuilder()),
	}
}

// Build builds the resource metrics array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ResourceMetricsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new resource metrics to the builder.
func (b *ResourceMetricsBuilder) Append(rmg *ResourceMetricsGroup) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(rmg, func() error {
		if err := b.rb.Append(rmg.Resource); err != nil {
			return werror.Wrap(err)
		}
		b.schb.AppendNonEmpty(rmg.ResourceSchemaUrl)
		sc := len(rmg.ScopeMetrics)
		return b.spsb.Append(sc, func() error {
			for _, smg := range rmg.ScopeMetrics {
				if err := b.smb.Append(smg); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		})
	})
}

// Release releases the memory allocated by the builder.
func (b *ResourceMetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
