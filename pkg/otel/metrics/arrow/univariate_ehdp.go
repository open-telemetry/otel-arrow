// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		arrow.Field{Name: constants.HistogramCount, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.HistogramSum, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.ExpHistogramScale, Type: arrow.PrimitiveTypes.Int32},
		arrow.Field{Name: constants.ExpHistogramZeroCount, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.ExpHistogramPositive, Type: EHistogramDataPointBucketsDT},
		arrow.Field{Name: constants.ExpHistogramNegative, Type: EHistogramDataPointBucketsDT},
		arrow.Field{Name: constants.Exemplars, Type: arrow.ListOf(ExemplarDT)},
		arrow.Field{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32},
		arrow.Field{Name: constants.HistogramMin, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.HistogramMax, Type: arrow.PrimitiveTypes.Float64},
	)
)

// EHistogramDataPointBuilder is a builder for exponential histogram data points.
type EHistogramDataPointBuilder struct {
	released bool

	builder *array.StructBuilder

	ab    *acommon.AttributesBuilder         // attributes builder
	stunb *array.TimestampBuilder            // start_time_unix_nano builder
	tunb  *array.TimestampBuilder            // time_unix_nano builder
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
		stunb: b.FieldBuilder(1).(*array.TimestampBuilder),
		tunb:  b.FieldBuilder(2).(*array.TimestampBuilder),
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
		b.stunb.Append(arrow.Timestamp(hdp.StartTimestamp()))
	} else {
		b.stunb.AppendNull()
	}
	if smdata.Time == nil && mdata.Time == nil {
		b.tunb.Append(arrow.Timestamp(hdp.Timestamp()))
	} else {
		b.tunb.AppendNull()
	}
	b.AppendCountSum(hdp)
	b.sb.Append(hdp.Scale())
	b.zcb.Append(hdp.ZeroCount())
	if err := b.pb.Append(hdp.Positive()); err != nil {
		return err
	}
	if err := b.nb.Append(hdp.Negative()); err != nil {
		return err
	}

	err := b.AppendExemplars(hdp)
	if err != nil {
		return err
	}
	if hdp.Flags() != 0 {
		b.fb.Append(uint32(hdp.Flags()))
	} else {
		b.fb.AppendNull()
	}

	b.AppendMinMax(hdp)

	return nil
}

func (b *EHistogramDataPointBuilder) AppendExemplars(hdp pmetric.ExponentialHistogramDataPoint) error {
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
	return nil
}

func (b *EHistogramDataPointBuilder) AppendCountSum(hdp pmetric.ExponentialHistogramDataPoint) {
	b.hcb.Append(hdp.Count())
	if hdp.HasSum() {
		b.hsb.Append(hdp.Sum())
	} else {
		b.hsb.AppendNull()
	}
}

func (b *EHistogramDataPointBuilder) AppendMinMax(hdp pmetric.ExponentialHistogramDataPoint) {
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
}
