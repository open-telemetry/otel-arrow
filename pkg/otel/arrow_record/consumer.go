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
	"context"
	"fmt"
	"math/rand"

	"github.com/apache/arrow/go/v12/arrow/ipc"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"

	colarspb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	common "github.com/open-telemetry/otel-arrow/pkg/otel/common/arrow"
	logsotlp "github.com/open-telemetry/otel-arrow/pkg/otel/logs/otlp"
	metricsotlp "github.com/open-telemetry/otel-arrow/pkg/otel/metrics/otlp"
	"github.com/open-telemetry/otel-arrow/pkg/otel/traces/arrow"
	tracesotlp "github.com/open-telemetry/otel-arrow/pkg/otel/traces/otlp"
	"github.com/open-telemetry/otel-arrow/pkg/record_message"
	"github.com/open-telemetry/otel-arrow/pkg/werror"
)

const defaultMemoryLimit = 70 << 20

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

var ErrConsumerMemoryLimit = fmt.Errorf(
	"The number of decoded records is smaller than the number of received payloads. " +
		"Please increase the memory limit of the consumer.")

// Consumer is a BatchArrowRecords consumer.
type Consumer struct {
	// streamConsumers is a map of reader state by SchemaID.
	streamConsumers map[string]*streamConsumer

	// Config embeds the configurable parameters.
	Config

	// allocator is the one instrumented in calls to Consume,
	// it is reused across multiple IPC readers.
	allocator *common.LimitedAllocator
	// lastInuseValue is the previously-captured value for
	// allocator.Inuse().  This is used to work around a
	// limitation in the OTel synchronous instrument API, which we
	// are using because we reject the use of atomic operations
	// for allocators used here, since there is otherwise no
	// concurrency.  See inuseChangeByCaller() and
	// inuseChangeObserve().
	lastInuseValue uint64

	// counts of the number of records consumed.
	recordsCounter metric.Int64Counter
	// counts of the number of schema resets by data type.
	schemaResetCounter metric.Int64Counter
	// tracks allocator.Inuse()
	memoryCounter metric.Int64UpDownCounter

	// uniqueAttr is set to an 8-byte hex digit string with
	// 32-bits of randomness, applied to all metric events.
	uniqueAttr attribute.KeyValue
}

type Config struct {
	memLimit uint64

	tracesConfig *arrow.Config

	// from component.TelemetrySettings
	meterProvider metric.MeterProvider
}

// WithMemoryLimit configures the Arrow limited memory allocator.
func WithMemoryLimit(bytes uint64) Option {
	return func(cfg *Config) {
		cfg.memLimit = bytes
	}
}

// WithTracesConfig configures trace-specific Arrow encoding options.
func WithTracesConfig(tcfg *arrow.Config) Option {
	return func(cfg *Config) {
		cfg.tracesConfig = tcfg
	}
}

// WithMeterProvider configures an OTel metrics provider.  If none is
// configured, the global meter provider will be used.
func WithMeterProvider(p metric.MeterProvider) Option {
	return func(cfg *Config) {
		cfg.meterProvider = p
	}
}

type streamConsumer struct {
	bufReader   *bytes.Reader
	ipcReader   *ipc.Reader
	payloadType record_message.PayloadType
}

type Option func(*Config)

// NewConsumer creates a new BatchArrowRecords consumer, i.e. a decoder consuming BatchArrowRecords and returning
// the corresponding OTLP representation (pmetric,Metrics, plog.Logs, ptrace.Traces).
func NewConsumer(opts ...Option) *Consumer {
	cfg := Config{
		memLimit:      defaultMemoryLimit,
		tracesConfig:  arrow.DefaultConfig(),
		meterProvider: otel.GetMeterProvider(),
	}
	for _, opt := range opts {
		opt(&cfg)
	}
	meter := cfg.meterProvider.Meter("otel-arrow/pkg/otel/arrow_record")
	allocator := common.NewLimitedAllocator(memory.NewGoAllocator(), cfg.memLimit)
	return &Consumer{
		Config:             cfg,
		allocator:          allocator,
		uniqueAttr:         attribute.String("stream_unique", fmt.Sprintf("%08x", rand.Uint32())),
		streamConsumers:    make(map[string]*streamConsumer),
		recordsCounter:     mustWarn(meter.Int64Counter("arrow_batch_records")),
		schemaResetCounter: mustWarn(meter.Int64Counter("arrow_schema_resets")),
		memoryCounter:      mustWarn(meter.Int64UpDownCounter("arrow_memory_inuse")),
	}
}

