package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pmetric"
)

type MetricsIds struct {
	ResourceMetrics *ResourceMetricsIds
}

// MetricsFrom creates a [pmetric.Metrics] from the given Arrow Record.
func MetricsFrom(record arrow.Record) (pmetric.Metrics, error) {
	metrics := pmetric.NewMetrics()

	metricsIds, err := SchemaToIds(record.Schema())
	if err != nil {
		return metrics, err
	}

	resMetricsSlice := metrics.ResourceMetrics()
	resSpansCount := int(record.NumRows())
	resMetricsSlice.EnsureCapacity(resSpansCount)

	// TODO there is probably two nested lists that could be replaced by a single list (metrics, resource spans). This could simplify a future query layer.

	err = AppendResourceMetricsInto(metrics, record, metricsIds)
	return metrics, err
}

func SchemaToIds(schema *arrow.Schema) (*MetricsIds, error) {
	resMetricsIds, err := NewResourceMetricsIds(schema)
	if err != nil {
		return nil, err
	}
	return &MetricsIds{
		ResourceMetrics: resMetricsIds,
	}, nil
}
