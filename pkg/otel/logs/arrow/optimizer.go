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
	"go.opentelemetry.io/collector/pdata/plog"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
)

type LogsOptimizer struct {
	sort  bool
	stats *LogsStats
}

type LogsOptimized struct {
	ResourceLogsIdx map[string]int // resource logs id -> resource logs group
	ResourceLogs    []*ResourceLogGroup
}

type ResourceLogGroup struct {
	Resource          *pcommon.Resource
	ResourceSchemaUrl string
	ScopeLogsIdx      map[string]int // scope logs id -> scope logs group
	ScopeLogs         []*ScopeLogGroup
}

type ScopeLogGroup struct {
	Scope          *pcommon.InstrumentationScope
	ScopeSchemaUrl string

	Logs []*plog.LogRecord
}

type LogsStats struct {
	LogsCount              int
	ResourceLogsHistogram  *hdrhistogram.Histogram
	ResourceAttrsHistogram *carrow.AttributesStats
	ScopeLogsHistogram     *hdrhistogram.Histogram
	ScopeAttrsHistogram    *carrow.AttributesStats
	LogsHistogram          *hdrhistogram.Histogram
	LogsAttrsHistogram     *carrow.AttributesStats
	IntBodyHistogram       *hdrhistogram.Histogram
	DoubleBodyHistogram    *hdrhistogram.Histogram
	StringBodyHistogram    *hdrhistogram.Histogram
	BoolBodyHistogram      *hdrhistogram.Histogram
	ListBodyHistogram      *hdrhistogram.Histogram
	MapBodyHistogram       *hdrhistogram.Histogram
	BytesBodyHistogram     *hdrhistogram.Histogram
}

func NewLogsOptimizer(cfg ...func(*carrow.Options)) *LogsOptimizer {
	options := carrow.Options{
		Sort:  false,
		Stats: false,
	}
	for _, c := range cfg {
		c(&options)
	}

	var s *LogsStats
	if options.Stats {
		s = &LogsStats{
			LogsCount:              0,
			ResourceLogsHistogram:  hdrhistogram.New(1, 1000000, 1),
			ResourceAttrsHistogram: carrow.NewAttributesStats(),
			ScopeLogsHistogram:     hdrhistogram.New(1, 1000000, 1),
			ScopeAttrsHistogram:    carrow.NewAttributesStats(),
			LogsHistogram:          hdrhistogram.New(1, 1000000, 1),
			LogsAttrsHistogram:     carrow.NewAttributesStats(),
			IntBodyHistogram:       hdrhistogram.New(1, 1000000, 1),
			DoubleBodyHistogram:    hdrhistogram.New(1, 1000000, 1),
			StringBodyHistogram:    hdrhistogram.New(1, 1000000, 1),
			BoolBodyHistogram:      hdrhistogram.New(1, 1000000, 1),
			ListBodyHistogram:      hdrhistogram.New(1, 1000000, 1),
			MapBodyHistogram:       hdrhistogram.New(1, 1000000, 1),
			BytesBodyHistogram:     hdrhistogram.New(1, 1000000, 1),
		}
	}

	return &LogsOptimizer{
		sort:  options.Sort,
		stats: s,
	}
}

func (t *LogsOptimizer) Optimize(logs plog.Logs) *LogsOptimized {
	logsOptimized := &LogsOptimized{
		ResourceLogsIdx: make(map[string]int),
		ResourceLogs:    make([]*ResourceLogGroup, 0),
	}

	resLogsSlice := logs.ResourceLogs()
	for i := 0; i < resLogsSlice.Len(); i++ {
		resLogs := resLogsSlice.At(i)
		logsOptimized.AddResourceLogs(&resLogs)
	}

	if t.sort {
		for _, resLogGroup := range logsOptimized.ResourceLogs {
			resLogGroup.Sort()
		}
	}

	if t.stats != nil {
		logsOptimized.RecordStats(t.stats)
	}

	return logsOptimized
}

func (t *LogsOptimizer) Stats() *LogsStats {
	return t.stats
}

func (t *LogsOptimized) RecordStats(stats *LogsStats) {
	stats.LogsCount++
	if err := stats.ResourceLogsHistogram.RecordValue(int64(len(t.ResourceLogs))); err != nil {
		panic(fmt.Sprintf("number of resource logs is out of range: %v", err))
	}
	for _, resLogsGroup := range t.ResourceLogs {
		attrs := resLogsGroup.Resource.Attributes()
		stats.ResourceAttrsHistogram.UpdateStats(attrs)
		resLogsGroup.RecordStats(stats)
	}
}

