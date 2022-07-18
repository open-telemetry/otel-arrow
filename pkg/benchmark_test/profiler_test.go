package benchmark

import (
	"google.golang.org/protobuf/proto"
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/benchmark"
	"testing"
)

func TestProfiler(t *testing.T) {
	t.Parallel()

	systemToProfile := &SystemToProfile{
		dataset: benchmark.NewFakeMetricsDataset(1000),
	}
	profiler := benchmark.NewProfiler([]int{10, 100, 1000})
	if err := profiler.Profile(systemToProfile, 10); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	profiler.CheckProcessingResults()
	profiler.PrintResults()
	profiler.ExportMetricsTimesCSV("test")
	profiler.ExportMetricsBytesCSV("test")
}

type SystemToProfile struct {
	dataset benchmark.MetricsDataset
	metrics []*v1.ExportMetricsServiceRequest
}

func (s *SystemToProfile) Name() string {
	return "SystemToProfile"
}

func (s *SystemToProfile) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *SystemToProfile) DatasetSize() int { return s.dataset.Len() }
func (s *SystemToProfile) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return benchmark.Zstd
}
func (s *SystemToProfile) InitBatchSize(_ int)   {}
func (s *SystemToProfile) PrepareBatch(_, _ int) {}
func (s *SystemToProfile) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *SystemToProfile) Process() string { return "" }
func (s *SystemToProfile) Serialize() ([][]byte, error) {
	buffers := make([][]byte, len(s.metrics))
	for i, m := range s.metrics {
		bytes, err := proto.Marshal(m)
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}
func (s *SystemToProfile) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportMetricsServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportMetricsServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *SystemToProfile) Clear() {
	s.metrics = nil
}
func (s *SystemToProfile) ShowStats() {}
