/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package otlp

import (
	"io"

	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark/dataset"

	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

type TracesProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     dataset.TraceDataset
	traces      []ptrace.Traces
}

func New(compression benchmark.CompressionAlgorithm) *TracesProfileable {
	return &TracesProfileable{compression: compression}
}

func NewTraceProfileable(dataset dataset.TraceDataset, compression benchmark.CompressionAlgorithm) *TracesProfileable {
	return &TracesProfileable{dataset: dataset, compression: compression}
}

func (s *TracesProfileable) Name() string {
	return "OTLP"
}

func (s *TracesProfileable) Tags() []string {
	return []string{s.compression.String()}
}

func (s *TracesProfileable) DatasetSize() int { return s.dataset.Len() }

func (s *TracesProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}

func (s *TracesProfileable) SetDataset(dataset dataset.TraceDataset) {
	s.dataset = dataset
}

func (s *TracesProfileable) StartProfiling(io.Writer) {}

func (s *TracesProfileable) EndProfiling(io.Writer) {}

func (s *TracesProfileable) InitBatchSize(_ io.Writer, _ int) {}

func (s *TracesProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.traces = s.dataset.Traces(startAt, size)
}

func (s *TracesProfileable) ConvertOtlpToOtlpArrow(_ io.Writer, _, _ int) {
	// In the standard OTLP exporter the incoming messages are already OTLP messages,
	// so we don't need to create or convert them.
}

func (s *TracesProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}

func (s *TracesProfileable) Serialize(io.Writer) ([][]byte, error) {
	// In the standard OTLP exporter, the incoming messages are serialized before being
	// sent via the standard protobuf serialization process.
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

func (s *TracesProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	// In the standard OTLP receiver the incoming messages are deserialized before being
	// sent to the collector pipeline.
	s.traces = make([]ptrace.Traces, len(buffers))
	for i, b := range buffers {
		r := ptraceotlp.NewExportRequest()
		if err := r.UnmarshalProto(b); err != nil {
			panic(err)
		}
		s.traces[i] = r.Traces()
	}
}

func (s *TracesProfileable) ConvertOtlpArrowToOtlp(_ io.Writer) {
	// In the standard OTLP receiver the incoming messages are already OTLP messages.
}

func (s *TracesProfileable) Clear() {
	s.traces = nil
}

func (s *TracesProfileable) ShowStats() {}
