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

package datagen

import (
	"time"

	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
	resourcepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/resource/v1"
)

type MetricsGenerator struct {
	resourceAttributes    [][]*commonpb.KeyValue
	defaultSchemaUrl      string
	instrumentationScopes []*commonpb.InstrumentationScope
	dataGenerator         *DataGenerator
	generation            int
}

func NewMetricsGenerator(resourceAttributes [][]*commonpb.KeyValue, instrumentationScopes []*commonpb.InstrumentationScope) *MetricsGenerator {
	return &MetricsGenerator{
		resourceAttributes:    resourceAttributes,
		defaultSchemaUrl:      "",
		instrumentationScopes: instrumentationScopes,
		dataGenerator:         NewDataGenerator(uint64(time.Now().UnixNano() / int64(time.Millisecond))),
		generation:            0,
	}
}

func (lg *MetricsGenerator) Generate(batchSize int, collectInterval time.Duration) *collogspb.ExportMetricsServiceRequest {
	var resourceMetrics []*metricspb.ResourceMetrics

	var metrics []*metricspb.Metric

	for i := 0; i < batchSize; i++ {
		lg.dataGenerator.AdvanceTime(collectInterval)

		metrics = append(metrics, SystemCpuTime(lg.dataGenerator, 1))
		metrics = append(metrics, SystemMemoryUsage(lg.dataGenerator))
		metrics = append(metrics, SystemCpuLoadAverage1m(lg.dataGenerator))
	}
	resourceMetrics = append(resourceMetrics, &metricspb.ResourceMetrics{
		Resource: &resourcepb.Resource{
			Attributes:             lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)],
			DroppedAttributesCount: 0,
		},
		SchemaUrl: lg.defaultSchemaUrl,
		ScopeMetrics: []*metricspb.ScopeMetrics{
			{
				Scope:     lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)],
				Metrics:   metrics,
				SchemaUrl: "",
			},
		},
	})

	lg.generation++

	return &collogspb.ExportMetricsServiceRequest{
		ResourceMetrics: resourceMetrics,
	}
}

func (lg *MetricsGenerator) GenerateSystemCpuTime(batchSize int, collectInterval time.Duration) *collogspb.ExportMetricsServiceRequest {
	var resourceMetrics []*metricspb.ResourceMetrics

	var metrics []*metricspb.Metric

	for i := 0; i < batchSize; i++ {
		lg.dataGenerator.AdvanceTime(collectInterval)

		metrics = append(metrics, SystemCpuTime(lg.dataGenerator, 1))
	}
	resourceMetrics = append(resourceMetrics, &metricspb.ResourceMetrics{
		Resource: &resourcepb.Resource{
			Attributes:             lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)],
			DroppedAttributesCount: 0,
		},
		SchemaUrl: lg.defaultSchemaUrl,
		ScopeMetrics: []*metricspb.ScopeMetrics{
			{
				Scope:     lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)],
				Metrics:   metrics,
				SchemaUrl: "",
			},
		},
	})

	lg.generation++

	return &collogspb.ExportMetricsServiceRequest{
		ResourceMetrics: resourceMetrics,
	}
}

func (lg *MetricsGenerator) GenerateSystemMemoryUsage(batchSize int, collectInterval time.Duration) *collogspb.ExportMetricsServiceRequest {
	var resourceMetrics []*metricspb.ResourceMetrics

	var metrics []*metricspb.Metric

	for i := 0; i < batchSize; i++ {
		lg.dataGenerator.AdvanceTime(collectInterval)

		metrics = append(metrics, SystemMemoryUsage(lg.dataGenerator))
	}
	resourceMetrics = append(resourceMetrics, &metricspb.ResourceMetrics{
		Resource: &resourcepb.Resource{
			Attributes:             lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)],
			DroppedAttributesCount: 0,
		},
		SchemaUrl: lg.defaultSchemaUrl,
		ScopeMetrics: []*metricspb.ScopeMetrics{
			{
				Scope:     lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)],
				Metrics:   metrics,
				SchemaUrl: "",
			},
		},
	})

	lg.generation++

	return &collogspb.ExportMetricsServiceRequest{
		ResourceMetrics: resourceMetrics,
	}
}

func (lg *MetricsGenerator) GenerateSystemCpuLoadAverage1m(batchSize int, collectInterval time.Duration) *collogspb.ExportMetricsServiceRequest {
	var resourceMetrics []*metricspb.ResourceMetrics

	var metrics []*metricspb.Metric

	for i := 0; i < batchSize; i++ {
		lg.dataGenerator.AdvanceTime(collectInterval)

		metrics = append(metrics, SystemCpuLoadAverage1m(lg.dataGenerator))
	}
	resourceMetrics = append(resourceMetrics, &metricspb.ResourceMetrics{
		Resource: &resourcepb.Resource{
			Attributes:             lg.resourceAttributes[lg.generation%len(lg.resourceAttributes)],
			DroppedAttributesCount: 0,
		},
		SchemaUrl: lg.defaultSchemaUrl,
		ScopeMetrics: []*metricspb.ScopeMetrics{
			{
				Scope:     lg.instrumentationScopes[lg.generation%len(lg.instrumentationScopes)],
				Metrics:   metrics,
				SchemaUrl: "",
			},
		},
	})

	lg.generation++

	return &collogspb.ExportMetricsServiceRequest{
		ResourceMetrics: resourceMetrics,
	}
}

