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
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// UnivariateEHistogramDataPointDT is the Arrow Data Type describing a univariate exponential histogram number data point.
var (
	UnivariateEHistogramDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramSum, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.ExpHistogramScale, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.ExpHistogramZeroCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.ExpHistogramPositive, Type: EHistogramDataPointBucketsDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.ExpHistogramNegative, Type: EHistogramDataPointBucketsDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Exemplars, Type: arrow.ListOf(ExemplarDT), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramMin, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramMax, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
	)
)

// EHistogramDataPointBuilder is a builder for exponential histogram data points.
type EHistogramDataPointBuilder struct {
	released bool

	builder *builder.StructBuilder

	ab    *acommon.AttributesBuilder         // attributes builder
	stunb *builder.TimestampBuilder          // start_time_unix_nano builder
	tunb  *builder.TimestampBuilder          // time_unix_nano builder
	hcb   *builder.Uint64Builder             // histogram_count builder
	hsb   *builder.Float64Builder            // histogram_sum builder
	sb    *builder.Int32Builder              // scale builder
	zcb   *builder.Uint64Builder             // zero_count builder
	pb    *EHistogramDataPointBucketsBuilder // positive buckets builder
	nb    *EHistogramDataPointBucketsBuilder // negative buckets builder
	elb   *builder.ListBuilder               // exemplars builder
	eb    *ExemplarBuilder                   // exemplar builder
	fb    *builder.Uint32Builder             // flags builder
	hmib  *builder.Float64Builder            // histogram_min builder
	hmab  *builder.Float64Builder            // histogram_max builder
}

// EHistogramDataPointBuilderFrom creates a new EHistogramDataPointBuilder from an existing StructBuilder.
func EHistogramDataPointBuilderFrom(b *builder.StructBuilder) *EHistogramDataPointBuilder {
	elb := b.ListBuilder(constants.Exemplars)

	return &EHistogramDataPointBuilder{
		released: false,
		builder:  b,

		ab:    acommon.AttributesBuilderFrom(b.MapBuilder(constants.Attributes)),
		stunb: b.TimestampBuilder(constants.StartTimeUnixNano),
		tunb:  b.TimestampBuilder(constants.TimeUnixNano),
		hcb:   b.Uint64Builder(constants.HistogramCount),
		hsb:   b.Float64Builder(constants.HistogramSum),
		sb:    b.Int32Builder(constants.ExpHistogramScale),
		zcb:   b.Uint64Builder(constants.ExpHistogramZeroCount),
		pb:    EHistogramDataPointBucketsBuilderFrom(b.StructBuilder(constants.ExpHistogramPositive)),
		nb:    EHistogramDataPointBucketsBuilderFrom(b.StructBuilder(constants.ExpHistogramNegative)),
		elb:   elb,
		eb:    ExemplarBuilderFrom(elb.StructBuilder()),
		fb:    b.Uint32Builder(constants.Flags),
		hmib:  b.Float64Builder(constants.HistogramMin),
		hmab:  b.Float64Builder(constants.HistogramMax),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *EHistogramDataPointBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
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
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(hdp, func() error {
		if err := b.ab.AppendUniqueAttributes(hdp.Attributes(), smdata.Attributes, mdata.Attributes); err != nil {
			return werror.Wrap(err)
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
		b.sb.AppendNonZero(hdp.Scale())
		b.zcb.Append(hdp.ZeroCount())
		if err := b.pb.Append(hdp.Positive()); err != nil {
			return werror.Wrap(err)
		}
		if err := b.nb.Append(hdp.Negative()); err != nil {
			return werror.Wrap(err)
		}

		err := b.AppendExemplars(hdp)
		if err != nil {
			return werror.Wrap(err)
		}
		b.fb.Append(uint32(hdp.Flags()))

		b.AppendMinMax(hdp)

		return nil
	})
}

func (b *EHistogramDataPointBuilder) AppendExemplars(hdp pmetric.ExponentialHistogramDataPoint) error {
	exs := hdp.Exemplars()
	ec := exs.Len()
	return b.elb.Append(ec, func() error {
		for i := 0; i < ec; i++ {
			if err := b.eb.Append(exs.At(i)); err != nil {
				return werror.Wrap(err)
			}
		}
		return nil
	})
}

func (b *EHistogramDataPointBuilder) AppendCountSum(hdp pmetric.ExponentialHistogramDataPoint) {
	b.hcb.Append(hdp.Count())
	if hdp.HasSum() {
		b.hsb.AppendNonZero(hdp.Sum())
	} else {
		b.hsb.AppendNull()
	}
}

func (b *EHistogramDataPointBuilder) AppendMinMax(hdp pmetric.ExponentialHistogramDataPoint) {
	if hdp.HasMin() {
		b.hmib.AppendNonZero(hdp.Min())
	} else {
		b.hmib.AppendNull()
	}
	if hdp.HasMax() {
		b.hmab.AppendNonZero(hdp.Max())
	} else {
		b.hmab.AppendNull()
	}
}
