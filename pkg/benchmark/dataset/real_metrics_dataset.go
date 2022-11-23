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
	"path/filepath"

	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
)

// RealMetricsDataset represents a dataset of real metrics read from an Metrics serialized to a binary file.
type RealMetricsDataset struct {
	metrics []metrics
}

type metrics struct {
	metric   pmetric.Metric
	resource pmetric.ResourceMetrics
	scope    pmetric.ScopeMetrics
}

// NewRealMetricsDataset creates a new RealMetricsDataset from a binary file.
func NewRealMetricsDataset(path string) *RealMetricsDataset {
	data, err := os.ReadFile(filepath.Clean(path))
	if err != nil {
		log.Fatal("read file:", err)
	}
	otlp := pmetricotlp.NewExportRequest()
	if err := otlp.UnmarshalProto(data); err != nil {
		log.Fatal("unmarshal:", err)
	}
	mdata := otlp.Metrics()

	ds := &RealMetricsDataset{metrics: []metrics{}}

	for ri := 0; ri < mdata.ResourceMetrics().Len(); ri++ {
		rm := mdata.ResourceMetrics().At(ri)
		for si := 0; si < rm.ScopeMetrics().Len(); si++ {
			sm := rm.ScopeMetrics().At(si)
			for mi := 0; mi < sm.Metrics().Len(); mi++ {
				m := sm.Metrics().At(mi)
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
func (d *RealMetricsDataset) Metrics(offset, size int) []pmetric.Metrics {
	resMetrics := map[pmetric.ResourceMetrics]map[pmetric.ScopeMetrics][]pmetric.Metric{}

	for _, metric := range d.metrics[offset : offset+size] {
		if _, ok := resMetrics[metric.resource]; !ok {
			resMetrics[metric.resource] = map[pmetric.ScopeMetrics][]pmetric.Metric{}
		}

		resMetrics[metric.resource][metric.scope] =
			append(resMetrics[metric.resource][metric.scope], metric.metric)
	}

	request := pmetric.NewMetrics()

	for rm, smm := range resMetrics {
		outRm := request.ResourceMetrics().AppendEmpty()
		rm.Resource().CopyTo(outRm.Resource())

		for sm, ms := range smm {
			outSm := outRm.ScopeMetrics().AppendEmpty()
			sm.Scope().CopyTo(outSm.Scope())

			for _, m := range ms {
				outM := outSm.Metrics().AppendEmpty()
				m.CopyTo(outM)
			}
		}
	}

	return []pmetric.Metrics{request}
}
