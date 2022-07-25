package otlp

import (
	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type LogsProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.LogsDataset
	metrics     []*v1.ExportLogsServiceRequest
}

func NewLogsProfileable(dataset benchmark.LogsDataset, compression benchmark.CompressionAlgorithm) *LogsProfileable {
	return &LogsProfileable{dataset: dataset, compression: compression}
}

func (s *LogsProfileable) Name() string {
	return "OTLP"
}

func (s *LogsProfileable) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *LogsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *LogsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *LogsProfileable) InitBatchSize(_ int)   {}
func (s *LogsProfileable) PrepareBatch(_, _ int) {}
func (s *LogsProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Logs(startAt, size)
}
func (s *LogsProfileable) Process() string { return "" }
func (s *LogsProfileable) Serialize() ([][]byte, error) {
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
func (s *LogsProfileable) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportLogsServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportLogsServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *LogsProfileable) Clear() {
	s.metrics = nil
}
func (s *LogsProfileable) ShowStats() {}
