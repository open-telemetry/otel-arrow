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
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateSummaryDataPointDT is the Arrow Data Type describing a univariate summary data point.
var (
	UnivariateSummaryDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT},
		arrow.Field{Name: constants.StartTimeUnixNano, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.SummaryCount, Type: arrow.PrimitiveTypes.Uint64},
		arrow.Field{Name: constants.SummarySum, Type: arrow.PrimitiveTypes.Float64},
		arrow.Field{Name: constants.SummaryQuantileValues, Type: arrow.ListOf(QuantileValueDT)},
		arrow.Field{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32},
	)
)

// UnivariateSummaryDataPointBuilder is a builder for a summary data point.
type UnivariateSummaryDataPointBuilder struct {
	released bool

	builder *array.StructBuilder

	ab    *acommon.AttributesBuilder // attributes builder
	stunb *array.Uint64Builder       // start_time_unix_nano builder
	tunb  *array.Uint64Builder       // time_unix_nano builder
	scb   *array.Uint64Builder       // count builder
	ssb   *array.Float64Builder      // sum builder
	qvlb  *array.ListBuilder         // summary quantile value list builder
	qvb   *QuantileValueBuilder      // summary quantile value builder
	fb    *array.Uint32Builder       // flags builder
}

// NewUnivariateSummaryDataPointBuilder creates a new UnivariateSummaryDataPointBuilder with a given memory allocator.
func NewUnivariateSummaryDataPointBuilder(pool memory.Allocator) *UnivariateSummaryDataPointBuilder {
	return UnivariateSummaryDataPointBuilderFrom(array.NewStructBuilder(pool, UnivariateSummaryDataPointDT))
}

// UnivariateSummaryDataPointBuilderFrom creates a new UnivariateSummaryDataPointBuilder from an existing StructBuilder.
func UnivariateSummaryDataPointBuilderFrom(ndpb *array.StructBuilder) *UnivariateSummaryDataPointBuilder {
	return &UnivariateSummaryDataPointBuilder{
		released: false,
		builder:  ndpb,

		ab:    acommon.AttributesBuilderFrom(ndpb.FieldBuilder(0).(*array.MapBuilder)),
		stunb: ndpb.FieldBuilder(1).(*array.Uint64Builder),
		tunb:  ndpb.FieldBuilder(2).(*array.Uint64Builder),
		scb:   ndpb.FieldBuilder(3).(*array.Uint64Builder),
		ssb:   ndpb.FieldBuilder(4).(*array.Float64Builder),
		qvlb:  ndpb.FieldBuilder(5).(*array.ListBuilder),
		qvb:   QuantileValueBuilderFrom(ndpb.FieldBuilder(5).(*array.ListBuilder).ValueBuilder().(*array.StructBuilder)),
		fb:    ndpb.FieldBuilder(6).(*array.Uint32Builder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *UnivariateSummaryDataPointBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("UnivariateSummaryDataPointBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *UnivariateSummaryDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new summary data point to the builder.
func (b *UnivariateSummaryDataPointBuilder) Append(sdp pmetric.SummaryDataPoint, smdata *ScopeMetricsSharedData, mdata *MetricSharedData) error {
	if b.released {
		return fmt.Errorf("UnivariateSummaryDataPointBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	if err := b.ab.AppendUniqueAttributes(sdp.Attributes(), smdata.Attributes, mdata.Attributes); err != nil {
		return err
	}

	if smdata.StartTime == nil && mdata.StartTime == nil {
		b.stunb.Append(uint64(sdp.StartTimestamp()))
	} else {
		b.stunb.AppendNull()
	}
	if smdata.Time == nil && mdata.Time == nil {
		b.tunb.Append(uint64(sdp.Timestamp()))
	} else {
		b.tunb.AppendNull()
	}

	b.scb.Append(sdp.Count())
	b.ssb.Append(sdp.Sum())
	qvs := sdp.QuantileValues()
	qvc := qvs.Len()
	if qvc > 0 {
		b.qvlb.Append(true)
		b.qvlb.Reserve(qvc)
		for i := 0; i < qvc; i++ {
			if err := b.qvb.Append(qvs.At(i)); err != nil {
				return err
			}
		}
	} else {
		b.qvlb.Append(false)
	}
	if sdp.Flags() != 0 {
		b.fb.Append(uint32(sdp.Flags()))
	} else {
		b.fb.AppendNull()
	}
	return nil
}
