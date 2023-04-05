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

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// UnivariateSumDT is the Arrow Data Type describing a univariate sum.
var (
	UnivariateSumDT = arrow.StructOf(
		arrow.Field{Name: constants.DataPoints, Type: arrow.ListOf(UnivariateNumberDataPointDT), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.AggregationTemporality, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		arrow.Field{Name: constants.IsMonotonic, Type: arrow.FixedWidthTypes.Boolean, Metadata: schema.Metadata(schema.Optional)},
	)
)

// UnivariateSumBuilder is a builder for sum metrics.
type UnivariateSumBuilder struct {
	released bool

	builder *builder.StructBuilder

	dplb *builder.ListBuilder    // data_points builder
	dpb  *NumberDataPointBuilder // number data point builder
	atb  *builder.Int32Builder   // aggregation_temporality builder
	imb  *builder.BooleanBuilder // is_monotonic builder
}

// UnivariateSumBuilderFrom creates a new UnivariateSumBuilder from an existing StructBuilder.
func UnivariateSumBuilderFrom(ndpb *builder.StructBuilder) *UnivariateSumBuilder {
	dplb := ndpb.ListBuilder(constants.DataPoints)

	return &UnivariateSumBuilder{
		released: false,
		builder:  ndpb,

		dplb: dplb,
		dpb:  NumberDataPointBuilderFrom(dplb.StructBuilder()),
		atb:  ndpb.Int32Builder(constants.AggregationTemporality),
		imb:  ndpb.BooleanBuilder(constants.IsMonotonic),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateSumBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
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
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(sum, func() error {
		dps := sum.DataPoints()
		dpc := dps.Len()
		if err := b.dplb.Append(dpc, func() error {
			for i := 0; i < dpc; i++ {
				if err := b.dpb.Append(dps.At(i), smdata, mdata); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		}); err != nil {
			return werror.Wrap(err)
		}
		if sum.AggregationTemporality() == pmetric.AggregationTemporalityUnspecified {
			b.atb.AppendNull()
		} else {
			b.atb.AppendNonZero(int32(sum.AggregationTemporality()))
		}
		if sum.IsMonotonic() {
			b.imb.AppendNonFalse(sum.IsMonotonic())
		} else {
			b.imb.AppendNull()
		}

		return nil
	})
}

func (b *UnivariateSumBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.AppendNull()
}
