package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// Schema is the Arrow schema for the OTLP Arrow Logs record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.RESOURCE_LOGS, Type: arrow.ListOf(ResourceLogsDT)},
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
func NewLogsBuilder(pool memory.Allocator, schema *acommon.AdaptiveSchema) (*LogsBuilder, error) {
	builder := array.NewRecordBuilder(pool, schema.Schema())
	err := schema.InitDictionaryBuilders(builder)
	if err != nil {
		return nil, err
	}
	rlb, ok := builder.Field(0).(*array.ListBuilder)
	if !ok {
		return nil, fmt.Errorf("expected field 0 to be a list, got %T", builder.Field(0))
	}
	return &LogsBuilder{
		released: false,
		schema:   schema,
		builder:  builder,
		rlb:      rlb,
		rlp:      ResourceLogsBuilderFrom(rlb.ValueBuilder().(*array.StructBuilder)),
	}, nil
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
func (b *LogsBuilder) Build() (arrow.Record, error) {
	if b.released {
		return nil, fmt.Errorf("resource logs builder already released")
	}

	defer b.Release()

	record := b.builder.NewRecord()

	overflowDetected, updates := b.schema.Analyze(record)
	if overflowDetected {
		record.Release()

		// Build a list of fields that overflowed
		var fieldNames []string
		for _, update := range updates {
			fieldNames = append(fieldNames, b.schema.DictionaryPath(update.DictIdx))
		}

		b.schema.UpdateSchema(updates)

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
		b.rlp.Release()
		b.released = true
	}
}
