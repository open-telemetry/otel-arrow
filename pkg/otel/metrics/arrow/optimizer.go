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

package arrow

import (
	"sort"
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
)

type (
	MetricsOptimizer struct {
		sort bool
	}

	MetricsOptimized struct {
		ResourceMetricsIdx map[string]int // resource metrics id -> resource metrics group
		ResourceMetrics    []*ResourceMetricsGroup
	}

	ResourceMetricsGroup struct {
		Resource          *pcommon.Resource
		ResourceSchemaUrl string
		ScopeMetricsIdx   map[string]int // scope metrics id -> scope metrics group
		ScopeMetrics      []*ScopeMetricsGroup
	}

	ScopeMetricsGroup struct {
		Scope          *pcommon.InstrumentationScope
		ScopeSchemaUrl string

		Metrics []*pmetric.Metric
	}
)

func NewMetricsOptimizer(cfg ...func(*carrow.Options)) *MetricsOptimizer {
	options := carrow.Options{
		Sort:  false,
		Stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	return &MetricsOptimizer{
		sort: options.Sort,
	}
}

func (t *MetricsOptimizer) Optimize(metrics pmetric.Metrics) *MetricsOptimized {
	metricsOptimized := &MetricsOptimized{
		ResourceMetricsIdx: make(map[string]int),
		ResourceMetrics:    make([]*ResourceMetricsGroup, 0),
	}

	resMetricsSlice := metrics.ResourceMetrics()
	for i := 0; i < resMetricsSlice.Len(); i++ {
		resMetrics := resMetricsSlice.At(i)
		metricsOptimized.AddResourceMetrics(&resMetrics)
	}

	if t.sort {
		for _, resMetricsGroup := range metricsOptimized.ResourceMetrics {
			resMetricsGroup.Sort()
		}
	}

	return metricsOptimized
}

func (t *MetricsOptimized) AddResourceMetrics(resMetrics *pmetric.ResourceMetrics) {
	resMetricsID := otlp.ResourceID(resMetrics.Resource(), resMetrics.SchemaUrl())
	resMetricsGroupIdx, found := t.ResourceMetricsIdx[resMetricsID]
	if !found {
		res := resMetrics.Resource()
		resMetricsGroup := &ResourceMetricsGroup{
			Resource:          &res,
			ResourceSchemaUrl: resMetrics.SchemaUrl(),
			ScopeMetricsIdx:   make(map[string]int),
			ScopeMetrics:      make([]*ScopeMetricsGroup, 0),
		}
		t.ResourceMetrics = append(t.ResourceMetrics, resMetricsGroup)
		resMetricsGroupIdx = len(t.ResourceMetrics) - 1
		t.ResourceMetricsIdx[resMetricsID] = resMetricsGroupIdx
	}
	scopeMetricsSlice := resMetrics.ScopeMetrics()
	for i := 0; i < scopeMetricsSlice.Len(); i++ {
		scopeMetrics := scopeMetricsSlice.At(i)
		t.ResourceMetrics[resMetricsGroupIdx].AddScopeMetrics(&scopeMetrics)
	}
}

func (r *ResourceMetricsGroup) AddScopeMetrics(scopeMetrics *pmetric.ScopeMetrics) {
	scopeMetricsID := otlp.ScopeID(scopeMetrics.Scope(), scopeMetrics.SchemaUrl())
	scopeMetricsGroupIdx, found := r.ScopeMetricsIdx[scopeMetricsID]
	if !found {
		scope := scopeMetrics.Scope()
		scopeMetricsGroup := &ScopeMetricsGroup{
			Scope:          &scope,
			ScopeSchemaUrl: scopeMetrics.SchemaUrl(),
			Metrics:        make([]*pmetric.Metric, 0),
		}
		r.ScopeMetrics = append(r.ScopeMetrics, scopeMetricsGroup)
		scopeMetricsGroupIdx = len(r.ScopeMetrics) - 1
		r.ScopeMetricsIdx[scopeMetricsID] = scopeMetricsGroupIdx
	}
	metricsSlice := scopeMetrics.Metrics()
	for i := 0; i < metricsSlice.Len(); i++ {
		metric := metricsSlice.At(i)
		sm := r.ScopeMetrics[scopeMetricsGroupIdx]
		sm.Metrics = append(sm.Metrics, &metric)
	}
}

func (r *ResourceMetricsGroup) Sort() {
	for _, scopeMetricsGroup := range r.ScopeMetrics {
		sort.Slice(scopeMetricsGroup.Metrics, func(i, j int) bool {
			return strings.Compare(
				scopeMetricsGroup.Metrics[i].Name(),
				scopeMetricsGroup.Metrics[j].Name(),
			) == -1
		})
	}
}
