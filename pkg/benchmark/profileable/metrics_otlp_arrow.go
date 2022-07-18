package profileable

import (
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type OtlpArrowProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.MetricsDataset
	metrics     []*v1.ExportMetricsServiceRequest
}

func NewOtlpArrowProfileable(dataset benchmark.MetricsDataset, compression benchmark.CompressionAlgorithm) *OtlpArrowProfileable {
	return &OtlpArrowProfileable{dataset: dataset, compression: compression}
}

func (s *OtlpArrowProfileable) Name() string {
	return "OtlpArrowProfileable"
}

func (s *OtlpArrowProfileable) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *OtlpArrowProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *OtlpArrowProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *OtlpArrowProfileable) InitBatchSize(_ int)   {}
func (s *OtlpArrowProfileable) PrepareBatch(_, _ int) {}
func (s *OtlpArrowProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *OtlpArrowProfileable) Process() string { return "" }
func (s *OtlpArrowProfileable) Serialize() ([][]byte, error) {
	return nil, nil
}
func (s *OtlpArrowProfileable) Deserialize(buffers [][]byte) {
}
func (s *OtlpArrowProfileable) Clear() {
	s.metrics = nil
}
func (s *OtlpArrowProfileable) ShowStats() {}
