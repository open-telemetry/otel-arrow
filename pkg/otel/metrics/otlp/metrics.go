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