func SystemCpuTime(dg *DataGenerator, cpuCount int) *metricspb.Metric {
	cpuStates := CpuStates()
	var dataPoint []*metricspb.NumberDataPoint

	for cpu := 0; cpu < cpuCount; cpu++ {
		for _, state := range cpuStates {
			dataPoint = append(dataPoint, &metricspb.NumberDataPoint{
				Attributes: []*commonpb.KeyValue{
					{
						Key:   "state",
						Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: state}},
					},
					{
						Key:   "cpu",
						Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_IntValue{IntValue: int64(cpu)}},
					},
				},
				StartTimeUnixNano: dg.PrevTime(),
				TimeUnixNano:      dg.CurrentTime(),
				Flags:             0,
				Value: &metricspb.NumberDataPoint_AsDouble{
					AsDouble: dg.GenF64Range(0.0, 1.0),
				},
			})
		}
	}
	return &metricspb.Metric{
		Name:        "system.cpu.time",
		Description: "",
		Unit:        "s",
		Data: &metricspb.Metric_Sum{
			Sum: &metricspb.Sum{
				DataPoints:             dataPoint,
				AggregationTemporality: metricspb.AggregationTemporality_AGGREGATION_TEMPORALITY_CUMULATIVE,
				IsMonotonic:            false,
			},
		},
	}
}

func SystemMemoryUsage(dg *DataGenerator) *metricspb.Metric {
	return &metricspb.Metric{
		Name:        "system.memory.usage",
		Description: "Bytes of memory in use.",
		Unit:        "By",
		Data: &metricspb.Metric_Sum{
			Sum: &metricspb.Sum{
				DataPoints: []*metricspb.NumberDataPoint{
					{
						Attributes: []*commonpb.KeyValue{
							{
								Key:   "state",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "used"}},
							},
						},
						StartTimeUnixNano: dg.PrevTime(),
						TimeUnixNano:      dg.CurrentTime(),
						Flags:             0,
						Value: &metricspb.NumberDataPoint_AsInt{
							AsInt: dg.GenI64Range(10_000_000_000, 13_000_000_000),
						},
					},
					{
						Attributes: []*commonpb.KeyValue{
							{
								Key:   "state",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "free"}},
							},
						},
						StartTimeUnixNano: dg.PrevTime(),
						TimeUnixNano:      dg.CurrentTime(),
						Flags:             0,
						Value: &metricspb.NumberDataPoint_AsInt{
							AsInt: dg.GenI64Range(300_000_000, 500_000_000),
						},
					},
					{
						Attributes: []*commonpb.KeyValue{
							{
								Key:   "state",
								Value: &commonpb.AnyValue{Value: &commonpb.AnyValue_StringValue{StringValue: "inactive"}},
							},
						},
						StartTimeUnixNano: dg.PrevTime(),
						TimeUnixNano:      dg.CurrentTime(),
						Flags:             0,
						Value: &metricspb.NumberDataPoint_AsInt{
							AsInt: 4_000_000_000,
						},
					},
				},
				AggregationTemporality: metricspb.AggregationTemporality_AGGREGATION_TEMPORALITY_CUMULATIVE,
				IsMonotonic:            false,
			},
		},
	}
}

func SystemCpuLoadAverage1m(dg *DataGenerator) *metricspb.Metric {
	return &metricspb.Metric{
		Name:        "system.cpu.load_average.1m",
		Description: "Average CPU Load over 1 minute.",
		Unit:        "1",
		Data: &metricspb.Metric_Sum{
			Sum: &metricspb.Sum{
				DataPoints: []*metricspb.NumberDataPoint{
					{
						StartTimeUnixNano: dg.PrevTime(),
						TimeUnixNano:      dg.CurrentTime(),
						Flags:             0,
						Value: &metricspb.NumberDataPoint_AsDouble{
							AsDouble: dg.GenF64Range(1.0, 100.0),
						},
					},
				},
				AggregationTemporality: metricspb.AggregationTemporality_AGGREGATION_TEMPORALITY_CUMULATIVE,
				IsMonotonic:            false,
			},
		},
	}
}

func CpuStates() [5]string {
	return [...]string{"idle", "user", "system", "iowait", "interrupt"}
}
