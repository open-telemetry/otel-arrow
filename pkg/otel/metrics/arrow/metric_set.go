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
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

var (
	// UnivariateMetricSetDT is the Arrow Data Type describing a set of univariate metrics.
	UnivariateMetricSetDT = arrow.StructOf(
		arrow.Field{Name: constants.Name, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		arrow.Field{Name: constants.Description, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		arrow.Field{Name: constants.Unit, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		arrow.Field{Name: constants.Data, Type: UnivariateMetricDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SharedAttributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SharedStartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SharedTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
	)
)

// MetricSetBuilder is a helper to build a metric set.
type MetricSetBuilder struct {
	released bool

	builder *builder.StructBuilder

	nb     *builder.StringBuilder     // metric name builder
	db     *builder.StringBuilder     // metric description builder
	ub     *builder.StringBuilder     // metric unit builder
	dtb    *UnivariateMetricBuilder   // univariate metric builder
	sab    *acommon.AttributesBuilder // shared attributes builder
	sstunb *builder.TimestampBuilder  // shared start time unix nano builder
	stunb  *builder.TimestampBuilder  // shared time unix nano builder
}

func MetricSetBuilderFrom(sb *builder.StructBuilder) *MetricSetBuilder {
	return &MetricSetBuilder{
		released: false,
		builder:  sb,
		nb:       sb.StringBuilder(constants.Name),
		db:       sb.StringBuilder(constants.Description),
		ub:       sb.StringBuilder(constants.Unit),
		dtb:      UnivariateMetricBuilderFrom(sb.SparseUnionBuilder(constants.Data)),
		sab:      acommon.AttributesBuilderFrom(sb.MapBuilder(constants.SharedAttributes)),
		sstunb:   sb.TimestampBuilder(constants.SharedStartTimeUnixNano),
		stunb:    sb.TimestampBuilder(constants.SharedTimeUnixNano),
	}
}

// Build builds the span array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *MetricSetBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new metric to the builder.
func (b *MetricSetBuilder) Append(metric pmetric.Metric, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(metric, func() error {
		b.nb.AppendNonEmpty(metric.Name())
		b.db.AppendNonEmpty(metric.Description())
		b.ub.AppendNonEmpty(metric.Unit())
		if err := b.dtb.Append(metric, smdata, mdata); err != nil {
			return werror.Wrap(err)
		}

		attrs := pcommon.NewMap()
		if mdata.Attributes != nil && mdata.Attributes.Len() > 0 {
			mdata.Attributes.CopyTo(attrs)
		}
		err := b.sab.Append(attrs)
		if err != nil {
			return werror.Wrap(err)
		}

		if mdata != nil && mdata.StartTime != nil {
			b.sstunb.Append(arrow.Timestamp(*mdata.StartTime))
		} else {
			b.sstunb.AppendNull()
		}

		if mdata != nil && mdata.Time != nil {
			b.stunb.Append(arrow.Timestamp(*mdata.Time))
		} else {
			b.stunb.AppendNull()
		}

		return nil
	})
}

// Release releases the memory allocated by the builder.
func (b *MetricSetBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
