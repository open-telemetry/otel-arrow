package otlp_arrow

import (
	"io"

	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/dataset"
	"otel-arrow-adapter/pkg/otel/batch_event"
	"otel-arrow-adapter/pkg/otel/metrics"
)

type MetricsProfileable struct {
	tags               []string
	compression        benchmark.CompressionAlgorithm
	dataset            dataset.MetricsDataset
	metrics            []*metricspb.ExportMetricsServiceRequest
	rr                 *air.RecordRepository
	producer           *batch_event.Producer
	batchEvents        []*v1.BatchEvent
	config             *config.Config
	multivariateConfig *metrics.MultivariateMetricsConfig
}

func NewMetricsProfileable(tags []string, dataset dataset.MetricsDataset, config *config.Config, multivariateConfig *metrics.MultivariateMetricsConfig, compression benchmark.CompressionAlgorithm) *MetricsProfileable {
	return &MetricsProfileable{
		tags:               tags,
		dataset:            dataset,
		compression:        compression,
		rr:                 nil,
		producer:           batch_event.NewProducer(),
		batchEvents:        make([]*v1.BatchEvent, 0, 10),
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
	// Conversion of OTLP metrics to OTLP Arrow events
	s.batchEvents = make([]*v1.BatchEvent, 0, len(s.metrics))
	for _, metricsServiceRequest := range s.metrics {
		records, err := metrics.OtlpMetricsToArrowRecords(s.rr, metricsServiceRequest, s.multivariateConfig)
		if err != nil {
			panic(err)
		}
		for _, record := range records {
			//fmt.Fprintf(writer, "IPC Message\n")
			//fmt.Fprintf(writer, "\t- schema id = %s\n", schemaId)
			//fmt.Fprintf(writer, "\t- record #row = %d\n", record.Column(0).Len())
			batchEvent, err := s.producer.Produce(batch_event.NewMetricsMessage(record, v1.DeliveryType_BEST_EFFORT))
			if err != nil {
				panic(err)
			}
			//fmt.Fprintf(writer, "\t- batch-id = %s\n", batchEvent.BatchId)
			//fmt.Fprintf(writer, "\t- sub-stream-id = %s\n", batchEvent.SubStreamId)
			//for _, p := range batchEvent.OtlpArrowPayloads {
			//	fmt.Fprintf(writer, "\t- IPC message size = %d\n", len(p.Schema))
			//}
			s.batchEvents = append(s.batchEvents, batchEvent)
		}
	}
}
func (s *MetricsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *MetricsProfileable) Serialize(io.Writer) ([][]byte, error) {
	buffers := make([][]byte, len(s.batchEvents))
	for i, be := range s.batchEvents {
		bytes, err := proto.Marshal(be)
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}
func (s *MetricsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.batchEvents = make([]*v1.BatchEvent, len(buffers))
	for i, b := range buffers {
		be := &v1.BatchEvent{}
		if err := proto.Unmarshal(b, be); err != nil {
			panic(err)
		}
		s.batchEvents[i] = be
	}
}
func (s *MetricsProfileable) Clear() {
	s.metrics = nil
	s.batchEvents = s.batchEvents[:0]
}
func (s *MetricsProfileable) ShowStats() {}
