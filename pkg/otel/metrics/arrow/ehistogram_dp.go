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

// This file contains the Arrow schema and encoding logic for exponential
// histogram metrics and associated data points. Our Arrow schema employs a
// flattened structure, optimally utilizing Arrow's columnar format. This
// denormalization obviates the need for an auxiliary pair of ID and ParentID,
// which would otherwise be necessary to denote the relationship between a
// metric and its data points. Consequently, the metric fields `Name`, `Description`,
// `Unit`, `AggregationTemporality`, and `IsMonotonic` will appear duplicated across
// each data point. However, due to the high compressibility of these fields,
// this redundancy isn't a concern. The corresponding Arrow records are sorted
// by the metric name, further enhancing the schema's efficiency and accessibility.

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
	// EHistogramDataPointSchema is the Arrow schema describing a
	// histogram data point.
	// Related record.
	EHistogramDataPointSchema = arrow.NewSchema([]arrow.Field{
		// Unique identifier of the EHDP. This ID is used to identify the
		// relationship between the EHDP, its attributes and exemplars.
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
		{Name: constants.HistogramCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.HistogramSum, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.ExpHistogramScale, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.ExpHistogramZeroCount, Type: arrow.PrimitiveTypes.Uint64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.ExpHistogramPositive, Type: EHistogramDataPointBucketsDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.ExpHistogramNegative, Type: EHistogramDataPointBucketsDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Exemplars, Type: arrow.ListOf(ExemplarDT), Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.HistogramMin, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.HistogramMax, Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
	}, nil)
)

type (
	// EHistogramDataPointBuilder is a builder for exponential histogram data points.
	EHistogramDataPointBuilder struct {
		released bool

		builder *builder.RecordBuilderExt

		ib  *builder.Uint32DeltaBuilder // id builder
		pib *builder.Uint16Builder      // parent_id builder

		nb  *builder.StringBuilder  // metric name builder
		db  *builder.StringBuilder  // metric description builder
		ub  *builder.StringBuilder  // metric unit builder
		atb *builder.Int32Builder   // aggregation temporality builder
		imb *builder.BooleanBuilder // is monotonic builder

		stunb *builder.TimestampBuilder          // start_time_unix_nano builder
		tunb  *builder.TimestampBuilder          // time_unix_nano builder
		hcb   *builder.Uint64Builder             // histogram_count builder
		hsb   *builder.Float64Builder            // histogram_sum builder
		sb    *builder.Int32Builder              // scale builder
		zcb   *builder.Uint64Builder             // zero_count builder
		pb    *EHistogramDataPointBucketsBuilder // positive buckets builder
		nbb   *EHistogramDataPointBucketsBuilder // negative buckets builder
		elb   *builder.ListBuilder               // exemplars builder
		eb    *ExemplarBuilder                   // exemplar builder
		fb    *builder.Uint32Builder             // flags builder
		hmib  *builder.Float64Builder            // histogram_min builder
		hmab  *builder.Float64Builder            // histogram_max builder

		accumulator *EHDPAccumulator
		attrsAccu   *carrow.Attributes32Accumulator
	}

	EHDP struct {
		ParentID               uint16
		Metric                 *pmetric.Metric
		AggregationTemporality pmetric.AggregationTemporality
		IsMonotonic            bool
		Orig                   *pmetric.ExponentialHistogramDataPoint
	}

	EHDPAccumulator struct {
		groupCount uint32
		ehdps      []EHDP
	}
)

// EHistogramDataPointBuilderFrom creates a new EHistogramDataPointBuilder from an existing StructBuilder.
func NewEHistogramDataPointBuilder(rBuilder *builder.RecordBuilderExt) *EHistogramDataPointBuilder {
	b := &EHistogramDataPointBuilder{
		released:    false,
		builder:     rBuilder,
		accumulator: NewEHDPAccumulator(),
	}

	b.init()
	return b
}

func (b *EHistogramDataPointBuilder) init() {
	b.ib = b.builder.Uint32DeltaBuilder(constants.ID)
	b.ib.SetMaxDelta(1)
	b.pib = b.builder.Uint16Builder(constants.ParentID)

	b.nb = b.builder.StringBuilder(constants.Name)
	b.db = b.builder.StringBuilder(constants.Description)
	b.ub = b.builder.StringBuilder(constants.Unit)
	b.atb = b.builder.Int32Builder(constants.AggregationTemporality)
	b.imb = b.builder.BooleanBuilder(constants.IsMonotonic)

	b.stunb = b.builder.TimestampBuilder(constants.StartTimeUnixNano)
	b.tunb = b.builder.TimestampBuilder(constants.TimeUnixNano)
	b.hcb = b.builder.Uint64Builder(constants.HistogramCount)
	b.hsb = b.builder.Float64Builder(constants.HistogramSum)
	b.sb = b.builder.Int32Builder(constants.ExpHistogramScale)
	b.zcb = b.builder.Uint64Builder(constants.ExpHistogramZeroCount)
	b.pb = EHistogramDataPointBucketsBuilderFrom(b.builder.StructBuilder(constants.ExpHistogramPositive))
	b.nbb = EHistogramDataPointBucketsBuilderFrom(b.builder.StructBuilder(constants.ExpHistogramNegative))
	b.elb = b.builder.ListBuilder(constants.Exemplars)
	b.eb = ExemplarBuilderFrom(b.elb.StructBuilder())
	b.fb = b.builder.Uint32Builder(constants.Flags)
	b.hmib = b.builder.Float64Builder(constants.HistogramMin)
	b.hmab = b.builder.Float64Builder(constants.HistogramMax)
}

