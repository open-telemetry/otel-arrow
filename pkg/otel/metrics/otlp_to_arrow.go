package metrics

import (
	"fmt"
	"github.com/apache/arrow/go/v9/arrow"
	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/field_value"
)

type MultivariateMetricsConfig struct {
	Metrics map[string]string
}

func OtlpMetricsToArrowEvents(rbr *rbb.RecordBatchRepository, request *collogspb.ExportMetricsServiceRequest, multivariateConf *MultivariateMetricsConfig) ([]arrow.Record, error) {
	for _, resourceMetrics := range request.ResourceMetrics {
		for _, scopeMetrics := range resourceMetrics.ScopeMetrics {
			for _, metric := range scopeMetrics.Metrics {
				if metric.Data != nil {
					switch metric.Data.(type) {
					case *metricspb.Metric_Gauge:
						metricGauge(rbr, resourceMetrics, scopeMetrics, metric, multivariateConf)
					case *metricspb.Metric_Sum:
						// ToDo Metric Sum
						return nil, nil
					case *metricspb.Metric_Histogram:
						// ToDo Metric Histogram
						return nil, nil
					case *metricspb.Metric_Summary:
						// ToDo Metric Summary
						return nil, nil
					case *metricspb.Metric_ExponentialHistogram:
						// ToDo Metric Exponential Histogram
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

func metricGauge(rbr *rbb.RecordBatchRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metric *metricspb.Metric, config *MultivariateMetricsConfig) {
	if mvKey, ok := config.Metrics[metric.Name]; ok {
		multivariateMetricGauge(rbr, resMetrics, scopeMetrics, metric.Name, metric.Data.(*metricspb.Metric_Gauge), mvKey)
	} else {
		univariateMetricGauge(rbr, resMetrics, scopeMetrics, metric.Name, metric.Data.(*metricspb.Metric_Gauge))
	}
}

func multivariateMetricGauge(rbr *rbb.RecordBatchRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, metric *metricspb.Metric_Gauge, multivariateKey string) {
	//record := rbb.NewRecord()

	//for _, ndp := range metric.Gauge.DataPoints {
	//	sig := DataPointSig(ndp, multivariateKey)
	//}
}

func univariateMetricGauge(rbr *rbb.RecordBatchRepository, resMetrics *metricspb.ResourceMetrics, scopeMetrics *metricspb.ScopeMetrics, metricName string, metric *metricspb.Metric_Gauge) {
	for _, ndp := range metric.Gauge.DataPoints {
		record := rbb.NewRecord()

		if resMetrics.Resource != nil {
			common.AddResource(record, resMetrics.Resource)
		}
		if scopeMetrics.Scope != nil {
			common.AddScope(record, constants.SCOPE_METRICS, scopeMetrics.Scope)
		}

		record.U64Field(constants.TIME_UNIX_NANO, ndp.TimeUnixNano)
		if ndp.StartTimeUnixNano > 0 {
			record.U64Field(constants.START_TIME_UNIX_NANO, ndp.StartTimeUnixNano)
		}

		attributes := common.NewAttributes(ndp.Attributes)
		if attributes != nil {
			record.AddField(*attributes)
		}

		if ndp.Value != nil {
			switch ndp.Value.(type) {
			case *metricspb.NumberDataPoint_AsDouble:
				record.StructField(constants.METRICS, field_value.Struct{
					Fields: []field_value.Field{
						field_value.MakeF64Field(metricName, ndp.Value.(*metricspb.NumberDataPoint_AsDouble).AsDouble),
					},
				})
			case *metricspb.NumberDataPoint_AsInt:
				record.StructField(constants.METRICS, field_value.Struct{
					Fields: []field_value.Field{
						field_value.MakeI64Field(metricName, ndp.Value.(*metricspb.NumberDataPoint_AsInt).AsInt),
					},
				})
			default:
				panic("Unsupported number data point value type")
			}
		}

		// ToDo Exemplar

		if ndp.Flags > 0 {
			record.U32Field(constants.FLAGS, ndp.Flags)
		}

		rbr.AddRecord(record)
	}
}
