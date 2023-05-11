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
	"errors"
	"math"
	"sort"

	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pmetric"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

var (
	// SummaryDataPointSchema is the Arrow Schema describing a summary
	// data point.
	// Related record.
	SummaryDataPointSchema = arrow.NewSchema([]arrow.Field{
		// Unique identifier of the NDP. This ID is used to identify the
		// relationship between the NDP, its attributes and exemplars.
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional, schema.DeltaEncoding)},
		// The ID of the parent metric.
		{Name: constants.ParentID, Type: arrow.PrimitiveTypes.Uint16},
		{Name: constants.Name, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Dictionary8)},
		{Name: constants.Description, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.Unit, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.AggregationTemporality, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.IsMonotonic, Type: arrow.FixedWidthTypes.Boolean, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.StartTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SummaryCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SummarySum, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.SummaryQuantileValues, Type: arrow.ListOf(QuantileValueDT), Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
	}, nil)
)

type (
	// SummaryDataPointBuilder is a builder for a summary data point.
	SummaryDataPointBuilder struct {
		released bool

		builder *builder.RecordBuilderExt

		ib  *builder.Uint32DeltaBuilder // id builder
		pib *builder.Uint16Builder      // parent_id builder

		nb  *builder.StringBuilder  // metric name builder
		db  *builder.StringBuilder  // metric description builder
		ub  *builder.StringBuilder  // metric unit builder
		atb *builder.Int32Builder   // aggregation temporality builder
		imb *builder.BooleanBuilder // is monotonic builder

		stunb *builder.TimestampBuilder // start_time_unix_nano builder
		tunb  *builder.TimestampBuilder // time_unix_nano builder
		scb   *builder.Uint64Builder    // count builder
		ssb   *builder.Float64Builder   // sum builder
		qvlb  *builder.ListBuilder      // summary quantile value list builder
		qvb   *QuantileValueBuilder     // summary quantile value builder
		fb    *builder.Uint32Builder    // flags builder

		accumulator *SummaryAccumulator
		attrsAccu   *carrow.Attributes32Accumulator
	}

	Summary struct {
		ParentID               uint16
		Metric                 *pmetric.Metric
		AggregationTemporality pmetric.AggregationTemporality
		IsMonotonic            bool
		Orig                   *pmetric.SummaryDataPoint
	}

	SummaryAccumulator struct {
		groupCount uint32
		summaries  []Summary
	}
)

// NewSummaryDataPointBuilder creates a new SummaryDataPointBuilder.
func NewSummaryDataPointBuilder(rBuilder *builder.RecordBuilderExt) *SummaryDataPointBuilder {
	b := &SummaryDataPointBuilder{
		released:    false,
		builder:     rBuilder,
		accumulator: NewSummaryAccumulator(),
	}

	b.init()
	return b
}

func (b *SummaryDataPointBuilder) init() {
	b.ib = b.builder.Uint32DeltaBuilder(constants.ID)
	// As the attributes are sorted before insertion, the delta between two
	// consecutive attributes ID should always be <=1.
	b.ib.SetMaxDelta(1)
	b.pib = b.builder.Uint16Builder(constants.ParentID)

	b.nb = b.builder.StringBuilder(constants.Name)
	b.db = b.builder.StringBuilder(constants.Description)
	b.ub = b.builder.StringBuilder(constants.Unit)
	b.atb = b.builder.Int32Builder(constants.AggregationTemporality)
	b.imb = b.builder.BooleanBuilder(constants.IsMonotonic)

	qvlb := b.builder.ListBuilder(constants.SummaryQuantileValues)

	b.stunb = b.builder.TimestampBuilder(constants.StartTimeUnixNano)
	b.tunb = b.builder.TimestampBuilder(constants.TimeUnixNano)
	b.scb = b.builder.Uint64Builder(constants.SummaryCount)
	b.ssb = b.builder.Float64Builder(constants.SummarySum)
	b.qvlb = qvlb
	b.qvb = QuantileValueBuilderFrom(qvlb.StructBuilder())
	b.fb = b.builder.Uint32Builder(constants.Flags)
}

func (b *SummaryDataPointBuilder) SetAttributesAccumulator(accu *carrow.Attributes32Accumulator) {
	b.attrsAccu = accu
}

func (b *SummaryDataPointBuilder) SchemaID() string {
	return b.builder.SchemaID()
}

func (b *SummaryDataPointBuilder) IsEmpty() bool {
	return b.accumulator.IsEmpty()
}

