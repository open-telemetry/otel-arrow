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

	colarspb "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
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
	bufReader   *bytes.Reader
	ipcReader   *ipc.Reader
	payloadType record_message.PayloadType
}

// NewConsumer creates a new BatchArrowRecords consumer, i.e. a decoder consuming BatchArrowRecords and returning
// the corresponding OTLP representation (pmetric,Metrics, plog.Logs, ptrace.Traces).
func NewConsumer() *Consumer {
	return &Consumer{
		streamConsumers: make(map[string]*streamConsumer),

		// TODO: configure this limit with a functional option
		memLimit: 70 << 20,
	}
}

// MetricsFrom produces an array of [pmetric.Metrics] from a BatchArrowRecords message.
func (c *Consumer) MetricsFrom(bar *colarspb.BatchArrowRecords) ([]pmetric.Metrics, error) {
	// extracts the records from the BatchArrowRecords message
	records, err := c.Consume(bar)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	result := make([]pmetric.Metrics, 0, len(records))

	// builds the related entities (i.e. Attributes, Summaries, Histograms, ...)
	// from the records and returns the main record.
	relatedData, metricsRecord, err := metricsotlp.RelatedDataFrom(records)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	// Process the main record with the related entities.
	if metricsRecord != nil {
		// Decode OTLP metrics from the combination of the main record and the
		// related records.
		metrics, err := metricsotlp.MetricsFrom(metricsRecord.Record(), relatedData)
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

	result := make([]plog.Logs, 0, len(records))

	// Compute all related records (i.e. Attributes)
	relatedData, logsRecord, err := logsotlp.RelatedDataFrom(records)

	if logsRecord != nil {
		// Decode OTLP logs from the combination of the main record and the
		// related records.
		logs, err := logsotlp.LogsFrom(logsRecord.Record(), relatedData)
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
			// cleanup previous stream consumer if any that have the same
			// PayloadType. The reasoning is that if we have a new
			// sub-stream ID (i.e. schema change) we should no longer use
			// the previous stream consumer for this PayloadType as schema
			// changes are only additive.
			// This will release the resources associated with the previous
			// stream consumer.
			for scID, sc := range c.streamConsumers {
				if sc.payloadType == payload.Type {
					sc.ipcReader.Release()
					delete(c.streamConsumers, scID)
				}
			}

			bufReader := bytes.NewReader([]byte{})
			sc = &streamConsumer{
				bufReader:   bufReader,
				payloadType: payload.Type,
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

	if len(ibes) < len(bar.OtlpArrowPayloads) {
		println("Something is wrong! " +
			"The number of decoded records is smaller than the number of received payloads. " +
			"Please consider to increase the memory limit of the consumer.")
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
