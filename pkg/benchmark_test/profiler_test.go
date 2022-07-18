package benchmark

import (
	"otel-arrow-adapter/pkg/benchmark"
	"testing"
)

func TestProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := &SystemToProfile{}
	profiler := benchmark.NewProfiler([]uint64{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults()
	profiler.ExportMetricsTimesCSV("test")
	profiler.ExportMetricsBytesCSV("test")
}

type SystemToProfile struct{}

func (s *SystemToProfile) Name() string {
	return "SystemToProfile"
}

func (s *SystemToProfile) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *SystemToProfile) DatasetSize() uint64 { return 10 }
func (s *SystemToProfile) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return benchmark.Zstd
}
func (s *SystemToProfile) InitBatchSize(batchSize uint64)    {}
func (s *SystemToProfile) PrepareBatch(startAt, size uint64) {}
func (s *SystemToProfile) CreateBatch(startAt, size uint64)  {}
func (s *SystemToProfile) Process() string                   { return "" }
func (s *SystemToProfile) Serialize() ([][]byte, error) {
	return nil, nil
}
func (s *SystemToProfile) Deserialize(buffers [][]byte) {}
func (s *SystemToProfile) Clear()                       {}
func (s *SystemToProfile) ShowStats()                   {}
