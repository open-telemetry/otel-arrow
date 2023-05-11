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
	"sort"
	"time"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/ipc"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"

	colarspb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	carrow "github.com/f5/otel-arrow-adapter/pkg/arrow"
	config2 "github.com/f5/otel-arrow-adapter/pkg/config"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	config "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	logsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	metricsarrow "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	pstats "github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	tracesarrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
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
	nextSubStreamId int64
	batchId         int64

	// Builder for each OTEL entities
	metricsBuilder *metricsarrow.MetricsBuilder
	logsBuilder    *logsarrow.LogsBuilder
	tracesBuilder  *tracesarrow.TracesBuilder

	// Record builder for each OTEL entities
	metricsRecordBuilder *builder.RecordBuilderExt
	logsRecordBuilder    *builder.RecordBuilderExt
	tracesRecordBuilder  *builder.RecordBuilderExt

	// General stats for the producer
	stats *pstats.ProducerStats
}

type streamProducer struct {
	output         bytes.Buffer
	ipcWriter      *ipc.Writer
	subStreamId    string
	lastProduction time.Time
	schema         *arrow.Schema
	payloadType    record_message.PayloadType
}

// NewProducer creates a new BatchArrowRecords producer.
//
// The method close MUST be called when the producer is not used anymore to release the memory and avoid memory leaks.
func NewProducer() *Producer {
	return NewProducerWithOptions( /* use default options */ )
}

// NewProducerWithOptions creates a new BatchArrowRecords producer with a set of options.
//
// The method close MUST be called when the producer is not used anymore to release the memory and avoid memory leaks.
func NewProducerWithOptions(options ...config2.Option) *Producer {
	// Default configuration
	cfg := config2.DefaultConfig()
	for _, opt := range options {
		opt(cfg)
	}

	stats := pstats.NewProducerStats()
	if cfg.Stats {
		stats.SchemaStatsEnabled = true
	}

	// Record builders
	metricsRecordBuilder := builder.NewRecordBuilderExt(cfg.Pool, metricsarrow.MetricsSchema, config.NewDictionary(cfg.LimitIndexSize), stats)
	metricsRecordBuilder.SetLabel("metrics")
	logsRecordBuilder := builder.NewRecordBuilderExt(cfg.Pool, logsarrow.Schema, config.NewDictionary(cfg.LimitIndexSize), stats)
	logsRecordBuilder.SetLabel("logs")
	tracesRecordBuilder := builder.NewRecordBuilderExt(cfg.Pool, tracesarrow.Schema, config.NewDictionary(cfg.LimitIndexSize), stats)
	tracesRecordBuilder.SetLabel("traces")

	// Entity builders
	metricsBuilder, err := metricsarrow.NewMetricsBuilder(metricsRecordBuilder, cfg, stats)
	if err != nil {
		panic(err)
	}

	logsBuidler, err := logsarrow.NewLogsBuilder(logsRecordBuilder, cfg, stats)
	if err != nil {
		panic(err)
	}

	tracesBuilder, err := tracesarrow.NewTracesBuilder(tracesRecordBuilder, cfg, stats)
	if err != nil {
		panic(err)
	}

	return &Producer{
		pool:            cfg.Pool,
		zstd:            cfg.Zstd,
		streamProducers: make(map[string]*streamProducer),
		batchId:         0,

		metricsBuilder: metricsBuilder,
		logsBuilder:    logsBuidler,
		tracesBuilder:  tracesBuilder,

		metricsRecordBuilder: metricsRecordBuilder,
		logsRecordBuilder:    logsRecordBuilder,
		tracesRecordBuilder:  tracesRecordBuilder,

		stats: stats,
	}
}

// BatchArrowRecordsFromMetrics produces a BatchArrowRecords message from a [pmetric.Metrics] messages.
func (p *Producer) BatchArrowRecordsFromMetrics(metrics pmetric.Metrics) (*colarspb.BatchArrowRecords, error) {
	// Builds a main Record and n related Records from the metrics passed in
	// parameter. All these Arrow records are wrapped into a BatchArrowRecords
	// and will be released by the Producer.Produce method.
	record, err := recordBuilder[pmetric.Metrics](func() (acommon.EntityBuilder[pmetric.Metrics], error) {
		// Related entity builder must be reset before each use.
		// This is especially important after a schema update.
		p.metricsBuilder.RelatedData().Reset()
		return p.metricsBuilder, nil
	}, metrics)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	// builds the related records (e.g. INT_SUM, INT_GAUGE, INT_GAUGE_ATTRS, ...)
	rms, err := p.metricsBuilder.RelatedData().BuildRecordMessages()
	if err != nil {
		return nil, werror.Wrap(err)
	}

	schemaID := p.metricsRecordBuilder.SchemaID()

	// The main record must be the first one to simplify the decoding
	// in the collector.
	rms = append([]*record_message.RecordMessage{record_message.NewMetricsMessage(schemaID, record)}, rms...)

	bar, err := p.Produce(rms)
	if err != nil {
		return nil, werror.Wrap(err)
	}
	p.stats.MetricsBatchesProduced++
	return bar, nil
}

