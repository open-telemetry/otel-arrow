package profileable

import (
	"google.golang.org/protobuf/proto"
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type MetricsOtlpProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.MetricsDataset
	metrics     []*v1.ExportMetricsServiceRequest
}

func NewMetricsOtlpProfileable(dataset benchmark.MetricsDataset, compression benchmark.CompressionAlgorithm) *MetricsOtlpProfileable {
	return &MetricsOtlpProfileable{dataset: dataset, compression: compression}
}

func (s *MetricsOtlpProfileable) Name() string {
	return "MetricsOtlpProfileable"
}

func (s *MetricsOtlpProfileable) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *MetricsOtlpProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *MetricsOtlpProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *MetricsOtlpProfileable) InitBatchSize(_ int)   {}
func (s *MetricsOtlpProfileable) PrepareBatch(_, _ int) {}
func (s *MetricsOtlpProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsOtlpProfileable) Process() string { return "" }
func (s *MetricsOtlpProfileable) Serialize() ([][]byte, error) {
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
func (s *MetricsOtlpProfileable) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportMetricsServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportMetricsServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *MetricsOtlpProfileable) Clear() {
	s.metrics = nil
}
func (s *MetricsOtlpProfileable) ShowStats() {}
