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

// Constants used to identify the type of univariate metric in the union.
const (
	GaugeCode        int8 = 0
	SumCode          int8 = 1
	SummaryCode      int8 = 2
	HistogramCode    int8 = 3
	ExpHistogramCode int8 = 4
)

// UnivariateMetricDT is the Arrow Data Type describing a univariate metric.
var (
	UnivariateMetricDT = arrow.SparseUnionOf([]arrow.Field{
		{Name: constants.GaugeMetrics, Type: UnivariateGaugeDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SumMetrics, Type: UnivariateSumDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SummaryMetrics, Type: UnivariateSummaryDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.HistogramMetrics, Type: UnivariateHistogramDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.ExpHistogramMetrics, Type: UnivariateEHistogramDT, Metadata: schema.Metadata(schema.Optional)},
	},
		[]arrow.UnionTypeCode{
			GaugeCode,
			SumCode,
			SummaryCode,
			HistogramCode,
			ExpHistogramCode,
		},
	)
)

// UnivariateMetricBuilder is a builder for univariate metrics.
type UnivariateMetricBuilder struct {
	released bool

	builder *builder.SparseUnionBuilder

	gb  *UnivariateGaugeBuilder      // univariate gauge builder
	sb  *UnivariateSumBuilder        // univariate sum builder
	syb *UnivariateSummaryBuilder    // univariate summary builder
	hb  *UnivariateHistogramBuilder  // univariate histogram builder
	ehb *UnivariateEHistogramBuilder // univariate exponential histogram builder
}

// UnivariateMetricBuilderFrom creates a new UnivariateMetricBuilder from an existing StructBuilder.
func UnivariateMetricBuilderFrom(umb *builder.SparseUnionBuilder) *UnivariateMetricBuilder {
	return &UnivariateMetricBuilder{
		released: false,
		builder:  umb,

		gb:  UnivariateGaugeBuilderFrom(umb.StructBuilder(GaugeCode)),
		sb:  UnivariateSumBuilderFrom(umb.StructBuilder(SumCode)),
		syb: UnivariateSummaryBuilderFrom(umb.StructBuilder(SummaryCode)),
		hb:  UnivariateHistogramBuilderFrom(umb.StructBuilder(HistogramCode)),
		ehb: UnivariateEHistogramBuilderFrom(umb.StructBuilder(ExpHistogramCode)),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateMetricBuilder) Build() (*array.SparseUnion, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewSparseUnionArray(), nil
}

// Release releases the underlying memory.
func (b *UnivariateMetricBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new univariate metric to the builder.
func (b *UnivariateMetricBuilder) Append(metric pmetric.Metric, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	switch metric.Type() {
	case pmetric.MetricTypeGauge:
		b.builder.Append(GaugeCode)
		if err := b.gb.Append(metric.Gauge(), smdata, mdata); err != nil {
			return werror.Wrap(err)
		}
		b.sb.AppendNull()
		b.syb.AppendNull()
		b.hb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeSum:
		b.builder.Append(SumCode)
		if err := b.sb.Append(metric.Sum(), smdata, mdata); err != nil {
			return werror.Wrap(err)
		}
		b.gb.AppendNull()
		b.syb.AppendNull()
		b.hb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeSummary:
		b.builder.Append(SummaryCode)
		if err := b.syb.Append(metric.Summary(), smdata, mdata); err != nil {
			return werror.Wrap(err)
		}
		b.gb.AppendNull()
		b.sb.AppendNull()
		b.hb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeHistogram:
		b.builder.Append(HistogramCode)
		if err := b.hb.Append(metric.Histogram(), smdata, mdata); err != nil {
			return werror.Wrap(err)
		}
		b.gb.AppendNull()
		b.sb.AppendNull()
		b.syb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeExponentialHistogram:
		b.builder.Append(ExpHistogramCode)
		if err := b.ehb.Append(metric.ExponentialHistogram(), smdata, mdata); err != nil {
			return werror.Wrap(err)
		}
		b.gb.AppendNull()
		b.sb.AppendNull()
		b.syb.AppendNull()
		b.hb.AppendNull()
	case pmetric.MetricTypeEmpty:
		// ignore empty metric
	}

	return nil
}
