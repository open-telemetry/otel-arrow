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
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// UnivariateSummaryDataPointDT is the Arrow Data Type describing a univariate summary data point.
var (
	UnivariateSummaryDataPointDT = arrow.StructOf(
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SummaryCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SummarySum, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SummaryQuantileValues, Type: arrow.ListOf(QuantileValueDT), Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
	)
)

// UnivariateSummaryDataPointBuilder is a builder for a summary data point.
type UnivariateSummaryDataPointBuilder struct {
	released bool

	builder *builder.StructBuilder

	ab    *acommon.AttributesBuilder // attributes builder
	stunb *builder.TimestampBuilder  // start_time_unix_nano builder
	tunb  *builder.TimestampBuilder  // time_unix_nano builder
	scb   *builder.Uint64Builder     // count builder
	ssb   *builder.Float64Builder    // sum builder
	qvlb  *builder.ListBuilder       // summary quantile value list builder
	qvb   *QuantileValueBuilder      // summary quantile value builder
	fb    *builder.Uint32Builder     // flags builder
}

// UnivariateSummaryDataPointBuilderFrom creates a new UnivariateSummaryDataPointBuilder from an existing StructBuilder.
func UnivariateSummaryDataPointBuilderFrom(ndpb *builder.StructBuilder) *UnivariateSummaryDataPointBuilder {
	qvlb := ndpb.ListBuilder(constants.SummaryQuantileValues)

	return &UnivariateSummaryDataPointBuilder{
		released: false,
		builder:  ndpb,

		ab:    acommon.AttributesBuilderFrom(ndpb.MapBuilder(constants.Attributes)),
		stunb: ndpb.TimestampBuilder(constants.StartTimeUnixNano),
		tunb:  ndpb.TimestampBuilder(constants.TimeUnixNano),
		scb:   ndpb.Uint64Builder(constants.SummaryCount),
		ssb:   ndpb.Float64Builder(constants.SummarySum),
		qvlb:  qvlb,
		qvb:   QuantileValueBuilderFrom(qvlb.StructBuilder()),
		fb:    ndpb.Uint32Builder(constants.Flags),
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
		return fmt.Errorf("UnivariateSummaryDataPointBuilder: Reserve() called after Release()")
	}

	return b.builder.Append(sdp, func() error {
		if err := b.ab.AppendUniqueAttributes(sdp.Attributes(), smdata.Attributes, mdata.Attributes); err != nil {
			return err
		}

		if smdata.StartTime == nil && mdata.StartTime == nil {
			b.stunb.Append(arrow.Timestamp(sdp.StartTimestamp()))
		} else {
			b.stunb.AppendNull()
		}
		if smdata.Time == nil && mdata.Time == nil {
			b.tunb.Append(arrow.Timestamp(sdp.Timestamp()))
		} else {
			b.tunb.AppendNull()
		}

		b.scb.Append(sdp.Count())
		b.ssb.AppendNonZero(sdp.Sum())

		b.fb.Append(uint32(sdp.Flags()))

		qvs := sdp.QuantileValues()
		qvc := qvs.Len()
		return b.qvlb.Append(qvc, func() error {
			for i := 0; i < qvc; i++ {
				if err := b.qvb.Append(qvs.At(i)); err != nil {
					return err
				}
			}
			return nil
		})
	})
}
