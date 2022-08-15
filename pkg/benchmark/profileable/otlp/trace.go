package otlp

import (
	"io"

	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/dataset"
)

type TraceProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     dataset.TraceDataset
	traces      []*v1.ExportTraceServiceRequest
}

func NewTraceProfileable(dataset dataset.TraceDataset, compression benchmark.CompressionAlgorithm) *TraceProfileable {
	return &TraceProfileable{dataset: dataset, compression: compression}
}

func (s *TraceProfileable) Name() string {
	return "OTLP"
}

func (s *TraceProfileable) Tags() []string {
	return []string{s.compression.String()}
}
func (s *TraceProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *TraceProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *TraceProfileable) StartProfiling(io.Writer)           {}
func (s *TraceProfileable) EndProfiling(io.Writer)             {}
func (s *TraceProfileable) InitBatchSize(_ io.Writer, _ int)   {}
func (s *TraceProfileable) PrepareBatch(_ io.Writer, _, _ int) {}
func (s *TraceProfileable) CreateBatch(_ io.Writer, startAt, size int) {
	s.traces = s.dataset.Traces(startAt, size)
}
func (s *TraceProfileable) Process(io.Writer) string { return "" }
func (s *TraceProfileable) Serialize(io.Writer) ([][]byte, error) {
	buffers := make([][]byte, len(s.traces))
	for i, m := range s.traces {
		bytes, err := proto.Marshal(m)
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}
func (s *TraceProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.traces = make([]*v1.ExportTraceServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportTraceServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.traces[i] = m
	}
}
func (s *TraceProfileable) Clear() {
	s.traces = nil
}
func (s *TraceProfileable) ShowStats() {}
