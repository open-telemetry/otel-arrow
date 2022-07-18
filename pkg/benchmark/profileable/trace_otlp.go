package profileable

import (
	"google.golang.org/protobuf/proto"
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type TraceOtlpProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.TraceDataset
	metrics     []*v1.ExportTraceServiceRequest
}

func NewTraceOtlpProfileable(dataset benchmark.TraceDataset, compression benchmark.CompressionAlgorithm) *TraceOtlpProfileable {
	return &TraceOtlpProfileable{dataset: dataset, compression: compression}
}

func (s *TraceOtlpProfileable) Name() string {
	return "TraceOtlpProfileable"
}

func (s *TraceOtlpProfileable) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *TraceOtlpProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *TraceOtlpProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *TraceOtlpProfileable) InitBatchSize(_ int)   {}
func (s *TraceOtlpProfileable) PrepareBatch(_, _ int) {}
func (s *TraceOtlpProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Traces(startAt, size)
}
func (s *TraceOtlpProfileable) Process() string { return "" }
func (s *TraceOtlpProfileable) Serialize() ([][]byte, error) {
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
func (s *TraceOtlpProfileable) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportTraceServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportTraceServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *TraceOtlpProfileable) Clear() {
	s.metrics = nil
}
func (s *TraceOtlpProfileable) ShowStats() {}
