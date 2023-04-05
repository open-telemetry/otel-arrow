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
	"fmt"
	"sort"
	"strings"

	"github.com/HdrHistogram/hdrhistogram-go"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
)

type MetricsOptimizer struct {
	sort  bool
	stats *MetricsStats
}

type MetricsOptimized struct {
	ResourceMetricsIdx map[string]int // resource metrics id -> resource metrics group
	ResourceMetrics    []*ResourceMetricsGroup
}

type ResourceMetricsGroup struct {
	Resource          *pcommon.Resource
	ResourceSchemaUrl string
	ScopeMetricsIdx   map[string]int // scope metrics id -> scope metrics group
	ScopeMetrics      []*ScopeMetricsGroup
}

type ScopeMetricsGroup struct {
	Scope          *pcommon.InstrumentationScope
	ScopeSchemaUrl string

	Metrics []*pmetric.Metric
}

type MetricsStats struct {
	MetricsCount               int
	ResourceMetricsHistogram   *hdrhistogram.Histogram
	ResourceAttrsHistogram     *carrow.AttributesStats
	ScopeMetricsHistogram      *hdrhistogram.Histogram
	ScopeAttrsHistogram        *carrow.AttributesStats
	MetricsHistogram           *hdrhistogram.Histogram
	SumHistogram               *hdrhistogram.Histogram
	SumAttrsHistogram          *carrow.AttributesStats
	GaugeHistogram             *hdrhistogram.Histogram
	GaugeAttrsHistogram        *carrow.AttributesStats
	HistogramHistogram         *hdrhistogram.Histogram
	HistogramAttrsHistogram    *carrow.AttributesStats
	SummaryHistogram           *hdrhistogram.Histogram
	SummaryAttrsHistogram      *carrow.AttributesStats
	ExpHistogramHistogram      *hdrhistogram.Histogram
	ExpHistogramAttrsHistogram *carrow.AttributesStats
}

