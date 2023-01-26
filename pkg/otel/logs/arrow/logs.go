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
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// Schema is the Arrow schema for the OTLP Arrow Logs record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceLogs, Type: arrow.ListOf(ResourceLogsDT)},
	}, nil)
)

// LogsBuilder is a helper to build a list of resource logs.
type LogsBuilder struct {
	released bool

	schema  *acommon.AdaptiveSchema // Trace schema
	builder *array.RecordBuilder    // Record builder
	rlb     *array.ListBuilder      // ResourceLogs list builder
	rlp     *ResourceLogsBuilder    // resource logs builder
}

// NewLogsBuilder creates a new LogsBuilder with a given allocator.
func NewLogsBuilder(schema *acommon.AdaptiveSchema) (*LogsBuilder, error) {
	builder := &LogsBuilder{
		released: false,
		schema:   schema,
	}
	if err := builder.init(); err != nil {
		return nil, err
	}
	return builder, nil
}

func (b *LogsBuilder) init() error {
	if b.builder != nil {
		b.builder.Release()
	}

	b.builder = b.schema.RecordBuilder()
	rlb, ok := b.builder.Field(0).(*array.ListBuilder)
	if !ok {
		return fmt.Errorf("expected field 0 to be a list builder, got %T", b.builder.Field(0))
	}
	b.rlb = rlb
	b.rlp = ResourceLogsBuilderFrom(rlb.ValueBuilder().(*array.StructBuilder))
	return nil
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
func (b *LogsBuilder) Build() (arrow.Record, error) {
	if b.released {
		return nil, fmt.Errorf("resource logs builder already released")
	}

	record := b.builder.NewRecord()

	overflowDetected, updates := b.schema.Analyze(record)
	if overflowDetected {
		// Dictionary overflow detected =>
		// * Update the schema
		// * Reinitialize the trace builder

		// The existing record is no longer valid, release it.
		// A new record will be built after the schema update.
		record.Release()

		// Build a list of fields that overflowed
		var fieldNames []string
		for _, update := range updates {
			fieldNames = append(fieldNames, update.DictPath)
		}

		b.schema.UpdateSchema(updates)
		if err := b.init(); err != nil {
			return nil, err
		}

		return nil, &acommon.DictionaryOverflowError{FieldNames: fieldNames}
	}

	return record, nil
}

// Append appends a new set of resource logs to the builder.
func (b *LogsBuilder) Append(logs plog.Logs) error {
	if b.released {
		return fmt.Errorf("traces builder already released")
	}

	rl := logs.ResourceLogs()
	rc := rl.Len()
	if rc > 0 {
		b.rlb.Append(true)
		b.builder.Reserve(rc)
		for i := 0; i < rc; i++ {
			if err := b.rlp.Append(rl.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.rlb.AppendNull()
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *LogsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
