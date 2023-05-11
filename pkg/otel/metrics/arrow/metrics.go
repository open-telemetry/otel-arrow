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
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/config"
	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
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

// MetricsSchema is the Arrow schema for the OTLP Arrow Metrics record.
var (
	MetricsSchema = arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceMetrics, Type: arrow.ListOf(ResourceMetricsDT)},
	}, nil)
)

// MetricsBuilder is a helper to build a list of resource metrics.
type MetricsBuilder struct {
	released bool

	builder *builder.RecordBuilderExt // Record builder
	rmb     *builder.ListBuilder      // resource metrics list builder
	rmp     *ResourceMetricsBuilder   // resource metrics builder

	optimizer *MetricsOptimizer
	analyzer  *MetricsAnalyzer

	relatedData *RelatedData
}

// NewMetricsBuilder creates a new MetricsBuilder with a given allocator.
func NewMetricsBuilder(
	rBuilder *builder.RecordBuilderExt,
	cfg *config.Config,
	stats *stats.ProducerStats,
) (*MetricsBuilder, error) {
	var optimizer *MetricsOptimizer
	var analyzer *MetricsAnalyzer

	relatedData, err := NewRelatedData(cfg, stats)
	if err != nil {
		panic(err)
	}

	if stats.SchemaStatsEnabled {
		optimizer = NewMetricsOptimizer(carrow.WithSort(), carrow.WithStats())
		analyzer = NewMetricsAnalyzer()
	} else {
		optimizer = NewMetricsOptimizer(carrow.WithSort())
	}

	b := &MetricsBuilder{
		released:    false,
		builder:     rBuilder,
		optimizer:   optimizer,
		analyzer:    analyzer,
		relatedData: relatedData,
	}

	if err := b.init(); err != nil {
		return nil, werror.Wrap(err)
	}

	return b, nil
}

func (b *MetricsBuilder) init() error {
	rmb := b.builder.ListBuilder(constants.ResourceMetrics)
	b.rmb = rmb
	b.rmp = ResourceMetricsBuilderFrom(rmb.StructBuilder())
	return nil
}

func (b *MetricsBuilder) RelatedData() *RelatedData {
	return b.relatedData
}

// Build builds an Arrow Record from the builder.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the record.
func (b *MetricsBuilder) Build() (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(carrow.ErrBuilderAlreadyReleased)
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		initErr := b.init()
		if initErr != nil {
			err = werror.Wrap(initErr)
		}
	}

	return
}

// Append appends a new set of resource metrics to the builder.
func (b *MetricsBuilder) Append(metrics pmetric.Metrics) error {
	if b.released {
		return werror.Wrap(carrow.ErrBuilderAlreadyReleased)
	}

	optimizedMetrics := b.optimizer.Optimize(metrics)
	if b.analyzer != nil {
		b.analyzer.Analyze(optimizedMetrics)
		b.analyzer.ShowStats("")
	}

	rc := len(optimizedMetrics.ResourceMetrics)
	return b.rmb.Append(rc, func() error {
		for _, resMetricsGroup := range optimizedMetrics.ResourceMetrics {
			if err := b.rmp.Append(resMetricsGroup, b.relatedData); err != nil {
				return werror.Wrap(err)
			}
		}
		return nil
	})
}

// Release releases the memory allocated by the builder.
func (b *MetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.released = true

		b.relatedData.Release()
	}
}

func (b *MetricsBuilder) ShowSchema() {
	b.builder.ShowSchema()
}
