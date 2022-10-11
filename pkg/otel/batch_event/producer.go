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
	"fmt"

	"github.com/apache/arrow/go/v9/arrow/ipc"
	"go.opentelemetry.io/collector/pdata/ptrace"

	coleventspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	"otel-arrow-adapter/pkg/otel/traces"
)

type Producer struct {
	streamProducers         map[string]*streamProducer
	otlpArrowTracesProducer *traces.OtlpArrowProducer
}

type streamProducer struct {
	output      bytes.Buffer
	ipcWriter   *ipc.Writer
	batchId     int64
	subStreamId string
}

// NewProducer creates a new BatchEvent producer.
func NewProducer() *Producer {
	return &Producer{
		streamProducers:         make(map[string]*streamProducer),
		otlpArrowTracesProducer: traces.NewOtlpArrowProducer(),
	}
}

// BatchEventsFrom produces a BatchEvent message from a ptrace.Traces messages.
func (p *Producer) BatchEventsFrom(traces ptrace.Traces) ([]*coleventspb.BatchEvent, error) {
	records, err := p.otlpArrowTracesProducer.ProduceFrom(traces)
	if err != nil {
		return nil, err
	}
	batchEvents := make([]*coleventspb.BatchEvent, len(records))
	for i, record := range records {
		batchEvent, err := p.Produce(NewTraceMessage(record, coleventspb.DeliveryType_BEST_EFFORT))
		if err != nil {
			return nil, err
		}
		batchEvents[i] = batchEvent
	}
	return batchEvents, nil
}

// Produce takes an RecordMessage and returns the corresponding BatchEvent protobuf message.
func (p *Producer) Produce(rm *RecordMessage) (*coleventspb.BatchEvent, error) {
	// Retrieves (or creates) the stream Producer for the sub-stream id defined in the RecordMessage.
	sp := p.streamProducers[rm.subStreamId]
	if sp == nil {
		var buf bytes.Buffer
		sp = &streamProducer{
			output:      buf,
			batchId:     0,
			subStreamId: fmt.Sprintf("%d", len(p.streamProducers)),
		}
		p.streamProducers[rm.subStreamId] = sp
	}

	if sp.ipcWriter == nil {
		sp.ipcWriter = ipc.NewWriter(&sp.output, ipc.WithSchema(rm.record.Schema()))
	}
	err := sp.ipcWriter.Write(rm.record)
	if err != nil {
		return nil, err
	}
	buf := sp.output.Bytes()

	// Reset the buffer
	sp.output.Reset()

	batchId := fmt.Sprintf("%d", sp.batchId)
	sp.batchId++

	return &coleventspb.BatchEvent{
		BatchId:     batchId,
		SubStreamId: sp.subStreamId,
		OtlpArrowPayloads: []*coleventspb.OtlpArrowPayload{
			{
				Type:   rm.recordType,
				Schema: buf,
			},
		},
		DeliveryType: rm.deliveryType,
	}, nil
}