func (t *LogsOptimized) AddResourceLogs(resLogs *plog.ResourceLogs) {
	resLogsID := otlp.ResourceID(resLogs.Resource(), resLogs.SchemaUrl())
	resLogGroupIdx, found := t.ResourceLogsIdx[resLogsID]
	if !found {
		res := resLogs.Resource()
		resLogGroup := &ResourceLogGroup{
			Resource:          &res,
			ResourceSchemaUrl: resLogs.SchemaUrl(),
			ScopeLogsIdx:      make(map[string]int),
			ScopeLogs:         make([]*ScopeLogGroup, 0),
		}
		t.ResourceLogs = append(t.ResourceLogs, resLogGroup)
		resLogGroupIdx = len(t.ResourceLogs) - 1
		t.ResourceLogsIdx[resLogsID] = resLogGroupIdx
	}
	scopeLogsSlice := resLogs.ScopeLogs()
	for i := 0; i < scopeLogsSlice.Len(); i++ {
		scopeLogs := scopeLogsSlice.At(i)
		t.ResourceLogs[resLogGroupIdx].AddScopeLogs(&scopeLogs)
	}
}

func (r *ResourceLogGroup) AddScopeLogs(scopeLogs *plog.ScopeLogs) {
	scopeLogID := otlp.ScopeID(scopeLogs.Scope(), scopeLogs.SchemaUrl())
	scopeLogGroupIdx, found := r.ScopeLogsIdx[scopeLogID]
	if !found {
		scope := scopeLogs.Scope()
		scopeLogGroup := &ScopeLogGroup{
			Scope:          &scope,
			ScopeSchemaUrl: scopeLogs.SchemaUrl(),
			Logs:           make([]*plog.LogRecord, 0),
		}
		r.ScopeLogs = append(r.ScopeLogs, scopeLogGroup)
		scopeLogGroupIdx = len(r.ScopeLogs) - 1
		r.ScopeLogsIdx[scopeLogID] = scopeLogGroupIdx
	}
	logsSlice := scopeLogs.LogRecords()
	for i := 0; i < logsSlice.Len(); i++ {
		log := logsSlice.At(i)
		sl := r.ScopeLogs[scopeLogGroupIdx]
		sl.Logs = append(sl.Logs, &log)
	}
}

func (r *ResourceLogGroup) Sort() {
	for _, scopeLogGroup := range r.ScopeLogs {
		sort.Slice(scopeLogGroup.Logs, func(i, j int) bool {
			return strings.Compare(
				scopeLogGroup.Logs[i].TraceID().String(),
				scopeLogGroup.Logs[j].TraceID().String(),
			) == -1
		})
	}
}

func (t *ResourceLogGroup) RecordStats(stats *LogsStats) {
	if err := stats.ScopeLogsHistogram.RecordValue(int64(len(t.ScopeLogs))); err != nil {
		panic(fmt.Sprintf("number of scope logs is out of range: %v", err))
	}
	for _, scopeLogsGroup := range t.ScopeLogs {
		attrs := scopeLogsGroup.Scope.Attributes()
		stats.ScopeAttrsHistogram.UpdateStats(attrs)
		scopeLogsGroup.RecordStats(stats)
	}
}

func (t *ScopeLogGroup) RecordStats(stats *LogsStats) {
	if err := stats.LogsHistogram.RecordValue(int64(len(t.Logs))); err != nil {
		panic(fmt.Sprintf("number of logs is out of range: %v", err))
	}

	intCount := 0
	doubleCount := 0
	stringCount := 0
	boolCount := 0
	listCount := 0
	mapCount := 0
	bytesCount := 0

	for _, log := range t.Logs {
		stats.LogsAttrsHistogram.UpdateStats(log.Attributes())

		switch log.Body().Type() {
		case pcommon.ValueTypeInt:
			intCount++
		case pcommon.ValueTypeDouble:
			doubleCount++
		case pcommon.ValueTypeStr:
			stringCount++
		case pcommon.ValueTypeBool:
			boolCount++
		case pcommon.ValueTypeSlice:
			listCount++
		case pcommon.ValueTypeMap:
			mapCount++
		case pcommon.ValueTypeBytes:
			bytesCount++
		default: /* ignore */
		}
	}

	if err := stats.IntBodyHistogram.RecordValue(int64(intCount)); err != nil {
		panic(fmt.Sprintf("number of int body is out of range: %v", err))
	}
	if err := stats.DoubleBodyHistogram.RecordValue(int64(doubleCount)); err != nil {
		panic(fmt.Sprintf("number of double body is out of range: %v", err))
	}
	if err := stats.StringBodyHistogram.RecordValue(int64(stringCount)); err != nil {
		panic(fmt.Sprintf("number of string body is out of range: %v", err))
	}
	if err := stats.BoolBodyHistogram.RecordValue(int64(boolCount)); err != nil {
		panic(fmt.Sprintf("number of bool body is out of range: %v", err))
	}
	if err := stats.ListBodyHistogram.RecordValue(int64(listCount)); err != nil {
		panic(fmt.Sprintf("number of list body is out of range: %v", err))
	}
	if err := stats.MapBodyHistogram.RecordValue(int64(mapCount)); err != nil {
		panic(fmt.Sprintf("number of map body is out of range: %v", err))
	}
	if err := stats.BytesBodyHistogram.RecordValue(int64(bytesCount)); err != nil {
		panic(fmt.Sprintf("number of bytes body is out of range: %v", err))
	}
}

