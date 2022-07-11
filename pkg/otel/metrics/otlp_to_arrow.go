package metrics

import (
	"fmt"
	"github.com/apache/arrow/go/v9/arrow"
	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	"otel-arrow-adapter/pkg/rbb"
)

func OtlpMetricsToArrowEvents(rbr *rbb.RecordBatchRepository, request *collogspb.ExportMetricsServiceRequest) ([]arrow.Record, error) {
	for _, resourceMetrics := range request.ResourceMetrics {
		for _, scopeMetrics := range resourceMetrics.ScopeMetrics {
			for _, metric := range scopeMetrics.Metrics {
				if metric.Data != nil {
					switch metric.Data.(type) {
					case *metricspb.Metric_Gauge:
						return nil, nil
					case *metricspb.Metric_Sum:
						return nil, nil
					case *metricspb.Metric_Histogram:
						return nil, nil
					case *metricspb.Metric_Summary:
						return nil, nil
					case *metricspb.Metric_ExponentialHistogram:
						return nil, nil
					default:
						errorString := fmt.Sprintf("Unsupported metric type: %v", metric.Data)
						panic(errorString)
					}
				}
			}
			records, err := rbr.Build()
			if err != nil {
				return nil, err
			}
			result := make([]arrow.Record, len(records))
			for _, record := range records {
				result = append(result, record)
			}
			return result, nil
		}
	}
	return nil, nil
}
