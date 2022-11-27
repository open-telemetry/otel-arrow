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

// ScopeMetricsDT is the Arrow Data Type describing a scope span.
var (
	ScopeMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.SCOPE, Type: acommon.ScopeDT},
		{Name: constants.SCHEMA_URL, Type: acommon.DefaultDictString},
		{Name: constants.UNIVARIATE_METRICS, Type: arrow.ListOf(UnivariateMetricSetDT)},
	}...)
)

// ScopeMetricsBuilder is a helper to build a scope spans.
type ScopeMetricsBuilder struct {
	released bool

	builder *array.StructBuilder

	scb  *acommon.ScopeBuilder              // scope builder
	schb *acommon.AdaptiveDictionaryBuilder // schema url builder
	smb  *array.ListBuilder                 // metrics list builder
	mb   *MetricSetBuilder                  // metrics builder
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
	mc := metrics.Len()
	if mc > 0 {
		b.smb.Append(true)
		b.smb.Reserve(mc)
		for i := 0; i < mc; i++ {
			if err := b.mb.Append(metrics.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.smb.Append(false)
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *ScopeMetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.scb.Release()
		b.schb.Release()
		b.smb.Release()
		b.mb.Release()

		b.released = true
	}
}

type ScopeMetricsSharedData struct {
	StartTime  *pcommon.Timestamp
	Time       *pcommon.Timestamp
	Attributes *SharedAttributes
	Metrics    []*MetricSharedData
}

type MetricSharedData struct {
	// number of data points
	NumDP      int
	StartTime  *pcommon.Timestamp
	Time       *pcommon.Timestamp
	Attributes *SharedAttributes
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
		if msd.StartTime != nil && uint64(*sharedData.StartTime) != uint64(*msd.StartTime) {
			sharedData.StartTime = nil
		}
		if msd.Time != nil && uint64(*sharedData.Time) != uint64(*msd.Time) {
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
		for k := range sharedData.Attributes.attributes {
			for i := 0; i < len(sharedData.Metrics); i++ {
				delete(sharedData.Metrics[i].Attributes.attributes, k)
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
	sharedData.Attributes = NewSharedAttributesFrom(initDataPoint.Attributes())
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

// SharedAttributes is a data structure representing the shared attributes of a set of metrics.
type SharedAttributes struct {
	attributes map[string]pcommon.Value
}

// NewSharedAttributesFrom creates a new SharedAttributes from a [pcommon.Map] of attributes.
func NewSharedAttributesFrom(attrs pcommon.Map) *SharedAttributes {
	attributes := make(map[string]pcommon.Value)
	attrs.Range(func(k string, v pcommon.Value) bool {
		attributes[k] = v
		return true
	})
	return &SharedAttributes{
		attributes: attributes,
	}
}

func (sa *SharedAttributes) Clone() *SharedAttributes {
	attributes := make(map[string]pcommon.Value)
	for k, v := range sa.attributes {
		attributes[k] = v
	}
	return &SharedAttributes{
		attributes: attributes,
	}
}

// IntersectWithMap intersects the current SharedAttributes with a [pcommon.Map] of attributes
// and returns the number of shared attributes after the intersection.
func (sa *SharedAttributes) IntersectWithMap(attrs pcommon.Map) int {
	for k, v := range sa.attributes {
		if otherV, ok := attrs.Get(k); ok {
			if !v.Equal(otherV) {
				delete(sa.attributes, k)
			}
		} else {
			delete(sa.attributes, k)
		}
	}
	return len(sa.attributes)
}

func (sa *SharedAttributes) IntersectWith(other *SharedAttributes) int {
	for k, v := range sa.attributes {
		if otherV, ok := other.attributes[k]; ok {
			if !v.Equal(otherV) {
				delete(sa.attributes, k)
			}
		} else {
			delete(sa.attributes, k)
		}
	}
	return len(sa.attributes)
}

// Has returns true if the current SharedAttributes has the given attribute.
func (sa *SharedAttributes) Has(k string) bool {
	_, ok := sa.attributes[k]
	return ok
}

// Len returns the number of attributes in the current SharedAttributes.
func (sa *SharedAttributes) Len() int {
	return len(sa.attributes)
}
