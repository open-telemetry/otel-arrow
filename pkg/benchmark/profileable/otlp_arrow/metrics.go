package otlp_arrow

import (
	"io"

	"google.golang.org/protobuf/proto"

	v1 "github.com/lquerel/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/lquerel/otel-arrow-adapter/pkg/air"
	"github.com/lquerel/otel-arrow-adapter/pkg/air/config"
	"github.com/lquerel/otel-arrow-adapter/pkg/benchmark"
	"github.com/lquerel/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/arrow_record"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/metrics"

	"go.opentelemetry.io/collector/pdata/pmetric"
)

type MetricsProfileable struct {
	tags               []string
	compression        benchmark.CompressionAlgorithm
	dataset            dataset.MetricsDataset
	metrics            []pmetric.Metrics
	rr                 *air.RecordRepository
	producer           *arrow_record.Producer
	batchArrowRecords  []*v1.BatchArrowRecords
	config             *config.Config
	multivariateConfig *metrics.MultivariateMetricsConfig
}

func NewMetricsProfileable(tags []string, dataset dataset.MetricsDataset, config *config.Config, multivariateConfig *metrics.MultivariateMetricsConfig, compression benchmark.CompressionAlgorithm) *MetricsProfileable {
	return &MetricsProfileable{
		tags:               tags,
		dataset:            dataset,
		compression:        compression,
		rr:                 nil,
		producer:           arrow_record.NewProducer(),
		batchArrowRecords:  make([]*v1.BatchArrowRecords, 0, 10),
		config:             config,
		multivariateConfig: multivariateConfig,
	}
}

func (s *MetricsProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *MetricsProfileable) Tags() []string {
	tags := []string{s.compression.String()}
	tags = append(tags, s.tags...)
	return tags
}
func (s *MetricsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *MetricsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *MetricsProfileable) StartProfiling(_ io.Writer) {
	s.rr = air.NewRecordRepository(s.config)
}
func (s *MetricsProfileable) EndProfiling(writer io.Writer) {
	s.rr.DumpMetadata(writer)
	s.rr = nil
}
func (s *MetricsProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *MetricsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsProfileable) CreateBatch(_ io.Writer, _, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow Records
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, 0, len(s.metrics))
	for _, metricsServiceRequest := range s.metrics {
		records, err := metrics.OtlpMetricsToArrowRecords(s.rr, metricsServiceRequest, s.multivariateConfig, s.config)
		if err != nil {
			panic(err)
		}
		rms := make([]*arrow_record.RecordMessage, len(records))
		for i, record := range records {
			rms[i] = arrow_record.NewMetricsMessage(record, v1.DeliveryType_BEST_EFFORT)
		}
		bar, err := s.producer.Produce(rms, v1.DeliveryType_BEST_EFFORT)
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
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, len(buffers))
	for i, b := range buffers {
		be := &v1.BatchArrowRecords{}
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
