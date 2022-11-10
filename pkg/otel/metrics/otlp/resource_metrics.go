package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
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
	id, rsDT, err := arrow_utils.ListOfStructsFieldIdFromSchema(schema, constants.RESOURCE_METRICS)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrow_utils.FieldIdFromStruct(rsDT, constants.SCHEMA_URL)
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
		arrowResEnts, err := arrow_utils.ListOfStructsFromRecordBis(record, metricsIds.ResourceMetrics.Id, metricsIdx)
		if err != nil {
			return err
		}
		resMetricsSlice.EnsureCapacity(resMetricsSlice.Len() + arrowResEnts.End() - arrowResEnts.Start())

		for resMetricsIdx := arrowResEnts.Start(); resMetricsIdx < arrowResEnts.End(); resMetricsIdx++ {
			resMetrics := resMetricsSlice.AppendEmpty()

			if err = otlp.UpdateResourceWith(resMetrics.Resource(), arrowResEnts, resMetricsIdx, metricsIds.ResourceMetrics.Resource); err != nil {
				return err
			}

			schemaUrl, err := arrowResEnts.StringFieldById(metricsIds.ResourceMetrics.SchemaUrl, resMetricsIdx)
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
