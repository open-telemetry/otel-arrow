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

package otlp

import (
	"strconv"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	HistogramDataPointIDs struct {
		ID                     int
		ParentID               int
		Name                   int
		Description            int
		Unit                   int
		AggregationTemporality int
		StartTimeUnixNano      int
		TimeUnixNano           int
		Count                  int
		Sum                    int
		BucketCounts           int // List of uint64
		ExplicitBounds         int // List of float64
		Exemplars              *ExemplarIds
		Flags                  int
		Min                    int
		Max                    int
	}

	HistogramDataPointsStore struct {
		nextID      uint16
		metricByIDs map[uint16]map[string]*pmetric.Metric
	}
)

func NewHistogramDataPointsStore() *HistogramDataPointsStore {
	return &HistogramDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}
}

func (s *HistogramDataPointsStore) HistogramMetricsByID(ID uint16) []*pmetric.Metric {
	histograms, ok := s.metricByIDs[ID]
	if !ok {
		return make([]*pmetric.Metric, 0)
	}
	metrics := make([]*pmetric.Metric, 0, len(histograms))
	for _, metric := range histograms {
		metrics = append(metrics, metric)
	}
	return metrics
}

func SchemaToHistogramIDs(schema *arrow.Schema) (*HistogramDataPointIDs, error) {
	ID, err := arrowutils.FieldIDFromSchema(schema, constants.ID)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	parentID, err := arrowutils.FieldIDFromSchema(schema, constants.ParentID)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	name, err := arrowutils.FieldIDFromSchema(schema, constants.Name)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	description, err := arrowutils.FieldIDFromSchema(schema, constants.Description)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	unit, err := arrowutils.FieldIDFromSchema(schema, constants.Unit)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	aggregationTemporality, err := arrowutils.FieldIDFromSchema(schema, constants.AggregationTemporality)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	startTimeUnixNano, err := arrowutils.FieldIDFromSchema(schema, constants.StartTimeUnixNano)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	timeUnixNano, err := arrowutils.FieldIDFromSchema(schema, constants.TimeUnixNano)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	count, err := arrowutils.FieldIDFromSchema(schema, constants.HistogramCount)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	sum, err := arrowutils.FieldIDFromSchema(schema, constants.HistogramSum)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	bucketCounts, err := arrowutils.FieldIDFromSchema(schema, constants.HistogramBucketCounts)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	explicitBounds, err := arrowutils.FieldIDFromSchema(schema, constants.HistogramExplicitBounds)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	exemplars, err := NewExemplarIds(schema)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	flags, err := arrowutils.FieldIDFromSchema(schema, constants.Flags)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	min, err := arrowutils.FieldIDFromSchema(schema, constants.HistogramMin)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	max, err := arrowutils.FieldIDFromSchema(schema, constants.HistogramMax)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &HistogramDataPointIDs{
		ID:                     ID,
		ParentID:               parentID,
		Name:                   name,
		Description:            description,
		Unit:                   unit,
		AggregationTemporality: aggregationTemporality,
		StartTimeUnixNano:      startTimeUnixNano,
		TimeUnixNano:           timeUnixNano,
		Count:                  count,
		Sum:                    sum,
		BucketCounts:           bucketCounts,
		ExplicitBounds:         explicitBounds,
		Exemplars:              exemplars,
		Flags:                  flags,
		Min:                    min,
		Max:                    max,
	}, nil
}

