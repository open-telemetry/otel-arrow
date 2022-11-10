package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ScopeMetricsIds struct {
	Id           int
	SchemaUrl    int
	ScopeIds     *otlp.ScopeIds
	MetricSetIds *MetricSetIds
}

func NewScopeMetricsIds(dt *arrow.StructType) (*ScopeMetricsIds, error) {
	id, scopeMetricsDT, err := arrow_utils.ListOfStructsFieldIdFromStruct(dt, constants.SCOPE_METRICS)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrow_utils.FieldIdFromStruct(scopeMetricsDT, constants.SCHEMA_URL)
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

	return &ScopeMetricsIds{
		Id:           id,
		SchemaUrl:    schemaId,
		ScopeIds:     scopeIds,
		MetricSetIds: metricSetIds,
	}, nil
}

func AppendScopeMetricsInto(resMetrics pmetric.ResourceMetrics, arrowResMetrics *arrow_utils.ListOfStructs, resMetricsIdx int, ids *ScopeMetricsIds) error {
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

		schemaUrl, err := arrowScopeMetrics.StringFieldById(ids.SchemaUrl, scopeMetricsIdx)
		if err != nil {
			return err
		}
		scopeMetrics.SetSchemaUrl(schemaUrl)

		arrowMetrics, err := arrowScopeMetrics.ListOfStructsById(scopeMetricsIdx, ids.MetricSetIds.Id)
		if err != nil {
			return err
		}
		metricsSlice := scopeMetrics.Metrics()
		metricsSlice.EnsureCapacity(arrowMetrics.End() - arrowMetrics.Start())
		for entityIdx := arrowMetrics.Start(); entityIdx < arrowMetrics.End(); entityIdx++ {
			err = AppendMetricSetInto(metricsSlice, arrowMetrics, entityIdx, ids.MetricSetIds)
			if err != nil {
				return err
			}
		}
	}

	return nil
}
