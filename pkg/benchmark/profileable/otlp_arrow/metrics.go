package otlp_arrow

import (
	"io"

	"github.com/apache/arrow/go/v10/arrow/memory"
	"google.golang.org/protobuf/proto"

	"go.opentelemetry.io/collector/pdata/pmetric"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
)

type MetricsProfileable struct {
	tags              []string
	compression       benchmark.CompressionAlgorithm
	dataset           dataset.MetricsDataset
	metrics           []pmetric.Metrics
	producer          *arrow_record.Producer
	batchArrowRecords []*colarspb.BatchArrowRecords
	config            *benchmark.Config
	pool              *memory.GoAllocator
}

func NewMetricsProfileable(tags []string, dataset dataset.MetricsDataset, config *benchmark.Config) *MetricsProfileable {
	return &MetricsProfileable{
		tags:              tags,
		dataset:           dataset,
		compression:       benchmark.Zstd(), // ToDo replace Zstd with NoCompression when this bug will be fixed: https://github.com/apache/arrow/issues/14883
		producer:          arrow_record.NewProducer(),
		batchArrowRecords: make([]*colarspb.BatchArrowRecords, 0, 10),
		config:            config,
		pool:              memory.NewGoAllocator(),
	}
}

func (s *MetricsProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *MetricsProfileable) Tags() []string {
	var tags []string
	compression := s.compression.String()
	if compression != "" {
		tags = append(tags, compression)
	}
	tags = append(tags, s.tags...)
	return tags
}
func (s *MetricsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *MetricsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *MetricsProfileable) StartProfiling(_ io.Writer)       {}
func (s *MetricsProfileable) EndProfiling(_ io.Writer)         {}
func (s *MetricsProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *MetricsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsProfileable) CreateBatch(_ io.Writer, _, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow Records
	s.batchArrowRecords = make([]*colarspb.BatchArrowRecords, 0, len(s.metrics))
	for _, metricsServiceRequest := range s.metrics {
		bar, err := s.producer.BatchArrowRecordsFromMetrics(metricsServiceRequest)
		if err != nil {
			panic(err)
		}
		s.batchArrowRecords = append(s.batchArrowRecords, bar)
	}
}
func (s *MetricsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *MetricsProfileable) Serialize(io.Writer) ([][]byte, error) {
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
func (s *MetricsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.batchArrowRecords = make([]*colarspb.BatchArrowRecords, len(buffers))
	for i, b := range buffers {
		be := &colarspb.BatchArrowRecords{}
		if err := proto.Unmarshal(b, be); err != nil {
			panic(err)
		}
		s.batchArrowRecords[i] = be
	}
}
func (s *MetricsProfileable) Clear() {
	s.metrics = nil
	s.batchArrowRecords = s.batchArrowRecords[:0]
}
func (s *MetricsProfileable) ShowStats() {}
