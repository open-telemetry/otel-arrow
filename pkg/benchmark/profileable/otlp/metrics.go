package otlp

import (
	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type MetricsProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.MetricsDataset
	metrics     []*v1.ExportMetricsServiceRequest
}

func NewMetricsProfileable(dataset benchmark.MetricsDataset, compression benchmark.CompressionAlgorithm) *MetricsProfileable {
	return &MetricsProfileable{dataset: dataset, compression: compression}
}

func (s *MetricsProfileable) Name() string {
	return "OTLP"
}

func (s *MetricsProfileable) Tags() []string {
	return []string{s.compression.String()}
}
func (s *MetricsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *MetricsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *MetricsProfileable) StartProfiling()       {}
func (s *MetricsProfileable) EndProfiling()         {}
func (s *MetricsProfileable) InitBatchSize(_ int)   {}
func (s *MetricsProfileable) PrepareBatch(_, _ int) {}
func (s *MetricsProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsProfileable) Process() string { return "" }
func (s *MetricsProfileable) Serialize() ([][]byte, error) {
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
func (s *MetricsProfileable) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportMetricsServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportMetricsServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *MetricsProfileable) Clear() {
	s.metrics = nil
}
func (s *MetricsProfileable) ShowStats() {}
