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
	"errors"
	"fmt"
	"math"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/ipc"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	logsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	metricsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	tracesarrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
)

// This file implements a generic producer API used to encode BatchArrowRecords messages from
// OTLP entities (i.e. pmetric.Metrics, plog.Logs, ptrace.Traces).
// The producer API is used by the OTLP Arrow exporter.

// ProducerAPI is the interface of a Producer considering all signals.
// This is useful for mock testing.
type ProducerAPI interface {
	BatchArrowRecordsFromTraces(ptrace.Traces) (*colarspb.BatchArrowRecords, error)
	BatchArrowRecordsFromLogs(plog.Logs) (*colarspb.BatchArrowRecords, error)
	BatchArrowRecordsFromMetrics(pmetric.Metrics) (*colarspb.BatchArrowRecords, error)
	Close() error
}

var _ ProducerAPI = &Producer{}

// Producer is a BatchArrowRecords producer.
type Producer struct {
	pool            memory.Allocator // Use a custom memory allocator
	zstd            bool             // Use IPC ZSTD compression
	streamProducers map[string]*streamProducer
	batchId         int64

	// Adaptive schemas for each OTEL entities
	metricsSchema *acommon.AdaptiveSchema
	logsSchema    *acommon.AdaptiveSchema
	tracesSchema  *acommon.AdaptiveSchema

	// Builder for each OTEL entities
	metricsBuilder *metricsarrow.MetricsBuilder
	logsBuilder    *logsarrow.LogsBuilder
	tracesBuilder  *tracesarrow.TracesBuilder
}

type streamProducer struct {
	output      bytes.Buffer
	ipcWriter   *ipc.Writer
	subStreamId string
}

type Config struct {
	pool           memory.Allocator
	initIndexSize  uint64
	limitIndexSize uint64
	zstd           bool // Use IPC ZSTD compression
}

type Option func(*Config)

// NewProducer creates a new BatchArrowRecords producer.
//
// The method close MUST be called when the producer is not used anymore to release the memory and avoid memory leaks.
func NewProducer() *Producer {
	return NewProducerWithOptions( /* use default options */ )
}

// NewProducerWithOptions creates a new BatchArrowRecords producer with a set of options.
//
// The method close MUST be called when the producer is not used anymore to release the memory and avoid memory leaks.
func NewProducerWithOptions(options ...Option) *Producer {
	// Default configuration
	cfg := &Config{
		pool:           memory.NewGoAllocator(),
		initIndexSize:  math.MaxUint16,
		limitIndexSize: math.MaxUint16,
		zstd:           true,
	}
	for _, opt := range options {
		opt(cfg)
	}

	metricsSchema := acommon.NewAdaptiveSchema(
		cfg.pool,
		metricsarrow.Schema,
		acommon.WithDictInitIndexSize(cfg.initIndexSize),
		acommon.WithDictLimitIndexSize(cfg.limitIndexSize))

	logsSchema := acommon.NewAdaptiveSchema(
		cfg.pool,
		logsarrow.Schema,
		acommon.WithDictInitIndexSize(cfg.initIndexSize),
		acommon.WithDictLimitIndexSize(cfg.limitIndexSize))

	tracesSchema := acommon.NewAdaptiveSchema(
		cfg.pool,
		tracesarrow.Schema,
		acommon.WithDictInitIndexSize(cfg.initIndexSize),
		acommon.WithDictLimitIndexSize(cfg.limitIndexSize))

	metricsBuilder, err := metricsarrow.NewMetricsBuilder(metricsSchema)
	if err != nil {
		panic(err)
	}

	logsBuidler, err := logsarrow.NewLogsBuilder(logsSchema)
	if err != nil {
		panic(err)
	}

	tracesBuilder, err := tracesarrow.NewTracesBuilder(tracesSchema)
	if err != nil {
		panic(err)
	}

	return &Producer{
		pool:            cfg.pool,
		zstd:            cfg.zstd,
		streamProducers: make(map[string]*streamProducer),
		batchId:         0,

		metricsSchema: metricsSchema,
		logsSchema:    logsSchema,
		tracesSchema:  tracesSchema,

		metricsBuilder: metricsBuilder,
		logsBuilder:    logsBuidler,
		tracesBuilder:  tracesBuilder,
	}
}

