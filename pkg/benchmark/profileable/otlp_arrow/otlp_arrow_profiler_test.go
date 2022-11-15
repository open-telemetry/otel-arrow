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

package otlp_arrow

import (
	"path/filepath"
	"testing"

	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/otel/metrics"
)

func TestOtlpArrowMetricsProfiler(t *testing.T) {
	// t.Skip("Skipping this test because it's not fully implemented yet")
	t.Parallel()

	// Configuration
	cfg := config.NewUint8DefaultConfig()
	multivariateConf := &metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	maxIter := uint64(10)
	systemToProfile := NewMetricsProfileable([]string{"multivariate"}, dataset.NewFakeMetricsDataset(1000), cfg, multivariateConf, benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, filepath.Join(t.TempDir(), "tmpfile"))
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
