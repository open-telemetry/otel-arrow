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

// ScopeLogsDT is the Arrow Data Type describing a scope span.
var (
	ScopeLogsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Scope, Type: acommon.ScopeDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
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
		return nil, fmt.Errorf("scope logs builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new scope logs to the builder.
func (b *ScopeLogsBuilder) Append(sl plog.ScopeLogs) error {
	if b.released {
		return fmt.Errorf("scope logs builder already released")
	}

	return b.builder.Append(sl, func() error {
		if err := b.scb.Append(sl.Scope()); err != nil {
			return err
		}
		b.schb.AppendNonEmpty(sl.SchemaUrl())
		logRecords := sl.LogRecords()
		lrc := logRecords.Len()
		return b.lrsb.Append(lrc, func() error {
			for i := 0; i < lrc; i++ {
				if err := b.lrb.Append(logRecords.At(i)); err != nil {
					return err
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
