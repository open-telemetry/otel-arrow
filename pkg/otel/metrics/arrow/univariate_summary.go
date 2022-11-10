package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateSummaryDT is the Arrow Data Type describing a univariate summary.
var (
	UnivariateSummaryDT = arrow.StructOf(
		arrow.Field{Name: constants.DATA_POINTS, Type: arrow.ListOf(UnivariateSummaryDataPointDT)},
	)
)

// UnivariateSummaryBuilder is a builder for summary metrics.
type UnivariateSummaryBuilder struct {
	released bool

	builder *array.StructBuilder

	dplb *array.ListBuilder                 // data points builder
	dpb  *UnivariateSummaryDataPointBuilder // summary data point builder
}

// NewUnivariateSummaryBuilder creates a new UnivariateSummaryBuilder with a given memory allocator.
func NewUnivariateSummaryBuilder(pool memory.Allocator) *UnivariateSummaryBuilder {
	return UnivariateSummaryBuilderFrom(array.NewStructBuilder(pool, UnivariateSummaryDT))
}

// UnivariateSummaryBuilderFrom creates a new UnivariateSummaryBuilder from an existing StructBuilder.
func UnivariateSummaryBuilderFrom(ndpb *array.StructBuilder) *UnivariateSummaryBuilder {
	return &UnivariateSummaryBuilder{
		released: false,
		builder:  ndpb,

		dplb: ndpb.FieldBuilder(0).(*array.ListBuilder),
		dpb:  UnivariateSummaryDataPointBuilderFrom(ndpb.FieldBuilder(0).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateSummaryBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateSummaryBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *UnivariateSummaryBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new univariate summary to the builder.
func (b *UnivariateSummaryBuilder) Append(summary pmetric.Summary) error {
	if b.released {
		return fmt.Errorf("UnivariateSummaryBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	dps := summary.DataPoints()
	dpc := dps.Len()
	if dpc > 0 {
		b.dplb.Append(true)
		for i := 0; i < dpc; i++ {
			if err := b.dpb.Append(dps.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.dplb.Append(false)
	}

	return nil
}

func (b *UnivariateSummaryBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.Append(false)
}
