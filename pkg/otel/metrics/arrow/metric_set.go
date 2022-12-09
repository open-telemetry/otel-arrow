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
		arrow.Field{Name: constants.NAME, Type: acommon.DefaultDictString},
		arrow.Field{Name: constants.DESCRIPTION, Type: acommon.DefaultDictString},
		arrow.Field{Name: constants.UNIT, Type: acommon.DefaultDictString},
		arrow.Field{Name: constants.DATA, Type: UnivariateMetricDT},
		arrow.Field{Name: constants.SHARED_ATTRIBUTES, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.SHARED_START_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.SHARED_TIME_UNIX_NANO, Type: arrow.PrimitiveTypes.Uint64},
	)

	// MultivariateMetricsDT is the Arrow Data Type describing a set of multivariate metrics.
	// Multivariate metrics are metrics sharing the same attributes, start time, and end time.
	MultivariateMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.NAME, Type: acommon.DefaultDictString},
		{Name: constants.DESCRIPTION, Type: acommon.DefaultDictString},
		{Name: constants.UNIT, Type: acommon.DefaultDictString},
		// TODO
	}...)
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
	sstunb *array.Uint64Builder               // shared start time unix nano builder
	stunb  *array.Uint64Builder               // shared time unix nano builder
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
		sstunb:   sb.FieldBuilder(5).(*array.Uint64Builder),
		stunb:    sb.FieldBuilder(6).(*array.Uint64Builder),
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
		b.sstunb.Append(uint64(*mdata.StartTime))
	} else {
		b.sstunb.AppendNull()
	}

	if mdata != nil && mdata.Time != nil {
		b.stunb.Append(uint64(*mdata.Time))
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
