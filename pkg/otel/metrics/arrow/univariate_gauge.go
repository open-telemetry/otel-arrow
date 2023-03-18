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

	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

var (
	// UnivariateGaugeDT is the Arrow Data Type describing a univariate gauge.
	UnivariateGaugeDT = arrow.StructOf(
		arrow.Field{Name: constants.DataPoints, Type: arrow.ListOf(UnivariateNumberDataPointDT), Metadata: schema.Metadata(schema.Optional)},
	)
)

// UnivariateGaugeBuilder is a builder for gauge metrics.
type UnivariateGaugeBuilder struct {
	released bool

	builder *builder.StructBuilder

	dplb *builder.ListBuilder    // data_points builder
	dpb  *NumberDataPointBuilder // number data point builder
}

// UnivariateGaugeBuilderFrom creates a new UnivariateMetricBuilder from an existing StructBuilder.
func UnivariateGaugeBuilderFrom(ndpb *builder.StructBuilder) *UnivariateGaugeBuilder {
	dplb := ndpb.ListBuilder(constants.DataPoints)

	return &UnivariateGaugeBuilder{
		released: false,
		builder:  ndpb,

		dplb: dplb,
		dpb:  NumberDataPointBuilderFrom(dplb.StructBuilder()),
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
		return fmt.Errorf("UnivariateGaugeBuilder: Reserve() called after Release()")
	}

	return b.builder.Append(gauge, func() error {
		dps := gauge.DataPoints()
		dpc := dps.Len()
		return b.dplb.Append(dpc, func() error {
			for i := 0; i < dpc; i++ {
				if err := b.dpb.Append(dps.At(i), smdata, mdata); err != nil {
					return err
				}
			}
			return nil
		})
	})
}

func (b *UnivariateGaugeBuilder) AppendNull() {
	if b.released {
		return
	}

	b.builder.AppendNull()
}
