/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

import (
	"fmt"

	"github.com/HdrHistogram/hdrhistogram-go"
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// Arrow data types used to build the attribute map.
var (
	// KDT is the Arrow key data type.
	KDT = arrow.BinaryTypes.String

	// AttributesDT is the Arrow attribute data type.
	AttributesDT = arrow.MapOfWithMetadata(
		KDT, schema.Metadata(schema.Dictionary8),
		AnyValueDT, schema.Metadata(),
	)
)

type AttributesStats struct {
	AttrsHistogram *hdrhistogram.Histogram
	AnyValueStats  *AnyValueStats
}

// AttributesBuilder is a helper to build a map of attributes.
type AttributesBuilder struct {
	released bool

	builder *builder.MapBuilder    // map builder
	kb      *builder.StringBuilder // key builder
	ib      *AnyValueBuilder       // item any value builder
}

// NewAttributesBuilder creates a new AttributesBuilder with a given allocator.
//
// Once the builder is no longer needed, Build() or Release() must be called to free the
// memory allocated by the builder.
func NewAttributesBuilder(builder *builder.MapBuilder) *AttributesBuilder {
	return AttributesBuilderFrom(builder)
}

// AttributesBuilderFrom creates a new AttributesBuilder from an existing MapBuilder.
func AttributesBuilderFrom(mb *builder.MapBuilder) *AttributesBuilder {
	ib := AnyValueBuilderFrom(mb.ItemSparseUnionBuilder())

	return &AttributesBuilder{
		released: false,
		builder:  mb,
		kb:       mb.KeyStringBuilder(),
		ib:       ib,
	}
}

// Build builds the attribute array map.
//
// Once the returned array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *AttributesBuilder) Build() (*array.Map, error) {
	if b.released {
		return nil, werror.Wrap(ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewMapArray(), nil
}

// Append appends a new set of attributes to the builder.
// Note: empty keys are skipped.
func (b *AttributesBuilder) Append(attrs pcommon.Map) error {
	if b.released {
		return werror.Wrap(ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(attrs.Len(), func() error {
		var err error
		attrs.Range(func(key string, v pcommon.Value) bool {
			if key == "" {
				// Skip entries with empty keys
				return true
			}
			b.kb.AppendNonEmpty(key)
			return b.ib.Append(v) == nil
		})
		return werror.Wrap(err)
	})
}

func (b *AttributesBuilder) AppendUniqueAttributes(attrs pcommon.Map, smattrs *common.SharedAttributes, mattrs *common.SharedAttributes) error {
	if b.released {
		return werror.Wrap(ErrBuilderAlreadyReleased)
	}

	uniqueAttrsCount := attrs.Len()
	if smattrs != nil {
		uniqueAttrsCount -= smattrs.Len()
	}
	if mattrs != nil {
		uniqueAttrsCount -= mattrs.Len()
	}

	return b.builder.Append(uniqueAttrsCount, func() error {
		var err error
		attrs.Range(func(key string, v pcommon.Value) bool {
			// Skip the current attribute if it is a scope metric shared attribute
			// or a metric shared attribute
			smattrsFound := false
			mattrsFound := false
			if smattrs != nil {
				_, smattrsFound = smattrs.Attributes[key]
			}
			if mattrs != nil {
				_, mattrsFound = mattrs.Attributes[key]
			}
			if smattrsFound || mattrsFound {
				return true
			}

			b.kb.AppendNonEmpty(key)
			err = werror.WrapWithContext(b.ib.Append(v), map[string]interface{}{"key": key, "value": v})

			uniqueAttrsCount--
			return err == nil && uniqueAttrsCount > 0
		})
		return err
	})
}

// Release releases the memory allocated by the builder.
func (b *AttributesBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}

func NewAttributesStats() *AttributesStats {
	return &AttributesStats{
		AttrsHistogram: hdrhistogram.New(0, 1000000, 1),
		AnyValueStats:  NewAnyValueStats(),
	}
}

func (a *AttributesStats) UpdateStats(attrs pcommon.Map) {
	counters := ValueTypeCounters{}

	if err := a.AttrsHistogram.RecordValue(int64(attrs.Len())); err != nil {
		panic(fmt.Sprintf("failed to record attrs count: %v", err))
	}

	attrs.Range(func(key string, v pcommon.Value) bool {
		a.AnyValueStats.UpdateStats(v, &counters)
		return true
	})

	if err := a.AnyValueStats.StrHistogram.RecordValue(counters.strCount); err != nil {
		panic(fmt.Sprintf("failed to record str count: %v", err))
	}
	if err := a.AnyValueStats.I64Histogram.RecordValue(counters.i64Count); err != nil {
		panic(fmt.Sprintf("failed to record i64 count: %v", err))
	}
	if err := a.AnyValueStats.F64Histogram.RecordValue(counters.f64Count); err != nil {
		panic(fmt.Sprintf("failed to record f64 count: %v", err))
	}
	if err := a.AnyValueStats.BoolHistogram.RecordValue(counters.boolCount); err != nil {
		panic(fmt.Sprintf("failed to record bool count: %v", err))
	}
	if err := a.AnyValueStats.BinaryHistogram.RecordValue(counters.binaryCount); err != nil {
		panic(fmt.Sprintf("failed to record binary count: %v", err))
	}
	if err := a.AnyValueStats.ListHistogram.RecordValue(counters.listCount); err != nil {
		panic(fmt.Sprintf("failed to record list count: %v", err))
	}
	if err := a.AnyValueStats.MapHistogram.RecordValue(counters.mapCount); err != nil {
		panic(fmt.Sprintf("failed to record map count: %v", err))
	}
}

func (a *AttributesStats) Show(prefix string) {
	if a.AttrsHistogram.Mean() == 0 {
		return
	}
	fmt.Printf("%sAttributes -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		prefix,
		a.AttrsHistogram.Mean(),
		a.AttrsHistogram.Min(),
		a.AttrsHistogram.Max(),
		a.AttrsHistogram.StdDev(),
		a.AttrsHistogram.ValueAtQuantile(50),
		a.AttrsHistogram.ValueAtQuantile(99),
	)
	a.AnyValueStats.Show(prefix + "  ")
}
