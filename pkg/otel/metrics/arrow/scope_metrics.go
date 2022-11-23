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
	"github.com/f5/otel-arrow-adapter/pkg/otel/metrics"
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

type SharedData struct {
	startTime  *pcommon.Timestamp
	time       *pcommon.Timestamp
	attributes *SharedAttributes
}

func NewSharedDataFrom(metric pmetric.Metric) (sharedData *SharedData, err error) {
	sharedData = &SharedData{}
	var dpLen func() int
	var dpAt func(int) metrics.DataPoint

	switch metric.Type() {
	case pmetric.MetricTypeGauge:
		dps := metric.Gauge().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) metrics.DataPoint { return dps.At(i) }
	case pmetric.MetricTypeSum:
		dps := metric.Sum().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) metrics.DataPoint { return dps.At(i) }
	case pmetric.MetricTypeHistogram:
		dps := metric.Histogram().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) metrics.DataPoint { return dps.At(i) }
	case pmetric.MetricTypeSummary:
		dps := metric.Summary().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) metrics.DataPoint { return dps.At(i) }
	case pmetric.MetricTypeExponentialHistogram:
		dps := metric.ExponentialHistogram().DataPoints()
		dpLen = func() int { return dps.Len() }
		dpAt = func(i int) metrics.DataPoint { return dps.At(i) }
	default:
		err = fmt.Errorf("unknown metric type: %v", metric.Type())
		return
	}

	if dpLen() > 0 {
		initSharedDataFrom(sharedData, dpAt(0))
		for i := 1; i < dpLen(); i++ {
			updateSharedDataWith(sharedData, dpAt(i))
		}
	}

	return
}

func initSharedDataFrom(sharedData *SharedData, initDataPoint metrics.DataPoint) {
	startTime := initDataPoint.StartTimestamp()
	sharedData.startTime = &startTime
	time := initDataPoint.Timestamp()
	sharedData.time = &time
	sharedData.attributes = NewSharedAttributesFrom(initDataPoint.Attributes())
}

func updateSharedDataWith(sharedData *SharedData, dp metrics.DataPoint) int {
	if sharedData.startTime != nil && uint64(*sharedData.startTime) != uint64(dp.StartTimestamp()) {
		sharedData.startTime = nil
	}
	if sharedData.time != nil && uint64(*sharedData.time) != uint64(dp.Timestamp()) {
		sharedData.time = nil
	}
	return sharedData.attributes.IntersectWith(dp.Attributes())
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

// IntersectWith intersects the current SharedAttributes with a [pcommon.Map] of attributes
// and returns the number of shared attributes after the intersection.
func (sa *SharedAttributes) IntersectWith(attrs pcommon.Map) int {
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

// Has returns true if the current SharedAttributes has the given attribute.
func (sa *SharedAttributes) Has(k string) bool {
	_, ok := sa.attributes[k]
	return ok
}

// Len returns the number of attributes in the current SharedAttributes.
func (sa *SharedAttributes) Len() int {
	return len(sa.attributes)
}
