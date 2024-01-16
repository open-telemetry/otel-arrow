// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package concurrentbatchprocessor

import (
	"context"
	"errors"
	"fmt"
	"math"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/zap"
	"golang.org/x/sync/semaphore"

	"github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor/testdata"
	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/component/componenttest"
	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/consumer/consumertest"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/exporter/exporterhelper"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/processor"
	"go.opentelemetry.io/collector/processor/processortest"
	"go.opentelemetry.io/otel"
	noopmetric "go.opentelemetry.io/otel/metric/noop"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/sdk/trace/tracetest"
)

func TestProcessorShutdown(t *testing.T) {
	factory := NewFactory()

	ctx := context.Background()
	processorCreationSet := processortest.NewNopCreateSettings()

	for i := 0; i < 5; i++ {
		require.NotPanics(t, func() {
			tProc, err := factory.CreateTracesProcessor(ctx, processorCreationSet, factory.CreateDefaultConfig(), consumertest.NewNop())
			require.NoError(t, err)
			_ = tProc.Shutdown(ctx)
		})

		require.NotPanics(t, func() {
			mProc, err := factory.CreateMetricsProcessor(ctx, processorCreationSet, factory.CreateDefaultConfig(), consumertest.NewNop())
			require.NoError(t, err)
			_ = mProc.Shutdown(ctx)
		})

		require.NotPanics(t, func() {
			lProc, err := factory.CreateLogsProcessor(ctx, processorCreationSet, factory.CreateDefaultConfig(), consumertest.NewNop())
			require.NoError(t, err)
			_ = lProc.Shutdown(ctx)
		})
	}
}

func TestProcessorLifecycle(t *testing.T) {
	factory := NewFactory()

	ctx := context.Background()
	processorCreationSet := processortest.NewNopCreateSettings()

	for i := 0; i < 5; i++ {
		tProc, err := factory.CreateTracesProcessor(ctx, processorCreationSet, factory.CreateDefaultConfig(), consumertest.NewNop())
		require.NoError(t, err)
		require.NoError(t, tProc.Start(ctx, componenttest.NewNopHost()))
		require.NoError(t, tProc.Shutdown(ctx))

		mProc, err := factory.CreateMetricsProcessor(ctx, processorCreationSet, factory.CreateDefaultConfig(), consumertest.NewNop())
		require.NoError(t, err)
		require.NoError(t, mProc.Start(ctx, componenttest.NewNopHost()))
		require.NoError(t, mProc.Shutdown(ctx))

		lProc, err := factory.CreateLogsProcessor(ctx, processorCreationSet, factory.CreateDefaultConfig(), consumertest.NewNop())
		require.NoError(t, err)
		require.NoError(t, lProc.Start(ctx, componenttest.NewNopHost()))
		require.NoError(t, lProc.Shutdown(ctx))
	}
}

type panicConsumer struct {
}

func (pc *panicConsumer) ConsumeTraces(ctx context.Context, td ptrace.Traces) error {
	panic("testing panic")
	return nil
}
func (pc *panicConsumer) ConsumeMetrics(ctx context.Context, md pmetric.Metrics) error {
	panic("testing panic")
	return nil
}
func (pc *panicConsumer) ConsumeLogs(ctx context.Context, ld plog.Logs) error {
	panic("testing panic")
	return nil
}

func (pc *panicConsumer) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}

func TestBatchProcessorSpansPanicRecover(t *testing.T) {
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 128
	cfg.Timeout = 10 * time.Second
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	bp, err := newBatchTracesProcessor(creationSet, &panicConsumer{}, cfg)

	require.NoError(t, err)
	require.NoError(t, bp.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 10
	spansPerRequest := 100
	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())
		// ConsumeTraces is a blocking function and should be run in a go routine
		// until batch size reached to unblock.
		wg.Add(1)
		go func() {
			err = bp.ConsumeTraces(context.Background(), td)
			assert.Contains(t, err.Error(), "testing panic")
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, bp.Shutdown(context.Background()))
}

func TestBatchProcessorMetricsPanicRecover(t *testing.T) {
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 128
	cfg.Timeout = 10 * time.Second
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	bp, err := newBatchMetricsProcessor(creationSet, &panicConsumer{}, cfg)

	require.NoError(t, err)
	require.NoError(t, bp.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 10
	metricsPerRequest := 100
	sentResourceMetrics := pmetric.NewMetrics().ResourceMetrics()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		md := testdata.GenerateMetrics(metricsPerRequest)
		metrics := md.ResourceMetrics().At(0).ScopeMetrics().At(0).Metrics()
		for metricIndex := 0; metricIndex < metricsPerRequest; metricIndex++ {
			metrics.At(metricIndex).SetName(getTestMetricName(requestNum, metricIndex))
		}
		md.ResourceMetrics().At(0).CopyTo(sentResourceMetrics.AppendEmpty())
		wg.Add(1)
		go func() {
			err = bp.ConsumeMetrics(context.Background(), md)
			assert.Contains(t, err.Error(), "testing panic")
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, bp.Shutdown(context.Background()))
}

func TestBatchProcessorLogsPanicRecover(t *testing.T) {
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 128
	cfg.Timeout = 10 * time.Second
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	bp, err := newBatchLogsProcessor(creationSet, &panicConsumer{}, cfg)

	require.NoError(t, err)
	require.NoError(t, bp.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 10
	logsPerRequest := 100
	sentResourceLogs := plog.NewLogs().ResourceLogs()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		ld := testdata.GenerateLogs(logsPerRequest)
		logs := ld.ResourceLogs().At(0).ScopeLogs().At(0).LogRecords()
		for logIndex := 0; logIndex < logsPerRequest; logIndex++ {
			logs.At(logIndex).SetSeverityText(getTestLogSeverityText(requestNum, logIndex))
		}
		ld.ResourceLogs().At(0).CopyTo(sentResourceLogs.AppendEmpty())
		wg.Add(1)
		go func() {
			err = bp.ConsumeLogs(context.Background(), ld)
			assert.Contains(t, err.Error(), "testing panic")
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, bp.Shutdown(context.Background()))
}

type blockingConsumer struct {
	lock             sync.Mutex
	numItems         int
	numBytesAcquired int64
	blocking         chan struct{}
	sem              *semaphore.Weighted
	szr              *ptrace.ProtoMarshaler
}

func (bc *blockingConsumer) getItemsWaiting() int {
	bc.lock.Lock()
	defer bc.lock.Unlock()
	return bc.numItems
}

func (bc *blockingConsumer) ConsumeTraces(ctx context.Context, td ptrace.Traces) error {
	sz := int64(bc.szr.TracesSize(td))
	bc.lock.Lock()

	bc.numItems += td.SpanCount()
	bc.numBytesAcquired += sz

	bc.lock.Unlock()

	bc.sem.Acquire(ctx, sz)
	defer bc.sem.Release(sz)
	<-bc.blocking

	return nil
}

func (bc *blockingConsumer) unblock() {
	bc.lock.Lock()
	defer bc.lock.Unlock()
	close(bc.blocking)
	bc.numItems = 0
}

func (bc *blockingConsumer) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}

// helper function to help determine a setting for cfg.MaxInFlightSizeMiB based
// on the number of requests and number of spans per request.
func calculateMaxInFlightSizeMiB(numRequests, spansPerRequest int) uint32 {
	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	td := testdata.GenerateTraces(spansPerRequest)
	spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
	for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
		spans.At(spanIndex).SetName(getTestSpanName(0, spanIndex))
	}
	td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())

	szr := &ptrace.ProtoMarshaler{}
	singleSz := szr.TracesSize(td)
	numBytes := uint32(singleSz * numRequests)
	numMiB := (numBytes - 1 + 1<<20) >> 20

	return numMiB
}

