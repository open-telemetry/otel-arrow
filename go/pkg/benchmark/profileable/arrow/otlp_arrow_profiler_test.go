/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow

import (
	"path/filepath"
	"testing"

	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark/dataset"
)

func TestOtlpArrowMetricsProfiler(t *testing.T) {
	t.Skip("Skipping this test because it's not fully implemented yet")
	t.Parallel()

	// Configuration
	cfg := &benchmark.Config{}

	maxIter := uint64(10)
	systemToProfile := NewMetricsProfileable([]string{"multivariate"}, dataset.NewFakeMetricsDataset(1000), cfg)
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, filepath.Join(t.TempDir(), "tmpfile"), 2)
	if err := profiler.Profile(systemToProfile, maxIter); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults(maxIter)
	profiler.ExportMetricsTimesCSV("otlp_arrow_metrics")
	profiler.ExportMetricsBytesCSV("otlp_arrow_metrics")
}

func TestOtlpArrowLogsProfiler(t *testing.T) {
	t.Parallel()
}

func TestOtlpArrowTracesProfiler(t *testing.T) {
	t.Parallel()
}
