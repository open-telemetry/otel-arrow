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
	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	SummaryDataPointIDs struct {
		ID                int
		ParentID          int
		Name              int
		Description       int
		Unit              int
		StartTimeUnixNano int
		TimeUnixNano      int
		Count             int
		Sum               int
		QuantileValues    *QuantileValueIds
		Flags             int
	}

	SummaryDataPointsStore struct {
		nextID      uint16
		metricByIDs map[uint16]map[string]*pmetric.Metric
	}
)

func NewSummaryDataPointsStore() *SummaryDataPointsStore {
	return &SummaryDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}
}

func (s *SummaryDataPointsStore) SummaryMetricsByID(ID uint16) []*pmetric.Metric {
	summaries, ok := s.metricByIDs[ID]
	if !ok {
		return make([]*pmetric.Metric, 0)
	}
	metrics := make([]*pmetric.Metric, 0, len(summaries))
	for _, metric := range summaries {
		metrics = append(metrics, metric)
	}
	return metrics
}

func SchemaToSummaryIDs(schema *arrow.Schema) (*SummaryDataPointIDs, error) {
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

	startTimeUnixNano, err := arrowutils.FieldIDFromSchema(schema, constants.StartTimeUnixNano)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	timeUnixNano, err := arrowutils.FieldIDFromSchema(schema, constants.TimeUnixNano)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	count, err := arrowutils.FieldIDFromSchema(schema, constants.SummaryCount)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	sum, err := arrowutils.FieldIDFromSchema(schema, constants.SummarySum)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	quantileValues, err := NewQuantileValueIds(schema)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	flags, err := arrowutils.FieldIDFromSchema(schema, constants.Flags)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &SummaryDataPointIDs{
		ID:                ID,
		ParentID:          parentID,
		Name:              name,
		Description:       description,
		Unit:              unit,
		StartTimeUnixNano: startTimeUnixNano,
		TimeUnixNano:      timeUnixNano,
		Count:             count,
		Sum:               sum,
		QuantileValues:    quantileValues,
		Flags:             flags,
	}, nil
}

func SummaryDataPointsStoreFrom(record arrow.Record, attrsStore *otlp.Attributes32Store) (*SummaryDataPointsStore, error) {
	defer record.Release()

	store := &SummaryDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}

	fieldIDs, err := SchemaToSummaryIDs(record.Schema())
	if err != nil {
		return nil, werror.Wrap(err)
	}

	count := int(record.NumRows())

	for row := 0; row < count; row++ {
		// Number Data Point ID
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

		metricSig := name + ":" + description + ":" + unit
		metric := metrics[metricSig]
		var summary pmetric.Summary

		if metric == nil {
			metricObj := pmetric.NewMetric()
			metric = &metricObj
			metric.SetName(name)
			metric.SetDescription(description)
			metric.SetUnit(unit)
			summary = metric.SetEmptySummary()
			metrics[metricSig] = metric
		} else {
			summary = metric.Summary()
		}
		sdp := summary.DataPoints().AppendEmpty()

		startTimeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.StartTimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		sdp.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.TimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		sdp.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		summaryCount, err := arrowutils.U64FromRecord(record, fieldIDs.Count, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		sdp.SetCount(summaryCount)

		sum, err := arrowutils.F64FromRecord(record, fieldIDs.Sum, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		sdp.SetSum(sum)

		err = AppendQuantileValuesInto(sdp.QuantileValues(), record, row, fieldIDs.QuantileValues)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		flags, err := arrowutils.U32FromRecord(record, fieldIDs.Flags, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		sdp.SetFlags(pmetric.DataPointFlags(flags))

		if ID != nil {
			attrs := attrsStore.AttributesByDeltaID(*ID)
			if attrs != nil {
				attrs.CopyTo(sdp.Attributes())
			}
		}
	}

	return store, nil
}
