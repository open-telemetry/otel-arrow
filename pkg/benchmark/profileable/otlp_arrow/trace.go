package otlp_arrow

import (
	"bytes"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/ipc"

	tracepb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
)

type TraceProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     benchmark.TraceDataset
	traces      []*tracepb.ExportTraceServiceRequest
	rr          *air.RecordRepository
	records     []arrow.Record
}

func NewTraceProfileable(dataset benchmark.TraceDataset, compression benchmark.CompressionAlgorithm) *TraceProfileable {
	return &TraceProfileable{dataset: dataset, compression: compression, rr: air.NewRecordRepository(config.NewDefaultConfig()), records: []arrow.Record{}}
}

func (s *TraceProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *TraceProfileable) Tags() []string {
	return []string{s.compression.String()}
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

}
func (s *TraceProfileable) Process() string {
	// Not used in this benchmark
	return ""
}
func (s *TraceProfileable) Serialize() ([][]byte, error) {
	buffers := make([][]byte, len(s.records))
	for _, r := range s.records {
		var buf bytes.Buffer
		w := ipc.NewWriter(&buf, ipc.WithSchema(r.Schema()))
		err := w.Write(r)
		if err != nil {
			return nil, err
		}
		err = w.Close()
		if err != nil {
			return nil, err
		}
		r.Release()
		buffers = append(buffers, buf.Bytes())
	}
	return buffers, nil
}
func (s *TraceProfileable) Deserialize(buffers [][]byte) {
	println("ToDo Deserialize")
	//for _, b := range buffers {
	//	reader, err := ipc.NewReader(bytes.NewReader(b))
	//	if err != nil {
	//		panic(err)
	//	}
	//	record, err := reader.Read()
	//	if err != nil {
	//		panic(err)
	//	}
	//	record.Release()
	//}
}
func (s *TraceProfileable) Clear() {
	s.traces = nil
	s.records = s.records[:0]
}
func (s *TraceProfileable) ShowStats() {}