// This test is meant to confirm that semaphore is still
// released if the client context is canceled.
func TestBatchProcessorCancelContext(t *testing.T) {
	requestCount := 10
	spansPerRequest := 250
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 128
	cfg.Timeout = 10 * time.Second
	cfg.MaxInFlightSizeMiB = calculateMaxInFlightSizeMiB(requestCount, spansPerRequest)
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	bc := &blockingConsumer{
		blocking: make(chan struct{}, 1),
		sem:      semaphore.NewWeighted(int64(cfg.MaxInFlightSizeMiB << 20)),
		szr:      &ptrace.ProtoMarshaler{},
	}
	bp, err := newBatchTracesProcessor(creationSet, bc, cfg)
	require.NoError(t, err)
	require.NoError(t, bp.Start(context.Background(), componenttest.NewNopHost()))

	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	var wg sync.WaitGroup
	ctx, cancel := context.WithCancel(context.Background())
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())
		// ConsumeTraces is a blocking function and should be run in a go routine
		// until batch size reached to unblock.
		wg.Add(1)
		go func() {
			err = bp.ConsumeTraces(ctx, td)
			assert.Contains(t, err.Error(), "context canceled")
			wg.Done()
		}()
	}

	// check all spans arrived in blockingConsumer.
	require.Eventually(t, func() bool {
		numSpans := (requestCount) * spansPerRequest
		return bc.getItemsWaiting() == numSpans
	}, 5*time.Second, 10*time.Millisecond)

	// MaxInFlightSizeMiB is the upperbound on in flight bytes, so calculate
	// how many free bytes the semaphore has.
	excess := int64(cfg.MaxInFlightSizeMiB<<20) - bc.numBytesAcquired
	assert.False(t, bp.sem.TryAcquire(excess+1))

	// cancel context and wait for ConsumeTraces to return.
	cancel()
	wg.Wait()
	assert.False(t, bp.sem.TryAcquire(excess+1))

	// signal to the blockingConsumer to return response to waiters.
	bc.unblock()

	// Semaphore should be released once all responses are returned. Confirm we can acquire MaxInFlightSizeMiB bytes.
	require.Eventually(t, func() bool {
		return bp.sem.TryAcquire(int64(cfg.MaxInFlightSizeMiB << 20))
	}, 5*time.Second, 10*time.Millisecond)
	require.NoError(t, bp.Shutdown(context.Background()))
}

func TestBatchProcessorUnbrokenParentContext(t *testing.T) {
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 100
	cfg.SendBatchMaxSize = 100
	cfg.Timeout = 3 * time.Second
	cfg.MaxInFlightSizeMiB = 2
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	requestCount := 10
	spansPerRequest := 5249
	exp := tracetest.NewInMemoryExporter()
	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exp),
	)
	otel.SetTracerProvider(tp)
	tracer := tp.Tracer("otel")
	bg, rootSp := tracer.Start(context.Background(), "test_start_parent")
	rootSp.End()

	createSet := exporter.CreateSettings{
		TelemetrySettings: component.TelemetrySettings{
			TracerProvider: tp,
			MeterProvider:  noopmetric.MeterProvider{},
			Logger:         zap.NewNop(),
		},
	}

	opt := exporterhelper.WithQueue(exporterhelper.QueueSettings{
		Enabled: false,
	})
	next, err := exporterhelper.NewTracesExporter(bg, createSet, Config{}, func(ctx context.Context, td ptrace.Traces) error { return nil }, opt)
	require.NoError(t, err)

	bp, err := newBatchTracesProcessor(creationSet, next, cfg)
	require.NoError(t, err)
	require.NoError(t, bp.Start(context.Background(), componenttest.NewNopHost()))

	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())
		// ConsumeTraces is a blocking function and should be run in a go routine
		// until batch size reached to unblock.
		wg.Add(1)
		go func() {
			assert.NoError(t, bp.ConsumeTraces(bg, td))
			wg.Done()
		}()
	}
	wg.Wait()

	// need to flush tracerprovider
	tp.ForceFlush(bg)
	td := exp.GetSpans()

	numBatches := float64(spansPerRequest*requestCount) / float64(cfg.SendBatchMaxSize)
	assert.Equal(t, int(math.Ceil(numBatches))+1, len(td))
	for i := range td {
		if !td[i].Parent.HasTraceID() {
			assert.Equal(t, td[i].SpanContext, rootSp.SpanContext())
			continue
		}
		// confirm parent is rootSp
		assert.Equal(t, td[i].Parent, rootSp.SpanContext())
	}

	require.NoError(t, bp.Shutdown(context.Background()))
}

