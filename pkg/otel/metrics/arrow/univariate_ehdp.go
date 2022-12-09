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

// UnivariateEHistogramDataPointDT is the Arrow Data Type describing a univariate exponential histogram number data point.
var (
	UnivariateEHistogramDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.ATTRIBUTES, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.START_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.HISTOGRAM_COUNT, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.HISTOGRAM_SUM, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.EXP_HISTOGRAM_SCALE, Type: arrow.PrimitiveTypes.Int32},
		arrow.Field{Name: constants.EXP_HISTOGRAM_ZERO_COUNT, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.EXP_HISTOGRAM_POSITIVE, Type: EHistogramDataPointBucketsDT},
		arrow.Field{Name: constants.EXP_HISTOGRAM_NEGATIVE, Type: EHistogramDataPointBucketsDT},
		arrow.Field{Name: constants.EXEMPLARS, Type: arrow.ListOf(ExemplarDT)},
		arrow.Field{Name: constants.FLAGS, Type: arrow.PrimitiveTypes.Uint32},
		arrow.Field{Name: constants.HISTOGRAM_MIN, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.HISTOGRAM_MAX, Type: arrow.PrimitiveTypes.Float64},
	)
)

// EHistogramDataPointBuilder is a builder for exponential histogram data points.
type EHistogramDataPointBuilder struct {
	released bool

	builder *array.StructBuilder

	ab    *acommon.AttributesBuilder         // attributes builder
	stunb *array.Uint64Builder               // start_time_unix_nano builder
	tunb  *array.Uint64Builder               // time_unix_nano builder
	hcb   *array.Uint64Builder               // histogram_count builder
	hsb   *array.Float64Builder              // histogram_sum builder
	sb    *array.Int32Builder                // scale builder
	zcb   *array.Uint64Builder               // zero_count builder
	pb    *EHistogramDataPointBucketsBuilder // positive buckets builder
	nb    *EHistogramDataPointBucketsBuilder // negative buckets builder
	elb   *array.ListBuilder                 // exemplars builder
	eb    *ExemplarBuilder                   // exemplar builder
	fb    *array.Uint32Builder               // flags builder
	hmib  *array.Float64Builder              // histogram_min builder
	hmab  *array.Float64Builder              // histogram_max builder
}

// NewEHistogramDataPointBuilder creates a new EHistogramDataPointBuilder with a given memory allocator.
func NewEHistogramDataPointBuilder(pool memory.Allocator) *EHistogramDataPointBuilder {
	return EHistogramDataPointBuilderFrom(array.NewStructBuilder(pool, UnivariateEHistogramDataPointDT))
}

// EHistogramDataPointBuilderFrom creates a new EHistogramDataPointBuilder from an existing StructBuilder.
func EHistogramDataPointBuilderFrom(b *array.StructBuilder) *EHistogramDataPointBuilder {
	return &EHistogramDataPointBuilder{
		released: false,
		builder:  b,

		ab:    acommon.AttributesBuilderFrom(b.FieldBuilder(0).(*array.MapBuilder)),
		stunb: b.FieldBuilder(1).(*array.Uint64Builder),
		tunb:  b.FieldBuilder(2).(*array.Uint64Builder),
		hcb:   b.FieldBuilder(3).(*array.Uint64Builder),
		hsb:   b.FieldBuilder(4).(*array.Float64Builder),
		sb:    b.FieldBuilder(5).(*array.Int32Builder),
		zcb:   b.FieldBuilder(6).(*array.Uint64Builder),
		pb:    EHistogramDataPointBucketsBuilderFrom(b.FieldBuilder(7).(*array.StructBuilder)),
		nb:    EHistogramDataPointBucketsBuilderFrom(b.FieldBuilder(8).(*array.StructBuilder)),
		elb:   b.FieldBuilder(9).(*array.ListBuilder),
		eb:    ExemplarBuilderFrom(b.FieldBuilder(9).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		fb:    b.FieldBuilder(10).(*array.Uint32Builder),
		hmib:  b.FieldBuilder(11).(*array.Float64Builder),
		hmab:  b.FieldBuilder(12).(*array.Float64Builder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *EHistogramDataPointBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("EHistogramDataPointBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *EHistogramDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new histogram data point to the builder.
func (b *EHistogramDataPointBuilder) Append(hdp pmetric.ExponentialHistogramDataPoint, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("EHistogramDataPointBuilder: Append() called after Release()")
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
	b.sb.Append(hdp.Scale())
	b.zcb.Append(hdp.ZeroCount())
	if err := b.pb.Append(hdp.Positive()); err != nil {
		return err
	}
	if err := b.nb.Append(hdp.Negative()); err != nil {
		return err
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
