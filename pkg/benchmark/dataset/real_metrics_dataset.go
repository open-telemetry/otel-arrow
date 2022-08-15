/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package dataset

import (
	"log"
	"os"

	"google.golang.org/protobuf/proto"

	colmetrics "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	otlpmetrics "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/metrics/v1"
)

// RealMetricsDataset represents a dataset of real metrics read from an ExportMetricsServiceRequest serialized to a binary file.
type RealMetricsDataset struct {
	metrics []metrics
}

type metrics struct {
	metric   *otlpmetrics.Metric
	resource *otlpmetrics.ResourceMetrics
	scope    *otlpmetrics.ScopeMetrics
}

// NewRealMetricsDataset creates a new RealMetricsDataset from a binary file.
func NewRealMetricsDataset(path string) *RealMetricsDataset {
	data, err := os.ReadFile(path)
	if err != nil {
		log.Fatal("read file:", err)
	}
	var otlp colmetrics.ExportMetricsServiceRequest
	if err := proto.Unmarshal(data, &otlp); err != nil {
		log.Fatal("unmarshal:", err)
	}

	ds := &RealMetricsDataset{metrics: []metrics{}}

	for _, rm := range otlp.ResourceMetrics {
		for _, sm := range rm.ScopeMetrics {
			for _, m := range sm.Metrics {
				ds.metrics = append(ds.metrics, metrics{metric: m, resource: rm, scope: sm})
			}
		}
	}

	return ds
}

// Len returns the number of metrics in the dataset.
func (d *RealMetricsDataset) Len() int {
	return len(d.metrics)
}

// Metrics returns a subset of metrics from the original dataset.
func (d *RealMetricsDataset) Metrics(offset, size int) []*colmetrics.ExportMetricsServiceRequest {
	resMetrics := map[*otlpmetrics.ResourceMetrics]map[*otlpmetrics.ScopeMetrics][]*otlpmetrics.Metric{}

	for _, metric := range d.metrics[offset : offset+size] {
		if rl, ok := resMetrics[metric.resource]; !ok {
			resMetrics[metric.resource] = map[*otlpmetrics.ScopeMetrics][]*otlpmetrics.Metric{}
		} else if _, ok := rl[metric.scope]; !ok {
			rl[metric.scope] = []*otlpmetrics.Metric{}
		}

		metrics := resMetrics[metric.resource][metric.scope]
		metrics = append(metrics, metric.metric)
	}

	request := colmetrics.ExportMetricsServiceRequest{
		ResourceMetrics: make([]*otlpmetrics.ResourceMetrics, 0, len(resMetrics)),
	}

	for rm, sm := range resMetrics {
		scopeMetrics := make([]*otlpmetrics.ScopeMetrics, 0, len(sm))
		for sl, mrs := range sm {
			scopeMetrics = append(scopeMetrics, &otlpmetrics.ScopeMetrics{
				Scope:     sl.Scope,
				Metrics:   mrs,
				SchemaUrl: sl.SchemaUrl,
			})
		}

		request.ResourceMetrics = append(request.ResourceMetrics, &otlpmetrics.ResourceMetrics{
			Resource:     rm.Resource,
			ScopeMetrics: scopeMetrics,
			SchemaUrl:    rm.SchemaUrl,
		})
	}

	return []*colmetrics.ExportMetricsServiceRequest{&request}
}
