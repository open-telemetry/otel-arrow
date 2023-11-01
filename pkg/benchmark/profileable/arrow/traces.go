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

	"go.opentelemetry.io/collector/pdata/ptrace"

	v1 "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"
	cfg "github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
)

type TracesProfileable struct {
	tags                  []string
	compression           benchmark.CompressionAlgorithm
	dataset               dataset.TraceDataset
	traces                []ptrace.Traces
	producer              *arrow_record.Producer
	consumer              *arrow_record.Consumer
	batchArrowRecords     []*v1.BatchArrowRecords
	config                *benchmark.Config
	pool                  *memory.GoAllocator
	unaryRpcMode          bool
	stats                 bool
	tracesProducerOptions []cfg.Option

	observer arrow_record.ProducerObserver
}

func WithOption(tags []string, options ...cfg.Option) *TracesProfileable {
	return &TracesProfileable{
		tags:                  tags,
		compression:           benchmark.Zstd(),
		producer:              arrow_record.NewProducerWithOptions(options...),
		consumer:              arrow_record.NewConsumer(),
		batchArrowRecords:     make([]*v1.BatchArrowRecords, 0, 10),
		pool:                  memory.NewGoAllocator(),
		unaryRpcMode:          false,
		tracesProducerOptions: options,
	}
}

func NewTraceProfileable(tags []string, dataset dataset.TraceDataset, config *benchmark.Config) *TracesProfileable {
	var tracesProducerOptions []cfg.Option

	if config.Compression {
		tracesProducerOptions = append(tracesProducerOptions, cfg.WithZstd())
	} else {
		tracesProducerOptions = append(tracesProducerOptions, cfg.WithNoZstd())
	}
	if config.Stats {
		tracesProducerOptions = append(tracesProducerOptions, cfg.WithSchemaStats())
	}

	return &TracesProfileable{
		tags:                  tags,
		dataset:               dataset,
		compression:           benchmark.Zstd(),
		producer:              arrow_record.NewProducerWithOptions(tracesProducerOptions...),
		consumer:              arrow_record.NewConsumer(),
		batchArrowRecords:     make([]*v1.BatchArrowRecords, 0, 10),
		config:                config,
		pool:                  memory.NewGoAllocator(),
		unaryRpcMode:          false,
		stats:                 config.Stats,
		tracesProducerOptions: tracesProducerOptions,
	}
}

func (s *TracesProfileable) SetDataset(dataset dataset.TraceDataset) {
	s.dataset = dataset
}

func (s *TracesProfileable) SetObserver(observer arrow_record.ProducerObserver) {
	s.observer = observer
}

func (s *TracesProfileable) Name() string {
	return "OTel_ARROW"
}

func (s *TracesProfileable) EnableUnaryRpcMode() {
	s.unaryRpcMode = true
}

func (s *TracesProfileable) Tags() []string {
	var tags []string
	compression := s.compression.String()
	if compression != "" {
		tags = append(tags, compression)
	}
	tags = append(tags, s.tags...)
	return tags
}
func (s *TracesProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *TracesProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *TracesProfileable) StartProfiling(_ io.Writer) {
	if !s.unaryRpcMode {
		s.producer = arrow_record.NewProducerWithOptions(s.tracesProducerOptions...)
		s.consumer = arrow_record.NewConsumer()
		s.producer.SetObserver(s.observer)
	}
}
func (s *TracesProfileable) EndProfiling(_ io.Writer) {
	if !s.unaryRpcMode {
		if err := s.producer.Close(); err != nil {
			panic(err)
		}
		if err := s.consumer.Close(); err != nil {
			panic(err)
		}
	}
}
func (s *TracesProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *TracesProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	if s.unaryRpcMode {
		s.producer = arrow_record.NewProducerWithOptions(s.tracesProducerOptions...)
		s.consumer = arrow_record.NewConsumer()
		s.producer.SetObserver(s.observer)
	}

	s.traces = s.dataset.Traces(startAt, size)
}
func (s *TracesProfileable) ConvertOtlpToOtlpArrow(_ io.Writer, _, _ int) {
	// In the OTLP Arrow exporter, incoming OTLP messages must be converted to
	// OTLP Arrow messages.
	// This step contains the conversion from OTLP to OTLP Arrow, the conversion to Arrow IPC,
	// and the compression.
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, 0, len(s.traces))
	for _, traceReq := range s.traces {
		bar, err := s.producer.BatchArrowRecordsFromTraces(traceReq)
		if err != nil {
			panic(err)
		}
		s.batchArrowRecords = append(s.batchArrowRecords, bar)
	}
}
func (s *TracesProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *TracesProfileable) Serialize(io.Writer) ([][]byte, error) {
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

func (s *TracesProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	// In the OTLP Arrow exporter, OTLP Arrow messages are deserialized via the
	// standard protobuf deserialization process.
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, len(buffers))
	for i, b := range buffers {
		batchArrowRecords := &v1.BatchArrowRecords{}
		if err := proto.Unmarshal(b, batchArrowRecords); err != nil {
			panic(err)
		}
		s.batchArrowRecords[i] = batchArrowRecords
	}
}

func (s *TracesProfileable) ConvertOtlpArrowToOtlp(_ io.Writer) {
	for _, batchArrowRecords := range s.batchArrowRecords {
		traces, err := s.consumer.TracesFrom(batchArrowRecords)
		if err != nil {
			panic(err)
		}
		if len(traces) == 0 {
			println("no traces")
		}
	}
}

func (s *TracesProfileable) Clear() {
	s.traces = nil
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
func (s *TracesProfileable) ShowStats() {
	if s.stats {
		s.producer.ShowStats()
	}
}
