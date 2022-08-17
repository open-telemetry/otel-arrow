package otlp_arrow

import (
	"io"

	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	tracepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/dataset"
	"otel-arrow-adapter/pkg/otel/batch_event"
	trace2 "otel-arrow-adapter/pkg/otel/trace"
)

type TraceProfileable struct {
	tags        []string
	compression benchmark.CompressionAlgorithm
	dataset     dataset.TraceDataset
	traces      []*tracepb.ExportTraceServiceRequest
	rr          *air.RecordRepository
	producer    *batch_event.Producer
	consumer    *batch_event.Consumer
	batchEvents []*v1.BatchEvent
	config      *config.Config
}

func NewTraceProfileable(tags []string, dataset dataset.TraceDataset, config *config.Config, compression benchmark.CompressionAlgorithm) *TraceProfileable {
	return &TraceProfileable{
		tags:        tags,
		dataset:     dataset,
		compression: compression,
		rr:          nil,
		producer:    batch_event.NewProducer(),
		consumer:    batch_event.NewConsumer(),
		batchEvents: make([]*v1.BatchEvent, 0, 10),
		config:      config,
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
func (s *TraceProfileable) StartProfiling(_ io.Writer) {
	s.rr = air.NewRecordRepository(s.config)
}
func (s *TraceProfileable) EndProfiling(writer io.Writer) {
	s.rr.DumpMetadata(writer)
	s.rr = nil
}
func (s *TraceProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *TraceProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.traces = s.dataset.Traces(startAt, size)
}
func (s *TraceProfileable) CreateBatch(_ io.Writer, _, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow events
	s.batchEvents = make([]*v1.BatchEvent, 0, len(s.traces))
	for _, trace := range s.traces {
		records, err := trace2.OtlpTraceToArrowRecords(s.rr, trace)
		if err != nil {
			panic(err)
		}
		for _, record := range records {
			//fmt.Fprintf(writer, "IPC Message\n")
			//fmt.Fprintf(writer, "\t- schema id = %s\n", schemaId)
			//fmt.Fprintf(writer, "\t- record #row = %d\n", record.Column(0).Len())
			batchEvent, err := s.producer.Produce(batch_event.NewTraceMessage(record, v1.DeliveryType_BEST_EFFORT))
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
func (s *TraceProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *TraceProfileable) Serialize(io.Writer) ([][]byte, error) {
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
func (s *TraceProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.batchEvents = make([]*v1.BatchEvent, len(buffers))
	for i, b := range buffers {
		be := &v1.BatchEvent{}
		if err := proto.Unmarshal(b, be); err != nil {
			panic(err)
		}
		s.batchEvents[i] = be

		// ToDo TMP
		//ibes, err := s.consumer.Consume(be)
		//if err != nil {
		//	panic(err)
		//}
		//for _, ibe := range ibes {
		//	request, err := trace2.ArrowRecordsToOtlpTrace(ibe.Record())
		//	if err != nil {
		//		panic(err)
		//	}
		//	if len(request.ResourceSpans) == 0 {
		//		panic("no resource spans")
		//	}
		//}
	}
}
func (s *TraceProfileable) Clear() {
	s.traces = nil
	s.batchEvents = s.batchEvents[:0]
}
func (s *TraceProfileable) ShowStats() {}