func TestBatchProcessorUnbrokenParentContextMultiple(t *testing.T) {
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 100
	cfg.SendBatchMaxSize = 100
	cfg.Timeout = 3 * time.Second
	cfg.MaxInFlightSizeMiB = 2
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	requestCount := 50
	// keep spansPerRequest small to ensure multiple contexts end up in the same batch.
	spansPerRequest := 5
	exp := tracetest.NewInMemoryExporter()
	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exp),
	)
	otel.SetTracerProvider(tp)
	tracer := tp.Tracer("otel")
	bg, rootSp := tracer.Start(context.Background(), "test_start_parent")
	rootSp.End()

	createSet := exporter.CreateSettings{
		TelemetrySettings: component.TelemetrySettings{
			TracerProvider: tp,
			MeterProvider:  noopmetric.MeterProvider{},
			Logger:         zap.NewNop(),
		},
	}
	opt := exporterhelper.WithQueue(exporterhelper.QueueSettings{
		Enabled: false,
	})
	next, err := exporterhelper.NewTracesExporter(bg, createSet, Config{}, func(ctx context.Context, td ptrace.Traces) error { return nil }, opt)
	require.NoError(t, err)

	bp, err := newBatchTracesProcessor(creationSet, next, cfg)
	require.NoError(t, err)
	require.NoError(t, bp.Start(context.Background(), componenttest.NewNopHost()))

	callCtxs := []context.Context{
		bg,
		client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token1": {"single"},
				"token3": {"n/a"},
			}),
		}),
		client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token1": {"single"},
				"token2": {"one", "two"},
				"token4": {"n/a"},
			}),
		}),
	}

	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		num := requestNum % len(callCtxs)
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())
		// ConsumeTraces is a blocking function and should be run in a go routine
		// until batch size reached to unblock.
		wg.Add(1)
		go func() {
			assert.NoError(t, bp.ConsumeTraces(callCtxs[num], td))
			wg.Done()
		}()
	}
	wg.Wait()

	// need to flush tracerprovider
	tp.ForceFlush(bg)
	td := exp.GetSpans()
	numBatches := float64(spansPerRequest*requestCount) / float64(cfg.SendBatchMaxSize)
	assert.Equal(t, 2*int(math.Ceil(numBatches))+1, len(td))
	for i := range td {
		if len(td[i].Links) != 0 {
			assert.Equal(t, len(td[i].Links), len(callCtxs))
			assert.Equal(t, td[i].Links[0].SpanContext, rootSp.SpanContext())
		}
	}

	require.NoError(t, bp.Shutdown(context.Background()))
}

func TestBatchProcessorSpansDelivered(t *testing.T) {
	sink := new(consumertest.TracesSink)
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 128
	cfg.Timeout = 10 * time.Second
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 1000
	spansPerRequest := 100
	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())
		// ConsumeTraces is a blocking function and should be run in a go routine
		// until batch size reached to unblock.
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
			wg.Done()
		}()
	}

	// Added to test logic that check for empty resources.
	td := ptrace.NewTraces()
	wg.Add(1)
	go func() {
		assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
		wg.Done()
	}()

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*spansPerRequest, sink.SpanCount())
	receivedTraces := sink.AllTraces()
	spansReceivedByName := spansReceivedByName(receivedTraces)
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		spans := sentResourceSpans.At(requestNum).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			require.EqualValues(t,
				spans.At(spanIndex),
				spansReceivedByName[getTestSpanName(requestNum, spanIndex)])
		}
	}
}

func TestBatchProcessorSpansDeliveredEnforceBatchSize(t *testing.T) {
	sink := new(consumertest.TracesSink)
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 128
	cfg.SendBatchMaxSize = 130
	cfg.Timeout = 2 * time.Second
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 1000
	spansPerRequest := 150
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
			wg.Done()
		}()
	}

	// Added to test logic that check for empty resources.
	td := ptrace.NewTraces()
	wg.Add(1)
	go func() {
		require.NoError(t, batcher.ConsumeTraces(context.Background(), td))
		wg.Done()
	}()

	// shutdown will flush any remaining spans
	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*spansPerRequest, sink.SpanCount())
	for i := 0; i < len(sink.AllTraces())-1; i++ {
		assert.Equal(t, int(cfg.SendBatchMaxSize), sink.AllTraces()[i].SpanCount())
	}
	// the last batch has the remaining size
	assert.Equal(t, (requestCount*spansPerRequest)%int(cfg.SendBatchMaxSize), sink.AllTraces()[len(sink.AllTraces())-1].SpanCount())
}

func TestBatchProcessorSentBySize(t *testing.T) {
	telemetryTest(t, testBatchProcessorSentBySize)
}

func testBatchProcessorSentBySize(t *testing.T, tel testTelemetry) {
	sizer := &ptrace.ProtoMarshaler{}
	sink := new(consumertest.TracesSink)
	cfg := createDefaultConfig().(*Config)
	sendBatchSize := 20
	cfg.SendBatchSize = uint32(sendBatchSize)
	cfg.Timeout = 5 * time.Second
	creationSet := tel.NewProcessorCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 100
	spansPerRequest := 5

	start := time.Now()
	sizeSum := 0
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		sizeSum += sizer.TracesSize(td)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	elapsed := time.Since(start)
	require.LessOrEqual(t, elapsed.Nanoseconds(), cfg.Timeout.Nanoseconds())

	expectedBatchesNum := requestCount * spansPerRequest / sendBatchSize
	expectedBatchingFactor := sendBatchSize / spansPerRequest

	require.Equal(t, requestCount*spansPerRequest, sink.SpanCount())
	receivedTraces := sink.AllTraces()
	require.EqualValues(t, expectedBatchesNum, len(receivedTraces))
	for _, td := range receivedTraces {
		rss := td.ResourceSpans()
		require.Equal(t, expectedBatchingFactor, rss.Len())
		for i := 0; i < expectedBatchingFactor; i++ {
			require.Equal(t, spansPerRequest, rss.At(i).ScopeSpans().At(0).Spans().Len())
		}
	}

	tel.assertMetrics(t, expectedMetrics{
		sendCount:        float64(expectedBatchesNum),
		sendSizeSum:      float64(sink.SpanCount()),
		sendSizeBytesSum: float64(sizeSum),
		sizeTrigger:      float64(expectedBatchesNum),
	})
}

