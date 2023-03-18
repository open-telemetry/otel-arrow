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
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateHistogramDataPointDT is the Arrow Data Type describing a univariate histogram number data point.
var (
	UnivariateHistogramDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramSum, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramBucketCounts, Type: arrow.ListOf(arrow.PrimitiveTypes.Uint64), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramExplicitBounds, Type: arrow.ListOf(arrow.PrimitiveTypes.Float64), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Exemplars, Type: arrow.ListOf(ExemplarDT), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramMin, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.HistogramMax, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
	)
)

// HistogramDataPointBuilder is a builder for histogram data points.
type HistogramDataPointBuilder struct {
	released bool

	builder *builder.StructBuilder

	ab    *acommon.AttributesBuilder // attributes builder
	stunb *builder.TimestampBuilder  // start_time_unix_nano builder
	tunb  *builder.TimestampBuilder  // time_unix_nano builder
	hcb   *builder.Uint64Builder     // histogram_count builder
	hsb   *builder.Float64Builder    // histogram_sum builder
	hbclb *builder.ListBuilder       // histogram_bucket_counts list builder
	hbcb  *builder.Uint64Builder     // histogram_bucket_counts builder
	heblb *builder.ListBuilder       // histogram_explicit_bounds list builder
	hebb  *builder.Float64Builder    // histogram_explicit_bounds builder
	elb   *builder.ListBuilder       // exemplars builder
	eb    *ExemplarBuilder           // exemplar builder
	fb    *builder.Uint32Builder     // flags builder
	hmib  *builder.Float64Builder    // histogram_min builder
	hmab  *builder.Float64Builder    // histogram_max builder
}

// HistogramDataPointBuilderFrom creates a new HistogramDataPointBuilder from an existing StructBuilder.
func HistogramDataPointBuilderFrom(b *builder.StructBuilder) *HistogramDataPointBuilder {
	hbclb := b.ListBuilder(constants.HistogramBucketCounts)
	heblb := b.ListBuilder(constants.HistogramExplicitBounds)
	elb := b.ListBuilder(constants.Exemplars)

	return &HistogramDataPointBuilder{
		released: false,
		builder:  b,

		ab:    acommon.AttributesBuilderFrom(b.MapBuilder(constants.Attributes)),
		stunb: b.TimestampBuilder(constants.StartTimeUnixNano),
		tunb:  b.TimestampBuilder(constants.TimeUnixNano),
		hcb:   b.Uint64Builder(constants.HistogramCount),
		hsb:   b.Float64Builder(constants.HistogramSum),
		hbclb: hbclb,
		hbcb:  hbclb.Uint64Builder(),
		heblb: heblb,
		hebb:  heblb.Float64Builder(),
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
		return fmt.Errorf("HistogramDataPointBuilder: Reserve() called after Release()")
	}

	return b.builder.Append(hdp, func() error {
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
		b.hcb.Append(hdp.Count())
		if hdp.HasSum() {
			b.hsb.AppendNonZero(hdp.Sum())
		} else {
			b.hsb.AppendNull()
		}

		hbc := hdp.BucketCounts()
		hbcc := hbc.Len()
		if err := b.hbclb.Append(hbcc, func() error {
			for i := 0; i < hbcc; i++ {
				b.hbcb.Append(hbc.At(i))
			}
			return nil
		}); err != nil {
			return err
		}

		heb := hdp.ExplicitBounds()
		hebc := heb.Len()
		if err := b.heblb.Append(hebc, func() error {
			for i := 0; i < hebc; i++ {
				b.hebb.AppendNonZero(heb.At(i))
			}
			return nil
		}); err != nil {
			return err
		}

		exs := hdp.Exemplars()
		ec := exs.Len()
		if err := b.elb.Append(ec, func() error {
			for i := 0; i < ec; i++ {
				if err := b.eb.Append(exs.At(i)); err != nil {
					return err
				}
			}
			return nil
		}); err != nil {
			return err
		}
		b.fb.Append(uint32(hdp.Flags()))

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

		return nil
	})
}
