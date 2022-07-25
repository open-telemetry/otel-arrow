package benchmark

import (
	"testing"

	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/profileable/otlp"
)

func TestOtlpMetricsProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := otlp.NewMetricsProfileable(benchmark.NewFakeMetricsDataset(1000), benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	//profiler.PrintResults()
	//profiler.ExportMetricsTimesCSV("otlp_metrics")
	//profiler.ExportMetricsBytesCSV("otlp_metrics")
}

func TestOtlpLogsProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := otlp.NewLogsProfileable(benchmark.NewFakeLogsDataset(1000), benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	//profiler.PrintResults()
	//profiler.ExportMetricsTimesCSV("otlp_logs")
	//profiler.ExportMetricsBytesCSV("otlp_logs")
}

func TestOtlpTracesProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := otlp.NewTraceProfileable(benchmark.NewFakeTraceDataset(1000), benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	//profiler.PrintResults()
	//profiler.ExportMetricsTimesCSV("otlp_traces")
	//profiler.ExportMetricsBytesCSV("otlp_traces")
}