func TestBatchProcessorSentBySizeWithMaxSize(t *testing.T) {
	telemetryTest(t, testBatchProcessorSentBySizeWithMaxSize)
}

func testBatchProcessorSentBySizeWithMaxSize(t *testing.T, tel testTelemetry) {
	sink := new(consumertest.TracesSink)
	cfg := createDefaultConfig().(*Config)
	sendBatchSize := 20
	sendBatchMaxSize := 37
	cfg.SendBatchSize = uint32(sendBatchSize)
	cfg.SendBatchMaxSize = uint32(sendBatchMaxSize)
	cfg.Timeout = 5 * time.Second
	creationSet := tel.NewProcessorCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 1
	spansPerRequest := 500
	totalSpans := requestCount * spansPerRequest

	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		// this should be a noerr but need to separate triggerTimeout from triggerShutdown
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	// The max batch size is not a divisor of the total number of spans
	expectedBatchesNum := int(math.Ceil(float64(totalSpans) / float64(sendBatchMaxSize)))

	require.Equal(t, totalSpans, sink.SpanCount())
	receivedTraces := sink.AllTraces()
	require.EqualValues(t, expectedBatchesNum, len(receivedTraces))

	tel.assertMetrics(t, expectedMetrics{
		sendCount:   float64(expectedBatchesNum),
		sendSizeSum: float64(sink.SpanCount()),
		sizeTrigger: math.Floor(float64(totalSpans) / float64(sendBatchMaxSize)),
	})
}

func TestBatchProcessorSentByTimeout(t *testing.T) {
	sink := new(consumertest.TracesSink)
	cfg := createDefaultConfig().(*Config)
	sendBatchSize := 100
	cfg.SendBatchSize = uint32(sendBatchSize)
	cfg.Timeout = 100 * time.Millisecond

	requestCount := 5
	spansPerRequest := 10

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	var wg sync.WaitGroup
	start := time.Now()
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
			wg.Done()
		}()
	}

	wg.Wait()
	elapsed := time.Since(start)
	require.LessOrEqual(t, cfg.Timeout.Nanoseconds(), elapsed.Nanoseconds())
	require.NoError(t, batcher.Shutdown(context.Background()))

	expectedBatchesNum := 1
	expectedBatchingFactor := 5

	require.Equal(t, requestCount*spansPerRequest, sink.SpanCount())
	receivedTraces := sink.AllTraces()
	require.EqualValues(t, expectedBatchesNum, len(receivedTraces))
	for _, td := range receivedTraces {
		rss := td.ResourceSpans()
		require.Equal(t, expectedBatchingFactor, rss.Len())
		for i := 0; i < expectedBatchingFactor; i++ {
			require.Equal(t, spansPerRequest, rss.At(i).ScopeSpans().At(0).Spans().Len())
		}
	}
}

func TestBatchProcessorTraceSendWhenClosing(t *testing.T) {
	cfg := Config{
		Timeout:            3 * time.Second,
		SendBatchSize:      1000,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	sink := new(consumertest.TracesSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	requestCount := 10
	spansPerRequest := 10
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(context.Background(), td))
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*spansPerRequest, sink.SpanCount())
	require.Equal(t, 1, len(sink.AllTraces()))
}

func TestBatchMetricProcessor_ReceivingData(t *testing.T) {
	// Instantiate the batch processor with low config values to test data
	// gets sent through the processor.
	cfg := Config{
		Timeout:            200 * time.Millisecond,
		SendBatchSize:      50,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}

	requestCount := 100
	metricsPerRequest := 5
	sink := new(consumertest.MetricsSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchMetricsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	sentResourceMetrics := pmetric.NewMetrics().ResourceMetrics()

	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		md := testdata.GenerateMetrics(metricsPerRequest)
		metrics := md.ResourceMetrics().At(0).ScopeMetrics().At(0).Metrics()
		for metricIndex := 0; metricIndex < metricsPerRequest; metricIndex++ {
			metrics.At(metricIndex).SetName(getTestMetricName(requestNum, metricIndex))
		}
		md.ResourceMetrics().At(0).CopyTo(sentResourceMetrics.AppendEmpty())
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeMetrics(context.Background(), md))
			wg.Done()
		}()
	}

	// Added to test case with empty resources sent.
	md := pmetric.NewMetrics()
	wg.Add(1)
	go func() {
		assert.NoError(t, batcher.ConsumeMetrics(context.Background(), md))
		wg.Done()
	}()

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*2*metricsPerRequest, sink.DataPointCount())
	receivedMds := sink.AllMetrics()
	metricsReceivedByName := metricsReceivedByName(receivedMds)
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		metrics := sentResourceMetrics.At(requestNum).ScopeMetrics().At(0).Metrics()
		for metricIndex := 0; metricIndex < metricsPerRequest; metricIndex++ {
			require.EqualValues(t,
				metrics.At(metricIndex),
				metricsReceivedByName[getTestMetricName(requestNum, metricIndex)])
		}
	}
}

func TestBatchMetricProcessorBatchSize(t *testing.T) {
	telemetryTest(t, testBatchMetricProcessorBatchSize)
}

