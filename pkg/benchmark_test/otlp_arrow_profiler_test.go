package benchmark

import (
	"testing"

	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/profileable/otlp_arrow"
	"otel-arrow-adapter/pkg/otel/metrics"
)

func TestOtlpArrowMetricsProfiler(t *testing.T) {
	t.Skip("Skipping this test because it's not fully implemented yet")
	t.Parallel()

	// Configuration
	cfg := config.NewDefaultConfig()
	multivariateConf := &metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	systemToProfile := otlp_arrow.NewMetricsProfileable(benchmark.NewFakeMetricsDataset(1000), cfg, multivariateConf, benchmark.Zstd)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults()
	profiler.ExportMetricsTimesCSV("otlp_arrow_metrics")
	profiler.ExportMetricsBytesCSV("otlp_arrow_metrics")
}

func TestOtlpArrowLogsProfiler(t *testing.T) {
	t.Parallel()
}

func TestOtlpArrowTracesProfiler(t *testing.T) {
	t.Parallel()
}