func NewMetricsOptimizer(cfg ...func(*carrow.Options)) *MetricsOptimizer {
	options := carrow.Options{
		Sort:  false,
		Stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	var s *MetricsStats
	if options.Stats {
		s = &MetricsStats{
			MetricsCount:               0,
			ResourceMetricsHistogram:   hdrhistogram.New(1, 1000000, 1),
			ResourceAttrsHistogram:     carrow.NewAttributesStats(),
			ScopeMetricsHistogram:      hdrhistogram.New(1, 1000000, 1),
			ScopeAttrsHistogram:        carrow.NewAttributesStats(),
			MetricsHistogram:           hdrhistogram.New(1, 1000000, 1),
			SumHistogram:               hdrhistogram.New(1, 1000000, 1),
			SumAttrsHistogram:          carrow.NewAttributesStats(),
			GaugeHistogram:             hdrhistogram.New(1, 1000000, 1),
			GaugeAttrsHistogram:        carrow.NewAttributesStats(),
			HistogramHistogram:         hdrhistogram.New(1, 1000000, 1),
			HistogramAttrsHistogram:    carrow.NewAttributesStats(),
			SummaryHistogram:           hdrhistogram.New(1, 1000000, 1),
			SummaryAttrsHistogram:      carrow.NewAttributesStats(),
			ExpHistogramHistogram:      hdrhistogram.New(1, 1000000, 1),
			ExpHistogramAttrsHistogram: carrow.NewAttributesStats(),
		}
	}

	return &MetricsOptimizer{
		sort:  options.Sort,
		stats: s,
	}
}

func (t *MetricsOptimizer) Stats() *MetricsStats {
	return t.stats
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

	if t.stats != nil {
		metricsOptimized.RecordStats(t.stats)
	}

	return metricsOptimized
}

func (t *MetricsOptimized) RecordStats(stats *MetricsStats) {
	stats.MetricsCount++
	if err := stats.ResourceMetricsHistogram.RecordValue(int64(len(t.ResourceMetrics))); err != nil {
		panic(fmt.Sprintf("number of resource metrics is out of range: %v", err))
	}
	for _, resMetricsGroup := range t.ResourceMetrics {
		attrs := resMetricsGroup.Resource.Attributes()
		stats.ResourceAttrsHistogram.UpdateStats(attrs)
		resMetricsGroup.RecordStats(stats)
	}
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

func (t *ResourceMetricsGroup) RecordStats(stats *MetricsStats) {
	if err := stats.ScopeMetricsHistogram.RecordValue(int64(len(t.ScopeMetrics))); err != nil {
		panic(fmt.Sprintf("number of scope metrics is out of range: %v", err))
	}
	for _, scopeMetricsGroup := range t.ScopeMetrics {
		attrs := scopeMetricsGroup.Scope.Attributes()
		stats.ScopeAttrsHistogram.UpdateStats(attrs)
		scopeMetricsGroup.RecordStats(stats)
	}
}

func (t *ScopeMetricsGroup) RecordStats(stats *MetricsStats) {
	if err := stats.MetricsHistogram.RecordValue(int64(len(t.Metrics))); err != nil {
		panic(fmt.Sprintf("number of metrics is out of range: %v", err))
	}

	sumCount := 0
	gaugeCount := 0
	summaryCount := 0
	histogramCount := 0
	expHistogramCount := 0

	for _, metric := range t.Metrics {
		switch metric.Type() {
		case pmetric.MetricTypeSum:
			dps := metric.Sum().DataPoints()
			for i := 0; i < dps.Len(); i++ {
				dp := dps.At(i)
				stats.SumAttrsHistogram.UpdateStats(dp.Attributes())
			}
			sumCount++
		case pmetric.MetricTypeGauge:
			dps := metric.Gauge().DataPoints()
			for i := 0; i < dps.Len(); i++ {
				dp := dps.At(i)
				stats.GaugeAttrsHistogram.UpdateStats(dp.Attributes())
			}
			gaugeCount++
		case pmetric.MetricTypeSummary:
			dps := metric.Summary().DataPoints()
			for i := 0; i < dps.Len(); i++ {
				dp := dps.At(i)
				stats.SummaryAttrsHistogram.UpdateStats(dp.Attributes())
			}
			summaryCount++
		case pmetric.MetricTypeHistogram:
			dps := metric.Histogram().DataPoints()
			for i := 0; i < dps.Len(); i++ {
				dp := dps.At(i)
				stats.HistogramAttrsHistogram.UpdateStats(dp.Attributes())
			}
			histogramCount++
		case pmetric.MetricTypeExponentialHistogram:
			dps := metric.ExponentialHistogram().DataPoints()
			for i := 0; i < dps.Len(); i++ {
				dp := dps.At(i)
				stats.ExpHistogramAttrsHistogram.UpdateStats(dp.Attributes())
			}
			expHistogramCount++
		default: /* ignore */
		}
	}

	if err := stats.SumHistogram.RecordValue(int64(sumCount)); err != nil {
		panic(fmt.Sprintf("number of sum metrics is out of range: %v", err))
	}
	if err := stats.GaugeHistogram.RecordValue(int64(gaugeCount)); err != nil {
		panic(fmt.Sprintf("number of gauge metrics is out of range: %v", err))
	}
	if err := stats.SummaryHistogram.RecordValue(int64(summaryCount)); err != nil {
		panic(fmt.Sprintf("number of summary metrics is out of range: %v", err))
	}
	if err := stats.HistogramHistogram.RecordValue(int64(histogramCount)); err != nil {
		panic(fmt.Sprintf("number of histogram metrics is out of range: %v", err))
	}
	if err := stats.ExpHistogramHistogram.RecordValue(int64(expHistogramCount)); err != nil {
		panic(fmt.Sprintf("number of exponential histogram metrics is out of range: %v", err))
	}
}

func (t *MetricsStats) Show() {
	println("\n== Metrics structure distribution ============================================================")
	fmt.Printf("Metrics (total): %d\n", t.MetricsCount)
	fmt.Printf("  ResourceMetrics  -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ResourceMetricsHistogram.Mean(),
		t.ResourceMetricsHistogram.Min(),
		t.ResourceMetricsHistogram.Max(),
		t.ResourceMetricsHistogram.StdDev(),
		t.ResourceMetricsHistogram.ValueAtQuantile(50),
		t.ResourceMetricsHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("    Resource\n")
	t.ResourceAttrsHistogram.Show("      ")
	fmt.Printf("    ScopeMetrics   -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ScopeMetricsHistogram.Mean(),
		t.ScopeMetricsHistogram.Min(),
		t.ScopeMetricsHistogram.Max(),
		t.ScopeMetricsHistogram.StdDev(),
		t.ScopeMetricsHistogram.ValueAtQuantile(50),
		t.ScopeMetricsHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("      Scope\n")
	t.ScopeAttrsHistogram.Show("        ")
	fmt.Printf("      Metrics      -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.MetricsHistogram.Mean(),
		t.MetricsHistogram.Min(),
		t.MetricsHistogram.Max(),
		t.MetricsHistogram.StdDev(),
		t.MetricsHistogram.ValueAtQuantile(50),
		t.MetricsHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("        Sum           -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.SumHistogram.Mean(),
		t.SumHistogram.Min(),
		t.SumHistogram.Max(),
		t.SumHistogram.StdDev(),
		t.SumHistogram.ValueAtQuantile(50),
		t.SumHistogram.ValueAtQuantile(99),
	)
	t.SumAttrsHistogram.Show("           ")
	fmt.Printf("        Gauge         -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.GaugeHistogram.Mean(),
		t.GaugeHistogram.Min(),
		t.GaugeHistogram.Max(),
		t.GaugeHistogram.StdDev(),
		t.GaugeHistogram.ValueAtQuantile(50),
		t.GaugeHistogram.ValueAtQuantile(99),
	)
	t.GaugeAttrsHistogram.Show("           ")
	fmt.Printf("        Summary       -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.SummaryHistogram.Mean(),
		t.SummaryHistogram.Min(),
		t.SummaryHistogram.Max(),
		t.SummaryHistogram.StdDev(),
		t.SummaryHistogram.ValueAtQuantile(50),
		t.SummaryHistogram.ValueAtQuantile(99),
	)
	t.SummaryAttrsHistogram.Show("           ")
	fmt.Printf("        Histogram     -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.HistogramHistogram.Mean(),
		t.HistogramHistogram.Min(),
		t.HistogramHistogram.Max(),
		t.HistogramHistogram.StdDev(),
		t.HistogramHistogram.ValueAtQuantile(50),
		t.HistogramHistogram.ValueAtQuantile(99),
	)
	t.HistogramAttrsHistogram.Show("           ")
	fmt.Printf("        Exp Histogram -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ExpHistogramHistogram.Mean(),
		t.ExpHistogramHistogram.Min(),
		t.ExpHistogramHistogram.Max(),
		t.ExpHistogramHistogram.StdDev(),
		t.ExpHistogramHistogram.ValueAtQuantile(50),
		t.ExpHistogramHistogram.ValueAtQuantile(99),
	)
	t.ExpHistogramAttrsHistogram.Show("           ")
}