func testBatchMetricProcessorBatchSize(t *testing.T, tel testTelemetry) {
	sizer := &pmetric.ProtoMarshaler{}

	// Instantiate the batch processor with low config values to test data
	// gets sent through the processor.
	cfg := Config{
		Timeout:            2 * time.Second,
		SendBatchSize:      50,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}

	requestCount := 100
	metricsPerRequest := 5
	dataPointsPerMetric := 2 // Since the int counter uses two datapoints.
	dataPointsPerRequest := metricsPerRequest * dataPointsPerMetric
	sink := new(consumertest.MetricsSink)

	creationSet := tel.NewProcessorCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchMetricsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	size := 0
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		md := testdata.GenerateMetrics(metricsPerRequest)
		size += sizer.MetricsSize(md)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeMetrics(context.Background(), md))
			wg.Done()
		}()
	}
	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	expectedBatchesNum := requestCount * dataPointsPerRequest / int(cfg.SendBatchSize)
	expectedBatchingFactor := int(cfg.SendBatchSize) / dataPointsPerRequest

	require.Equal(t, requestCount*2*metricsPerRequest, sink.DataPointCount())
	receivedMds := sink.AllMetrics()
	require.Equal(t, expectedBatchesNum, len(receivedMds))
	for _, md := range receivedMds {
		require.Equal(t, expectedBatchingFactor, md.ResourceMetrics().Len())
		for i := 0; i < expectedBatchingFactor; i++ {
			require.Equal(t, metricsPerRequest, md.ResourceMetrics().At(i).ScopeMetrics().At(0).Metrics().Len())
		}
	}

	tel.assertMetrics(t, expectedMetrics{
		sendCount:        float64(expectedBatchesNum),
		sendSizeSum:      float64(sink.DataPointCount()),
		sendSizeBytesSum: float64(size),
		sizeTrigger:      20,
	})
}

func TestBatchMetrics_UnevenBatchMaxSize(t *testing.T) {
	ctx := context.Background()
	sink := new(metricsSink)
	metricsCount := 50
	dataPointsPerMetric := 2
	sendBatchMaxSize := 99

	batchMetrics := newBatchMetrics(sink)
	md := testdata.GenerateMetrics(metricsCount)

	batchMetrics.add(md)
	require.Equal(t, dataPointsPerMetric*metricsCount, batchMetrics.dataPointCount)
	sent, req := batchMetrics.splitBatch(ctx, sendBatchMaxSize, true)
	sendErr := batchMetrics.export(ctx, req)
	require.NoError(t, sendErr)
	require.Equal(t, sendBatchMaxSize, sent)
	remainingDataPointCount := metricsCount*dataPointsPerMetric - sendBatchMaxSize
	require.Equal(t, remainingDataPointCount, batchMetrics.dataPointCount)
}

func TestBatchMetricsProcessor_Timeout(t *testing.T) {
	cfg := Config{
		Timeout:            100 * time.Millisecond,
		SendBatchSize:      101,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	requestCount := 5
	metricsPerRequest := 10
	sink := new(consumertest.MetricsSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchMetricsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	var wg sync.WaitGroup
	start := time.Now()
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		md := testdata.GenerateMetrics(metricsPerRequest)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeMetrics(context.Background(), md))
			wg.Done()
		}()
	}

	wg.Wait()
	elapsed := time.Since(start)
	require.LessOrEqual(t, cfg.Timeout.Nanoseconds(), elapsed.Nanoseconds())
	require.NoError(t, batcher.Shutdown(context.Background()))

	expectedBatchesNum := 1
	expectedBatchingFactor := 5

	require.Equal(t, requestCount*2*metricsPerRequest, sink.DataPointCount())
	receivedMds := sink.AllMetrics()
	require.Equal(t, expectedBatchesNum, len(receivedMds))
	for _, md := range receivedMds {
		require.Equal(t, expectedBatchingFactor, md.ResourceMetrics().Len())
		for i := 0; i < expectedBatchingFactor; i++ {
			require.Equal(t, metricsPerRequest, md.ResourceMetrics().At(i).ScopeMetrics().At(0).Metrics().Len())
		}
	}
}

func TestBatchMetricProcessor_Shutdown(t *testing.T) {
	cfg := Config{
		Timeout:            3 * time.Second,
		SendBatchSize:      1000,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	requestCount := 5
	metricsPerRequest := 10
	sink := new(consumertest.MetricsSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchMetricsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		md := testdata.GenerateMetrics(metricsPerRequest)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeMetrics(context.Background(), md))
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*2*metricsPerRequest, sink.DataPointCount())
	require.Equal(t, 1, len(sink.AllMetrics()))
}

func getTestSpanName(requestNum, index int) string {
	return fmt.Sprintf("test-span-%d-%d", requestNum, index)
}

func spansReceivedByName(tds []ptrace.Traces) map[string]ptrace.Span {
	spansReceivedByName := map[string]ptrace.Span{}
	for i := range tds {
		rss := tds[i].ResourceSpans()
		for i := 0; i < rss.Len(); i++ {
			ilss := rss.At(i).ScopeSpans()
			for j := 0; j < ilss.Len(); j++ {
				spans := ilss.At(j).Spans()
				for k := 0; k < spans.Len(); k++ {
					span := spans.At(k)
					spansReceivedByName[spans.At(k).Name()] = span
				}
			}
		}
	}
	return spansReceivedByName
}

func metricsReceivedByName(mds []pmetric.Metrics) map[string]pmetric.Metric {
	metricsReceivedByName := map[string]pmetric.Metric{}
	for _, md := range mds {
		rms := md.ResourceMetrics()
		for i := 0; i < rms.Len(); i++ {
			ilms := rms.At(i).ScopeMetrics()
			for j := 0; j < ilms.Len(); j++ {
				metrics := ilms.At(j).Metrics()
				for k := 0; k < metrics.Len(); k++ {
					metric := metrics.At(k)
					metricsReceivedByName[metric.Name()] = metric
				}
			}
		}
	}
	return metricsReceivedByName
}

