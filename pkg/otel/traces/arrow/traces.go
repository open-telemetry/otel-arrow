package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// Schema is the Arrow schema for the OTLP Arrow Traces record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.RESOURCE_SPANS, Type: arrow.ListOf(ResourceSpansDT)},
	}, nil)
)

// TracesBuilder is a helper to build a list of resource spans.
type TracesBuilder struct {
	released bool

	builder *array.ListBuilder    // resource spans list builder
	rsp     *ResourceSpansBuilder // resource spans builder
}

// NewTracesBuilder creates a new TracesBuilder with a given allocator.
func NewTracesBuilder(pool memory.Allocator) *TracesBuilder {
	rsb := array.NewListBuilder(pool, ResourceSpansDT)
	return &TracesBuilder{
		released: false,
		builder:  rsb,
		rsp:      ResourceSpansBuilderFrom(rsb.ValueBuilder().(*array.StructBuilder)),
	}
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
func (b *TracesBuilder) Build() (arrow.Record, error) {
	if b.released {
		return nil, fmt.Errorf("resource spans builder already released")
	}

	defer b.Release()

	arr := b.builder.NewArray()
	defer arr.Release()
	return array.NewRecord(Schema, []arrow.Array{arr}, int64(arr.Len())), nil
}

// Append appends a new set of resource spans to the builder.
func (b *TracesBuilder) Append(traces ptrace.Traces) error {
	if b.released {
		return fmt.Errorf("traces builder already released")
	}

	rs := traces.ResourceSpans()
	rc := rs.Len()
	if rc > 0 {
		b.builder.Append(true)
		b.builder.Reserve(rc)
		for i := 0; i < rc; i++ {
			if err := b.rsp.Append(rs.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.builder.AppendNull()
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *TracesBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.rsp.Release()
		b.released = true
	}
}
