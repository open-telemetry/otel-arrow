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

	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"

	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark/dataset"
)

type MetricsProfileable struct {
	compression benchmark.CompressionAlgorithm
	dataset     dataset.MetricsDataset
	metrics     []pmetric.Metrics
}

func NewMetricsProfileable(dataset dataset.MetricsDataset, compression benchmark.CompressionAlgorithm) *MetricsProfileable {
	return &MetricsProfileable{dataset: dataset, compression: compression}
}

func (s *MetricsProfileable) Name() string {
	return "OTLP"
}

func (s *MetricsProfileable) Tags() []string {
	return []string{s.compression.String()}
}

func (s *MetricsProfileable) DatasetSize() int { return s.dataset.Len() }

func (s *MetricsProfileable) CompressionAlgorithm() benchmark.CompressionAlgorithm {
	return s.compression
}

func (s *MetricsProfileable) StartProfiling(io.Writer) {}

func (s *MetricsProfileable) EndProfiling(io.Writer) {}

func (s *MetricsProfileable) InitBatchSize(_ io.Writer, _ int) {}

func (s *MetricsProfileable) PrepareBatch(_ io.Writer, startAt, size int) {
	s.metrics = s.dataset.Metrics(startAt, size)
}

func (s *MetricsProfileable) ConvertOtlpToOtlpArrow(_ io.Writer, _, _ int) {
	// In the standard OTLP exporter the incoming messages are already OTLP messages,
	// so we don't need to create or convert them.
}

func (s *MetricsProfileable) Process(io.Writer) string {
	// Not used in this benchmark
	return ""
}

func (s *MetricsProfileable) Serialize(io.Writer) ([][]byte, error) {
	// In the standard OTLP exporter, the incoming messages are serialized before being
	// sent via the standard protobuf serialization process.
	buffers := make([][]byte, len(s.metrics))
	for i, m := range s.metrics {
		r := pmetricotlp.NewExportRequestFromMetrics(m)
		bytes, err := r.MarshalProto()
		if err != nil {
			return nil, err
		}
		buffers[i] = bytes
	}
	return buffers, nil
}

func (s *MetricsProfileable) Deserialize(_ io.Writer, buffers [][]byte) {
	// In the standard OTLP receiver the incoming messages are deserialized before being
	// sent to the collector pipeline.
	s.metrics = make([]pmetric.Metrics, len(buffers))
	for i, b := range buffers {
		r := pmetricotlp.NewExportRequest()
		if err := r.UnmarshalProto(b); err != nil {
			panic(err)
		}
		s.metrics[i] = r.Metrics()
	}
}

func (s *MetricsProfileable) ConvertOtlpArrowToOtlp(_ io.Writer) {
	// In the standard OTLP receiver the incoming messages are already OTLP messages.
}

func (s *MetricsProfileable) Clear() {
	s.metrics = nil
}

func (s *MetricsProfileable) ShowStats() {}
