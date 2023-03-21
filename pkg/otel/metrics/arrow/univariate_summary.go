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

// UnivariateSummaryDT is the Arrow Data Type describing a univariate summary.
var (
	UnivariateSummaryDT = arrow.StructOf(
		arrow.Field{Name: constants.DataPoints, Type: arrow.ListOf(UnivariateSummaryDataPointDT), Metadata: schema.Metadata(schema.Optional)},
	)
)

// UnivariateSummaryBuilder is a builder for summary metrics.
type UnivariateSummaryBuilder struct {
	released bool

	builder *builder.StructBuilder

	dplb *builder.ListBuilder               // data points builder
	dpb  *UnivariateSummaryDataPointBuilder // summary data point builder
}

// UnivariateSummaryBuilderFrom creates a new UnivariateSummaryBuilder from an existing StructBuilder.
func UnivariateSummaryBuilderFrom(ndpb *builder.StructBuilder) *UnivariateSummaryBuilder {
	dplb := ndpb.ListBuilder(constants.DataPoints)

	return &UnivariateSummaryBuilder{
		released: false,
		builder:  ndpb,

		dplb: dplb,
		dpb:  UnivariateSummaryDataPointBuilderFrom(dplb.StructBuilder()),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateSummaryBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
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
func (b *UnivariateSummaryBuilder) Append(summary pmetric.Summary, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(summary, func() error {
		dps := summary.DataPoints()
		dpc := dps.Len()
		return b.dplb.Append(dpc, func() error {
			for i := 0; i < dpc; i++ {
				if err := b.dpb.Append(dps.At(i), smdata, mdata); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		})
	})
}

func (b *UnivariateSummaryBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.AppendNull()
}
