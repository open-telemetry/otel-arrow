package otlp_arrow

import (
	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	tracepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/otel/batch_event"
	trace2 "otel-arrow-adapter/pkg/otel/trace"
)

type TraceProfileable struct {
	tags        []string
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.TraceDataset
	traces      []*tracepb.ExportTraceServiceRequest
	rr          *air.RecordRepository
	producer    *batch_event.Producer
	batchEvents []*v1.BatchEvent
}

func NewTraceProfileable(tags []string, dataset benchmark.TraceDataset, config *config.Config, compression benchmark.CompressionAlgorithm) *TraceProfileable {
	return &TraceProfileable{
		tags:        tags,
		dataset:     dataset,
		compression: compression,
		rr:          air.NewRecordRepository(config),
		producer:    batch_event.NewProducer(),
		batchEvents: make([]*v1.BatchEvent, 0, 10),
	}
}

func (s *TraceProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *TraceProfileable) Tags() []string {
	tags := []string{s.compression.String()}
	tags = append(tags, s.tags...)
	return tags
}
func (s *TraceProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *TraceProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *TraceProfileable) InitBatchSize(_ int) {}
func (s *TraceProfileable) PrepareBatch(startAt, size int) {
	s.traces = s.dataset.Traces(startAt, size)
}
func (s *TraceProfileable) CreateBatch(_, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow events
	s.batchEvents = make([]*v1.BatchEvent, 0, len(s.traces))
	for _, trace := range s.traces {
		records, err := trace2.OtlpTraceToArrowRecords(s.rr, trace)
		if err != nil {
			panic(err)
		}
		for schemaId, record := range records {
			batchEvent, err := s.producer.Produce(batch_event.NewBatchEventOfTraces(schemaId, record, v1.DeliveryType_BEST_EFFORT))
			if err != nil {
				panic(err)
			}
			s.batchEvents = append(s.batchEvents, batchEvent)
		}
	}
}
func (s *TraceProfileable) Process() string {
	// Not used in this benchmark
	return ""
}
func (s *TraceProfileable) Serialize() ([][]byte, error) {
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
func (s *TraceProfileable) Deserialize(buffers [][]byte) {
	s.batchEvents = make([]*v1.BatchEvent, len(buffers))
	for i, b := range buffers {
		be := &v1.BatchEvent{}
		if err := proto.Unmarshal(b, be); err != nil {
			panic(err)
		}
		s.batchEvents[i] = be
	}
}
func (s *TraceProfileable) Clear() {
	s.traces = nil
	s.batchEvents = s.batchEvents[:0]
}
func (s *TraceProfileable) ShowStats() {}
