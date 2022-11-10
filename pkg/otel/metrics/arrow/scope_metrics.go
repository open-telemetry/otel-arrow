package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ScopeMetricsDT is the Arrow Data Type describing a scope span.
var (
	ScopeMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.SCOPE, Type: acommon.ScopeDT},
		{Name: constants.SCHEMA_URL, Type: acommon.DictU16String},
		{Name: constants.METRICS, Type: arrow.ListOf(UnivariateMetricSetDT)},
	}...)
)

// ScopeMetricsBuilder is a helper to build a scope spans.
type ScopeMetricsBuilder struct {
	released bool

	builder *array.StructBuilder

	scb  *acommon.ScopeBuilder          // scope builder
	schb *array.BinaryDictionaryBuilder // schema url builder
	smb  *array.ListBuilder             // metrics list builder
	mb   *MetricSetBuilder              // metrics builder
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
		schb:     builder.FieldBuilder(1).(*array.BinaryDictionaryBuilder),
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
