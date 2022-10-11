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

package benchmark

import (
	"testing"

	"github.com/lquerel/otel-arrow-adapter/pkg/benchmark"
	"github.com/lquerel/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/lquerel/otel-arrow-adapter/pkg/benchmark/profileable/otlp"
)

func TestOtlpMetricsProfiler(t *testing.T) {
	t.Skip("Too long test")
	t.Parallel()

	systemToProfile := otlp.NewMetricsProfileable(dataset.NewFakeMetricsDataset(1000), benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, "tmpfile")
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	//profiler.PrintResults()
	//profiler.ExportMetricsTimesCSV("otlp_metrics")
	//profiler.ExportMetricsBytesCSV("otlp_metrics")
}

func TestOtlpLogsProfiler(t *testing.T) {
	t.Skip("Too long test")
	t.Parallel()

	systemToProfile := otlp.NewLogsProfileable(dataset.NewFakeLogsDataset(1000), benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, "tmpfile")
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	//profiler.PrintResults()
	//profiler.ExportMetricsTimesCSV("otlp_logs")
	//profiler.ExportMetricsBytesCSV("otlp_logs")
}

func TestOtlpTracesProfiler(t *testing.T) {
	t.Skip("Too long test")
	t.Parallel()

	maxIter := uint64(10)
	systemToProfile := otlp.NewTraceProfileable(dataset.NewFakeTraceDataset(1000), benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, "tmpfile")
	if err := profiler.Profile(systemToProfile, maxIter); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults(maxIter)
	//profiler.ExportMetricsTimesCSV("otlp_traces")
	//profiler.ExportMetricsBytesCSV("otlp_traces")
}

func TestOtlpLightstepTracesProfiler(t *testing.T) {
	t.Skip("Lightstep specific test")
	t.Parallel()

	benchdata := dataset.NewRealTraceDataset("/Users/josh.macdonald/src/lightstep/forward_spans.bin.otlp.bin", []string{
		"trace_id",
	})

	maxIter := uint64(10)
	systemToProfile := otlp.NewTraceProfileable(benchdata, benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, "tmpfile")
	if err := profiler.Profile(systemToProfile, maxIter); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults(maxIter)
	//profiler.ExportMetricsTimesCSV("otlp_traces")
	//profiler.ExportMetricsBytesCSV("otlp_traces")
}
