package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

var (
	// UnivariateEHistogramDT is the Arrow Data Type describing a univariate exponential histogram.
	UnivariateEHistogramDT = arrow.StructOf(
		arrow.Field{Name: constants.DATA_POINTS, Type: arrow.ListOf(UnivariateEHistogramDataPointDT)},
		arrow.Field{Name: constants.AGGREGATION_TEMPORALITY, Type: arrow.PrimitiveTypes.Int32},
	)
)

// UnivariateEHistogramBuilder is a builder for exponential histogram metrics.
type UnivariateEHistogramBuilder struct {
	released bool

	builder *array.StructBuilder

	hdplb *array.ListBuilder          // data_points builder
	hdpb  *EHistogramDataPointBuilder // exponential histogram data point builder
	atb   *array.Int32Builder         // aggregation_temporality builder
}

// NewUnivariateEHistogramBuilder creates a new UnivariateEHistogramBuilder with a given memory allocator.
func NewUnivariateEHistogramBuilder(pool memory.Allocator) *UnivariateEHistogramBuilder {
	return UnivariateEHistogramBuilderFrom(array.NewStructBuilder(pool, UnivariateEHistogramDT))
}

// UnivariateEHistogramBuilderFrom creates a new UnivariateEHistogramBuilder from an existing StructBuilder.
func UnivariateEHistogramBuilderFrom(arr *array.StructBuilder) *UnivariateEHistogramBuilder {
	return &UnivariateEHistogramBuilder{
		released: false,
		builder:  arr,

		hdplb: arr.FieldBuilder(0).(*array.ListBuilder),
		hdpb:  EHistogramDataPointBuilderFrom(arr.FieldBuilder(0).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		atb:   arr.FieldBuilder(1).(*array.Int32Builder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateEHistogramBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateEHistogramBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory used by the builder.
func (b *UnivariateEHistogramBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new histogram to the builder.
func (b *UnivariateEHistogramBuilder) Append(eh pmetric.ExponentialHistogram, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("UnivariateEHistogramBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	dps := eh.DataPoints()
	dpc := dps.Len()
	if dpc > 0 {
		b.hdplb.Append(true)
		b.hdplb.Reserve(dpc)
		for i := 0; i < dpc; i++ {
			if err := b.hdpb.Append(dps.At(i), smdata, mdata); err != nil {
				return err
			}
		}
	} else {
		b.hdplb.Append(false)
	}
	if eh.AggregationTemporality() == pmetric.AggregationTemporalityUnspecified {
		b.atb.AppendNull()
	} else {
		b.atb.Append(int32(eh.AggregationTemporality()))
	}

	return nil
}

func (b *UnivariateEHistogramBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.Append(false)
}