// BatchArrowRecordsFromLogs produces a BatchArrowRecords message from a [plog.Logs] messages.
func (p *Producer) BatchArrowRecordsFromLogs(ls plog.Logs) (*colarspb.BatchArrowRecords, error) {
	// Builds a main Record and n related Records from the logs passed in
	// parameter. All these Arrow records are wrapped into a BatchArrowRecords
	// and will be released by the Producer.Produce method.
	record, err := recordBuilder[plog.Logs](func() (acommon.EntityBuilder[plog.Logs], error) {
		p.logsBuilder.RelatedData().Reset()
		return p.logsBuilder, nil
	}, ls)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	rms, err := p.logsBuilder.RelatedData().BuildRecordMessages()
	if err != nil {
		return nil, werror.Wrap(err)
	}

	schemaID := p.logsRecordBuilder.SchemaID()
	// The main record must be the first one to simplify the decoding
	// in the collector.
	rms = append([]*record_message.RecordMessage{record_message.NewLogsMessage(schemaID, record)}, rms...)

	bar, err := p.Produce(rms)
	if err != nil {
		return nil, werror.Wrap(err)
	}
	p.stats.LogsBatchesProduced++
	return bar, nil
}

// BatchArrowRecordsFromTraces produces a BatchArrowRecords message from a [ptrace.Traces] messages.
func (p *Producer) BatchArrowRecordsFromTraces(ts ptrace.Traces) (*colarspb.BatchArrowRecords, error) {
	// Builds a main Record and n related Records from the traces passed in
	// parameter. All these Arrow records are wrapped into a BatchArrowRecords
	// and will be released by the Producer.Produce method.
	record, err := recordBuilder[ptrace.Traces](func() (acommon.EntityBuilder[ptrace.Traces], error) {
		p.tracesBuilder.RelatedData().Reset()
		return p.tracesBuilder, nil
	}, ts)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	rms, err := p.tracesBuilder.RelatedData().BuildRecordMessages()
	if err != nil {
		return nil, werror.Wrap(err)
	}

	schemaID := p.tracesRecordBuilder.SchemaID()
	// The main record must be the first one to simplify the decoding
	// in the collector.
	rms = append([]*record_message.RecordMessage{record_message.NewTraceMessage(schemaID, record)}, rms...)

	bar, err := p.Produce(rms)
	if err != nil {
		return nil, werror.Wrap(err)
	}
	p.stats.TracesBatchesProduced++
	return bar, nil
}

// MetricsRecordBuilderExt returns the record builder used to encode metrics.
func (p *Producer) MetricsRecordBuilderExt() *builder.RecordBuilderExt {
	return p.metricsRecordBuilder
}

// LogsRecordBuilderExt returns the record builder used to encode logs.
func (p *Producer) LogsRecordBuilderExt() *builder.RecordBuilderExt {
	return p.logsRecordBuilder
}

// TracesRecordBuilderExt returns the record builder used to encode traces.
func (p *Producer) TracesRecordBuilderExt() *builder.RecordBuilderExt {
	return p.tracesRecordBuilder
}

func (p *Producer) MetricsBuilder() *metricsarrow.MetricsBuilder {
	return p.metricsBuilder
}

func (p *Producer) LogsBuilder() *logsarrow.LogsBuilder {
	return p.logsBuilder
}

func (p *Producer) TracesBuilder() *tracesarrow.TracesBuilder {
	return p.tracesBuilder
}

// Close closes all stream producers.
func (p *Producer) Close() error {
	p.metricsBuilder.Release()
	p.logsBuilder.Release()
	p.tracesBuilder.Release()

	p.metricsRecordBuilder.Release()
	p.logsRecordBuilder.Release()
	p.tracesRecordBuilder.Release()

	for _, sp := range p.streamProducers {
		if err := sp.ipcWriter.Close(); err != nil {
			return werror.Wrap(err)
		}
		p.stats.StreamProducersClosed++
	}
	return nil
}

