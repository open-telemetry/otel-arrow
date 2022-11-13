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

package arrow_record

import (
	"bytes"
	"fmt"

	"github.com/apache/arrow/go/v11/arrow/ipc"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	logs_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	metrics_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	traces_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
)

// ProducerAPI is the interface of a Producer consdiering all signals.
// This is useful for mock testing.
type ProducerAPI interface {
	BatchArrowRecordsFromTraces(ptrace.Traces) (*colarspb.BatchArrowRecords, error)
	BatchArrowRecordsFromLogs(plog.Logs) (*colarspb.BatchArrowRecords, error)
	// TODO: ...FromMetrics
}

var _ ProducerAPI = &Producer{}

// Producer is a BatchArrowRecords producer.
type Producer struct {
	pool            *memory.GoAllocator
	streamProducers map[string]*streamProducer
	batchId         int64
}

type streamProducer struct {
	output      bytes.Buffer
	ipcWriter   *ipc.Writer
	subStreamId string
}

// NewProducer creates a new BatchArrowRecords producer.
func NewProducer() *Producer {
	return &Producer{
		pool:            memory.NewGoAllocator(),
		streamProducers: make(map[string]*streamProducer),
		batchId:         0,
	}
}

// BatchArrowRecordsFromMetrics produces a BatchArrowRecords message from a [pmetric.Metrics] messages.
func (p *Producer) BatchArrowRecordsFromMetrics(metrics pmetric.Metrics) (*colarspb.BatchArrowRecords, error) {
	mb := metrics_arrow.NewMetricsBuilder(p.pool)
	if err := mb.Append(metrics); err != nil {
		return nil, err
	}
	record, err := mb.Build()
	if err != nil {
		return nil, err
	}
	defer record.Release()

	rms := []*RecordMessage{NewMetricsMessage(record, colarspb.DeliveryType_BEST_EFFORT)}

	bar, err := p.Produce(rms, colarspb.DeliveryType_BEST_EFFORT)
	if err != nil {
		return nil, err
	}
	return bar, nil
}

// BatchArrowRecordsFromLogs produces a BatchArrowRecords message from a [plog.Logs] messages.
func (p *Producer) BatchArrowRecordsFromLogs(ls plog.Logs) (*colarspb.BatchArrowRecords, error) {
	lb := logs_arrow.NewLogsBuilder(p.pool)
	if err := lb.Append(ls); err != nil {
		return nil, err
	}
	record, err := lb.Build()
	if err != nil {
		return nil, err
	}
	defer record.Release()

	rms := []*RecordMessage{NewLogsMessage(record, colarspb.DeliveryType_BEST_EFFORT)}

	bar, err := p.Produce(rms, colarspb.DeliveryType_BEST_EFFORT)
	if err != nil {
		return nil, err
	}
	return bar, nil
}

// BatchArrowRecordsFromTraces produces a BatchArrowRecords message from a [ptrace.Traces] messages.
func (p *Producer) BatchArrowRecordsFromTraces(ts ptrace.Traces) (*colarspb.BatchArrowRecords, error) {
	tb := traces_arrow.NewTracesBuilder(p.pool)
	if err := tb.Append(ts); err != nil {
		return nil, err
	}
	record, err := tb.Build()
	if err != nil {
		return nil, err
	}
	defer record.Release()

	rms := []*RecordMessage{NewTraceMessage(record, colarspb.DeliveryType_BEST_EFFORT)}

	bar, err := p.Produce(rms, colarspb.DeliveryType_BEST_EFFORT)
	if err != nil {
		return nil, err
	}
	return bar, nil
}

// Produce takes a slice of RecordMessage and returns the corresponding BatchArrowRecords protobuf message.
func (p *Producer) Produce(rms []*RecordMessage, deliveryType colarspb.DeliveryType) (*colarspb.BatchArrowRecords, error) {
	oapl := make([]*colarspb.OtlpArrowPayload, len(rms))

	for i, rm := range rms {
		// Retrieves (or creates) the stream Producer for the sub-stream id defined in the RecordMessage.
		sp := p.streamProducers[rm.subStreamId]
		if sp == nil {
			var buf bytes.Buffer
			sp = &streamProducer{
				output:      buf,
				subStreamId: fmt.Sprintf("%d", len(p.streamProducers)),
			}
			p.streamProducers[rm.subStreamId] = sp
		}

		if sp.ipcWriter == nil {
			sp.ipcWriter = ipc.NewWriter(&sp.output, ipc.WithSchema(rm.record.Schema()))
		}
		err := sp.ipcWriter.Write(rm.record)
		rm.record.Release()
		if err != nil {
			return nil, err
		}
		buf := sp.output.Bytes()

		// Reset the buffer
		sp.output.Reset()

		oapl[i] = &colarspb.OtlpArrowPayload{
			SubStreamId: sp.subStreamId,
			Type:        rm.payloadType,
			Record:      buf,
		}
	}

	batchId := fmt.Sprintf("%d", p.batchId)
	p.batchId++

	return &colarspb.BatchArrowRecords{
		BatchId:           batchId,
		OtlpArrowPayloads: oapl,
		DeliveryType:      deliveryType,
	}, nil
}
