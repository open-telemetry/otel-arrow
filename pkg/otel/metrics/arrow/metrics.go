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
	"go.opentelemetry.io/collector/pdata/pmetric"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// Schema is the Arrow schema for the OTLP Arrow Metrics record.
var (
	Schema = arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceMetrics, Type: arrow.ListOf(ResourceMetricsDT), Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
)

// MetricsBuilder is a helper to build a list of resource metrics.
type MetricsBuilder struct {
	released bool

	builder *builder.RecordBuilderExt // Record builder
	rmb     *builder.ListBuilder      // resource metrics list builder
	rmp     *ResourceMetricsBuilder   // resource metrics builder
}

// NewMetricsBuilder creates a new MetricsBuilder with a given allocator.
func NewMetricsBuilder(rBuilder *builder.RecordBuilderExt) (*MetricsBuilder, error) {
	metricsBuilder := &MetricsBuilder{
		released: false,
		builder:  rBuilder,
	}
	if err := metricsBuilder.init(); err != nil {
		return nil, werror.Wrap(err)
	}
	return metricsBuilder, nil
}

func (b *MetricsBuilder) init() error {
	rmb := b.builder.ListBuilder(constants.ResourceMetrics)
	b.rmb = rmb
	b.rmp = ResourceMetricsBuilderFrom(rmb.StructBuilder())
	return nil
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

	rm := metrics.ResourceMetrics()
	rc := rm.Len()
	return b.rmb.Append(rc, func() error {
		for i := 0; i < rc; i++ {
			if err := b.rmp.Append(rm.At(i)); err != nil {
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
	}
}
