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

var (
	ResourceMetricsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.RESOURCE, Type: acommon.ResourceDT},
		{Name: constants.SCHEMA_URL, Type: acommon.DefaultDictString},
		{Name: constants.SCOPE_METRICS, Type: arrow.ListOf(ScopeMetricsDT)},
	}...)
)

// ResourceMetricsBuilder is a helper to build resource metrics.
type ResourceMetricsBuilder struct {
	released bool

	builder *array.StructBuilder // builder for the resource metrics struct

	rb   *acommon.ResourceBuilder           // resource builder
	schb *acommon.AdaptiveDictionaryBuilder // schema url builder
	spsb *array.ListBuilder                 // scope metrics list builder
	smb  *ScopeMetricsBuilder               // scope metrics builder
}

// NewResourceMetricsBuilder creates a new ResourceMetricsBuilder with a given allocator.
//
// Once the builder is no longer needed, Build() or Release() must be called to free the
// memory allocated by the builder.
func NewResourceMetricsBuilder(pool memory.Allocator) *ResourceMetricsBuilder {
	builder := array.NewStructBuilder(pool, ResourceMetricsDT)
	return ResourceMetricsBuilderFrom(builder)
}

// ResourceMetricsBuilderFrom creates a new ResourceMetricsBuilder from an existing builder.
func ResourceMetricsBuilderFrom(builder *array.StructBuilder) *ResourceMetricsBuilder {
	return &ResourceMetricsBuilder{
		released: false,
		builder:  builder,
		rb:       acommon.ResourceBuilderFrom(builder.FieldBuilder(0).(*array.StructBuilder)),
		schb:     acommon.AdaptiveDictionaryBuilderFrom(builder.FieldBuilder(1)),
		spsb:     builder.FieldBuilder(2).(*array.ListBuilder),
		smb:      ScopeMetricsBuilderFrom(builder.FieldBuilder(2).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
	}
}

// Build builds the resource metrics array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ResourceMetricsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("resource metrics builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new resource metrics to the builder.
func (b *ResourceMetricsBuilder) Append(sm pmetric.ResourceMetrics) error {
	if b.released {
		return fmt.Errorf("resource metrics builder already released")
	}

	b.builder.Append(true)
	if err := b.rb.Append(sm.Resource()); err != nil {
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
	smetrics := sm.ScopeMetrics()
	sc := smetrics.Len()
	if sc > 0 {
		b.spsb.Append(true)
		b.spsb.Reserve(sc)
		for i := 0; i < sc; i++ {
			if err := b.smb.Append(smetrics.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.spsb.Append(false)
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *ResourceMetricsBuilder) Release() {
	if !b.released {
		b.builder.Release()
		b.rb.Release()
		b.schb.Release()
		b.spsb.Release()
		b.smb.Release()

		b.released = true
	}
}
