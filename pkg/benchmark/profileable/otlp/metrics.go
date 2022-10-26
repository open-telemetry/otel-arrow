package otlp

import (
	"io"

	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
)

type MetricsProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     dataset.MetricsDataset
	metrics     []pmetric.Metrics
}

func NewMetricsProfileable(dataset dataset.MetricsDataset, compression benchmark.CompressionAlgorithm) *MetricsProfileable {
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
func (s *MetricsProfileable) StartProfiling(io.Writer)           {}
func (s *MetricsProfileable) EndProfiling(io.Writer)             {}
func (s *MetricsProfileable) InitBatchSize(_ io.Writer, _ int)   {}
func (s *MetricsProfileable) PrepareBatch(_ io.Writer, _, _ int) {}
func (s *MetricsProfileable) CreateBatch(_ io.Writer, startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsProfileable) Process(io.Writer) string { return "" }
func (s *MetricsProfileable) Serialize(io.Writer) ([][]byte, error) {
	buffers := make([][]byte, len(s.metrics))
	for i, m := range s.metrics {
		r := pmetricotlp.NewRequestFromMetrics(m)
		bytes, err := r.MarshalProto()
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}
func (s *MetricsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.metrics = make([]pmetric.Metrics, len(buffers))
	for i, b := range buffers {
		r := pmetricotlp.NewRequest()
		if err := r.UnmarshalProto(b); err != nil {
			panic(err)
		}
		s.metrics[i] = r.Metrics()
	}
}
func (s *MetricsProfileable) Clear() {
	s.metrics = nil
}
func (s *MetricsProfileable) ShowStats() {}
