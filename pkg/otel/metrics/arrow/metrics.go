package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// Schema is the Arrow schema for the OTLP Arrow Metrics record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.RESOURCE_METRICS, Type: arrow.ListOf(ResourceMetricsDT)},
	}, nil)
)

// MetricsBuilder is a helper to build a list of resource metrics.
type MetricsBuilder struct {
	released bool

	builder *array.ListBuilder      // resource metrics list builder
	rmp     *ResourceMetricsBuilder // resource metrics builder
}

// NewMetricsBuilder creates a new MetricsBuilder with a given allocator.
func NewMetricsBuilder(pool memory.Allocator) *MetricsBuilder {
	rsb := array.NewListBuilder(pool, ResourceMetricsDT)
	return &MetricsBuilder{
		released: false,
		builder:  rsb,
		rmp:      ResourceMetricsBuilderFrom(rsb.ValueBuilder().(*array.StructBuilder)),
	}
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
func (b *MetricsBuilder) Build() (arrow.Record, error) {
	if b.released {
		return nil, fmt.Errorf("resource metrics builder already released")
	}

	defer b.Release()

	arr := b.builder.NewArray()
	defer arr.Release()
	return array.NewRecord(Schema, []arrow.Array{arr}, int64(arr.Len())), nil
}

// Append appends a new set of resource metrics to the builder.
func (b *MetricsBuilder) Append(metrics pmetric.Metrics) error {
	if b.released {
		return fmt.Errorf("metrics builder already released")
	}

	rm := metrics.ResourceMetrics()
	rc := rm.Len()
	if rc > 0 {
		b.builder.Append(true)
		b.builder.Reserve(rc)
		for i := 0; i < rc; i++ {
			if err := b.rmp.Append(rm.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.builder.AppendNull()
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *MetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.rmp.Release()
		b.released = true
	}
}