func releaseRecords(recs []*record_message.RecordMessage) {
	for _, rec := range recs {
		rec.Record().Release()
	}
}

func mustWarn[T any](t T, err error) T {
	if err != nil {
		// as it's an otel error, let someone else handle it
		otel.Handle(err)
	}
	return t
}

type placeholder struct{}

// inuseChangeByCaller records the change in allocated memory,
// attributing change to the caller.  The `placeholder` returned
// is to encourage this idiom at the top of any function that
// allocates memory in the library.
//
//	defer c.inuseChangeObserve(c.inuseChangeByCaller())
//
// Note: This is a sanity check. Currently there are no allocator
// operations in the caller, and the caller is not responsible
// for releasing memory. The count by where=caller should equal 0.
func (c *Consumer) inuseChangeByCaller() placeholder {
	c.inuseChangeObserveWhere("caller")
	return placeholder{}
}

// inuseChangeObserve records the change in allocated memory,
// attributing change to the library.
//
// Note if we have a memory accounting leak, we expect it will
// show up as a count `arrow_memory_inuse{where=library}`.
func (c *Consumer) inuseChangeObserve(_ placeholder) {
	c.inuseChangeObserveWhere("library")
}

// inuseChangeObserveWhere records synchronous UpDownCounter events
// tracking changes in allocator state.  If OTel had a synchronous
// cumulative updowncounter option, that would be easier to use.
func (c *Consumer) inuseChangeObserveWhere(where string) {
	ctx := context.Background()
	last := c.lastInuseValue
	inuse := c.allocator.Inuse()
	attrs := metric.WithAttributes(c.uniqueAttr, attribute.String("where", where))

	c.memoryCounter.Add(ctx, int64(inuse-last), attrs)
	c.lastInuseValue = inuse
}

// MetricsFrom produces an array of [pmetric.Metrics] from a BatchArrowRecords message.
func (c *Consumer) MetricsFrom(bar *colarspb.BatchArrowRecords) ([]pmetric.Metrics, error) {
	defer c.inuseChangeObserve(c.inuseChangeByCaller())

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
	defer c.inuseChangeObserve(c.inuseChangeByCaller())
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
	defer c.inuseChangeObserve(c.inuseChangeByCaller())
	records, err := c.Consume(bar)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	result := make([]ptrace.Traces, 0, len(records))

	// Compute all related records (i.e. Attributes, Events, and Links)
	relatedData, tracesRecord, err := tracesotlp.RelatedDataFrom(records, c.tracesConfig)

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
	ctx := context.Background()

	var ibes []*record_message.RecordMessage
	defer func() {
		c.recordsCounter.Add(ctx, int64(len(ibes)), metric.WithAttributes(c.uniqueAttr))
	}()

	// Transform each individual OtlpArrowPayload into RecordMessage
	for _, payload := range bar.ArrowPayloads {
		// Retrieves (or creates) the stream consumer for the schema id defined in the BatchArrowRecords message.
		sc := c.streamConsumers[payload.SchemaId]
		if sc == nil {
			// cleanup previous stream consumer if any that have the same
			// PayloadType. The reasoning is that if we have a new
			// schema ID (i.e. schema change) we should no longer use
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
			c.streamConsumers[payload.SchemaId] = sc
		}

		sc.bufReader.Reset(payload.Record)
		if sc.ipcReader == nil {
			c.schemaResetCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("payload_type", payload.Type.String()), c.uniqueAttr))
			ipcReader, err := ipc.NewReader(
				sc.bufReader,
				ipc.WithAllocator(c.allocator),
				ipc.WithDictionaryDeltas(true),
				ipc.WithZstd(),
			)
			if err != nil {
				releaseRecords(ibes)
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

	if len(ibes) < len(bar.ArrowPayloads) {
		releaseRecords(ibes)
		return nil, ErrConsumerMemoryLimit
	}

	return ibes, nil
}

// Close closes the consumer and all its ipc readers.
func (c *Consumer) Close() error {
	for _, sc := range c.streamConsumers {
		if sc.ipcReader != nil {
			sc.ipcReader.Release()
		}
	}
	return nil
}