// GetAndResetStats returns the stats and resets them.
func (p *Producer) GetAndResetStats() pstats.ProducerStats {
	return p.stats.GetAndReset()
}

// Produce takes a slice of RecordMessage and returns the corresponding BatchArrowRecords protobuf message.
func (p *Producer) Produce(rms []*record_message.RecordMessage) (*colarspb.BatchArrowRecords, error) {
	oapl := make([]*colarspb.OtlpArrowPayload, len(rms))

	for i, rm := range rms {
		err := func() error {
			defer func() {
				rm.Record().Release()
			}()

			// Retrieves (or creates) the stream Producer for the sub-stream id defined in the RecordMessage.
			sp := p.streamProducers[rm.SubStreamId()]
			if sp == nil {
				// cleanup previous stream producer if any that have the same
				// PayloadType. The reasoning is that if we have a new
				// sub-stream ID (i.e. schema change) we should no longer use
				// the previous stream producer for this PayloadType as schema
				// changes are only additive.
				// This will release the resources associated with the previous
				// stream producer.
				for ssID, sp := range p.streamProducers {
					if sp.payloadType == rm.PayloadType() {
						if err := sp.ipcWriter.Close(); err != nil {
							return werror.Wrap(err)
						}
						p.stats.StreamProducersClosed++
						delete(p.streamProducers, ssID)
					}
				}

				var buf bytes.Buffer
				sp = &streamProducer{
					output:      buf,
					subStreamId: fmt.Sprintf("%d", p.nextSubStreamId),
					payloadType: rm.PayloadType(),
				}
				p.streamProducers[rm.SubStreamId()] = sp
				p.nextSubStreamId++
				p.stats.StreamProducersCreated++
			}

			sp.lastProduction = time.Now()
			sp.schema = rm.Record().Schema()

			if sp.ipcWriter == nil {
				options := []ipc.Option{
					ipc.WithAllocator(p.pool), // use allocator of the `Producer`
					ipc.WithSchema(rm.Record().Schema()),
					ipc.WithDictionaryDeltas(true), // enable dictionary deltas
				}
				if p.zstd {
					options = append(options, ipc.WithZstd())
				}
				sp.ipcWriter = ipc.NewWriter(&sp.output, options...)
			}
			err := sp.ipcWriter.Write(rm.Record())
			if err != nil {
				return werror.Wrap(err)
			}
			outputBuf := sp.output.Bytes()
			buf := make([]byte, len(outputBuf))
			copy(buf, outputBuf)

			if p.stats.SchemaStatsEnabled {
				// ToDo Create option to display this info
				fmt.Printf("Record %q -> %d bytes\n", rm.PayloadType().String(), len(buf))
			}

			// Reset the buffer
			sp.output.Reset()

			oapl[i] = &colarspb.OtlpArrowPayload{
				SubStreamId: sp.subStreamId,
				Type:        rm.PayloadType(),
				Record:      buf,
			}
			return nil
		}()
		if err != nil {
			return nil, werror.Wrap(err)
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
	type TimeSchema struct {
		time   time.Time
		schema *arrow.Schema
	}

	var schemas []TimeSchema

	println("\n== Producer Stats ==============================================================================")
	p.stats.Show("")

	for _, producer := range p.streamProducers {
		schemas = append(schemas, TimeSchema{time: producer.lastProduction, schema: producer.schema})
	}
	sort.Slice(schemas, func(i, j int) bool {
		return schemas[i].time.Before(schemas[j].time)
	})
	fmt.Printf("\n== Schema (#stream-producers=%d) ============================================================\n", len(schemas))
	for _, s := range schemas {
		fmt.Printf(">> Schema last update at %s:\n", s.time)
		carrow.ShowSchema(s.schema, "  ")
	}
	println("------")
	p.tracesBuilder.ShowSchema()
}

func recordBuilder[T pmetric.Metrics | plog.Logs | ptrace.Traces](builder func() (acommon.EntityBuilder[T], error), entity T) (record arrow.Record, err error) {
	schemaNotUpToDateCount := 0

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
			if record != nil {
				record.Release()
			}

			switch {
			case errors.Is(err, schema.ErrSchemaNotUpToDate):
				schemaNotUpToDateCount++
				if schemaNotUpToDateCount > 5 {
					panic("Too many consecutive schema updates. This shouldn't happen.")
				}
			default:
				return
			}
		} else {
			break
		}
	}
	return record, werror.Wrap(err)
}
