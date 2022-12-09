package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// ScopeLogsDT is the Arrow Data Type describing a scope span.
var (
	ScopeLogsDT = arrow.StructOf([]arrow.Field{
		{Name: constants.SCOPE, Type: acommon.ScopeDT},
		{Name: constants.SCHEMA_URL, Type: acommon.DefaultDictString},
		{Name: constants.LOGS, Type: arrow.ListOf(LogRecordDT)},
	}...)
)

// ScopeLogsBuilder is a helper to build a scope logs.
type ScopeLogsBuilder struct {
	released bool

	builder *array.StructBuilder

	scb  *acommon.ScopeBuilder              // scope builder
	schb *acommon.AdaptiveDictionaryBuilder // schema url builder
	lrsb *array.ListBuilder                 // log recprd list builder
	lrb  *LogRecordBuilder                  // log record builder
}

// NewScopeLogsBuilder creates a new ResourceLogsBuilder with a given allocator.
//
// Once the builder is no longer needed, Release() must be called to free the
// memory allocated by the builder.
func NewScopeLogsBuilder(pool memory.Allocator) *ScopeLogsBuilder {
	builder := array.NewStructBuilder(pool, ScopeLogsDT)
	return ScopeLogsBuilderFrom(builder)
}

func ScopeLogsBuilderFrom(builder *array.StructBuilder) *ScopeLogsBuilder {
	return &ScopeLogsBuilder{
		released: false,
		builder:  builder,
		scb:      acommon.ScopeBuilderFrom(builder.FieldBuilder(0).(*array.StructBuilder)),
		schb:     acommon.AdaptiveDictionaryBuilderFrom(builder.FieldBuilder(1)),
		lrsb:     builder.FieldBuilder(2).(*array.ListBuilder),
		lrb:      LogRecordBuilderFrom(builder.FieldBuilder(2).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
	}
}

// Build builds the scope logs array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ScopeLogsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("scope logs builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new scope logs to the builder.
func (b *ScopeLogsBuilder) Append(sl plog.ScopeLogs) error {
	if b.released {
		return fmt.Errorf("scope logs builder already released")
	}

	b.builder.Append(true)
	if err := b.scb.Append(sl.Scope()); err != nil {
		return err
	}
	schemaUrl := sl.SchemaUrl()
	if schemaUrl == "" {
		b.schb.AppendNull()
	} else {
		if err := b.schb.AppendString(schemaUrl); err != nil {
			return err
		}
	}
	logRecords := sl.LogRecords()
	lrc := logRecords.Len()
	if lrc > 0 {
		b.lrsb.Append(true)
		b.lrsb.Reserve(lrc)
		for i := 0; i < lrc; i++ {
			if err := b.lrb.Append(logRecords.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.lrsb.Append(false)
	}
	return nil
}

// Release releases the memory allocated by the builder.
func (b *ScopeLogsBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
