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

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateSummaryDT is the Arrow Data Type describing a univariate summary.
var (
	UnivariateSummaryDT = arrow.StructOf(
		arrow.Field{Name: constants.DataPoints, Type: arrow.ListOf(UnivariateSummaryDataPointDT)},
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
func (b *UnivariateSummaryBuilder) Append(summary pmetric.Summary, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("UnivariateSummaryBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	dps := summary.DataPoints()
	dpc := dps.Len()
	if dpc > 0 {
		b.dplb.Append(true)
		for i := 0; i < dpc; i++ {
			if err := b.dpb.Append(dps.At(i), smdata, mdata); err != nil {
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
