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
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

var (
	ResourceLogsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Resource, Type: acommon.ResourceDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.ScopeLogs, Type: arrow.ListOf(ScopeLogsDT)},
	}...)
)

// ResourceLogsBuilder is a helper to build resource logs.
type ResourceLogsBuilder struct {
	released bool

	builder *builder.StructBuilder // builder for the resource logs struct

	rb   *acommon.ResourceBuilder // resource builder
	schb *builder.StringBuilder   // schema url builder
	slsb *builder.ListBuilder     // scope logs list builder
	slb  *ScopeLogsBuilder        // scope logs builder
}

// ResourceLogsBuilderFrom creates a new ResourceLogsBuilder from an existing builder.
func ResourceLogsBuilderFrom(builder *builder.StructBuilder) *ResourceLogsBuilder {
	scopeLogs := builder.ListBuilder(constants.ScopeLogs)
	return &ResourceLogsBuilder{
		released: false,
		builder:  builder,
		rb:       acommon.ResourceBuilderFrom(builder.StructBuilder(constants.Resource)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
		slsb:     scopeLogs,
		slb:      ScopeLogsBuilderFrom(scopeLogs.StructBuilder()),
	}
}

// Build builds the resource logs array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ResourceLogsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("resource logs builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new resource logs to the builder.
func (b *ResourceLogsBuilder) Append(rs plog.ResourceLogs) error {
	if b.released {
		return fmt.Errorf("resource logs builder already released")
	}

	return b.builder.Append(rs, func() error {
		if err := b.rb.Append(rs.Resource()); err != nil {
			return err
		}
		b.schb.AppendNonEmpty(rs.SchemaUrl())
		slogs := rs.ScopeLogs()
		sc := slogs.Len()
		return b.slsb.Append(sc, func() error {
			for i := 0; i < sc; i++ {
				if err := b.slb.Append(slogs.At(i)); err != nil {
					return err
				}
			}
			return nil
		})
	})
}

// Release releases the memory allocated by the builder.
func (b *ResourceLogsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
