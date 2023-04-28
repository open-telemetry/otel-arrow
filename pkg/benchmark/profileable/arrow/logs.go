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

	"go.opentelemetry.io/collector/pdata/plog"

	v1 "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	cfg "github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
)

const OtlpArrow = "OTLP_ARROW"

type LogsProfileable struct {
	tags                []string
	compression         benchmark.CompressionAlgorithm
	dataset             dataset.LogsDataset
	logs                []plog.Logs
	producer            *arrow_record.Producer
	consumer            *arrow_record.Consumer
	batchArrowRecords   []*v1.BatchArrowRecords
	config              *benchmark.Config
	pool                *memory.GoAllocator
	unaryRpcMode        bool
	stats               bool
	logsProducerOptions []cfg.Option
}

func NewLogsProfileable(tags []string, dataset dataset.LogsDataset, config *benchmark.Config) *LogsProfileable {
	var logsProducerOptions []cfg.Option

	if config.Compression {
		logsProducerOptions = append(logsProducerOptions, cfg.WithZstd())
	} else {
		logsProducerOptions = append(logsProducerOptions, cfg.WithNoZstd())
	}
	if config.Stats {
		logsProducerOptions = append(logsProducerOptions, cfg.WithStats())
	}

	return &LogsProfileable{
		tags:                tags,
		dataset:             dataset,
		compression:         benchmark.Zstd(),
		producer:            arrow_record.NewProducerWithOptions(logsProducerOptions...),
		consumer:            arrow_record.NewConsumer(),
		batchArrowRecords:   make([]*v1.BatchArrowRecords, 0, 10),
		config:              config,
		pool:                memory.NewGoAllocator(),
		unaryRpcMode:        false,
		stats:               config.Stats,
		logsProducerOptions: logsProducerOptions,
	}
}

func (s *LogsProfileable) Name() string {
	return OtlpArrow
}

func (s *LogsProfileable) EnableUnaryRpcMode() {
	s.unaryRpcMode = true
}

func (s *LogsProfileable) Tags() []string {
	var tags []string
	compression := s.compression.String()
	if compression != "" {
		tags = append(tags, compression)
	}
	tags = append(tags, s.tags...)
	return tags
}
func (s *LogsProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *LogsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *LogsProfileable) StartProfiling(_ io.Writer) {
	if !s.unaryRpcMode {
		s.producer = arrow_record.NewProducerWithOptions(s.logsProducerOptions...)
		s.consumer = arrow_record.NewConsumer()
	}
}
func (s *LogsProfileable) EndProfiling(_ io.Writer) {
	if !s.unaryRpcMode {
		if err := s.producer.Close(); err != nil {
			panic(err)
		}
		if err := s.consumer.Close(); err != nil {
			panic(err)
		}
	}
}
func (s *LogsProfileable) InitBatchSize(_ io.Writer, _ int) {}
func (s *LogsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	if s.unaryRpcMode {
		s.producer = arrow_record.NewProducerWithOptions(s.logsProducerOptions...)
		s.consumer = arrow_record.NewConsumer()
	}
	s.logs = s.dataset.Logs(startAt, size)
}
func (s *LogsProfileable) ConvertOtlpToOtlpArrow(_ io.Writer, _, _ int) {
	// In the OTLP Arrow exporter, incoming OTLP messages must be converted to
	// OTLP Arrow messages.
	// This step contains the conversion from OTLP to OTLP Arrow, the conversion to Arrow IPC,
	// and the compression.
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, 0, len(s.logs))
	for _, log := range s.logs {
		bar, err := s.producer.BatchArrowRecordsFromLogs(log)
		if err != nil {
			panic(err)
		}
		s.batchArrowRecords = append(s.batchArrowRecords, bar)
	}
}
func (s *LogsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}
func (s *LogsProfileable) Serialize(io.Writer) ([][]byte, error) {
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
func (s *LogsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	// In the OTLP Arrow exporter, OTLP Arrow messages are deserialized via the
	// standard protobuf deserialization process.
	s.batchArrowRecords = make([]*v1.BatchArrowRecords, len(buffers))
	for i, b := range buffers {
		bar := &v1.BatchArrowRecords{}
		if err := proto.Unmarshal(b, bar); err != nil {
			panic(err)
		}
		s.batchArrowRecords[i] = bar
	}
}
func (s *LogsProfileable) ConvertOtlpArrowToOtlp(_ io.Writer) {
	for _, batchArrowRecords := range s.batchArrowRecords {
		logs, err := s.consumer.LogsFrom(batchArrowRecords)
		if err != nil {
			panic(err)
		}
		if len(logs) == 0 {
			println("no logs")
		}
	}
}
func (s *LogsProfileable) Clear() {
	s.logs = nil
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
func (s *LogsProfileable) ShowStats() {
	if s.stats {
		s.producer.ShowStats()
	}
}
