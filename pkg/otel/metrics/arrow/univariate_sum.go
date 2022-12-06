package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateSumDT is the Arrow Data Type describing a univariate sum.
var (
	UnivariateSumDT = arrow.StructOf(
		arrow.Field{Name: constants.DATA_POINTS, Type: arrow.ListOf(UnivariateNumberDataPointDT)},
		arrow.Field{Name: constants.AGGREGATION_TEMPORALITY, Type: arrow.PrimitiveTypes.Int32},
		arrow.Field{Name: constants.IS_MONOTONIC, Type: arrow.FixedWidthTypes.Boolean},
	)
)

// UnivariateSumBuilder is a builder for sum metrics.
type UnivariateSumBuilder struct {
	released bool

	builder *array.StructBuilder

	dplb *array.ListBuilder      // data_points builder
	dpb  *NumberDataPointBuilder // number data point builder
	atb  *array.Int32Builder     // aggregation_temporality builder
	imb  *array.BooleanBuilder   // is_monotonic builder
}

// NewUnivariateSumBuilder creates a new UnivariateSumBuilder with a given memory allocator.
func NewUnivariateSumBuilder(pool memory.Allocator) *UnivariateSumBuilder {
	return UnivariateSumBuilderFrom(array.NewStructBuilder(pool, UnivariateSumDT))
}

// UnivariateSumBuilderFrom creates a new UnivariateSumBuilder from an existing StructBuilder.
func UnivariateSumBuilderFrom(ndpb *array.StructBuilder) *UnivariateSumBuilder {
	return &UnivariateSumBuilder{
		released: false,
		builder:  ndpb,

		dplb: ndpb.FieldBuilder(0).(*array.ListBuilder),
		dpb:  NumberDataPointBuilderFrom(ndpb.FieldBuilder(0).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		atb:  ndpb.FieldBuilder(1).(*array.Int32Builder),
		imb:  ndpb.FieldBuilder(2).(*array.BooleanBuilder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateSumBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateMetricBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *UnivariateSumBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new univariate sum to the builder.
func (b *UnivariateSumBuilder) Append(sum pmetric.Sum, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("UnivariateMetricBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	dps := sum.DataPoints()
	dpc := dps.Len()
	if dpc > 0 {
		b.dplb.Append(true)
		b.dplb.Reserve(dpc)
		for i := 0; i < dpc; i++ {
			if err := b.dpb.Append(dps.At(i), smdata, mdata); err != nil {
				return err
			}
		}
	} else {
		b.dplb.Append(false)
	}
	if sum.AggregationTemporality() == pmetric.AggregationTemporalityUnspecified {
		b.atb.AppendNull()
	} else {
		b.atb.Append(int32(sum.AggregationTemporality()))
	}
	if sum.IsMonotonic() {
		b.imb.Append(sum.IsMonotonic())
	} else {
		b.imb.AppendNull()
	}

	return nil
}

func (b *UnivariateSumBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.Append(false)
}
