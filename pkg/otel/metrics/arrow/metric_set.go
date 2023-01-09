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
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

var (
	// UnivariateMetricSetDT is the Arrow Data Type describing a set of univariate metrics.
	UnivariateMetricSetDT = arrow.StructOf(
		arrow.Field{Name: constants.Name, Type: acommon.DefaultDictString},
		arrow.Field{Name: constants.Description, Type: acommon.DefaultDictString},
		arrow.Field{Name: constants.Unit, Type: acommon.DefaultDictString},
		arrow.Field{Name: constants.Data, Type: UnivariateMetricDT},
		arrow.Field{Name: constants.SharedAttributes, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.SharedStartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		arrow.Field{Name: constants.SharedTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
	)
)

// MetricSetBuilder is a helper to build a metric set.
type MetricSetBuilder struct {
	released bool

	builder *array.StructBuilder

	nb     *acommon.AdaptiveDictionaryBuilder // metric name builder
	db     *acommon.AdaptiveDictionaryBuilder // metric description builder
	ub     *acommon.AdaptiveDictionaryBuilder // metric unit builder
	dtb    *UnivariateMetricBuilder           // univariate metric builder
	sab    *acommon.AttributesBuilder         // shared attributes builder
	sstunb *array.TimestampBuilder            // shared start time unix nano builder
	stunb  *array.TimestampBuilder            // shared time unix nano builder
}

// NewMetricSetBuilder creates a new SpansBuilder with a given allocator.
//
// Once the builder is no longer needed, Release() must be called to free the
// memory allocated by the builder.
func NewMetricSetBuilder(pool memory.Allocator) *MetricSetBuilder {
	sb := array.NewStructBuilder(pool, UnivariateMetricSetDT)
	return MetricSetBuilderFrom(sb)
}

func MetricSetBuilderFrom(sb *array.StructBuilder) *MetricSetBuilder {
	return &MetricSetBuilder{
		released: false,
		builder:  sb,
		nb:       acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(0)),
		db:       acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(1)),
		ub:       acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(2)),
		dtb:      UnivariateMetricBuilderFrom(sb.FieldBuilder(3).(*array.SparseUnionBuilder)),
		sab:      acommon.AttributesBuilderFrom(sb.FieldBuilder(4).(*array.MapBuilder)),
		sstunb:   sb.FieldBuilder(5).(*array.TimestampBuilder),
		stunb:    sb.FieldBuilder(6).(*array.TimestampBuilder),
	}
}

// Build builds the span array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *MetricSetBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("span builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new metric to the builder.
func (b *MetricSetBuilder) Append(metric pmetric.Metric, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("metric set builder already released")
	}

	b.builder.Append(true)

	name := metric.Name()
	if name == "" {
		b.nb.AppendNull()
	} else {
		if err := b.nb.AppendString(name); err != nil {
			return err
		}
	}
	desc := metric.Description()
	if desc == "" {
		b.db.AppendNull()
	} else {
		if err := b.db.AppendString(desc); err != nil {
			return err
		}
	}
	unit := metric.Unit()
	if unit == "" {
		b.ub.AppendNull()
	} else {
		if err := b.ub.AppendString(unit); err != nil {
			return err
		}
	}
	if err := b.dtb.Append(metric, smdata, mdata); err != nil {
		return err
	}

	attrs := pcommon.NewMap()
	if mdata.Attributes != nil && mdata.Attributes.Len() > 0 {
		mdata.Attributes.CopyTo(attrs)
	}
	err := b.sab.Append(attrs)
	if err != nil {
		return err
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
}

// Release releases the memory allocated by the builder.
func (b *MetricSetBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
