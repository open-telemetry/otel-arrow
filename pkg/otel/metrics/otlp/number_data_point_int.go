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
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	NumberDataPointIntIDs struct {
		ID                     int
		ParentID               int
		Name                   int
		Description            int
		Unit                   int
		AggregationTemporality int
		IsMonotonic            int
		StartTimeUnixNano      int
		TimeUnixNano           int
		MetricValue            int
		Exemplars              *ExemplarIds
		Flags                  int
	}

	SumIntDataPointsStore struct {
		nextID      uint16
		metricByIDs map[uint16]map[string]*pmetric.Metric
	}

	GaugeIntDataPointsStore struct {
		nextID      uint16
		metricByIDs map[uint16]map[string]*pmetric.Metric
	}
)

func NewSumIntDataPointsStore() *SumIntDataPointsStore {
	return &SumIntDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}
}

func NewGaugeIntDataPointsStore() *GaugeIntDataPointsStore {
	return &GaugeIntDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}
}

func (s *SumIntDataPointsStore) SumMetricsByID(ID uint16) []*pmetric.Metric {
	sums, ok := s.metricByIDs[ID]
	if !ok {
		return make([]*pmetric.Metric, 0)
	}
	metrics := make([]*pmetric.Metric, 0, len(sums))
	for _, metric := range sums {
		metrics = append(metrics, metric)
	}
	return metrics
}

func (s *GaugeIntDataPointsStore) GaugeMetricsByID(ID uint16) []*pmetric.Metric {
	sums, ok := s.metricByIDs[ID]
	if !ok {
		return make([]*pmetric.Metric, 0)
	}
	metrics := make([]*pmetric.Metric, 0, len(sums))
	for _, metric := range sums {
		metrics = append(metrics, metric)
	}
	return metrics
}

func SchemaToNDPIntIDs(schema *arrow.Schema) (*NumberDataPointIntIDs, error) {
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

	isMonotonic, err := arrowutils.FieldIDFromSchema(schema, constants.IsMonotonic)
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

	metricValue, err := arrowutils.FieldIDFromSchema(schema, constants.MetricValue)
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

	return &NumberDataPointIntIDs{
		ID:                     ID,
		ParentID:               parentID,
		Name:                   name,
		Description:            description,
		Unit:                   unit,
		AggregationTemporality: aggregationTemporality,
		IsMonotonic:            isMonotonic,
		StartTimeUnixNano:      startTimeUnixNano,
		TimeUnixNano:           timeUnixNano,
		MetricValue:            metricValue,
		Exemplars:              exemplars,
		Flags:                  flags,
	}, nil
}

func SumIntStoreFrom(record arrow.Record, attrsStore *otlp.Attributes32Store) (*SumIntDataPointsStore, error) {
	defer record.Release()

	store := &SumIntDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}

	fieldIDs, err := SchemaToNDPIntIDs(record.Schema())
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

		aggregationTemporality, err := arrowutils.I32FromRecord(record, fieldIDs.AggregationTemporality, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		isMonotonic, err := arrowutils.BoolFromRecord(record, fieldIDs.IsMonotonic, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}

		metricSig := name + ":" + description + ":" + unit + ":" + strconv.Itoa(int(aggregationTemporality)) + ":" + strconv.FormatBool(isMonotonic)
		metric := metrics[metricSig]
		var sum pmetric.Sum

		if metric == nil {
			metricObj := pmetric.NewMetric()
			metric = &metricObj
			metric.SetName(name)
			metric.SetDescription(description)
			metric.SetUnit(unit)
			sum = metric.SetEmptySum()
			sum.SetAggregationTemporality(pmetric.AggregationTemporality(aggregationTemporality))
			sum.SetIsMonotonic(isMonotonic)
			metrics[metricSig] = metric
		} else {
			sum = metric.Sum()
		}
		ndp := sum.DataPoints().AppendEmpty()

		startTimeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.StartTimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.TimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		metricValue, err := arrowutils.I64FromRecord(record, fieldIDs.MetricValue, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetIntValue(metricValue)

		if err := AppendExemplarsInto(ndp.Exemplars(), record, row, fieldIDs.Exemplars); err != nil {
			return nil, werror.Wrap(err)
		}

		flags, err := arrowutils.U32FromRecord(record, fieldIDs.Flags, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetFlags(pmetric.DataPointFlags(flags))

		if ID != nil {
			attrs := attrsStore.AttributesByDeltaID(*ID)
			if attrs != nil {
				attrs.CopyTo(ndp.Attributes())
			}
		}
	}

	return store, nil
}

func GaugeIntStoreFrom(record arrow.Record, attrsStore *otlp.Attributes32Store) (*GaugeIntDataPointsStore, error) {
	defer record.Release()

	store := &GaugeIntDataPointsStore{
		metricByIDs: make(map[uint16]map[string]*pmetric.Metric),
	}

	fieldIDs, err := SchemaToNDPIntIDs(record.Schema())
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
		var gauge pmetric.Gauge

		if metric == nil {
			metricObj := pmetric.NewMetric()
			metric = &metricObj
			metric.SetName(name)
			metric.SetDescription(description)
			metric.SetUnit(unit)
			gauge = metric.SetEmptyGauge()
			metrics[metricSig] = metric
		} else {
			gauge = metric.Gauge()
		}

		ndp := gauge.DataPoints().AppendEmpty()

		startTimeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.StartTimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))

		timeUnixNano, err := arrowutils.TimestampFromRecord(record, fieldIDs.TimeUnixNano, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		metricValue, err := arrowutils.I64FromRecord(record, fieldIDs.MetricValue, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetIntValue(metricValue)

		if err := AppendExemplarsInto(ndp.Exemplars(), record, row, fieldIDs.Exemplars); err != nil {
			return nil, werror.Wrap(err)
		}

		flags, err := arrowutils.U32FromRecord(record, fieldIDs.Flags, row)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		ndp.SetFlags(pmetric.DataPointFlags(flags))

		if ID != nil {
			attrs := attrsStore.AttributesByDeltaID(*ID)
			if attrs != nil {
				attrs.CopyTo(ndp.Attributes())
			}
		}
	}

	return store, nil
}