func HistogramDataPointsStoreFrom(record arrow.Record, attrsStore *otlp.Attributes32Store) (*HistogramDataPointsStore, error) {
	defer record.Release()

	store := &HistogramDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}

	fieldIDs, err := SchemaToHistogramIDs(record.Schema())
	if err != nil {
		return nil, werror.Wrap(err)
	}

	count := int(record.NumRows())

	for row := 0; row < count; row++ {
		// Data Point ID
		ID, err := arrowutils.NullableU32FromRecord(record, fieldIDs.ID, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		// ParentID = Scope ID
		parentID, err := arrowutils.U16FromRecord(record, fieldIDs.ParentID, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		metrics := store.metricByIDs[parentID]
		if metrics == nil {
			metrics = make(map[string]*pmetric.Metric)
			store.metricByIDs[parentID] = metrics
		}

		name, err := arrowutils.StringFromRecord(record, fieldIDs.Name, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		description, err := arrowutils.StringFromRecord(record, fieldIDs.Description, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		unit, err := arrowutils.StringFromRecord(record, fieldIDs.Unit, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		aggregationTemporality, err := arrowutils.I32FromRecord(record, fieldIDs.AggregationTemporality, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		metricSig := name + ":" + description + ":" + unit + ":" + strconv.Itoa(int(aggregationTemporality))
		metric := metrics[metricSig]
		var histogram pmetric.Histogram

		if metric == nil {
			metricObj := pmetric.NewMetric()
			metric = &metricObj
			metric.SetName(name)
			metric.SetDescription(description)
			metric.SetUnit(unit)
			histogram = metric.SetEmptyHistogram()
			histogram.SetAggregationTemporality(pmetric.AggregationTemporality(aggregationTemporality))
			metrics[metricSig] = metric
		} else {
			histogram = metric.Histogram()
		}
		hdp := histogram.DataPoints().AppendEmpty()

		startTimeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.StartTimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		hdp.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.TimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		hdp.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		histogramCount, err := arrowutils.U64FromRecord(record, fieldIDs.Count, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		hdp.SetCount(histogramCount)

		sum, err := arrowutils.F64OrNilFromRecord(record, fieldIDs.Sum, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		if sum != nil {
			hdp.SetSum(*sum)
		}

		bucketCounts, start, end, err := arrowutils.ListValuesByIDFromRecord(record, fieldIDs.BucketCounts, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		if values, ok := bucketCounts.(*array.Uint64); ok {
			bucketCountsSlice := hdp.BucketCounts()
			bucketCountsSlice.EnsureCapacity(end - start)
			for i := start; i < end; i++ {
				bucketCountsSlice.Append(values.Value(i))
			}
		} else {
			return nil, werror.Wrap(ErrNotArrayUint64)
		}

		explicitBounds, start, end, err := arrowutils.ListValuesByIDFromRecord(record, fieldIDs.ExplicitBounds, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		if values, ok := explicitBounds.(*array.Float64); ok {
			explicitBoundsSlice := hdp.ExplicitBounds()
			explicitBoundsSlice.EnsureCapacity(end - start)
			for i := start; i < end; i++ {
				explicitBoundsSlice.Append(values.Value(i))
			}
		} else {
			return nil, werror.Wrap(ErrNotArrayFloat64)
		}

		exemplars, err := arrowutils.ListOfStructsFromRecord(record, fieldIDs.Exemplars.ID, row)
		if exemplars != nil && err == nil {
			if err := AppendExemplarsInto(hdp.Exemplars(), record, row, fieldIDs.Exemplars); err != nil {
				return nil, werror.Wrap(err)
			}
		} else if err != nil {
			return nil, werror.Wrap(err)
		}

		flags, err := arrowutils.U32FromRecord(record, fieldIDs.Flags, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		hdp.SetFlags(pmetric.DataPointFlags(flags))

		min, err := arrowutils.F64OrNilFromRecord(record, fieldIDs.Min, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		if min != nil {
			hdp.SetMin(*min)
		}

		max, err := arrowutils.F64OrNilFromRecord(record, fieldIDs.Max, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		if max != nil {
			hdp.SetMax(*max)
		}

		if ID != nil {
			attrs := attrsStore.AttributesByDeltaID(*ID)
			if attrs != nil {
				attrs.CopyTo(hdp.Attributes())
			}
		}
	}

	return store, nil
}