// BatchArrowRecordsFromMetrics produces a BatchArrowRecords message from a [pmetric.Metrics] messages.
func (p *Producer) BatchArrowRecordsFromMetrics(metrics pmetric.Metrics) (*colarspb.BatchArrowRecords, error) {
	// Build the record from the logs passed as parameter
	// Note: The record returned is wrapped into a RecordMessage and will
	// be released by the Producer.Produce method.
	record, err := recordBuilder[pmetric.Metrics](func() (acommon.EntityBuilder[pmetric.Metrics], error) {
		return p.metricsBuilder, nil
	}, metrics)
	if err != nil {
		return nil, err
	}

	schemaID := p.metricsSchema.SchemaID()
	rms := []*RecordMessage{NewMetricsMessage(schemaID, record)}

	bar, err := p.Produce(rms)
	if err != nil {
		return nil, err
	}
	return bar, nil
}

// BatchArrowRecordsFromLogs produces a BatchArrowRecords message from a [plog.Logs] messages.
func (p *Producer) BatchArrowRecordsFromLogs(ls plog.Logs) (*colarspb.BatchArrowRecords, error) {
	// Build the record from the logs passed as parameter
	// Note: The record returned is wrapped into a RecordMessage and will
	// be released by the Producer.Produce method.
	record, err := recordBuilder[plog.Logs](func() (acommon.EntityBuilder[plog.Logs], error) {
		return p.logsBuilder, nil
	}, ls)
	if err != nil {
		return nil, err
	}

	schemaID := p.logsSchema.SchemaID()
	rms := []*RecordMessage{NewLogsMessage(schemaID, record)}

	bar, err := p.Produce(rms)
	if err != nil {
		return nil, err
	}
	return bar, nil
}

// BatchArrowRecordsFromTraces produces a BatchArrowRecords message from a [ptrace.Traces] messages.
func (p *Producer) BatchArrowRecordsFromTraces(ts ptrace.Traces) (*colarspb.BatchArrowRecords, error) {
	// Build the record from the traces passes as parameter
	// Note: The record returned is wrapped into a RecordMessage and will
	// be released by the Producer.Produce method.
	record, err := recordBuilder[ptrace.Traces](func() (acommon.EntityBuilder[ptrace.Traces], error) {
		return p.tracesBuilder, nil
	}, ts)
	if err != nil {
		return nil, err
	}

	schemaID := p.tracesSchema.SchemaID()
	rms := []*RecordMessage{NewTraceMessage(schemaID, record)}

	bar, err := p.Produce(rms)
	if err != nil {
		return nil, err
	}
	return bar, nil
}

// TracesAdaptiveSchema returns the adaptive schema used to encode traces.
func (p *Producer) TracesAdaptiveSchema() *acommon.AdaptiveSchema {
	return p.tracesSchema
}

// LogsAdaptiveSchema returns the adaptive schema used to encode logs.
func (p *Producer) LogsAdaptiveSchema() *acommon.AdaptiveSchema {
	return p.logsSchema
}

// MetricsAdaptiveSchema returns the adaptive schema used to encode metrics.
func (p *Producer) MetricsAdaptiveSchema() *acommon.AdaptiveSchema {
	return p.metricsSchema
}

// Close closes all stream producers.
func (p *Producer) Close() error {
	p.metricsSchema.Release()
	p.logsSchema.Release()
	p.tracesSchema.Release()
	p.metricsBuilder.Release()
	p.logsBuilder.Release()
	p.tracesBuilder.Release()
	for _, sp := range p.streamProducers {
		if err := sp.ipcWriter.Close(); err != nil {
			return err
		}
	}
	return nil
}

// Produce takes a slice of RecordMessage and returns the corresponding BatchArrowRecords protobuf message.
func (p *Producer) Produce(rms []*RecordMessage) (*colarspb.BatchArrowRecords, error) {
	oapl := make([]*colarspb.OtlpArrowPayload, len(rms))

	for i, rm := range rms {
		err := func() error {
			defer rm.record.Release()

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
				options := []ipc.Option{
					ipc.WithAllocator(p.pool), // use allocator of the `Producer`
					ipc.WithSchema(rm.record.Schema()),
					ipc.WithDictionaryDeltas(true), // enable dictionary deltas
				}
				if p.zstd {
					options = append(options, ipc.WithZstd())
				}
				sp.ipcWriter = ipc.NewWriter(&sp.output, options...)
			}
			err := sp.ipcWriter.Write(rm.record)
			if err != nil {
				return err
			}
			buf := sp.output.Bytes()

			// Reset the buffer
			sp.output.Reset()

			oapl[i] = &colarspb.OtlpArrowPayload{
				SubStreamId: sp.subStreamId,
				Type:        rm.payloadType,
				Record:      buf,
			}
			return nil
		}()
		if err != nil {
			return nil, err
		}
	}

	batchId := fmt.Sprintf("%d", p.batchId)
	p.batchId++

	return &colarspb.BatchArrowRecords{
		BatchId:           batchId,
		OtlpArrowPayloads: oapl,
	}, nil
}

