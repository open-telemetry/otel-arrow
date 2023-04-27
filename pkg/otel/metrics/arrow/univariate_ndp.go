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
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow_old"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

var (
	// UnivariateNumberDataPointDT is the data type for a single univariate number data point.
	UnivariateNumberDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.MetricValue, Type: MetricValueDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Exemplars, Type: arrow.ListOf(ExemplarDT), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
	)
)

// NumberDataPointBuilder is a builder for number data points.
type NumberDataPointBuilder struct {
	released bool

	builder *builder.StructBuilder

	ab    *acommon.AttributesBuilder // attributes builder
	stunb *builder.TimestampBuilder  // start_time_unix_nano builder
	tunb  *builder.TimestampBuilder  // time_unix_nano builder
	mvb   *MetricValueBuilder        // metric_value builder
	elb   *builder.ListBuilder       // exemplars builder
	eb    *ExemplarBuilder           // exemplar builder
	fb    *builder.Uint32Builder     // flags builder
}

// NumberDataPointBuilderFrom creates a new NumberDataPointBuilder from an existing StructBuilder.
func NumberDataPointBuilderFrom(ndpb *builder.StructBuilder) *NumberDataPointBuilder {
	exemplars := ndpb.ListBuilder(constants.Exemplars)
	return &NumberDataPointBuilder{
		released: false,
		builder:  ndpb,

		ab:    acommon.AttributesBuilderFrom(ndpb.MapBuilder(constants.Attributes)),
		stunb: ndpb.TimestampBuilder(constants.StartTimeUnixNano),
		tunb:  ndpb.TimestampBuilder(constants.TimeUnixNano),
		mvb:   MetricValueBuilderFrom(ndpb.SparseUnionBuilder(constants.MetricValue)),
		elb:   exemplars,
		eb:    ExemplarBuilderFrom(exemplars.StructBuilder()),
		fb:    ndpb.Uint32Builder(constants.Flags),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *NumberDataPointBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *NumberDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new data point to the builder.
func (b *NumberDataPointBuilder) Append(ndp pmetric.NumberDataPoint, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(ndp, func() error {
		if err := b.ab.AppendUniqueAttributes(ndp.Attributes(), smdata.Attributes, mdata.Attributes); err != nil {
			return werror.Wrap(err)
		}
		if smdata.StartTime == nil && mdata.StartTime == nil {
			b.stunb.Append(arrow.Timestamp(ndp.StartTimestamp()))
		} else {
			b.stunb.AppendNull()
		}
		if smdata.Time == nil && mdata.Time == nil {
			b.tunb.Append(arrow.Timestamp(ndp.Timestamp()))
		} else {
			b.tunb.AppendNull()
		}
		if err := b.mvb.AppendNumberDataPointValue(ndp); err != nil {
			return werror.Wrap(err)
		}

		b.fb.Append(uint32(ndp.Flags()))

		exs := ndp.Exemplars()
		ec := exs.Len()
		return b.elb.Append(ec, func() error {
			for i := 0; i < ec; i++ {
				if err := b.eb.Append(exs.At(i)); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		})
	})
}