func getTestMetricName(requestNum, index int) string {
	return fmt.Sprintf("test-metric-int-%d-%d", requestNum, index)
}

func BenchmarkTraceSizeBytes(b *testing.B) {
	sizer := &ptrace.ProtoMarshaler{}
	td := testdata.GenerateTraces(8192)
	for n := 0; n < b.N; n++ {
		fmt.Println(sizer.TracesSize(td))
	}
}

func BenchmarkTraceSizeSpanCount(b *testing.B) {
	td := testdata.GenerateTraces(8192)
	for n := 0; n < b.N; n++ {
		td.SpanCount()
	}
}

func BenchmarkBatchMetricProcessor(b *testing.B) {
	b.StopTimer()
	cfg := Config{
		Timeout:            100 * time.Millisecond,
		SendBatchSize:      2000,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	runMetricsProcessorBenchmark(b, cfg)
}

func BenchmarkMultiBatchMetricProcessor(b *testing.B) {
	b.StopTimer()
	cfg := Config{
		Timeout:            100 * time.Millisecond,
		SendBatchSize:      2000,
		MetadataKeys:       []string{"test", "test2"},
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	runMetricsProcessorBenchmark(b, cfg)
}

func runMetricsProcessorBenchmark(b *testing.B, cfg Config) {
	ctx := context.Background()
	sink := new(metricsSink)
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	metricsPerRequest := 1000
	batcher, err := newBatchMetricsProcessor(creationSet, sink, &cfg)
	require.NoError(b, err)
	require.NoError(b, batcher.Start(ctx, componenttest.NewNopHost()))

	b.StartTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			require.NoError(b, batcher.ConsumeMetrics(ctx, testdata.GenerateMetrics(metricsPerRequest)))
		}
	})
	b.StopTimer()
	require.NoError(b, batcher.Shutdown(ctx))
	require.Equal(b, b.N*metricsPerRequest, sink.metricsCount)
}

type metricsSink struct {
	mu           sync.Mutex
	metricsCount int
}

func (sme *metricsSink) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{
		MutatesData: false,
	}
}

func (sme *metricsSink) ConsumeMetrics(_ context.Context, md pmetric.Metrics) error {
	sme.mu.Lock()
	defer sme.mu.Unlock()
	sme.metricsCount += md.MetricCount()
	return nil
}

func TestBatchLogProcessor_ReceivingData(t *testing.T) {
	// Instantiate the batch processor with low config values to test data
	// gets sent through the processor.
	cfg := Config{
		Timeout:            200 * time.Millisecond,
		SendBatchSize:      50,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}

	requestCount := 100
	logsPerRequest := 5
	sink := new(consumertest.LogsSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	sentResourceLogs := plog.NewLogs().ResourceLogs()

	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		ld := testdata.GenerateLogs(logsPerRequest)
		logs := ld.ResourceLogs().At(0).ScopeLogs().At(0).LogRecords()
		for logIndex := 0; logIndex < logsPerRequest; logIndex++ {
			logs.At(logIndex).SetSeverityText(getTestLogSeverityText(requestNum, logIndex))
		}
		ld.ResourceLogs().At(0).CopyTo(sentResourceLogs.AppendEmpty())
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
			wg.Done()
		}()
	}

	// Added to test case with empty resources sent.
	ld := plog.NewLogs()
	wg.Add(1)
	go func() {
		assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
		wg.Done()
	}()

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*logsPerRequest, sink.LogRecordCount())
	receivedMds := sink.AllLogs()
	logsReceivedBySeverityText := logsReceivedBySeverityText(receivedMds)
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		logs := sentResourceLogs.At(requestNum).ScopeLogs().At(0).LogRecords()
		for logIndex := 0; logIndex < logsPerRequest; logIndex++ {
			require.EqualValues(t,
				logs.At(logIndex),
				logsReceivedBySeverityText[getTestLogSeverityText(requestNum, logIndex)])
		}
	}
}

func TestBatchLogProcessor_BatchSize(t *testing.T) {
	telemetryTest(t, testBatchLogProcessorBatchSize)
}

func testBatchLogProcessorBatchSize(t *testing.T, tel testTelemetry) {
	sizer := &plog.ProtoMarshaler{}

	// Instantiate the batch processor with low config values to test data
	// gets sent through the processor.
	cfg := Config{
		Timeout:            2 * time.Second,
		SendBatchSize:      50,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}

	requestCount := 100
	logsPerRequest := 5
	sink := new(consumertest.LogsSink)

	creationSet := tel.NewProcessorCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	size := 0
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		ld := testdata.GenerateLogs(logsPerRequest)
		size += sizer.LogsSize(ld)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
			wg.Done()
		}()
	}
	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	expectedBatchesNum := requestCount * logsPerRequest / int(cfg.SendBatchSize)
	expectedBatchingFactor := int(cfg.SendBatchSize) / logsPerRequest

	require.Equal(t, requestCount*logsPerRequest, sink.LogRecordCount())
	receivedMds := sink.AllLogs()
	require.Equal(t, expectedBatchesNum, len(receivedMds))
	for _, ld := range receivedMds {
		require.Equal(t, expectedBatchingFactor, ld.ResourceLogs().Len())
		for i := 0; i < expectedBatchingFactor; i++ {
			require.Equal(t, logsPerRequest, ld.ResourceLogs().At(i).ScopeLogs().At(0).LogRecords().Len())
		}
	}

	tel.assertMetrics(t, expectedMetrics{
		sendCount:        float64(expectedBatchesNum),
		sendSizeSum:      float64(sink.LogRecordCount()),
		sendSizeBytesSum: float64(size),
		sizeTrigger:      float64(expectedBatchesNum),
	})
}

