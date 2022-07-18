package benchmark

import (
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/profileable"
	"testing"
)

func TestMetricsProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := profileable.NewMetricsOtlpProfileable(benchmark.NewFakeMetricsDataset(1000), benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults()
	profiler.ExportMetricsTimesCSV("metrics")
	profiler.ExportMetricsBytesCSV("metrics")
}

func TestLogsProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := profileable.NewLogsOtlpProfileable(benchmark.NewFakeLogsDataset(1000), benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults()
	profiler.ExportMetricsTimesCSV("logs")
	profiler.ExportMetricsBytesCSV("logs")
}

func TestTracesProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := profileable.NewTraceOtlpProfileable(benchmark.NewFakeTraceDataset(1000), benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults()
	profiler.ExportMetricsTimesCSV("traces")
	profiler.ExportMetricsBytesCSV("traces")
}
