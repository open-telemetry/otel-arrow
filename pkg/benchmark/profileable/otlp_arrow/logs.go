package otlp_arrow

import (
	"io"

	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	logspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/dataset"
	"otel-arrow-adapter/pkg/otel/batch_event"
	logs "otel-arrow-adapter/pkg/otel/logs"
)

type LogsProfileable struct {
	tags        []string
	compression benchmark.CompressionAlgorithm
	dataset     dataset.LogsDataset
	logs        []*logspb.ExportLogsServiceRequest
	rr          *air.RecordRepository
	producer    *batch_event.Producer
	batchEvents []*v1.BatchEvent
	config      *config.Config
}

func NewLogsProfileable(tags []string, dataset dataset.LogsDataset, config *config.Config, compression benchmark.CompressionAlgorithm) *LogsProfileable {
	return &LogsProfileable{
		tags:        tags,
		dataset:     dataset,
		compression: compression,
		rr:          nil,
		producer:    batch_event.NewProducer(),
		batchEvents: make([]*v1.BatchEvent, 0, 10),
		config:      config,
	}
}

func (s *LogsProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *LogsProfileable) Tags() []string {
	tags := []string{s.compression.String()}
	tags = append(tags, s.tags...)
	return tags
}
func (s *LogsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *LogsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *LogsProfileable) StartProfiling(_ io.Writer) {
	s.rr = air.NewRecordRepository(s.config)
}
func (s *LogsProfileable) EndProfiling(writer io.Writer) {
	s.rr.DumpMetadata(writer)
	s.rr = nil
}
func (s *LogsProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *LogsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.logs = s.dataset.Logs(startAt, size)
}
func (s *LogsProfileable) CreateBatch(_ io.Writer, _, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow events
	s.batchEvents = make([]*v1.BatchEvent, 0, len(s.logs))
	for _, log := range s.logs {
		records, err := logs.OtlpLogsToArrowRecords(s.rr, log, s.config)
		if err != nil {
			panic(err)
		}
		for _, record := range records {
			//fmt.Fprintf(writer, "IPC Message\n")
			//fmt.Fprintf(writer, "\t- schema id = %s\n", schemaId)
			//fmt.Fprintf(writer, "\t- record #row = %d\n", record.Column(0).Len())
			batchEvent, err := s.producer.Produce(batch_event.NewLogsMessage(record, v1.DeliveryType_BEST_EFFORT))
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
func (s *LogsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *LogsProfileable) Serialize(io.Writer) ([][]byte, error) {
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
func (s *LogsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.batchEvents = make([]*v1.BatchEvent, len(buffers))
	for i, b := range buffers {
		be := &v1.BatchEvent{}
		if err := proto.Unmarshal(b, be); err != nil {
			panic(err)
		}
		s.batchEvents[i] = be
	}
}
func (s *LogsProfileable) Clear() {
	s.logs = nil
	s.batchEvents = s.batchEvents[:0]
}
func (s *LogsProfileable) ShowStats() {}
