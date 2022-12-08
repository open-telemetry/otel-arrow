package otlp_arrow

import (
	"io"

	"github.com/apache/arrow/go/v10/arrow/memory"
	"google.golang.org/protobuf/proto"

	"go.opentelemetry.io/collector/pdata/plog"

	v1 "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
)

type LogsProfileable struct {
	tags              []string
	compression       benchmark.CompressionAlgorithm
	dataset           dataset.LogsDataset
	logs              []plog.Logs
	producer          *arrow_record.Producer
	batchArrowRecords []*v1.BatchArrowRecords
	config            *benchmark.Config
	pool              *memory.GoAllocator
}

func NewLogsProfileable(tags []string, dataset dataset.LogsDataset, config *benchmark.Config) *LogsProfileable {
	return &LogsProfileable{
		tags:              tags,
		dataset:           dataset,
		compression:       benchmark.Zstd(), // ToDo replace Zstd with NoCompression when this bug will be fixed: https://github.com/apache/arrow/issues/14883
		producer:          arrow_record.NewProducer(),
		batchArrowRecords: make([]*v1.BatchArrowRecords, 0, 10),
		config:            config,
		pool:              memory.NewGoAllocator(),
	}
}

func (s *LogsProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *LogsProfileable) Tags() []string {
	var tags []string
	compression := s.compression.String()
	if compression != "" {
		tags = append(tags, compression)
	}
	tags = append(tags, s.tags...)
	return tags
}
func (s *LogsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *LogsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *LogsProfileable) StartProfiling(_ io.Writer)       {}
func (s *LogsProfileable) EndProfiling(_ io.Writer)         {}
func (s *LogsProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *LogsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.logs = s.dataset.Logs(startAt, size)
}
func (s *LogsProfileable) CreateBatch(_ io.Writer, _, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow Records
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, 0, len(s.logs))
	for _, log := range s.logs {
		bar, err := s.producer.BatchArrowRecordsFromLogs(log)
		if err != nil {
			panic(err)
		}
		s.batchArrowRecords = append(s.batchArrowRecords, bar)
	}
}
func (s *LogsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *LogsProfileable) Serialize(io.Writer) ([][]byte, error) {
	buffers := make([][]byte, len(s.batchArrowRecords))
	for i, be := range s.batchArrowRecords {
		bytes, err := proto.Marshal(be)
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}
func (s *LogsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, len(buffers))
	for i, b := range buffers {
		bar := &v1.BatchArrowRecords{}
		if err := proto.Unmarshal(b, bar); err != nil {
			panic(err)
		}
		s.batchArrowRecords[i] = bar
	}
}
func (s *LogsProfileable) Clear() {
	s.logs = nil
	s.batchArrowRecords = s.batchArrowRecords[:0]
}
func (s *LogsProfileable) ShowStats() {}
