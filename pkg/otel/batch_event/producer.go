/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package batch_event

import (
	"bytes"

	"github.com/apache/arrow/go/v9/arrow/ipc"

	coleventspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
)

type Producer struct {
	streamProducers map[string]*streamProducer
}

type streamProducer struct {
	output    bytes.Buffer
	ipcWriter *ipc.Writer
}

// NewProducer creates a new BatchEvent producer.
func NewProducer() *Producer {
	return &Producer{
		streamProducers: make(map[string]*streamProducer),
	}
}

// Produce takes an InternalBatchEvent and returns the corresponding BatchEvent protobuf message.
func (p *Producer) Produce(ibe *InternalBatchEvent) (*coleventspb.BatchEvent, error) {
	// Retrieves (or creates) the stream Producer for the sub-stream id defined in the InternalBatchEvent.
	sp := p.streamProducers[ibe.subStreamId]
	if sp == nil {
		var buf bytes.Buffer
		sp = &streamProducer{
			output: buf,
		}
	}

	if sp.ipcWriter == nil {
		sp.ipcWriter = ipc.NewWriter(&sp.output, ipc.WithSchema(ibe.record.Schema()))
	}
	err := sp.ipcWriter.Write(ibe.record)
	if err != nil {
		return nil, err
	}
	buf := sp.output.Bytes()

	// Reset the buffer
	sp.output.Reset()

	return &coleventspb.BatchEvent{
		BatchId:     ibe.batchId,
		SubStreamId: ibe.subStreamId,
		OtlpArrowPayloads: []*coleventspb.OtlpArrowPayload{
			{
				Type:   ibe.recordType,
				Schema: buf,
			},
		},
		DeliveryType: ibe.deliveryType,
	}, nil
}
