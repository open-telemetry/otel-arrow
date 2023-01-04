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

	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"

	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

type TraceProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     dataset.TraceDataset
	traces      []ptrace.Traces
}

func NewTraceProfileable(dataset dataset.TraceDataset, compression benchmark.CompressionAlgorithm) *TraceProfileable {
	return &TraceProfileable{dataset: dataset, compression: compression}
}

func (s *TraceProfileable) Name() string {
	return "OTLP"
}

func (s *TraceProfileable) Tags() []string {
	return []string{s.compression.String()}
}
func (s *TraceProfileable) DatasetSize() int { return s.dataset.Len() }
func (s *TraceProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}
func (s *TraceProfileable) StartProfiling(io.Writer)           {}
func (s *TraceProfileable) EndProfiling(io.Writer)             {}
func (s *TraceProfileable) InitBatchSize(_ io.Writer, _ int)   {}
func (s *TraceProfileable) PrepareBatch(_ io.Writer, _, _ int) {}
func (s *TraceProfileable) CreateBatch(_ io.Writer, startAt, size int) {
	s.traces = s.dataset.Traces(startAt, size)
}
func (s *TraceProfileable) Process(io.Writer) string { return "" }
func (s *TraceProfileable) Serialize(io.Writer) ([][]byte, error) {
	buffers := make([][]byte, len(s.traces))
	for i, t := range s.traces {
		r := ptraceotlp.NewExportRequestFromTraces(t)

		bytes, err := r.MarshalProto()
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}
func (s *TraceProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	s.traces = make([]ptrace.Traces, len(buffers))
	for i, b := range buffers {
		r := ptraceotlp.NewExportRequest()
		if err := r.UnmarshalProto(b); err != nil {
			panic(err)
		}
		s.traces[i] = r.Traces()
	}
}
func (s *TraceProfileable) Clear() {
	s.traces = nil
}
func (s *TraceProfileable) ShowStats() {}
