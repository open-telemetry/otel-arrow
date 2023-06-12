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

package otlp

import (
	"path/filepath"
	"testing"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
)

const WarmUpIter = 2

func TestOtlpMetricsProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := NewMetricsProfileable(dataset.NewFakeMetricsDataset(1000), benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, filepath.Join(t.TempDir(), "tmpfile"), WarmUpIter)
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
}

func TestOtlpLogsProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := NewLogsProfileable(dataset.NewFakeLogsDataset(1000), benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, filepath.Join(t.TempDir(), "tmpfile"), WarmUpIter)
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
}

func TestOtlpTracesProfiler(t *testing.T) {
	t.Parallel()

	maxIter := uint64(10)
	systemToProfile := NewTraceProfileable(dataset.NewFakeTraceDataset(1000), benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, filepath.Join(t.TempDir(), "tmpfile"), WarmUpIter)
	if err := profiler.Profile(systemToProfile, maxIter); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
}

func TestOtlpLightstepTracesProfiler(t *testing.T) {
	t.Skip("Lightstep specific test")
	t.Parallel()

	benchdata := dataset.NewRealTraceDataset("../../../../tools/trace_benchmark/data/otlp_traces.pb", benchmark.CompressionTypeZstd, "proto", []string{
		"trace_id",
	})

	maxIter := uint64(10)
	systemToProfile := NewTraceProfileable(benchdata, benchmark.Zstd())
	profiler := benchmark.NewProfiler([]int{10, 100, 1000}, "tmpfile", WarmUpIter)
	if err := profiler.Profile(systemToProfile, maxIter); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
}
