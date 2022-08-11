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

type consumer struct {
	streamConsumers map[string]*streamConsumer
}

type streamConsumer struct {
	bufReader *bytes.Reader
	ipcReader *ipc.Reader
}

// NewConsumer creates a new BatchEvent consumer.
func NewConsumer() *consumer {
	return &consumer{
		streamConsumers: make(map[string]*streamConsumer),
	}
}

// Consume takes a BatchEvent protobuf message and returns an array of InternalBatchEvent.
func (c *consumer) Consume(event *coleventspb.BatchEvent) ([]*InternalBatchEvent, error) {
	// Retrieves (or creates) the stream consumer for the sub-stream id defined in the BatchEvent message.
	sc := c.streamConsumers[event.SubStreamId]
	if sc == nil {
		bufReader := bytes.NewReader([]byte{})
		sc = &streamConsumer{
			bufReader: bufReader,
		}
		c.streamConsumers[event.SubStreamId] = sc
	}

	var ibes []*InternalBatchEvent

	// Transform each individual OtlpArrowPayload into InternalBatchEvent
	for _, payload := range event.OtlpArrowPayloads {
		sc.bufReader.Reset(payload.Schema) // ToDo change the protobuf definition to contain a single ipc_message
		if sc.ipcReader == nil {
			ipcReader, err := ipc.NewReader(sc.bufReader)
			if err != nil {
				return nil, err
			}
			sc.ipcReader = ipcReader
		}

		for sc.ipcReader.Next() {
			rec := sc.ipcReader.Record()
			ibes = append(ibes, &InternalBatchEvent{
				batchId:      event.BatchId,
				recordType:   payload.GetType(),
				record:       rec,
				deliveryType: event.DeliveryType,
			})
		}
	}

	return ibes, nil
}
