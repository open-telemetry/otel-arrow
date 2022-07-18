package profileable

import (
	"google.golang.org/protobuf/proto"
	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	"otel-arrow-adapter/pkg/benchmark"
)

type LogsOtlpProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.LogsDataset
	metrics     []*v1.ExportLogsServiceRequest
}

func NewLogsOtlpProfileable(dataset benchmark.LogsDataset, compression benchmark.CompressionAlgorithm) *LogsOtlpProfileable {
	return &LogsOtlpProfileable{dataset: dataset, compression: compression}
}

func (s *LogsOtlpProfileable) Name() string {
	return "LogsOtlpProfileable"
}

func (s *LogsOtlpProfileable) Tags() []string {
	return []string{"tag1", "tag2"}
}
func (s *LogsOtlpProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *LogsOtlpProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *LogsOtlpProfileable) InitBatchSize(_ int)   {}
func (s *LogsOtlpProfileable) PrepareBatch(_, _ int) {}
func (s *LogsOtlpProfileable) CreateBatch(startAt, size int) {
	s.metrics = s.dataset.Logs(startAt, size)
}
func (s *LogsOtlpProfileable) Process() string { return "" }
func (s *LogsOtlpProfileable) Serialize() ([][]byte, error) {
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
func (s *LogsOtlpProfileable) Deserialize(buffers [][]byte) {
	s.metrics = make([]*v1.ExportLogsServiceRequest, len(buffers))
	for i, b := range buffers {
		m := &v1.ExportLogsServiceRequest{}
		if err := proto.Unmarshal(b, m); err != nil {
			panic(err)
		}
		s.metrics[i] = m
	}
}
func (s *LogsOtlpProfileable) Clear() {
	s.metrics = nil
}
func (s *LogsOtlpProfileable) ShowStats() {}
