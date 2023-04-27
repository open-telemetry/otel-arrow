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

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
)

var metricsProducerOptions = []config.Option{
	config.WithNoZstd(),
	config.WithStats(),
}

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
}

func NewMetricsProfileable(tags []string, dataset dataset.MetricsDataset, config *benchmark.Config) *MetricsProfileable {
	return &MetricsProfileable{
		tags:              tags,
		dataset:           dataset,
		compression:       benchmark.Zstd(),
		producer:          arrow_record.NewProducerWithOptions(metricsProducerOptions...),
		consumer:          arrow_record.NewConsumer(),
		batchArrowRecords: make([]*colarspb.BatchArrowRecords, 0, 10),
		config:            config,
		pool:              memory.NewGoAllocator(),
		unaryRpcMode:      false,
	}
}

func (s *MetricsProfileable) Name() string {
	return "OTLP_ARROW"
}

func (s *MetricsProfileable) EnableUnaryRpcMode() {
	s.unaryRpcMode = true
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
		s.producer = arrow_record.NewProducerWithOptions(metricsProducerOptions...)
		s.consumer = arrow_record.NewConsumer()
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
		s.producer = arrow_record.NewProducerWithOptions(metricsProducerOptions...)
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
	stats := s.producer.MetricsStats()
	if stats != nil {
		stats.Show()
	}
}
