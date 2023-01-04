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
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ResourceMetricsIds struct {
	Id           int
	Resource     *otlp.ResourceIds
	SchemaUrl    int
	ScopeMetrics *ScopeMetricsIds
}

func NewResourceMetricsIds(schema *arrow.Schema) (*ResourceMetricsIds, error) {
	id, rsDT, err := arrowutils.ListOfStructsFieldIDFromSchema(schema, constants.ResourceMetrics)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrowutils.FieldIDFromStruct(rsDT, constants.SchemaUrl)
	if err != nil {
		return nil, err
	}

	scopeMetricsIds, err := NewScopeMetricsIds(rsDT)
	if err != nil {
		return nil, err
	}

	resourceIds, err := otlp.NewResourceIds(rsDT)
	if err != nil {
		return nil, err
	}

	return &ResourceMetricsIds{
		Id:           id,
		Resource:     resourceIds,
		SchemaUrl:    schemaId,
		ScopeMetrics: scopeMetricsIds,
	}, nil
}

func AppendResourceMetricsInto(metrics pmetric.Metrics, record arrow.Record, metricsIds *MetricsIds) error {
	resMetricsSlice := metrics.ResourceMetrics()
	resMetricsCount := int(record.NumRows())

	for metricsIdx := 0; metricsIdx < resMetricsCount; metricsIdx++ {
		arrowResEnts, err := arrowutils.ListOfStructsFromRecord(record, metricsIds.ResourceMetrics.Id, metricsIdx)
		if err != nil {
			return err
		}
		resMetricsSlice.EnsureCapacity(resMetricsSlice.Len() + arrowResEnts.End() - arrowResEnts.Start())

		for resMetricsIdx := arrowResEnts.Start(); resMetricsIdx < arrowResEnts.End(); resMetricsIdx++ {
			resMetrics := resMetricsSlice.AppendEmpty()

			if err = otlp.UpdateResourceWith(resMetrics.Resource(), arrowResEnts, resMetricsIdx, metricsIds.ResourceMetrics.Resource); err != nil {
				return err
			}

			schemaUrl, err := arrowResEnts.StringFieldByID(metricsIds.ResourceMetrics.SchemaUrl, resMetricsIdx)
			if err != nil {
				return err
			}
			resMetrics.SetSchemaUrl(schemaUrl)

			err = AppendScopeMetricsInto(resMetrics, arrowResEnts, resMetricsIdx, metricsIds.ResourceMetrics.ScopeMetrics)
			if err != nil {
				return err
			}
		}
	}

	return nil
}
