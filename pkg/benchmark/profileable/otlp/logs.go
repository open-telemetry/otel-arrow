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

package otlp

import (
	"io"

	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"

	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"
)

const Otlp = "OTLP"

type LogsProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     dataset.LogsDataset
	logs        []plog.Logs
}

func NewLogsProfileable(dataset dataset.LogsDataset, compression benchmark.CompressionAlgorithm) *LogsProfileable {
	return &LogsProfileable{dataset: dataset, compression: compression}
}

func (s *LogsProfileable) Name() string {
	return Otlp
}

func (s *LogsProfileable) Tags() []string {
	return []string{s.compression.String()}
}

func (s *LogsProfileable) DatasetSize() int { return s.dataset.Len() }

func (s *LogsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}

func (s *LogsProfileable) StartProfiling(io.Writer) {}

func (s *LogsProfileable) EndProfiling(io.Writer) {}

func (s *LogsProfileable) InitBatchSize(_ io.Writer, _ int) {}

func (s *LogsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.logs = s.dataset.Logs(startAt, size)
}
func (s *LogsProfileable) ConvertOtlpToOtlpArrow(_ io.Writer, _, _ int) {
	// In the standard OTLP exporter the incoming messages are already OTLP messages,
	// so we don't need to create or convert them.
}

func (s *LogsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}

func (s *LogsProfileable) Serialize(io.Writer) ([][]byte, error) {
	// In the standard OTLP exporter, the incoming messages are serialized before being
	// sent via the standard protobuf serialization process.
	buffers := make([][]byte, len(s.logs))
	for i, l := range s.logs {
		r := plogotlp.NewExportRequestFromLogs(l)
		bytes, err := r.MarshalProto()
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}

func (s *LogsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	// In the standard OTLP receiver the incoming messages are deserialized before being
	// sent to the collector pipeline.
	s.logs = make([]plog.Logs, len(buffers))
	for i, b := range buffers {
		r := plogotlp.NewExportRequest()
		if err := r.UnmarshalProto(b); err != nil {
			panic(err)
		}
		s.logs[i] = r.Logs()
	}
}

func (s *LogsProfileable) ConvertOtlpArrowToOtlp(_ io.Writer) {
	// In the standard OTLP receiver the incoming messages are already OTLP messages.
}

func (s *LogsProfileable) Clear() {
	s.logs = nil
}

func (s *LogsProfileable) ShowStats() {}
