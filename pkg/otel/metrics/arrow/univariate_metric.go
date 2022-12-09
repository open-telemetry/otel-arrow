package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
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
		{Name: constants.GAUGE_METRICS, Type: UnivariateGaugeDT},
		{Name: constants.SUM_METRICS, Type: UnivariateSumDT},
		{Name: constants.SUMMARY_METRICS, Type: UnivariateSummaryDT},
		{Name: constants.HISTOGRAM_METRICS, Type: UnivariateHistogramDT},
		{Name: constants.EXP_HISTOGRAM_METRICS, Type: UnivariateEHistogramDT},
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

	builder *array.SparseUnionBuilder

	gb  *UnivariateGaugeBuilder      // univariate gauge builder
	sb  *UnivariateSumBuilder        // univariate sum builder
	syb *UnivariateSummaryBuilder    // univariate summary builder
	hb  *UnivariateHistogramBuilder  // univariate histogram builder
	ehb *UnivariateEHistogramBuilder // univariate exponential histogram builder
}

// NewUnivariateMetricBuilder creates a new UnivariateMetricBuilder with a given memory allocator.
func NewUnivariateMetricBuilder(pool memory.Allocator) *UnivariateMetricBuilder {
	return UnivariateMetricBuilderFrom(array.NewSparseUnionBuilder(pool, UnivariateMetricDT))
}

// UnivariateMetricBuilderFrom creates a new UnivariateMetricBuilder from an existing StructBuilder.
func UnivariateMetricBuilderFrom(umb *array.SparseUnionBuilder) *UnivariateMetricBuilder {
	return &UnivariateMetricBuilder{
		released: false,
		builder:  umb,

		gb:  UnivariateGaugeBuilderFrom(umb.Child(0).(*array.StructBuilder)),
		sb:  UnivariateSumBuilderFrom(umb.Child(1).(*array.StructBuilder)),
		syb: UnivariateSummaryBuilderFrom(umb.Child(2).(*array.StructBuilder)),
		hb:  UnivariateHistogramBuilderFrom(umb.Child(3).(*array.StructBuilder)),
		ehb: UnivariateEHistogramBuilderFrom(umb.Child(4).(*array.StructBuilder)),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateMetricBuilder) Build() (*array.SparseUnion, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateMetricBuilder: Build() called after Release()")
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
		return fmt.Errorf("UnivariateMetricBuilder: Append() called after Release()")
	}

	switch metric.Type() {
	case pmetric.MetricTypeGauge:
		b.builder.Append(GaugeCode)
		if err := b.gb.Append(metric.Gauge(), smdata, mdata); err != nil {
			return err
		}
		b.sb.AppendNull()
		b.syb.AppendNull()
		b.hb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeSum:
		b.builder.Append(SumCode)
		if err := b.sb.Append(metric.Sum(), smdata, mdata); err != nil {
			return err
		}
		b.gb.AppendNull()
		b.syb.AppendNull()
		b.hb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeSummary:
		b.builder.Append(SummaryCode)
		if err := b.syb.Append(metric.Summary(), smdata, mdata); err != nil {
			return err
		}
		b.gb.AppendNull()
		b.sb.AppendNull()
		b.hb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeHistogram:
		b.builder.Append(HistogramCode)
		if err := b.hb.Append(metric.Histogram(), smdata, mdata); err != nil {
			return err
		}
		b.gb.AppendNull()
		b.sb.AppendNull()
		b.syb.AppendNull()
		b.ehb.AppendNull()
	case pmetric.MetricTypeExponentialHistogram:
		b.builder.Append(ExpHistogramCode)
		if err := b.ehb.Append(metric.ExponentialHistogram(), smdata, mdata); err != nil {
			return err
		}
		b.gb.AppendNull()
		b.sb.AppendNull()
		b.syb.AppendNull()
		b.hb.AppendNull()
	}

	return nil
}
