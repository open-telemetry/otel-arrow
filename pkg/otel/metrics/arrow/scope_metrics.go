package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ScopeMetricsDT is the Arrow Data Type describing a scope span.
var (
	ScopeMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.SCOPE, Type: acommon.ScopeDT},
		{Name: constants.SCHEMA_URL, Type: acommon.DefaultDictString},
		{Name: constants.UNIVARIATE_METRICS, Type: arrow.ListOf(UnivariateMetricSetDT)},
		{Name: constants.SHARED_ATTRIBUTES, Type: acommon.AttributesDT},
		{Name: constants.SHARED_START_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.SHARED_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
	}...)
)

// ScopeMetricsBuilder is a helper to build a scope spans.
type ScopeMetricsBuilder struct {
	released bool

	builder *array.StructBuilder

	scb    *acommon.ScopeBuilder              // scope builder
	schb   *acommon.AdaptiveDictionaryBuilder // schema url builder
	smb    *array.ListBuilder                 // metrics list builder
	mb     *MetricSetBuilder                  // metrics builder
	sab    *acommon.AttributesBuilder         // shared attributes builder
	sstunb *array.Uint64Builder               // shared start time unix nano builder
	stunb  *array.Uint64Builder               // shared time unix nano builder
}

type DataPoint interface {
	Attributes() pcommon.Map
	Timestamp() pcommon.Timestamp
	StartTimestamp() pcommon.Timestamp
}

// NewScopeMetricsBuilder creates a new ResourceMetricsBuilder with a given allocator.
//
// Once the builder is no longer needed, Release() must be called to free the
// memory allocated by the builder.
func NewScopeMetricsBuilder(pool memory.Allocator) *ScopeMetricsBuilder {
	builder := array.NewStructBuilder(pool, ScopeMetricsDT)
	return ScopeMetricsBuilderFrom(builder)
}

func ScopeMetricsBuilderFrom(builder *array.StructBuilder) *ScopeMetricsBuilder {
	return &ScopeMetricsBuilder{
		released: false,
		builder:  builder,
		scb:      acommon.ScopeBuilderFrom(builder.FieldBuilder(0).(*array.StructBuilder)),
		schb:     acommon.AdaptiveDictionaryBuilderFrom(builder.FieldBuilder(1)),
		smb:      builder.FieldBuilder(2).(*array.ListBuilder),
		mb:       MetricSetBuilderFrom(builder.FieldBuilder(2).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		sab:      acommon.AttributesBuilderFrom(builder.FieldBuilder(3).(*array.MapBuilder)),
		sstunb:   builder.FieldBuilder(4).(*array.Uint64Builder),
		stunb:    builder.FieldBuilder(5).(*array.Uint64Builder),
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

	b.builder.Append(true)
	if err := b.scb.Append(sm.Scope()); err != nil {
		return err
	}
	schemaUrl := sm.SchemaUrl()
	if schemaUrl == "" {
		b.schb.AppendNull()
	} else {
		if err := b.schb.AppendString(schemaUrl); err != nil {
			return err
		}
	}

	metrics := sm.Metrics()
	sharedData, err := NewMetricsSharedData(metrics)
	if err != nil {
		return err
	}
	mc := metrics.Len()
	if mc > 0 {
		b.smb.Append(true)
		b.smb.Reserve(mc)
		for i := 0; i < mc; i++ {
			if err := b.mb.Append(metrics.At(i), sharedData, sharedData.Metrics[i]); err != nil {
				return err
			}
		}
	} else {
		b.smb.Append(false)
	}

	attrs := pcommon.NewMap()
	if sharedData.Attributes != nil && sharedData.Attributes.Len() > 0 {
		sharedData.Attributes.CopyTo(attrs)
	}
	err = b.sab.Append(attrs)
	if err != nil {
		return err
	}

	if sharedData.StartTime != nil {
		b.sstunb.Append(uint64(*sharedData.StartTime))
	} else {
		b.sstunb.AppendNull()
	}

	if sharedData.Time != nil {
		b.stunb.Append(uint64(*sharedData.Time))
	} else {
		b.stunb.AppendNull()
	}

	return nil
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
	return
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
	default:
		err = fmt.Errorf("unknown metric type: %v", metric.Type())
		return
	}

	sharedData.NumDP = dpLen()
	if sharedData.NumDP > 0 {
		initSharedDataFrom(sharedData, dpAt(0))
		for i := 1; i < sharedData.NumDP; i++ {
			updateSharedDataWith(sharedData, dpAt(i))
		}
	}

	return
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
