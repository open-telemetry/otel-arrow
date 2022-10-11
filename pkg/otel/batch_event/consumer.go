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
	"go.opentelemetry.io/collector/pdata/ptrace"

	coleventspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	"otel-arrow-adapter/pkg/otel/traces"
)

type Consumer struct {
	streamConsumers   map[string]*streamConsumer
	otlpTraceProducer *traces.OtlpProducer
}

type streamConsumer struct {
	bufReader *bytes.Reader
	ipcReader *ipc.Reader
}

// NewConsumer creates a new BatchEvent consumer.
func NewConsumer() *Consumer {
	return &Consumer{
		streamConsumers:   make(map[string]*streamConsumer),
		otlpTraceProducer: traces.NewOtlpProducer(),
	}
}

// TracesFrom produces an array of ptrace.Traces from a BatchEvent message.
func (c *Consumer) TracesFrom(batchEvent *coleventspb.BatchEvent) ([]ptrace.Traces, error) {
	records, err := c.Consume(batchEvent)
	if err != nil {
		return nil, err
	}

	var result []ptrace.Traces
	for i := 1; i < len(records); i++ {
		record := records[i]
		tracesArr, err := c.otlpTraceProducer.ProduceFrom(record.record)
		if err != nil {
			return nil, err
		}
		result = append(result, tracesArr...)
	}
	return result, nil
}

// Consume takes a BatchEvent protobuf message and returns an array of RecordMessage.
func (c *Consumer) Consume(event *coleventspb.BatchEvent) ([]*RecordMessage, error) {
	// Retrieves (or creates) the stream consumer for the sub-stream id defined in the BatchEvent message.
	sc := c.streamConsumers[event.SubStreamId]
	if sc == nil {
		bufReader := bytes.NewReader([]byte{})
		sc = &streamConsumer{
			bufReader: bufReader,
		}
		c.streamConsumers[event.SubStreamId] = sc
	}

	var ibes []*RecordMessage

	// Transform each individual OtlpArrowPayload into RecordMessage
	for _, payload := range event.OtlpArrowPayloads {
		sc.bufReader.Reset(payload.Schema) // ToDo change the protobuf definition to contain a single ipc_message
		if sc.ipcReader == nil {
			ipcReader, err := ipc.NewReader(sc.bufReader)
			if err != nil {
				return nil, err
			}
			sc.ipcReader = ipcReader
		}

		if sc.ipcReader.Next() {
			rec := sc.ipcReader.Record()
			ibes = append(ibes, &RecordMessage{
				batchId:      event.BatchId,
				recordType:   payload.GetType(),
				record:       rec,
				deliveryType: event.DeliveryType,
			})
		}
	}

	return ibes, nil
}
