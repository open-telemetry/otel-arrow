package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateHistogramDataPointDT is the Arrow Data Type describing a univariate histogram number data point.
var (
	UnivariateHistogramDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.ATTRIBUTES, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.START_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.HISTOGRAM_COUNT, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.HISTOGRAM_SUM, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.HISTOGRAM_BUCKET_COUNTS, Type: arrow.ListOf(arrow.PrimitiveTypes.Uint64)},
		arrow.Field{Name: constants.HISTOGRAM_EXPLICIT_BOUNDS, Type: arrow.ListOf(arrow.PrimitiveTypes.Float64)},
		arrow.Field{Name: constants.EXEMPLARS, Type: arrow.ListOf(ExemplarDT)},
		arrow.Field{Name: constants.FLAGS, Type: arrow.PrimitiveTypes.Uint32},
		arrow.Field{Name: constants.HISTOGRAM_MIN, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.HISTOGRAM_MAX, Type: arrow.PrimitiveTypes.Float64},
	)
)

// HistogramDataPointBuilder is a builder for histogram data points.
type HistogramDataPointBuilder struct {
	released bool

	builder *array.StructBuilder

	ab    *acommon.AttributesBuilder // attributes builder
	stunb *array.Uint64Builder       // start_time_unix_nano builder
	tunb  *array.Uint64Builder       // time_unix_nano builder
	hcb   *array.Uint64Builder       // histogram_count builder
	hsb   *array.Float64Builder      // histogram_sum builder
	hbclb *array.ListBuilder         // histogram_bucket_counts list builder
	hbcb  *array.Uint64Builder       // histogram_bucket_counts builder
	heblb *array.ListBuilder         // histogram_explicit_bounds list builder
	hebb  *array.Float64Builder      // histogram_explicit_bounds builder
	elb   *array.ListBuilder         // exemplars builder
	eb    *ExemplarBuilder           // exemplar builder
	fb    *array.Uint32Builder       // flags builder
	hmib  *array.Float64Builder      // histogram_min builder
	hmab  *array.Float64Builder      // histogram_max builder
}

// NewHistogramDataPointBuilder creates a new HistogramDataPointBuilder with a given memory allocator.
func NewHistogramDataPointBuilder(pool memory.Allocator) *HistogramDataPointBuilder {
	return HistogramDataPointBuilderFrom(array.NewStructBuilder(pool, UnivariateHistogramDataPointDT))
}

// HistogramDataPointBuilderFrom creates a new HistogramDataPointBuilder from an existing StructBuilder.
func HistogramDataPointBuilderFrom(b *array.StructBuilder) *HistogramDataPointBuilder {
	return &HistogramDataPointBuilder{
		released: false,
		builder:  b,

		ab:    acommon.AttributesBuilderFrom(b.FieldBuilder(0).(*array.MapBuilder)),
		stunb: b.FieldBuilder(1).(*array.Uint64Builder),
		tunb:  b.FieldBuilder(2).(*array.Uint64Builder),
		hcb:   b.FieldBuilder(3).(*array.Uint64Builder),
		hsb:   b.FieldBuilder(4).(*array.Float64Builder),
		hbclb: b.FieldBuilder(5).(*array.ListBuilder),
		hbcb:  b.FieldBuilder(5).(*array.ListBuilder).ValueBuilder().(*array.Uint64Builder),
		heblb: b.FieldBuilder(6).(*array.ListBuilder),
		hebb:  b.FieldBuilder(6).(*array.ListBuilder).ValueBuilder().(*array.Float64Builder),
		elb:   b.FieldBuilder(7).(*array.ListBuilder),
		eb:    ExemplarBuilderFrom(b.FieldBuilder(7).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		fb:    b.FieldBuilder(8).(*array.Uint32Builder),
		hmib:  b.FieldBuilder(9).(*array.Float64Builder),
		hmab:  b.FieldBuilder(10).(*array.Float64Builder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *HistogramDataPointBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("HistogramDataPointBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *HistogramDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new histogram data point to the builder.
func (b *HistogramDataPointBuilder) Append(hdp pmetric.HistogramDataPoint, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("HistogramDataPointBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	if err := b.ab.AppendUniqueAttributes(hdp.Attributes(), smdata.Attributes, mdata.Attributes); err != nil {
		return err
	}
	if smdata.StartTime == nil && mdata.StartTime == nil {
		b.stunb.Append(uint64(hdp.StartTimestamp()))
	} else {
		b.stunb.AppendNull()
	}
	if smdata.Time == nil && mdata.Time == nil {
		b.tunb.Append(uint64(hdp.Timestamp()))
	} else {
		b.tunb.AppendNull()
	}
	b.hcb.Append(hdp.Count())
	if hdp.HasSum() {
		b.hsb.Append(hdp.Sum())
	} else {
		b.hsb.AppendNull()
	}

	hbc := hdp.BucketCounts()
	hbcc := hbc.Len()
	if hbcc > 0 {
		b.hbclb.Append(true)
		b.hbclb.Reserve(hbcc)
		for i := 0; i < hbcc; i++ {
			b.hbcb.Append(hbc.At(i))
		}
	} else {
		b.hbclb.Append(false)
	}

	heb := hdp.ExplicitBounds()
	hebc := heb.Len()
	if hebc > 0 {
		b.heblb.Append(true)
		b.heblb.Reserve(hebc)
		for i := 0; i < hebc; i++ {
			b.hebb.Append(heb.At(i))
		}
	} else {
		b.heblb.Append(false)
	}

	exs := hdp.Exemplars()
	ec := exs.Len()
	if ec > 0 {
		b.elb.Append(true)
		b.elb.Reserve(ec)
		for i := 0; i < ec; i++ {
			if err := b.eb.Append(exs.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.elb.Append(false)
	}
	if hdp.Flags() != 0 {
		b.fb.Append(uint32(hdp.Flags()))
	} else {
		b.fb.AppendNull()
	}

	if hdp.HasMin() {
		b.hmib.Append(hdp.Min())
	} else {
		b.hmib.AppendNull()
	}
	if hdp.HasMax() {
		b.hmab.Append(hdp.Max())
	} else {
		b.hmab.AppendNull()
	}

	return nil
}