func (p *Producer) ShowStats() {
	println("Metrics schema stats:")
	p.metricsSchema.ShowStats()
	println("Logs schema stats:")
	p.logsSchema.ShowStats()
	println("Traces schema stats:")
	p.tracesSchema.ShowStats()
}

func recordBuilder[T pmetric.Metrics | plog.Logs | ptrace.Traces](builder func() (acommon.EntityBuilder[T], error), entity T) (record arrow.Record, err error) {
	dictionaryOverflowCount := 0

	// Build an Arrow Record from an OTEL entity.
	//
	// If a dictionary overflow is observed (see AdaptiveSchema, index type), during
	// the conversion, the record must be build again with an updated schema.
	for {
		var tb acommon.EntityBuilder[T]

		if tb, err = builder(); err != nil {
			return
		}

		if err = tb.Append(entity); err != nil {
			return
		}

		record, err = tb.Build()
		if err != nil {
			var overflowErr *acommon.DictionaryOverflowError

			if record != nil {
				record.Release()
			}

			switch {
			case errors.As(err, &overflowErr):
				dictionaryOverflowCount++
				// 4 is the maximum number of dictionary overflow errors we can handle.
				// uint8 --> uint16
				// uint16 --> uint32
				// uint32 --> uint64
				// uint64 --> string | binary
				if dictionaryOverflowCount > 4 {
					panic("Dictionary overflowed too many times. This shouldn't happen.")
				}
			default:
				return
			}
		} else {
			break
		}
	}
	return record, err
}

// WithAllocator sets the allocator to use for the Producer.
func WithAllocator(allocator memory.Allocator) Option {
	return func(cfg *Config) {
		cfg.pool = allocator
	}
}

// WithNoDictionary sets the Producer to not use dictionary encoding.
func WithNoDictionary() Option {
	return func(cfg *Config) {
		cfg.initIndexSize = 0
		cfg.limitIndexSize = 0
	}
}

// WithUint8InitDictIndex sets the Producer to use an uint8 index for all dictionaries.
func WithUint8InitDictIndex() Option {
	return func(cfg *Config) {
		cfg.initIndexSize = math.MaxUint8
	}
}

// WithUint16InitDictIndex sets the Producer to use an uint16 index for all dictionaries.
func WithUint16InitDictIndex() Option {
	return func(cfg *Config) {
		cfg.initIndexSize = math.MaxUint16
	}
}

// WithUint32LinitDictIndex sets the Producer to use an uint32 index for all dictionaries.
func WithUint32LinitDictIndex() Option {
	return func(cfg *Config) {
		cfg.initIndexSize = math.MaxUint32
	}
}

// WithUint64InitDictIndex sets the Producer to use an uint64 index for all dictionaries.
func WithUint64InitDictIndex() Option {
	return func(cfg *Config) {
		cfg.initIndexSize = math.MaxUint64
	}
}

// WithUint8LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint8 index.
func WithUint8LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.limitIndexSize = math.MaxUint8
	}
}

// WithUint16LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint16 index.
func WithUint16LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.limitIndexSize = math.MaxUint16
	}
}

// WithUint32LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint32 index.
func WithUint32LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.limitIndexSize = math.MaxUint32
	}
}

// WithUint64LimitDictIndex sets the Producer to fall back to non dictionary encoding if the dictionary size exceeds an uint64 index.
func WithUint64LimitDictIndex() Option {
	return func(cfg *Config) {
		cfg.limitIndexSize = math.MaxUint64
	}
}

// WithZstd sets the Producer to use Zstd compression at the Arrow IPC level.
func WithZstd() Option {
	return func(cfg *Config) {
		cfg.zstd = true
	}
}

// WithNoZstd sets the Producer to not use Zstd compression at the Arrow IPC level.
func WithNoZstd() Option {
	return func(cfg *Config) {
		cfg.zstd = false
	}
}
