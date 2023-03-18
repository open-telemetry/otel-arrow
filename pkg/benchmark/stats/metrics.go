/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package stats

import (
	"fmt"

	"go.opentelemetry.io/collector/pdata/pmetric"
)

type MetricsStats struct {
	resourceMetrics    *Metric
	resourceAttributes *Metric
	scopeMetrics       *Metric
	scopeAttributes    *Metric
	metrics            *Metric
}

func NewMetricsStats() *MetricsStats {
	return &MetricsStats{
		resourceMetrics:    NewMetric(),
		resourceAttributes: NewMetric(),
		scopeMetrics:       NewMetric(),
		scopeAttributes:    NewMetric(),
		metrics:            NewMetric(),
	}
}

func (ms *MetricsStats) Analyze(metrics pmetric.Metrics) {
	resMetricsSlice := metrics.ResourceMetrics()
	ms.resourceMetrics.Record(float64(resMetricsSlice.Len()))

	for i := 0; i < resMetricsSlice.Len(); i++ {
		resMetrics := resMetricsSlice.At(i)
		scopeMetricsSlice := resMetrics.ScopeMetrics()

		ms.scopeMetrics.Record(float64(scopeMetricsSlice.Len()))
		ms.resourceAttributes.Record(float64(resMetrics.Resource().Attributes().Len()))

		for j := 0; j < scopeMetricsSlice.Len(); j++ {
			scopeMetrics := scopeMetricsSlice.At(j)
			metricsSlice := scopeMetrics.Metrics()

			ms.metrics.Record(float64(metricsSlice.Len()))
			ms.scopeAttributes.Record(float64(scopeMetrics.Scope().Attributes().Len()))
		}
	}
}

func (ms *MetricsStats) ShowStats() {
	fmt.Printf("\t- ResourceMetrics           => %s\n", ms.resourceMetrics.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Resource       => %s\n", ms.resourceAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- ScopeLogs/ResourceMetrics => %s\n", ms.scopeMetrics.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Scope          => %s\n", ms.scopeAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- LogRecord/ScopeMetrics    => %s\n", ms.metrics.ComputeSummary().ToString())
}
