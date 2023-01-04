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
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// Schema is the Arrow schema for the OTLP Arrow Traces record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceSpans, Type: arrow.ListOf(ResourceSpansDT)},
	}, nil)
)

// TracesBuilder is a helper to build a list of resource spans.
type TracesBuilder struct {
	released bool

	schema  *acommon.AdaptiveSchema // Trace schema
	builder *array.RecordBuilder    // Record builder
	rsb     *array.ListBuilder      // Resource spans builder
	rsp     *ResourceSpansBuilder   // resource spans builder
}

// NewTracesBuilder creates a new TracesBuilder with a given allocator.
func NewTracesBuilder(pool memory.Allocator, schema *acommon.AdaptiveSchema) (*TracesBuilder, error) {
	builder := array.NewRecordBuilder(pool, schema.Schema())
	err := schema.InitDictionaryBuilders(builder)
	if err != nil {
		return nil, err
	}
	rsb, ok := builder.Field(0).(*array.ListBuilder)
	if !ok {
		return nil, fmt.Errorf("expected field 0 to be a list builder, got %T", builder.Field(0))
	}
	return &TracesBuilder{
		released: false,
		schema:   schema,
		builder:  builder,
		rsb:      rsb,
		rsp:      ResourceSpansBuilderFrom(rsb.ValueBuilder().(*array.StructBuilder)),
	}, nil
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
//
// This method returns a DictionaryOverflowError if the cardinality of a dictionary
// (or several) exceeds the maximum allowed value.
func (b *TracesBuilder) Build() (arrow.Record, error) {
	if b.released {
		return nil, fmt.Errorf("resource spans builder already released")
	}

	defer b.Release()

	record := b.builder.NewRecord()

	overflowDetected, updates := b.schema.Analyze(record)
	if overflowDetected {
		record.Release()

		// Build a list of fields that overflowed
		var fieldNames []string
		for _, update := range updates {
			fieldNames = append(fieldNames, update.DictPath)
		}

		b.schema.UpdateSchema(updates)

		return nil, &acommon.DictionaryOverflowError{FieldNames: fieldNames}
	}

	return record, nil
}

// Append appends a new set of resource spans to the builder.
func (b *TracesBuilder) Append(traces ptrace.Traces) error {
	if b.released {
		return fmt.Errorf("traces builder already released")
	}

	rs := traces.ResourceSpans()
	rc := rs.Len()
	if rc > 0 {
		b.rsb.Append(true)
		b.builder.Reserve(rc)
		for i := 0; i < rc; i++ {
			if err := b.rsp.Append(rs.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.rsb.AppendNull()
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *TracesBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.released = true
	}
}
