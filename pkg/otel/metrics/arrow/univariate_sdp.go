package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"github.com/apache/arrow/go/v10/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateSummaryDataPointDT is the Arrow Data Type describing a univariate summary data point.
var (
	UnivariateSummaryDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.ATTRIBUTES, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.START_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.SUMMARY_COUNT, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.SUMMARY_SUM, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.SUMMARY_QUANTILE_VALUES, Type: arrow.ListOf(QuantileValueDT)},
		arrow.Field{Name: constants.FLAGS, Type: arrow.PrimitiveTypes.Uint32},
	)
)

// UnivariateSummaryDataPointBuilder is a builder for a summary data point.
type UnivariateSummaryDataPointBuilder struct {
	released bool

	builder *array.StructBuilder

	ab    *acommon.AttributesBuilder // attributes builder
	stunb *array.Uint64Builder       // start_time_unix_nano builder
	tunb  *array.Uint64Builder       // time_unix_nano builder
	scb   *array.Uint64Builder       // count builder
	ssb   *array.Float64Builder      // sum builder
	qvlb  *array.ListBuilder         // summary quantile value list builder
	qvb   *QuantileValueBuilder      // summary quantile value builder
	fb    *array.Uint32Builder       // flags builder
}

// NewUnivariateSummaryDataPointBuilder creates a new UnivariateSummaryDataPointBuilder with a given memory allocator.
func NewUnivariateSummaryDataPointBuilder(pool memory.Allocator) *UnivariateSummaryDataPointBuilder {
	return UnivariateSummaryDataPointBuilderFrom(array.NewStructBuilder(pool, UnivariateSummaryDataPointDT))
}

// UnivariateSummaryDataPointBuilderFrom creates a new UnivariateSummaryDataPointBuilder from an existing StructBuilder.
func UnivariateSummaryDataPointBuilderFrom(ndpb *array.StructBuilder) *UnivariateSummaryDataPointBuilder {
	return &UnivariateSummaryDataPointBuilder{
		released: false,
		builder:  ndpb,

		ab:    acommon.AttributesBuilderFrom(ndpb.FieldBuilder(0).(*array.MapBuilder)),
		stunb: ndpb.FieldBuilder(1).(*array.Uint64Builder),
		tunb:  ndpb.FieldBuilder(2).(*array.Uint64Builder),
		scb:   ndpb.FieldBuilder(3).(*array.Uint64Builder),
		ssb:   ndpb.FieldBuilder(4).(*array.Float64Builder),
		qvlb:  ndpb.FieldBuilder(5).(*array.ListBuilder),
		qvb:   QuantileValueBuilderFrom(ndpb.FieldBuilder(5).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		fb:    ndpb.FieldBuilder(6).(*array.Uint32Builder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateSummaryDataPointBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateSummaryDataPointBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *UnivariateSummaryDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new summary data point to the builder.
func (b *UnivariateSummaryDataPointBuilder) Append(sdp pmetric.SummaryDataPoint, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("UnivariateSummaryDataPointBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	if err := b.ab.AppendUniqueAttributes(sdp.Attributes(), smdata.Attributes, mdata.Attributes); err != nil {
		return err
	}

	if smdata.StartTime == nil && mdata.StartTime == nil {
		b.stunb.Append(uint64(sdp.StartTimestamp()))
	} else {
		b.stunb.AppendNull()
	}
	if smdata.Time == nil && mdata.Time == nil {
		b.tunb.Append(uint64(sdp.Timestamp()))
	} else {
		b.tunb.AppendNull()
	}

	b.scb.Append(sdp.Count())
	b.ssb.Append(sdp.Sum())
	qvs := sdp.QuantileValues()
	qvc := qvs.Len()
	if qvc > 0 {
		b.qvlb.Append(true)
		b.qvlb.Reserve(qvc)
		for i := 0; i < qvc; i++ {
			if err := b.qvb.Append(qvs.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.qvlb.Append(false)
	}
	if sdp.Flags() != 0 {
		b.fb.Append(uint32(sdp.Flags()))
	} else {
		b.fb.AppendNull()
	}
	return nil
}
