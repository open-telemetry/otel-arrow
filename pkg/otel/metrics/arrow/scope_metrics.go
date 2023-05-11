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

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// ScopeMetricsDT is the Arrow Data Type describing a scope span.
var (
	ScopeMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint16, Metadata: schema.Metadata(schema.DeltaEncoding)},
		{Name: constants.Scope, Type: acommon.ScopeDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
	}...)
)

type (
	// ScopeMetricsBuilder is a helper to build a scope spans.
	ScopeMetricsBuilder struct {
		released bool

		builder *builder.StructBuilder

		ib   *builder.Uint16DeltaBuilder // id builder
		scb  *acommon.ScopeBuilder       // scope builder
		schb *builder.StringBuilder      // schema url builder
	}
)

func ScopeMetricsBuilderFrom(builder *builder.StructBuilder) *ScopeMetricsBuilder {
	ib := builder.Uint16DeltaBuilder(constants.ID)
	ib.SetMaxDelta(1)

	return &ScopeMetricsBuilder{
		released: false,
		builder:  builder,
		ib:       ib,
		scb:      acommon.ScopeBuilderFrom(builder.StructBuilder(constants.Scope)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
	}
}

// Build builds the scope metrics array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ScopeMetricsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new scope metrics to the builder.
func (b *ScopeMetricsBuilder) Append(smg *ScopeMetricsGroup, relatedData *RelatedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(smg, func() error {
		ID := relatedData.NextMetricScopeID()
		scopeAttrsAccumulator := relatedData.AttrsBuilders().scope.Accumulator()

		b.ib.Append(ID)
		if err := b.scb.Append(smg.Scope, scopeAttrsAccumulator); err != nil {
			return werror.Wrap(err)
		}
		b.schb.AppendNonEmpty(smg.ScopeSchemaUrl)

		for _, metric := range smg.Metrics {
			switch metric.Type() {
			case pmetric.MetricTypeGauge:
				b.AppendGauge(ID, metric, relatedData)
			case pmetric.MetricTypeSum:
				b.AppendSum(ID, metric, relatedData)
			case pmetric.MetricTypeSummary:
				b.AppendSummary(ID, metric, relatedData)
			case pmetric.MetricTypeHistogram:
				b.AppendHistogram(ID, metric, relatedData)
			case pmetric.MetricTypeExponentialHistogram:
				b.AppendExponentialHistogram(ID, metric, relatedData)
			default:
				panic(fmt.Sprintf("unknown metric type %d", metric.Type()))
			}
		}

		return nil
	})
}

func (b *ScopeMetricsBuilder) AppendSum(ID uint16, metric *pmetric.Metric, relatedData *RelatedData) {
	sum := metric.Sum()

	aggrTempo := sum.AggregationTemporality()
	monotonic := sum.IsMonotonic()

	dps := sum.DataPoints()
	for i := 0; i < dps.Len(); i++ {
		dp := dps.At(i)
		switch dp.ValueType() {
		case pmetric.NumberDataPointValueTypeInt:
			relatedData.SumIDPBuilder().Accumulator().Append(ID, metric, aggrTempo, monotonic, &dp)
		case pmetric.NumberDataPointValueTypeDouble:
			relatedData.SumDDPBuilder().Accumulator().Append(ID, metric, aggrTempo, monotonic, &dp)
		default:
			panic(fmt.Sprintf("unknown value type %d", dp.ValueType()))
		}
	}
}

func (b *ScopeMetricsBuilder) AppendGauge(ID uint16, metric *pmetric.Metric, relatedData *RelatedData) {
	dps := metric.Gauge().DataPoints()
	for i := 0; i < dps.Len(); i++ {
		dp := dps.At(i)
		switch dp.ValueType() {
		case pmetric.NumberDataPointValueTypeInt:
			relatedData.GaugeIDPBuilder().Accumulator().Append(ID, metric, pmetric.AggregationTemporalityUnspecified, false, &dp)
		case pmetric.NumberDataPointValueTypeDouble:
			relatedData.GaugeDDPBuilder().Accumulator().Append(ID, metric, pmetric.AggregationTemporalityUnspecified, false, &dp)
		default:
			panic(fmt.Sprintf("unknown value type %d", dp.ValueType()))
		}
	}
}

func (b *ScopeMetricsBuilder) AppendSummary(ID uint16, metric *pmetric.Metric, relatedData *RelatedData) {
	relatedData.SummaryDPBuilder().Accumulator().Append(ID, metric, pmetric.AggregationTemporalityUnspecified, false, metric.Summary().DataPoints())
}

func (b *ScopeMetricsBuilder) AppendHistogram(ID uint16, metric *pmetric.Metric, relatedData *RelatedData) {
	histogram := metric.Histogram()
	aggrTempo := histogram.AggregationTemporality()
	relatedData.HistogramDPBuilder().Accumulator().Append(ID, metric, aggrTempo, false, histogram.DataPoints())
}

func (b *ScopeMetricsBuilder) AppendExponentialHistogram(ID uint16, metric *pmetric.Metric, relatedData *RelatedData) {
	histogram := metric.ExponentialHistogram()
	aggrTempo := histogram.AggregationTemporality()
	relatedData.EHistogramDPBuilder().Accumulator().Append(ID, metric, aggrTempo, false, histogram.DataPoints())
}

// Release releases the memory allocated by the builder.
func (b *ScopeMetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