func (b *EHistogramDataPointBuilder) SetAttributesAccumulator(accu *carrow.Attributes32Accumulator) {
	b.attrsAccu = accu
}

func (b *EHistogramDataPointBuilder) SchemaID() string {
	return b.builder.SchemaID()
}

func (b *EHistogramDataPointBuilder) IsEmpty() bool {
	return b.accumulator.IsEmpty()
}

func (b *EHistogramDataPointBuilder) Accumulator() *EHDPAccumulator {
	return b.accumulator
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *EHistogramDataPointBuilder) Build() (record arrow.Record, err error) {
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

func (b *EHistogramDataPointBuilder) Reset() {
	b.accumulator.Reset()
}

func (b *EHistogramDataPointBuilder) PayloadType() *carrow.PayloadType {
	return carrow.PayloadTypes.ExpHistogram
}

// Release releases the underlying memory.
func (b *EHistogramDataPointBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

func (b *EHistogramDataPointBuilder) TryBuild(attrsAccu *carrow.Attributes32Accumulator) (record arrow.Record, err error) {
	if b.released {
		return nil, werror.Wrap(carrow.ErrBuilderAlreadyReleased)
	}

	b.accumulator.Sort()

	for ID, ehdpRec := range b.accumulator.ehdps {
		ehdp := ehdpRec.Orig
		b.ib.Append(uint32(ID))
		b.pib.Append(ehdpRec.ParentID)

		// Attributes
		err = attrsAccu.Append(uint32(ID), ehdp.Attributes())
		if err != nil {
			return nil, werror.Wrap(err)
		}

		b.nb.AppendNonEmpty(ehdpRec.Metric.Name())
		b.db.AppendNonEmpty(ehdpRec.Metric.Description())
		b.ub.AppendNonEmpty(ehdpRec.Metric.Unit())
		b.atb.Append(int32(ehdpRec.AggregationTemporality))
		b.imb.Append(ehdpRec.IsMonotonic)

		b.stunb.Append(arrow.Timestamp(ehdp.StartTimestamp()))
		b.tunb.Append(arrow.Timestamp(ehdp.Timestamp()))

		b.AppendCountSum(*ehdp)
		b.sb.AppendNonZero(ehdp.Scale())
		b.zcb.Append(ehdp.ZeroCount())
		if err := b.pb.Append(ehdp.Positive()); err != nil {
			return nil, werror.Wrap(err)
		}
		if err := b.nbb.Append(ehdp.Negative()); err != nil {
			return nil, werror.Wrap(err)
		}

		err := b.AppendExemplars(*ehdp)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		b.fb.Append(uint32(ehdp.Flags()))

		b.AppendMinMax(*ehdp)
	}

	record, err = b.builder.NewRecord()
	if err != nil {
		b.init()
	}
	return
}

func (b *EHistogramDataPointBuilder) AppendExemplars(hdp pmetric.ExponentialHistogramDataPoint) error {
	exs := hdp.Exemplars()
	ec := exs.Len()
	return b.elb.Append(ec, func() error {
		for i := 0; i < ec; i++ {
			if err := b.eb.Append(exs.At(i)); err != nil {
				return werror.Wrap(err)
			}
		}
		return nil
	})
}

func (b *EHistogramDataPointBuilder) AppendCountSum(hdp pmetric.ExponentialHistogramDataPoint) {
	b.hcb.Append(hdp.Count())
	if hdp.HasSum() {
		b.hsb.AppendNonZero(hdp.Sum())
	} else {
		b.hsb.AppendNull()
	}
}

func (b *EHistogramDataPointBuilder) AppendMinMax(hdp pmetric.ExponentialHistogramDataPoint) {
	if hdp.HasMin() {
		b.hmib.AppendNonZero(hdp.Min())
	} else {
		b.hmib.AppendNull()
	}
	if hdp.HasMax() {
		b.hmab.AppendNonZero(hdp.Max())
	} else {
		b.hmab.AppendNull()
	}
}

func NewEHDPAccumulator() *EHDPAccumulator {
	return &EHDPAccumulator{
		groupCount: 0,
		ehdps:      make([]EHDP, 0),
	}
}

func (a *EHDPAccumulator) IsEmpty() bool {
	return len(a.ehdps) == 0
}

func (a *EHDPAccumulator) Append(
	metricID uint16,
	metric *pmetric.Metric,
	aggregationTemporality pmetric.AggregationTemporality,
	isMonotonic bool,
	ehdps pmetric.ExponentialHistogramDataPointSlice,
) {
	if a.groupCount == math.MaxUint32 {
		panic("The maximum number of group of exponential histogram data points has been reached (max is uint32).")
	}

	if ehdps.Len() == 0 {
		return
	}

	for i := 0; i < ehdps.Len(); i++ {
		ehdp := ehdps.At(i)

		a.ehdps = append(a.ehdps, EHDP{
			ParentID:               metricID,
			Metric:                 metric,
			AggregationTemporality: aggregationTemporality,
			IsMonotonic:            isMonotonic,
			Orig:                   &ehdp,
		})
	}

	a.groupCount++
}

func (a *EHDPAccumulator) Sort() {
	sort.Slice(a.ehdps, func(i, j int) bool {
		if a.ehdps[i].Metric.Name() == a.ehdps[j].Metric.Name() {
			return a.ehdps[i].Orig.Timestamp() < a.ehdps[j].Orig.Timestamp()
		} else {
			return a.ehdps[i].Metric.Name() < a.ehdps[j].Metric.Name()
		}
	})
}

func (a *EHDPAccumulator) Reset() {
	a.groupCount = 0
	a.ehdps = a.ehdps[:0]
}
