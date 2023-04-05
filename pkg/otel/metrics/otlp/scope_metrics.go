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
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type ScopeMetricsIds struct {
	SchemaUrl          int
	ScopeIds           *otlp.ScopeIds
	MetricSetIds       *MetricSetIds
	SharedAttributeIds *otlp.AttributeIds
	SharedStartTimeID  int
	SharedTimeID       int
}

func NewScopeMetricsIds(scopeMetricsDT *arrow.StructType) (*ScopeMetricsIds, error) {
	schemaId, _ := arrowutils.FieldIDFromStruct(scopeMetricsDT, constants.SchemaUrl)

	scopeIds, err := otlp.NewScopeIds(scopeMetricsDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	metricSetIds, err := NewMetricSetIds(scopeMetricsDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	sharedAttrIds := otlp.NewSharedAttributeIds(scopeMetricsDT)
	startTimeID := arrowutils.OptionalFieldIDFromStruct(scopeMetricsDT, constants.SharedStartTimeUnixNano)
	timeID := arrowutils.OptionalFieldIDFromStruct(scopeMetricsDT, constants.SharedTimeUnixNano)

	return &ScopeMetricsIds{
		SchemaUrl:          schemaId,
		ScopeIds:           scopeIds,
		MetricSetIds:       metricSetIds,
		SharedAttributeIds: sharedAttrIds,
		SharedStartTimeID:  startTimeID,
		SharedTimeID:       timeID,
	}, nil
}

func UpdateScopeMetricsFrom(scopeMetricsSlice pmetric.ScopeMetricsSlice, arrowScopeMetrics *arrowutils.ListOfStructs, ids *ScopeMetricsIds) error {
	scopeMetricsSlice.EnsureCapacity(arrowScopeMetrics.End() - arrowScopeMetrics.Start())

	for scopeMetricsIdx := arrowScopeMetrics.Start(); scopeMetricsIdx < arrowScopeMetrics.End(); scopeMetricsIdx++ {
		scopeMetrics := scopeMetricsSlice.AppendEmpty()

		if err := otlp.UpdateScopeWith(scopeMetrics.Scope(), arrowScopeMetrics, scopeMetricsIdx, ids.ScopeIds); err != nil {
			return werror.Wrap(err)
		}

		schemaUrl, err := arrowScopeMetrics.StringFieldByID(ids.SchemaUrl, scopeMetricsIdx)
		if err != nil {
			return werror.Wrap(err)
		}
		scopeMetrics.SetSchemaUrl(schemaUrl)

		sdata := &SharedData{}
		sdata.Attributes = pcommon.NewMap()
		if ids.SharedAttributeIds != nil {
			err = otlp.AppendAttributesInto(sdata.Attributes, arrowScopeMetrics.Array(), scopeMetricsIdx, ids.SharedAttributeIds)
			if err != nil {
				return werror.Wrap(err)
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
			return werror.Wrap(err)
		}
		metricsSlice := scopeMetrics.Metrics()
		metricsSlice.EnsureCapacity(arrowMetrics.End() - arrowMetrics.Start())
		for entityIdx := arrowMetrics.Start(); entityIdx < arrowMetrics.End(); entityIdx++ {
			err = AppendMetricSetInto(metricsSlice, arrowMetrics, entityIdx, ids.MetricSetIds, sdata)
			if err != nil {
				return werror.Wrap(err)
			}
		}
	}

	return nil
}
