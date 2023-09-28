// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package arrow

import (
	"io"

	"github.com/apache/arrow/go/v12/arrow/memory"
	"google.golang.org/protobuf/proto"

	"go.opentelemetry.io/collector/pdata/pmetric"

	colarspb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"
	cfg "github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
)

type MetricsProfileable struct {
	tags              []string
	compression       benchmark.CompressionAlgorithm
	dataset           dataset.MetricsDataset
	metrics           []pmetric.Metrics
	producer          *arrow_record.Producer
	consumer          *arrow_record.Consumer
	batchArrowRecords []*colarspb.BatchArrowRecords
	config            *benchmark.Config
	pool              *memory.GoAllocator
	unaryRpcMode      bool
	options           []cfg.Option

	observer arrow_record.ProducerObserver
}

func NewMetricsProfileable(tags []string, dataset dataset.MetricsDataset, config *benchmark.Config) *MetricsProfileable {
	var options []cfg.Option

	if config.Compression {
		options = append(options, cfg.WithZstd())
	} else {
		options = append(options, cfg.WithNoZstd())
	}
	if config.Stats {
		options = append(options, cfg.WithSchemaStats())
	}

	producer := arrow_record.NewProducerWithOptions(options...)

	return &MetricsProfileable{
		tags:              tags,
		dataset:           dataset,
		compression:       benchmark.Zstd(),
		producer:          producer,
		consumer:          arrow_record.NewConsumer(),
		batchArrowRecords: make([]*colarspb.BatchArrowRecords, 0, 10),
		config:            config,
		pool:              memory.NewGoAllocator(),
		unaryRpcMode:      false,
		options:           options,
	}
}

func (s *MetricsProfileable) Name() string {
	return "OTel_ARROW"
}

func (s *MetricsProfileable) EnableUnaryRpcMode() {
	s.unaryRpcMode = true
}

func (s *MetricsProfileable) SetObserver(observer arrow_record.ProducerObserver) {
	s.observer = observer
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
func (s *MetricsProfileable) StartProfiling(_ io.Writer) {
	if !s.unaryRpcMode {
		s.producer = arrow_record.NewProducerWithOptions(s.options...)
		s.consumer = arrow_record.NewConsumer()
		s.producer.SetObserver(s.observer)
	}
}
func (s *MetricsProfileable) EndProfiling(_ io.Writer) {
	if !s.unaryRpcMode {
		if err := s.producer.Close(); err != nil {
			panic(err)
		}
		if err := s.consumer.Close(); err != nil {
			panic(err)
		}
	}
}
func (s *MetricsProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *MetricsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	if s.unaryRpcMode {
		s.producer = arrow_record.NewProducerWithOptions(s.options...)
		s.producer.SetObserver(s.observer)
		s.consumer = arrow_record.NewConsumer()
	}

	s.metrics = s.dataset.Metrics(startAt, size)
}
func (s *MetricsProfileable) ConvertOtlpToOtlpArrow(_ io.Writer, _, _ int) {
	// In the OTLP Arrow exporter, incoming OTLP messages must be converted to
	// OTLP Arrow messages.
	// This step contains the conversion from OTLP to OTLP Arrow, the conversion to Arrow IPC,
	// and the compression.
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
	// In the OTLP Arrow exporter, OTLP Arrow messages are serialized via the
	// standard protobuf serialization process.
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

func (s *MetricsProfileable) ConvertOtlpArrowToOtlp(_ io.Writer) {
	for _, batchArrowRecords := range s.batchArrowRecords {
		metrics, err := s.consumer.MetricsFrom(batchArrowRecords)
		if err != nil {
			panic(err)
		}
		if len(metrics) == 0 {
			panic("no metrics")
		}
	}
}

func (s *MetricsProfileable) Clear() {
	s.metrics = nil
	s.batchArrowRecords = s.batchArrowRecords[:0]

	if s.unaryRpcMode {
		if err := s.producer.Close(); err != nil {
			panic(err)
		}
		if err := s.consumer.Close(); err != nil {
			panic(err)
		}
	}
}
func (s *MetricsProfileable) ShowStats() {
}
