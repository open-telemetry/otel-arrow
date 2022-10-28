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

package arrow_record

import (
	"bytes"

	"github.com/apache/arrow/go/v9/arrow/ipc"
	"go.opentelemetry.io/collector/pdata/ptrace"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	common_otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	traces_otlp "github.com/f5/otel-arrow-adapter/pkg/otel/traces/otlp"
)

// Consumer is a BatchArrowRecords consumer.
type Consumer struct {
	streamConsumers   map[string]*streamConsumer
	otlpTraceProducer *common_otlp.Producer[ptrace.Traces, ptrace.Span]
}

type streamConsumer struct {
	bufReader *bytes.Reader
	ipcReader *ipc.Reader
}

// NewConsumer creates a new BatchArrowRecords consumer.
func NewConsumer() *Consumer {
	return &Consumer{
		streamConsumers:   make(map[string]*streamConsumer),
		otlpTraceProducer: common_otlp.New[ptrace.Traces, ptrace.Span](traces_otlp.SpansProducer{}),
	}
}

// TracesFrom produces an array of ptrace.Traces from a BatchArrowRecords message.
func (c *Consumer) TracesFrom(bar *colarspb.BatchArrowRecords) ([]ptrace.Traces, error) {
	records, err := c.Consume(bar)
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

// Consume takes a BatchArrowRecords protobuf message and returns an array of RecordMessage.
func (c *Consumer) Consume(bar *colarspb.BatchArrowRecords) ([]*RecordMessage, error) {

	var ibes []*RecordMessage

	// Transform each individual OtlpArrowPayload into RecordMessage
	for _, payload := range bar.OtlpArrowPayloads {
		// Retrieves (or creates) the stream consumer for the sub-stream id defined in the BatchArrowRecords message.
		sc := c.streamConsumers[payload.SubStreamId]
		if sc == nil {
			bufReader := bytes.NewReader([]byte{})
			sc = &streamConsumer{
				bufReader: bufReader,
			}
			c.streamConsumers[payload.SubStreamId] = sc
		}

		sc.bufReader.Reset(payload.Record)
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
				batchId:      bar.BatchId,
				payloadType:  payload.GetType(),
				record:       rec,
				deliveryType: bar.DeliveryType,
			})
		}
	}

	return ibes, nil
}
