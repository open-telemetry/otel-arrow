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
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ScopeMetricsIds struct {
	Id                 int
	SchemaUrl          int
	ScopeIds           *otlp.ScopeIds
	MetricSetIds       *MetricSetIds
	SharedAttributeIds *otlp.AttributeIds
	SharedStartTimeID  int
	SharedTimeID       int
}

func NewScopeMetricsIds(dt *arrow.StructType) (*ScopeMetricsIds, error) {
	id, scopeMetricsDT, err := arrowutils.ListOfStructsFieldIDFromStruct(dt, constants.ScopeMetrics)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrowutils.FieldIDFromStruct(scopeMetricsDT, constants.SchemaUrl)
	if err != nil {
		return nil, err
	}

	scopeIds, err := otlp.NewScopeIds(scopeMetricsDT)
	if err != nil {
		return nil, err
	}

	metricSetIds, err := NewMetricSetIds(scopeMetricsDT)
	if err != nil {
		return nil, err
	}

	sharedAttrIds := otlp.NewSharedAttributeIds(scopeMetricsDT)
	startTimeID := arrowutils.OptionalFieldIDFromStruct(scopeMetricsDT, constants.SharedStartTimeUnixNano)
	timeID := arrowutils.OptionalFieldIDFromStruct(scopeMetricsDT, constants.SharedTimeUnixNano)

	return &ScopeMetricsIds{
		Id:                 id,
		SchemaUrl:          schemaId,
		ScopeIds:           scopeIds,
		MetricSetIds:       metricSetIds,
		SharedAttributeIds: sharedAttrIds,
		SharedStartTimeID:  startTimeID,
		SharedTimeID:       timeID,
	}, nil
}

func AppendScopeMetricsInto(resMetrics pmetric.ResourceMetrics, arrowResMetrics *arrowutils.ListOfStructs, resMetricsIdx int, ids *ScopeMetricsIds) error {
	arrowScopeMetrics, err := arrowResMetrics.ListOfStructsById(resMetricsIdx, ids.Id)
	if err != nil {
		return err
	}
	scopeMetricsSlice := resMetrics.ScopeMetrics()
	scopeMetricsSlice.EnsureCapacity(arrowScopeMetrics.End() - arrowResMetrics.Start())

	for scopeMetricsIdx := arrowScopeMetrics.Start(); scopeMetricsIdx < arrowScopeMetrics.End(); scopeMetricsIdx++ {
		scopeMetrics := scopeMetricsSlice.AppendEmpty()

		if err = otlp.UpdateScopeWith(scopeMetrics.Scope(), arrowScopeMetrics, scopeMetricsIdx, ids.ScopeIds); err != nil {
			return err
		}

		schemaUrl, err := arrowScopeMetrics.StringFieldByID(ids.SchemaUrl, scopeMetricsIdx)
		if err != nil {
			return err
		}
		scopeMetrics.SetSchemaUrl(schemaUrl)

		sdata := &SharedData{}
		if ids.SharedAttributeIds != nil {
			sdata.Attributes = pcommon.NewMap()
			err = otlp.AppendAttributesInto(sdata.Attributes, arrowScopeMetrics.Array(), scopeMetricsIdx, ids.SharedAttributeIds)
			if err != nil {
				return err
			}
		}
		if ids.SharedStartTimeID != -1 {
			sdata.StartTime = arrowScopeMetrics.OptionalTimestampFieldByID(ids.SharedStartTimeID, scopeMetricsIdx)
		}
		if ids.SharedTimeID != -1 {
			sdata.Time = arrowScopeMetrics.OptionalTimestampFieldByID(ids.SharedTimeID, scopeMetricsIdx)
		}

		arrowMetrics, err := arrowScopeMetrics.ListOfStructsById(scopeMetricsIdx, ids.MetricSetIds.Id)
		if err != nil {
			return err
		}
		metricsSlice := scopeMetrics.Metrics()
		metricsSlice.EnsureCapacity(arrowMetrics.End() - arrowMetrics.Start())
		for entityIdx := arrowMetrics.Start(); entityIdx < arrowMetrics.End(); entityIdx++ {
			err = AppendMetricSetInto(metricsSlice, arrowMetrics, entityIdx, ids.MetricSetIds, sdata)
			if err != nil {
				return err
			}
		}
	}

	return nil
}
