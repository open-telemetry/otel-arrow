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

	"github.com/apache/arrow/go/v12/arrow/ipc"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	common "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	logsotlp "github.com/f5/otel-arrow-adapter/pkg/otel/logs/otlp"
	metricsotlp "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/otlp"
	tracesotlp "github.com/f5/otel-arrow-adapter/pkg/otel/traces/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// This file implements a generic consumer API used to decode BatchArrowRecords messages into
// their corresponding OTLP representations (i.e. pmetric.Metrics, plog.Logs, ptrace.Traces).
// The consumer API is used by the OTLP Arrow receiver.

// ConsumerAPI is the interface of a Consumer considering all signals.
// This is useful for mock testing.
type ConsumerAPI interface {
	LogsFrom(*colarspb.BatchArrowRecords) ([]plog.Logs, error)
	TracesFrom(*colarspb.BatchArrowRecords) ([]ptrace.Traces, error)
	MetricsFrom(*colarspb.BatchArrowRecords) ([]pmetric.Metrics, error)
	Close() error
}

var _ ConsumerAPI = &Consumer{}

// Consumer is a BatchArrowRecords consumer.
type Consumer struct {
	streamConsumers map[string]*streamConsumer

	memLimit uint64
}

type streamConsumer struct {
	bufReader *bytes.Reader
	ipcReader *ipc.Reader
}

// NewConsumer creates a new BatchArrowRecords consumer, i.e. a decoder consuming BatchArrowRecords and returning
// the corresponding OTLP representation (pmetric,Metrics, plog.Logs, ptrace.Traces).
func NewConsumer() *Consumer {
	return &Consumer{
		streamConsumers: make(map[string]*streamConsumer),

		// TODO: configure this limit with a functional option
		memLimit: 50 << 20,
	}
}

// MetricsFrom produces an array of [pmetric.Metrics] from a BatchArrowRecords message.
func (c *Consumer) MetricsFrom(bar *colarspb.BatchArrowRecords) ([]pmetric.Metrics, error) {
	records, err := c.Consume(bar)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	record2Metrics := func(record *record_message.RecordMessage) (pmetric.Metrics, error) {
		defer record.Record().Release()
		return metricsotlp.MetricsFrom(record.Record())
	}

	result := make([]pmetric.Metrics, 0, len(records))
	for _, record := range records {
		metrics, err := record2Metrics(record)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		result = append(result, metrics)
	}
	return result, nil
}

// LogsFrom produces an array of [plog.Logs] from a BatchArrowRecords message.
func (c *Consumer) LogsFrom(bar *colarspb.BatchArrowRecords) ([]plog.Logs, error) {
	records, err := c.Consume(bar)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	record2Logs := func(record *record_message.RecordMessage) (plog.Logs, error) {
		defer record.Record().Release()
		return logsotlp.LogsFrom(record.Record())
	}

	result := make([]plog.Logs, 0, len(records))
	for _, record := range records {
		logs, err := record2Logs(record)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		result = append(result, logs)
	}
	return result, nil
}

// TracesFrom produces an array of [ptrace.Traces] from a BatchArrowRecords message.
func (c *Consumer) TracesFrom(bar *colarspb.BatchArrowRecords) ([]ptrace.Traces, error) {
	records, err := c.Consume(bar)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	result := make([]ptrace.Traces, 0, len(records))

	// Compute all related records (i.e. Attributes, Events, and Links)
	relatedData, tracesRecord, err := tracesotlp.RelatedDataFrom(records)

	if tracesRecord != nil {
		// Decode OTLP traces from the combination of the main record and the
		// related records.
		traces, err := tracesotlp.TracesFrom(tracesRecord.Record(), relatedData)
		if err != nil {
			return nil, werror.Wrap(err)
		}
		result = append(result, traces)
	}

	return result, nil
}

// Consume takes a BatchArrowRecords protobuf message and returns an array of RecordMessage.
// Note: the records wrapped in the RecordMessage must be released after use by the caller.
func (c *Consumer) Consume(bar *colarspb.BatchArrowRecords) ([]*record_message.RecordMessage, error) {
	var ibes []*record_message.RecordMessage

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
			ipcReader, err := ipc.NewReader(
				sc.bufReader,
				ipc.WithAllocator(common.NewLimitedAllocator(memory.NewGoAllocator(), c.memLimit)),
				ipc.WithDictionaryDeltas(true),
				ipc.WithZstd(),
			)
			if err != nil {
				return nil, werror.Wrap(err)
			}
			sc.ipcReader = ipcReader
		}

		if sc.ipcReader.Next() {
			rec := sc.ipcReader.Record()
			// The record returned by Reader.Record() is owned by the Reader.
			// We need to retain it to be able to use it after the Reader is closed
			// or after the next call to Reader.Next().
			rec.Retain()
			ibes = append(ibes, record_message.NewRecordMessage(bar.BatchId, payload.GetType(), rec))
		}
	}

	return ibes, nil
}

// Close closes the consumer and all its sub-stream ipc readers.
func (c *Consumer) Close() error {
	for _, sc := range c.streamConsumers {
		if sc.ipcReader != nil {
			sc.ipcReader.Release()
		}
	}
	return nil
}