func TestBatchLogsProcessor_Timeout(t *testing.T) {
	cfg := Config{
		Timeout:            3 * time.Second,
		SendBatchSize:      100,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	requestCount := 5
	logsPerRequest := 10
	sink := new(consumertest.LogsSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	var wg sync.WaitGroup
	start := time.Now()
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		ld := testdata.GenerateLogs(logsPerRequest)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
			wg.Done()
		}()
	}

	wg.Wait()
	elapsed := time.Since(start)
	require.LessOrEqual(t, cfg.Timeout.Nanoseconds(), elapsed.Nanoseconds())
	require.NoError(t, batcher.Shutdown(context.Background()))

	expectedBatchesNum := 1
	expectedBatchingFactor := 5

	require.Equal(t, requestCount*logsPerRequest, sink.LogRecordCount())
	receivedMds := sink.AllLogs()
	require.Equal(t, expectedBatchesNum, len(receivedMds))
	for _, ld := range receivedMds {
		require.Equal(t, expectedBatchingFactor, ld.ResourceLogs().Len())
		for i := 0; i < expectedBatchingFactor; i++ {
			require.Equal(t, logsPerRequest, ld.ResourceLogs().At(i).ScopeLogs().At(0).LogRecords().Len())
		}
	}
}

func TestBatchLogProcessor_Shutdown(t *testing.T) {
	cfg := Config{
		Timeout:            3 * time.Second,
		SendBatchSize:      1000,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}
	requestCount := 5
	logsPerRequest := 10
	sink := new(consumertest.LogsSink)

	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		ld := testdata.GenerateLogs(logsPerRequest)
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	require.Equal(t, requestCount*logsPerRequest, sink.LogRecordCount())
	require.Equal(t, 1, len(sink.AllLogs()))
}

func getTestLogSeverityText(requestNum, index int) string {
	return fmt.Sprintf("test-log-int-%d-%d", requestNum, index)
}

func logsReceivedBySeverityText(lds []plog.Logs) map[string]plog.LogRecord {
	logsReceivedBySeverityText := map[string]plog.LogRecord{}
	for i := range lds {
		ld := lds[i]
		rms := ld.ResourceLogs()
		for i := 0; i < rms.Len(); i++ {
			ilms := rms.At(i).ScopeLogs()
			for j := 0; j < ilms.Len(); j++ {
				logs := ilms.At(j).LogRecords()
				for k := 0; k < logs.Len(); k++ {
					log := logs.At(k)
					logsReceivedBySeverityText[log.SeverityText()] = log
				}
			}
		}
	}
	return logsReceivedBySeverityText
}

func TestShutdown(t *testing.T) {
	factory := NewFactory()
	verifyTracesDoesNotProduceAfterShutdown(t, factory, factory.CreateDefaultConfig())
}
func verifyTracesDoesNotProduceAfterShutdown(t *testing.T, factory processor.Factory, cfg component.Config) {
	// Create a proc and output its produce to a sink.
	nextSink := new(consumertest.TracesSink)
	proc, err := factory.CreateTracesProcessor(context.Background(), processortest.NewNopCreateSettings(), cfg, nextSink)
	if err != nil {
		if errors.Is(err, component.ErrDataTypeIsNotSupported) {
			return
		}
		require.NoError(t, err)
	}
	assert.NoError(t, proc.Start(context.Background(), componenttest.NewNopHost()))

	// Send some traces to the proc.
	const generatedCount = 10
	var wg sync.WaitGroup
	for i := 0; i < generatedCount; i++ {
		wg.Add(1)
		go func() {
			assert.NoError(t, proc.ConsumeTraces(context.Background(), testdata.GenerateTraces(1)))
			wg.Done()
		}()
	}

	// Now shutdown the proc.
	wg.Wait()
	assert.NoError(t, proc.Shutdown(context.Background()))

	// The Shutdown() is done. It means the proc must have sent everything we
	// gave it to the next sink.
	assert.EqualValues(t, generatedCount, nextSink.SpanCount())
}

type metadataTracesSink struct {
	*consumertest.TracesSink

	lock               sync.Mutex
	spanCountByToken12 map[string]int
}

func formatTwo(first, second []string) string {
	return fmt.Sprintf("%s;%s", first, second)
}

func (mts *metadataTracesSink) ConsumeTraces(ctx context.Context, td ptrace.Traces) error {
	info := client.FromContext(ctx)
	token1 := info.Metadata.Get("token1")
	token2 := info.Metadata.Get("token2")
	mts.lock.Lock()
	defer mts.lock.Unlock()

	mts.spanCountByToken12[formatTwo(
		token1,
		token2,
	)] += td.SpanCount()
	return mts.TracesSink.ConsumeTraces(ctx, td)
}

func TestBatchProcessorSpansBatchedByMetadata(t *testing.T) {
	sink := &metadataTracesSink{
		TracesSink:         &consumertest.TracesSink{},
		spanCountByToken12: map[string]int{},
	}
	cfg := createDefaultConfig().(*Config)
	cfg.SendBatchSize = 100
	cfg.Timeout = 1 * time.Second
	cfg.MetadataKeys = []string{"token1", "token2"}
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	bg := context.Background()
	callCtxs := []context.Context{
		client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token1": {"single"},
				"token3": {"n/a"},
			}),
		}),
		client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token1": {"single"},
				"token2": {"one", "two"},
				"token4": {"n/a"},
			}),
		}),
		client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token1": nil,
				"token2": {"single"},
			}),
		}),
		client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token1": {"one", "two", "three"},
				"token2": {"single"},
				"token3": {"n/a"},
				"token4": {"n/a", "d/c"},
			}),
		}),
	}
	expectByContext := make([]int, len(callCtxs))

	requestCount := 1000
	spansPerRequest := 33
	sentResourceSpans := ptrace.NewTraces().ResourceSpans()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		td := testdata.GenerateTraces(spansPerRequest)
		spans := td.ResourceSpans().At(0).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			spans.At(spanIndex).SetName(getTestSpanName(requestNum, spanIndex))
		}
		td.ResourceSpans().At(0).CopyTo(sentResourceSpans.AppendEmpty())
		// use round-robin to assign context.
		num := requestNum % len(callCtxs)
		expectByContext[num] += spansPerRequest
		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(callCtxs[num], td))
			wg.Done()
		}()
	}

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))

	// The following tests are the same as TestBatchProcessorSpansDelivered().
	require.Equal(t, requestCount*spansPerRequest, sink.SpanCount())
	receivedTraces := sink.AllTraces()
	spansReceivedByName := spansReceivedByName(receivedTraces)
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		spans := sentResourceSpans.At(requestNum).ScopeSpans().At(0).Spans()
		for spanIndex := 0; spanIndex < spansPerRequest; spanIndex++ {
			require.EqualValues(t,
				spans.At(spanIndex),
				spansReceivedByName[getTestSpanName(requestNum, spanIndex)])
		}
	}

	// This test ensures each context had the expected number of spans.
	require.Equal(t, len(callCtxs), len(sink.spanCountByToken12))
	for idx, ctx := range callCtxs {
		md := client.FromContext(ctx).Metadata
		exp := formatTwo(md.Get("token1"), md.Get("token2"))
		require.Equal(t, expectByContext[idx], sink.spanCountByToken12[exp])
	}
}

