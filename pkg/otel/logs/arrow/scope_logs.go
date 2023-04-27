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
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow_old"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// ScopeLogsDT is the Arrow Data Type describing a scope span.
var (
	ScopeLogsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Scope, Type: acommon.ScopeDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.Logs, Type: arrow.ListOf(LogRecordDT), Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// ScopeLogsBuilder is a helper to build a scope logs.
type ScopeLogsBuilder struct {
	released bool

	builder *builder.StructBuilder

	scb  *acommon.ScopeBuilder  // scope builder
	schb *builder.StringBuilder // schema url builder
	lrsb *builder.ListBuilder   // log recprd list builder
	lrb  *LogRecordBuilder      // log record builder
}

func ScopeLogsBuilderFrom(builder *builder.StructBuilder) *ScopeLogsBuilder {
	logs := builder.ListBuilder(constants.Logs)
	return &ScopeLogsBuilder{
		released: false,
		builder:  builder,
		scb:      acommon.ScopeBuilderFrom(builder.StructBuilder(constants.Scope)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
		lrsb:     logs,
		lrb:      LogRecordBuilderFrom(logs.StructBuilder()),
	}
}

// Build builds the scope logs array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ScopeLogsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new scope logs to the builder.
func (b *ScopeLogsBuilder) Append(slg *ScopeLogGroup) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(slg, func() error {
		if err := b.scb.Append(slg.Scope); err != nil {
			return werror.Wrap(err)
		}
		b.schb.AppendNonEmpty(slg.ScopeSchemaUrl)
		lrc := len(slg.Logs)
		return b.lrsb.Append(lrc, func() error {
			for i := 0; i < lrc; i++ {
				if err := b.lrb.Append(slg.Logs[i]); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		})
	})
}

// Release releases the memory allocated by the builder.
func (b *ScopeLogsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
