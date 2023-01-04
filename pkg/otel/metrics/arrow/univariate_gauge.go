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

var (
	// UnivariateGaugeDT is the Arrow Data Type describing a univariate gauge.
	UnivariateGaugeDT = arrow.StructOf(
		arrow.Field{Name: constants.DataPoints, Type: arrow.ListOf(UnivariateNumberDataPointDT)},
	)
)

// UnivariateGaugeBuilder is a builder for gauge metrics.
type UnivariateGaugeBuilder struct {
	released bool

	builder *array.StructBuilder

	dplb *array.ListBuilder      // data_points builder
	dpb  *NumberDataPointBuilder // number data point builder
}

// NewUnivariateGaugeBuilder creates a new UnivariateMetricBuilder with a given memory allocator.
func NewUnivariateGaugeBuilder(pool memory.Allocator) *UnivariateGaugeBuilder {
	return UnivariateGaugeBuilderFrom(array.NewStructBuilder(pool, UnivariateGaugeDT))
}

// UnivariateGaugeBuilderFrom creates a new UnivariateMetricBuilder from an existing StructBuilder.
func UnivariateGaugeBuilderFrom(ndpb *array.StructBuilder) *UnivariateGaugeBuilder {
	return &UnivariateGaugeBuilder{
		released: false,
		builder:  ndpb,

		dplb: ndpb.FieldBuilder(0).(*array.ListBuilder),
		dpb:  NumberDataPointBuilderFrom(ndpb.FieldBuilder(0).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateGaugeBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateGaugeBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *UnivariateGaugeBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new univariate gauge to the builder.
func (b *UnivariateGaugeBuilder) Append(gauge pmetric.Gauge, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("UnivariateGaugeBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	dps := gauge.DataPoints()
	dpc := dps.Len()
	if dpc > 0 {
		b.dplb.Append(true)
		b.dplb.Reserve(dpc)
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

func (b *UnivariateGaugeBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.Append(false)
}
