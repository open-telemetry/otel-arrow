package otlp

import (
	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type TraceProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.TraceDataset
	metrics     []*v1.ExportTraceServiceRequest
}

func NewTraceProfileable(dataset benchmark.TraceDataset, compression benchmark.CompressionAlgorithm) *TraceProfileable {
	return &TraceProfileable{dataset: dataset, compression: compression}
}

func (s *TraceProfileable) Name() string {
	return "OTLP"
}

func (s *TraceProfileable) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *TraceProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *TraceProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *TraceProfileable) InitBatchSize(_ int)   {}
func (s *TraceProfileable) PrepareBatch(_, _ int) {}
func (s *TraceProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Traces(startAt, size)
}
func (s *TraceProfileable) Process() string { return "" }
func (s *TraceProfileable) Serialize() ([][]byte, error) {
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
func (s *TraceProfileable) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportTraceServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportTraceServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *TraceProfileable) Clear() {
	s.metrics = nil
}
func (s *TraceProfileable) ShowStats() {}
