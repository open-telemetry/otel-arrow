package otlp_arrow

import (
	"bytes"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/ipc"

	metricspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/otel/metrics"
)

type MetricsProfileable struct {
	compression        benchmark.CompressionAlgorithm
	dataset            benchmark.MetricsDataset
	metrics            []*metricspb.ExportMetricsServiceRequest
	rr                 *air.RecordRepository
	multivariateConfig *metrics.MultivariateMetricsConfig
	records            []arrow.Record
	profileName        string
}

func NewMetricsProfileable(profileName string, dataset benchmark.MetricsDataset, dictionaryCfg *config.Config, multivariateCfg *metrics.MultivariateMetricsConfig, compression benchmark.CompressionAlgorithm) *MetricsProfileable {
	return &MetricsProfileable{dataset: dataset, compression: compression, rr: air.NewRecordRepository(dictionaryCfg), multivariateConfig: multivariateCfg, records: []arrow.Record{}, profileName: profileName}
}

func (s *MetricsProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *MetricsProfileable) Tags() []string {
	return []string{s.compression.String(), s.profileName}
}
func (s *MetricsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *MetricsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *MetricsProfileable) StartProfiling()     {}
func (s *MetricsProfileable) EndProfiling()       {}
func (s *MetricsProfileable) InitBatchSize(_ int) {}
func (s *MetricsProfileable) PrepareBatch(startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsProfileable) CreateBatch(_, _ int) {
	// Conversion of OTLP metrics to OTLP Arrow events
	for _, m := range s.metrics {
		records, err := metrics.OtlpMetricsToArrowRecords(s.rr, m, s.multivariateConfig)
		if err != nil {
			panic(err)
		}
		for _, r := range records {
			s.records = append(s.records, r...)
		}
	}
}
func (s *MetricsProfileable) Process() string {
	// Not used in this benchmark
	return ""
}
func (s *MetricsProfileable) Serialize() ([][]byte, error) {
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
func (s *MetricsProfileable) Deserialize(buffers [][]byte) {
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
func (s *MetricsProfileable) Clear() {
	s.metrics = nil
	s.records = s.records[:0]
}
func (s *MetricsProfileable) ShowStats() {}