func TestBatchProcessorDuplicateMetadataKeys(t *testing.T) {
	cfg := createDefaultConfig().(*Config)
	cfg.MetadataKeys = []string{"myTOKEN", "mytoken"}
	err := cfg.Validate()
	require.Error(t, err)
	require.Contains(t, err.Error(), "duplicate")
	require.Contains(t, err.Error(), "mytoken")
}

func TestBatchProcessorMetadataCardinalityLimit(t *testing.T) {
	const cardLimit = 10

	sink := new(consumertest.TracesSink)
	cfg := createDefaultConfig().(*Config)
	cfg.MetadataKeys = []string{"token"}
	cfg.MetadataCardinalityLimit = cardLimit
	cfg.Timeout = 1 * time.Second
	creationSet := processortest.NewNopCreateSettings()
	batcher, err := newBatchTracesProcessor(creationSet, sink, cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	bg := context.Background()
	var wg sync.WaitGroup
	for requestNum := 0; requestNum < cardLimit; requestNum++ {
		td := testdata.GenerateTraces(1)
		ctx := client.NewContext(bg, client.Info{
			Metadata: client.NewMetadata(map[string][]string{
				"token": {fmt.Sprint(requestNum)},
			}),
		})

		wg.Add(1)
		go func() {
			assert.NoError(t, batcher.ConsumeTraces(ctx, td))
			wg.Done()
		}()
	}

	wg.Wait()
	td := testdata.GenerateTraces(2)
	ctx := client.NewContext(bg, client.Info{
		Metadata: client.NewMetadata(map[string][]string{
			"token": {"limit_exceeded"},
		}),
	})

	wg.Add(1)
	go func() {
		err := batcher.ConsumeTraces(ctx, td)
		assert.ErrorIs(t, err, errTooManyBatchers)
		wg.Done()
	}()

	wg.Wait()
	require.NoError(t, batcher.Shutdown(context.Background()))
}

func TestBatchZeroConfig(t *testing.T) {
	// This is a no-op configuration. No need for a timer, no
	// minimum, no mxaimum, just a pass through.
	cfg := Config{
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}

	require.NoError(t, cfg.Validate())

	const requestCount = 5
	const logsPerRequest = 10
	sink := new(consumertest.LogsSink)
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	expect := 0
	for requestNum := 0; requestNum < requestCount; requestNum++ {
		cnt := logsPerRequest + requestNum
		expect += cnt
		ld := testdata.GenerateLogs(cnt)
		assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
	}

	// Wait for all batches.
	require.Eventually(t, func() bool {
		return sink.LogRecordCount() == expect
	}, time.Second, 5*time.Millisecond)

	// Expect them to be the original sizes.
	receivedMds := sink.AllLogs()
	require.Equal(t, requestCount, len(receivedMds))
	for i, ld := range receivedMds {
		require.Equal(t, 1, ld.ResourceLogs().Len())
		require.Equal(t, logsPerRequest+i, ld.LogRecordCount())
	}
}

func TestBatchSplitOnly(t *testing.T) {
	const maxBatch = 10
	const requestCount = 5
	const logsPerRequest = 100

	cfg := Config{
		SendBatchMaxSize:   maxBatch,
		MaxInFlightSizeMiB: defaultMaxInFlightSizeMiB,
	}

	require.NoError(t, cfg.Validate())

	sink := new(consumertest.LogsSink)
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	for requestNum := 0; requestNum < requestCount; requestNum++ {
		ld := testdata.GenerateLogs(logsPerRequest)
		assert.NoError(t, batcher.ConsumeLogs(context.Background(), ld))
	}

	// Wait for all batches.
	require.Eventually(t, func() bool {
		return sink.LogRecordCount() == logsPerRequest*requestCount
	}, time.Second, 5*time.Millisecond)

	// Expect them to be the limited by maxBatch.
	receivedMds := sink.AllLogs()
	require.Equal(t, requestCount*logsPerRequest/maxBatch, len(receivedMds))
	for _, ld := range receivedMds {
		require.Equal(t, maxBatch, ld.LogRecordCount())
	}
}

func TestBatchTooLarge(t *testing.T) {
	cfg := Config{
		SendBatchMaxSize:   100000,
		SendBatchSize:      100000,
		MaxInFlightSizeMiB: 1,
	}

	require.NoError(t, cfg.Validate())

	sink := new(consumertest.LogsSink)
	creationSet := processortest.NewNopCreateSettings()
	creationSet.MetricsLevel = configtelemetry.LevelDetailed
	batcher, err := newBatchLogsProcessor(creationSet, sink, &cfg)
	require.NoError(t, err)
	require.NoError(t, batcher.Start(context.Background(), componenttest.NewNopHost()))

	ld := testdata.GenerateLogs(100000)
	err = batcher.ConsumeLogs(context.Background(), ld)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "request size exceeds max-in-flight bytes")
}
