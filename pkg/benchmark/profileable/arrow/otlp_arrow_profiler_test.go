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

package arrow

import (
	"path/filepath"
	"testing"

	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"
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
