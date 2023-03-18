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
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ScopeMetricsDT is the Arrow Data Type describing a scope span.
var (
	ScopeMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.Scope, Type: acommon.ScopeDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SchemaUrl, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: constants.UnivariateMetrics, Type: arrow.ListOf(UnivariateMetricSetDT), Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SharedAttributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SharedStartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SharedTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// ScopeMetricsBuilder is a helper to build a scope spans.
type ScopeMetricsBuilder struct {
	released bool

	builder *builder.StructBuilder

	scb    *acommon.ScopeBuilder      // scope builder
	schb   *builder.StringBuilder     // schema url builder
	smb    *builder.ListBuilder       // metrics list builder
	mb     *MetricSetBuilder          // metrics builder
	sab    *acommon.AttributesBuilder // shared attributes builder
	sstunb *builder.TimestampBuilder  // shared start time unix nano builder
	stunb  *builder.TimestampBuilder  // shared time unix nano builder
}

type DataPoint interface {
	Attributes() pcommon.Map
	Timestamp() pcommon.Timestamp
	StartTimestamp() pcommon.Timestamp
}

func ScopeMetricsBuilderFrom(builder *builder.StructBuilder) *ScopeMetricsBuilder {
	smb := builder.ListBuilder(constants.UnivariateMetrics)
	return &ScopeMetricsBuilder{
		released: false,
		builder:  builder,
		scb:      acommon.ScopeBuilderFrom(builder.StructBuilder(constants.Scope)),
		schb:     builder.StringBuilder(constants.SchemaUrl),
		smb:      smb,
		mb:       MetricSetBuilderFrom(smb.StructBuilder()),
		sab:      acommon.AttributesBuilderFrom(builder.MapBuilder(constants.SharedAttributes)),
		sstunb:   builder.TimestampBuilder(constants.SharedStartTimeUnixNano),
		stunb:    builder.TimestampBuilder(constants.SharedTimeUnixNano),
	}
}

// Build builds the scope metrics array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ScopeMetricsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("scope metrics builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new scope metrics to the builder.
func (b *ScopeMetricsBuilder) Append(sm pmetric.ScopeMetrics) error {
	if b.released {
		return fmt.Errorf("scope metrics builder already released")
	}

	return b.builder.Append(sm, func() error {
		if err := b.scb.Append(sm.Scope()); err != nil {
			return err
		}
		b.schb.AppendNonEmpty(sm.SchemaUrl())

		metrics := sm.Metrics()
		sharedData, err := NewMetricsSharedData(metrics)
		if err != nil {
			return err
		}
		mc := metrics.Len()
		if err = b.smb.Append(mc, func() error {
			for i := 0; i < mc; i++ {
				if err := b.mb.Append(metrics.At(i), sharedData, sharedData.Metrics[i]); err != nil {
					return err
				}
			}
			return nil
		}); err != nil {
			return err
		}

		if sharedData.Attributes != nil && sharedData.Attributes.Len() > 0 {
			attrs := pcommon.NewMap()
			sharedData.Attributes.CopyTo(attrs)
			err = b.sab.Append(attrs)
			if err != nil {
				return err
			}
		}

		if sharedData.StartTime != nil {
			b.sstunb.Append(arrow.Timestamp(*sharedData.StartTime))
		} else {
			b.sstunb.AppendNull()
		}

		if sharedData.Time != nil {
			b.stunb.Append(arrow.Timestamp(*sharedData.Time))
		} else {
			b.stunb.AppendNull()
		}

		return nil
	})
}

// Release releases the memory allocated by the builder.
func (b *ScopeMetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}

type ScopeMetricsSharedData struct {
	StartTime  *pcommon.Timestamp
	Time       *pcommon.Timestamp
	Attributes *common.SharedAttributes
	Metrics    []*MetricSharedData
}

type MetricSharedData struct {
	// number of data points
	NumDP      int
	StartTime  *pcommon.Timestamp
	Time       *pcommon.Timestamp
	Attributes *common.SharedAttributes
}

func NewMetricsSharedData(metrics pmetric.MetricSlice) (sharedData *ScopeMetricsSharedData, err error) {
	if metrics.Len() > 0 {
		msd, err := NewMetricSharedData(metrics.At(0))
		if err != nil {
			return nil, err
		}
		sharedData = &ScopeMetricsSharedData{Metrics: make([]*MetricSharedData, metrics.Len())}
		sharedData.StartTime = msd.StartTime
		sharedData.Time = msd.Time
		sharedData.Attributes = msd.Attributes.Clone()
		sharedData.Metrics[0] = msd
	}
	for i := 1; i < metrics.Len(); i++ {
		msd, err := NewMetricSharedData(metrics.At(i))
		if err != nil {
			return nil, err
		}
		sharedData.Metrics[i] = msd
		if msd.StartTime != nil && sharedData.StartTime != nil && uint64(*sharedData.StartTime) != uint64(*msd.StartTime) {
			sharedData.StartTime = nil
		}
		if msd.Time != nil && sharedData.Time != nil && uint64(*sharedData.Time) != uint64(*msd.Time) {
			sharedData.Time = nil
		}
		if sharedData.Attributes.Len() > 0 {
			sharedData.Attributes.IntersectWith(msd.Attributes)
		}
	}
	if sharedData != nil {
		if sharedData.StartTime != nil {
			for i := 0; i < len(sharedData.Metrics); i++ {
				sharedData.Metrics[i].StartTime = nil
			}
		}
		if sharedData.Time != nil {
			for i := 0; i < len(sharedData.Metrics); i++ {
				sharedData.Metrics[i].Time = nil
			}
		}
		for k := range sharedData.Attributes.Attributes {
			for i := 0; i < len(sharedData.Metrics); i++ {
				delete(sharedData.Metrics[i].Attributes.Attributes, k)
			}
		}
		if sharedData.Attributes.Len() == 0 {
			sharedData.Attributes = nil
		}
	}
	return sharedData, err
}

func NewMetricSharedData(metric pmetric.Metric) (sharedData *MetricSharedData, err error) {
	sharedData = &MetricSharedData{}
	var dpLen func() int
	var dpAt func(int) DataPoint

	switch metric.Type() {
	case pmetric.MetricTypeGauge:
		dps := metric.Gauge().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) DataPoint { return dps.At(i) }
	case pmetric.MetricTypeSum:
		dps := metric.Sum().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) DataPoint { return dps.At(i) }
	case pmetric.MetricTypeHistogram:
		dps := metric.Histogram().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) DataPoint { return dps.At(i) }
	case pmetric.MetricTypeSummary:
		dps := metric.Summary().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) DataPoint { return dps.At(i) }
	case pmetric.MetricTypeExponentialHistogram:
		dps := metric.ExponentialHistogram().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) DataPoint { return dps.At(i) }
	case pmetric.MetricTypeEmpty:
		// ignore empty metric.
	default:
		err = fmt.Errorf("unknown metric type: %v", metric.Type())
		return sharedData, err
	}

	sharedData.NumDP = dpLen()
	if sharedData.NumDP > 0 {
		initSharedDataFrom(sharedData, dpAt(0))
		for i := 1; i < sharedData.NumDP; i++ {
			updateSharedDataWith(sharedData, dpAt(i))
		}
	}

	return sharedData, err
}

func initSharedDataFrom(sharedData *MetricSharedData, initDataPoint DataPoint) {
	startTime := initDataPoint.StartTimestamp()
	sharedData.StartTime = &startTime
	time := initDataPoint.Timestamp()
	sharedData.Time = &time
	sharedData.Attributes = common.NewSharedAttributesFrom(initDataPoint.Attributes())
}

func updateSharedDataWith(sharedData *MetricSharedData, dp DataPoint) int {
	if sharedData.StartTime != nil && uint64(*sharedData.StartTime) != uint64(dp.StartTimestamp()) {
		sharedData.StartTime = nil
	}
	if sharedData.Time != nil && uint64(*sharedData.Time) != uint64(dp.Timestamp()) {
		sharedData.Time = nil
	}
	return sharedData.Attributes.IntersectWithMap(dp.Attributes())
}
