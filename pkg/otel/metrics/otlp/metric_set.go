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
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type MetricSetIds struct {
	Id                 int
	Name               int
	Description        int
	Unit               int
	Data               *UnivariateMetricIds
	SharedAttributeIds *otlp.AttributeIds
	SharedStartTimeID  int
	SharedTimeID       int
}

func NewMetricSetIds(parentDT *arrow.StructType) (*MetricSetIds, error) {
	id, metricSetDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.UnivariateMetrics)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	name, _ := arrowutils.FieldIDFromStruct(metricSetDT, constants.Name)
	description, _ := arrowutils.FieldIDFromStruct(metricSetDT, constants.Description)
	unit, _ := arrowutils.FieldIDFromStruct(metricSetDT, constants.Unit)

	data, err := NewUnivariateMetricIds(metricSetDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	sharedAttrIds := otlp.NewSharedAttributeIds(metricSetDT)
	startTimeID := arrowutils.OptionalFieldIDFromStruct(metricSetDT, constants.SharedStartTimeUnixNano)
	timeID := arrowutils.OptionalFieldIDFromStruct(metricSetDT, constants.SharedTimeUnixNano)

	return &MetricSetIds{
		Id:                 id,
		Name:               name,
		Description:        description,
		Unit:               unit,
		Data:               data,
		SharedAttributeIds: sharedAttrIds,
		SharedStartTimeID:  startTimeID,
		SharedTimeID:       timeID,
	}, nil
}

func AppendMetricSetInto(metrics pmetric.MetricSlice, los *arrowutils.ListOfStructs, row int, ids *MetricSetIds, smdata *SharedData) error {
	metric := metrics.AppendEmpty()

	name, err := los.StringFieldByID(ids.Name, row)
	if err != nil {
		return werror.Wrap(err)
	}
	metric.SetName(name)

	description, err := los.StringFieldByID(ids.Description, row)
	if err != nil {
		return werror.Wrap(err)
	}
	metric.SetDescription(description)

	unit, err := los.StringFieldByID(ids.Unit, row)
	if err != nil {
		return werror.Wrap(err)
	}
	metric.SetUnit(unit)

	mdata := &SharedData{}
	mdata.Attributes = pcommon.NewMap()
	if ids.SharedAttributeIds != nil {
		err = otlp.AppendAttributesInto(mdata.Attributes, los.Array(), row, ids.SharedAttributeIds)
		if err != nil {
			return werror.Wrap(err)
		}
	}
	if ids.SharedStartTimeID != -1 {
		mdata.StartTime = los.OptionalTimestampFieldByID(ids.SharedStartTimeID, row)
	}
	if ids.SharedTimeID != -1 {
		mdata.Time = los.OptionalTimestampFieldByID(ids.SharedTimeID, row)
	}

	arr, ok := los.FieldByID(ids.Data.Id).(*array.SparseUnion)
	if !ok {
		return werror.Wrap(ErrNotArraySparseUnion)
	}
	return UpdateUnivariateMetricFrom(metric, arr, row, ids.Data, smdata, mdata)
}