func (b *SummaryDataPointBuilder) Accumulator() *SummaryAccumulator {
	return b.accumulator
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *SummaryDataPointBuilder) Build() (record arrow.Record, err error) {
	schemaNotUpToDateCount := 0

	// Loop until the record is built successfully.
	// Intermediaries steps may be required to update the schema.
	for {
		b.attrsAccu.Reset()
		record, err = b.TryBuild(b.attrsAccu)
		if err != nil {
			if record != nil {
				record.Release()
			}

			switch {
			case errors.Is(err, schema.ErrSchemaNotUpToDate):
				schemaNotUpToDateCount++
				if schemaNotUpToDateCount > 5 {
					panic("Too many consecutive schema updates. This shouldn't happen.")
				}
			default:
				return nil, werror.Wrap(err)
			}
		} else {
			break
		}
	}
	return record, werror.Wrap(err)
}

func (b *SummaryDataPointBuilder) Reset() {
	b.accumulator.Reset()
}

func (b *SummaryDataPointBuilder) PayloadType() *carrow.PayloadType {
	return carrow.PayloadTypes.Summary
}

// Release releases the underlying memory.
func (b *SummaryDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

func (b *SummaryDataPointBuilder) TryBuild(attrsAccu *carrow.Attributes32Accumulator) (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(carrow.ErrBuilderAlreadyReleased)
	}

	b.accumulator.Sort()

	for ID, summary := range b.accumulator.summaries {
		b.ib.Append(uint32(ID))
		b.pib.Append(summary.ParentID)

		// Attributes
		err = attrsAccu.Append(uint32(ID), summary.Orig.Attributes())
		if err != nil {
			return nil, werror.Wrap(err)
		}

		b.nb.AppendNonEmpty(summary.Metric.Name())
		b.db.AppendNonEmpty(summary.Metric.Description())
		b.ub.AppendNonEmpty(summary.Metric.Unit())
		b.atb.Append(int32(summary.AggregationTemporality))
		b.imb.Append(summary.IsMonotonic)

		b.stunb.Append(arrow.Timestamp(summary.Orig.StartTimestamp()))
		b.tunb.Append(arrow.Timestamp(summary.Orig.Timestamp()))

		b.scb.Append(summary.Orig.Count())
		b.ssb.AppendNonZero(summary.Orig.Sum())

		b.fb.Append(uint32(summary.Orig.Flags()))

		qvs := summary.Orig.QuantileValues()
		qvc := qvs.Len()
		err = b.qvlb.Append(qvc, func() error {
			for i := 0; i < qvc; i++ {
				if err := b.qvb.Append(qvs.At(i)); err != nil {
					return werror.Wrap(err)
				}
			}
			return nil
		})
		if err != nil {
			return nil, werror.Wrap(err)
		}
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		b.init()
	}
	return
}

// NewSummaryAccumulator creates a new SummaryAccumulator.
func NewSummaryAccumulator() *SummaryAccumulator {
	return &SummaryAccumulator{
		groupCount: 0,
		summaries:  make([]Summary, 0),
	}
}

func (a *SummaryAccumulator) IsEmpty() bool {
	return len(a.summaries) == 0
}

// Append appends a slice of number data points to the accumulator.
func (a *SummaryAccumulator) Append(
	parentID uint16,
	metric *pmetric.Metric,
	aggregationTemporality pmetric.AggregationTemporality,
	isMonotonic bool,
	summaries pmetric.SummaryDataPointSlice,
) {
	if a.groupCount == math.MaxUint32 {
		panic("The maximum number of group of summary data points has been reached (max is uint32).")
	}

	if summaries.Len() == 0 {
		return
	}

	for i := 0; i < summaries.Len(); i++ {
		summary := summaries.At(i)

		a.summaries = append(a.summaries, Summary{
			ParentID:               parentID,
			Orig:                   &summary,
			Metric:                 metric,
			AggregationTemporality: aggregationTemporality,
			IsMonotonic:            isMonotonic,
		})
	}

	a.groupCount++
}

func (a *SummaryAccumulator) Sort() {
	sort.Slice(a.summaries, func(i, j int) bool {
		if a.summaries[i].Metric.Name() == a.summaries[j].Metric.Name() {
			return a.summaries[i].ParentID < a.summaries[j].ParentID
		} else {
			return a.summaries[i].Metric.Name() < a.summaries[j].Metric.Name()
		}
	})
}

func (a *SummaryAccumulator) Reset() {
	a.groupCount = 0
	a.summaries = a.summaries[:0]
}
