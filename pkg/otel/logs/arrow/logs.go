package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"

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

	builder *array.ListBuilder   // resource logs list builder
	rlp     *ResourceLogsBuilder // resource logs builder
}

// NewLogsBuilder creates a new LogsBuilder with a given allocator.
func NewLogsBuilder(pool memory.Allocator) *LogsBuilder {
	rlb := array.NewListBuilder(pool, ResourceLogsDT)
	return &LogsBuilder{
		released: false,
		builder:  rlb,
		rlp:      ResourceLogsBuilderFrom(rlb.ValueBuilder().(*array.StructBuilder)),
	}
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

	arr := b.builder.NewArray()
	defer arr.Release()
	return array.NewRecord(Schema, []arrow.Array{arr}, int64(arr.Len())), nil
}

// Append appends a new set of resource logs to the builder.
func (b *LogsBuilder) Append(logs plog.Logs) error {
	if b.released {
		return fmt.Errorf("traces builder already released")
	}

	rl := logs.ResourceLogs()
	rc := rl.Len()
	if rc > 0 {
		b.builder.Append(true)
		b.builder.Reserve(rc)
		for i := 0; i < rc; i++ {
			if err := b.rlp.Append(rl.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.builder.AppendNull()
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
