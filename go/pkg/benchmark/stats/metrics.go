/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
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