func (t *LogsStats) Show() {
	println("\n == Logs structure distribution ============================================================")
	fmt.Printf("Logs (total): %d\n", t.LogsCount)
	fmt.Printf("  ResourceLogs   -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ResourceLogsHistogram.Mean(),
		t.ResourceLogsHistogram.Min(),
		t.ResourceLogsHistogram.Max(),
		t.ResourceLogsHistogram.StdDev(),
		t.ResourceLogsHistogram.ValueAtQuantile(50),
		t.ResourceLogsHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("    Resource\n")
	t.ResourceAttrsHistogram.Show("      ")
	fmt.Printf("    ScopeLogs    -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ScopeLogsHistogram.Mean(),
		t.ScopeLogsHistogram.Min(),
		t.ScopeLogsHistogram.Max(),
		t.ScopeLogsHistogram.StdDev(),
		t.ScopeLogsHistogram.ValueAtQuantile(50),
		t.ScopeLogsHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("      Scope\n")
	t.ScopeAttrsHistogram.Show("        ")
	fmt.Printf("      LogRecords   -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.LogsHistogram.Mean(),
		t.LogsHistogram.Min(),
		t.LogsHistogram.Max(),
		t.LogsHistogram.StdDev(),
		t.LogsHistogram.ValueAtQuantile(50),
		t.LogsHistogram.ValueAtQuantile(99),
	)
	t.LogsAttrsHistogram.Show("        ")
	fmt.Printf("        Body\n")
	fmt.Printf("          i64    -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.IntBodyHistogram.Mean(),
		t.IntBodyHistogram.Min(),
		t.IntBodyHistogram.Max(),
		t.IntBodyHistogram.StdDev(),
		t.IntBodyHistogram.ValueAtQuantile(50),
		t.IntBodyHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("          f64    -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.DoubleBodyHistogram.Mean(),
		t.DoubleBodyHistogram.Min(),
		t.DoubleBodyHistogram.Max(),
		t.DoubleBodyHistogram.StdDev(),
		t.DoubleBodyHistogram.ValueAtQuantile(50),
		t.DoubleBodyHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("          str    -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.StringBodyHistogram.Mean(),
		t.StringBodyHistogram.Min(),
		t.StringBodyHistogram.Max(),
		t.StringBodyHistogram.StdDev(),
		t.StringBodyHistogram.ValueAtQuantile(50),
		t.StringBodyHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("          bool   -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.BoolBodyHistogram.Mean(),
		t.BoolBodyHistogram.Min(),
		t.BoolBodyHistogram.Max(),
		t.BoolBodyHistogram.StdDev(),
		t.BoolBodyHistogram.ValueAtQuantile(50),
		t.BoolBodyHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("          map    -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.MapBodyHistogram.Mean(),
		t.MapBodyHistogram.Min(),
		t.MapBodyHistogram.Max(),
		t.MapBodyHistogram.StdDev(),
		t.MapBodyHistogram.ValueAtQuantile(50),
		t.MapBodyHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("          list   -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.ListBodyHistogram.Mean(),
		t.ListBodyHistogram.Min(),
		t.ListBodyHistogram.Max(),
		t.ListBodyHistogram.StdDev(),
		t.ListBodyHistogram.ValueAtQuantile(50),
		t.ListBodyHistogram.ValueAtQuantile(99),
	)
	fmt.Printf("          binary -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
		t.BytesBodyHistogram.Mean(),
		t.BytesBodyHistogram.Min(),
		t.BytesBodyHistogram.Max(),
		t.BytesBodyHistogram.StdDev(),
		t.BytesBodyHistogram.ValueAtQuantile(50),
		t.BytesBodyHistogram.ValueAtQuantile(99),
	)
}
