// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package concurrentbatchprocessor // import "github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor"

import (
	"context"
	"errors"
	"fmt"
	"runtime"
	"sort"
	"strings"
	"sync"
	"time"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/zap"
	"golang.org/x/sync/semaphore"

	"github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor/internal/metadata"
	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/consumer/consumererror"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/processor"
)

// errTooManyBatchers is returned when the MetadataCardinalityLimit has been reached.
var errTooManyBatchers = consumererror.NewPermanent(errors.New("too many batcher metadata-value combinations"))

// batch_processor is a component that accepts spans and metrics, places them
// into batches and sends downstream.
//
// batch_processor implements consumer.Traces and consumer.Metrics
//
// Batches are sent out with any of the following conditions:
// - batch size reaches cfg.SendBatchSize
// - cfg.Timeout is elapsed since the timestamp when the previous batch was sent out.
type batchProcessor struct {
	logger           *zap.Logger
	timeout          time.Duration
	sendBatchSize    int
	sendBatchMaxSize int

	// batchFunc is a factory for new batch objects corresponding
	// with the appropriate signal.
	batchFunc func() batch

	// metadataKeys is the configured list of metadata keys.  When
	// empty, the `singleton` batcher is used.  When non-empty,
	// each distinct combination of metadata keys and values
	// triggers a new batcher, counted in `goroutines`.
	metadataKeys []string

	// metadataLimit is the limiting size of the batchers map.
	metadataLimit int

	// sem controls the max_concurrency setting.  this field is nil
	// for unlimited concurrency.
	sem *semaphore.Weighted

	// earlyReturn is the value of Config.EarlyReturn.
	earlyReturn bool

	shutdownC  chan struct{}
	goroutines sync.WaitGroup

	telemetry *batchProcessorTelemetry

	// batcher will be either *singletonBatcher or *multiBatcher
	batcher batcher

	// tracer is the configured tracer
	tracer trace.Tracer
}

// batcher represents a single-shard or multi-shard batching process.
type batcher interface {
	start(ctx context.Context) error
	consume(ctx context.Context, data any) error
	currentMetadataCardinality() int
}

// shard is a single instance of the batch logic.  When metadata
// keys are in use, one of these is created per distinct combination
// of values.
type shard struct {
	// processor refers to this processor, for access to common
	// configuration.
	processor *batchProcessor

	// exportCtx is a context with the metadata key-values
	// corresponding with this shard set.
	exportCtx context.Context

	// timer informs the shard send a batch.
	timer *time.Timer

	// newItem is used to receive data items from producers.
	newItem chan dataItem

	// batch is an in-flight data item containing one of the
	// underlying data types.
	batch batch

	// pending describes the contributors to the current batch.
	pending []pendingItem

	// totalSent counts the number of items processed by the
	// shard in its lifetime.
	totalSent uint64
}

// pendingItem is stored parallel to a pending batch and records
// how many items the waiter submitted, which is used to link
// the incoming and outgoing traces.
type pendingItem struct {
	parentCtx context.Context
	numItems  int

	// respCh is non-nil when the caller is waiting for error
	// transmission.
	respCh chan countedError
}

// dataItem is exchanged between the waiter and the batching process
// includes the pendingItem and its data.
type dataItem struct {
	data any
	pendingItem
}

// batch is an interface generalizing the individual signal types.
type batch interface {
	// export the current batch
	export(ctx context.Context, req any) error

	// splitBatch returns a full request built from pending items.
	splitBatch(ctx context.Context, sendBatchMaxSize int) (sentBatchSize int, req any)

	// itemCount returns the size of the current batch
	itemCount() int

	// add item to the current batch
	add(item any)

	// sizeBytes counts the OTLP encoding size of the batch
	sizeBytes(item any) int
}

// countedError is useful when a producer adds items that are split
// between multiple batches. This signals that producers should continue
// waiting until all its items receive a response.
type countedError struct {
	err   error
	count int
}

func (ce countedError) Error() string {
	if ce.err == nil {
		return ""
	}
	return fmt.Sprintf("batch error: %s", ce.err.Error())
}

func (ce countedError) Unwrap() error {
	return ce.err
}

var _ consumer.Traces = (*batchProcessor)(nil)
var _ consumer.Metrics = (*batchProcessor)(nil)
var _ consumer.Logs = (*batchProcessor)(nil)

// newBatchProcessor returns a new batch processor component.
func newBatchProcessor(set processor.Settings, cfg *Config, batchFunc func() batch) (*batchProcessor, error) {
	// use lower-case, to be consistent with http/2 headers.
	mks := make([]string, len(cfg.MetadataKeys))
	for i, k := range cfg.MetadataKeys {
		mks[i] = strings.ToLower(k)
	}
	sort.Strings(mks)
	bp := &batchProcessor{
		logger: set.Logger,

		sendBatchSize:    int(cfg.SendBatchSize),
		sendBatchMaxSize: int(cfg.SendBatchMaxSize),
		timeout:          cfg.Timeout,
		batchFunc:        batchFunc,
		shutdownC:        make(chan struct{}, 1),
		metadataKeys:     mks,
		metadataLimit:    int(cfg.MetadataCardinalityLimit),
		earlyReturn:      cfg.EarlyReturn,
		tracer:           set.TelemetrySettings.TracerProvider.Tracer(metadata.ScopeName),
	}

	if cfg.MaxConcurrency > 0 {
		bp.sem = semaphore.NewWeighted(int64(cfg.MaxConcurrency))
	}

	asb := anyShardBatcher{processor: bp}
	if len(bp.metadataKeys) == 0 {
		bp.batcher = &singleShardBatcher{anyShardBatcher: asb}
	} else {
		bp.batcher = &multiShardBatcher{anyShardBatcher: asb}
	}

	bpt, err := newBatchProcessorTelemetry(set, bp.batcher.currentMetadataCardinality)
	if err != nil {
		return nil, fmt.Errorf("error creating batch processor telemetry: %w", err)
	}
	bp.telemetry = bpt

	return bp, nil
}

// anyShardBatcher contains common code for single and multi-shard batchers.
type anyShardBatcher struct {
	processor *batchProcessor
}

// newShard gets or creates a batcher corresponding with attrs.
func (bp *batchProcessor) newShard(md map[string][]string) *shard {
	exportCtx := client.NewContext(context.Background(), client.Info{
		Metadata: client.NewMetadata(md),
	})
	b := &shard{
		processor: bp,
		newItem:   make(chan dataItem, runtime.NumCPU()),
		exportCtx: exportCtx,
		batch:     bp.batchFunc(),
	}
	return b
}

func (bp *batchProcessor) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: true}
}

// Start is invoked during service startup.
func (bp *batchProcessor) Start(ctx context.Context, _ component.Host) error {
	return bp.batcher.start(ctx)
}

// Shutdown is invoked during service shutdown.
func (bp *batchProcessor) Shutdown(context.Context) error {
	close(bp.shutdownC)

	// Wait until all goroutines are done.
	bp.goroutines.Wait()
	return nil
}

func (b *shard) start() {
	b.processor.goroutines.Add(1)
	go b.startLoop()
}

func (b *shard) startLoop() {
	defer b.processor.goroutines.Done()

	// timerCh ensures we only block when there is a
	// timer, since <- from a nil channel is blocking.
	var timerCh <-chan time.Time
	if b.processor.timeout != 0 && b.processor.sendBatchSize != 0 {
		b.timer = time.NewTimer(b.processor.timeout)
		timerCh = b.timer.C
	}
	for {
		select {
		case <-b.processor.shutdownC:
		DONE:
			for {
				select {
				case item := <-b.newItem:
					b.processItem(item)
				default:
					break DONE
				}
			}
			// This is the close of the channel
			if b.batch.itemCount() > 0 {
				// TODO: Set a timeout on sendTraces or
				// make it cancellable using the context that Shutdown gets as a parameter
				b.sendItems(triggerTimeout)
			}
			return
		case item := <-b.newItem:
			if item.data == nil {
				continue
			}
			b.processItem(item)
		case <-timerCh:
			if b.batch.itemCount() > 0 {
				b.sendItems(triggerTimeout)
			}
			b.resetTimer()
		}
	}
}

func (b *shard) processItem(item dataItem) {
	before := b.batch.itemCount()
	b.batch.add(item.data)
	after := b.batch.itemCount()

	totalItems := after - before
	b.pending = append(b.pending, pendingItem{
		parentCtx: item.parentCtx,
		numItems:  totalItems,
		respCh:    item.respCh,
	})

	b.flushItems()
}

func (b *shard) flushItems() {
	sent := false
	for b.batch.itemCount() > 0 && (!b.hasTimer() || b.batch.itemCount() >= b.processor.sendBatchSize) {
		b.sendItems(triggerBatchSize)
		sent = true
	}

	if sent {
		b.stopTimer()
		b.resetTimer()
	}
}

func (b *shard) hasTimer() bool {
	return b.timer != nil
}

func (b *shard) stopTimer() {
	if b.hasTimer() && !b.timer.Stop() {
		<-b.timer.C
	}
}

func (b *shard) resetTimer() {
	if b.hasTimer() {
		b.timer.Reset(b.processor.timeout)
	}
}

func (b *shard) sendItems(trigger trigger) {
	sent, req := b.batch.splitBatch(b.exportCtx, b.processor.sendBatchMaxSize)

	var thisBatch []pendingTuple

	numItemsBefore := b.totalSent
	numItemsAfter := b.totalSent + uint64(sent)

	// The current batch can contain items from several different producers. Ensure each producer gets a response back.
	for len(b.pending) > 0 && numItemsBefore < numItemsAfter {
		if numItemsBefore+uint64(b.pending[0].numItems) > numItemsAfter {
			// Waiter only had some items in the current batch
			partialSent := int(numItemsAfter - numItemsBefore)
			numItemsBefore = numItemsAfter
			b.pending[0].numItems -= partialSent
			thisBatch = append(thisBatch, pendingTuple{
				waiter: b.pending[0].respCh,
				count:  partialSent,
				ctx:    b.pending[0].parentCtx,
			})
		} else {
			// waiter gets a complete response.
			numItemsBefore += uint64(b.pending[0].numItems)
			thisBatch = append(thisBatch, pendingTuple{
				waiter: b.pending[0].respCh,
				count:  b.pending[0].numItems,
				ctx:    b.pending[0].parentCtx,
			})

			// Shift the pending array, to allow it to be re-used
			// instead of re-allocated.
			copy(b.pending[0:len(b.pending)-1], b.pending[1:])
			b.pending[len(b.pending)-1] = pendingItem{}
			b.pending = b.pending[:len(b.pending)-1]
		}
	}

	b.totalSent = numItemsAfter

	if b.processor.sem != nil {
		b.processor.sem.Acquire(context.Background(), 1)
	}
	b.processor.goroutines.Add(1)
	go func() {
		if b.processor.sem != nil {
			defer b.processor.sem.Release(1)
		}
		defer b.processor.goroutines.Done()

		var err error

		var parentSpan trace.Span
		var parent context.Context
		isSingleCtx := allSameContext(thisBatch)

		// If incoming requests are sufficiently large, there
		// will be one context, in which case no need to create a new
		// root span.
		if isSingleCtx {
			parent = thisBatch[0].ctx
			parent, parentSpan = b.processor.tracer.Start(parent, "batch_processor/export")
		} else {
			spans := parentSpans(thisBatch)

			links := make([]trace.Link, len(spans))
			for i, span := range spans {
				links[i] = trace.Link{SpanContext: span.SpanContext()}
			}
			parent, parentSpan = b.processor.tracer.Start(b.exportCtx, "batch_processor/export", trace.WithLinks(links...))

			// Note: linking in the opposite direction.
			// This could be inferred by the trace
			// backend, but this adds helpful information
			// in cases where sampling may break links.
			// See https://github.com/open-telemetry/opentelemetry-specification/issues/1877
			for _, span := range spans {
				span.AddLink(trace.Link{SpanContext: parentSpan.SpanContext()})
			}
		}
		err = b.batch.export(parent, req)
		// Note: call End() before returning to caller contexts, otherwise
		// trace-based tests will not recognize unfinished spans when the test
		// terminates.
		parentSpan.End()

		if !b.processor.earlyReturn {
			for _, pending := range thisBatch {
				select {
				case pending.waiter <- countedError{err: err, count: pending.count}:
					// OK! Caller received error and count.
				case <-pending.ctx.Done():
					// OK! Caller context was canceled.
				}
			}
		}

		if err != nil {
			b.processor.logger.Warn("Sender failed", zap.Error(err))
		} else {
			var bytes int
			bpt := b.processor.telemetry

			// Check if the instrument is enabled to calculate the size of the batch in bytes.
			// See https://pkg.go.dev/go.opentelemetry.io/otel/sdk/metric/internal/x#readme-instrument-enabled
			batchSendSizeBytes := bpt.telemetryBuilder.ProcessorBatchBatchSendSizeBytes
			instr, ok := batchSendSizeBytes.(interface{ Enabled(context.Context) bool })
			if !ok || instr.Enabled(bpt.exportCtx) {
				bytes = b.batch.sizeBytes(req)
			}

			bpt.record(trigger, int64(sent), int64(bytes))
		}
	}()
}

func parentSpans(x []pendingTuple) []trace.Span {
	var spans []trace.Span
	unique := make(map[context.Context]bool)
	for i := range x {
		_, ok := unique[x[i].ctx]
		if ok {
			continue
		}

		unique[x[i].ctx] = true

		spans = append(spans, trace.SpanFromContext(x[i].ctx))
	}

	return spans
}

type pendingTuple struct {
	waiter chan countedError
	count  int
	ctx    context.Context
}

// allSameContext is a helper function to check if a slice of contexts
// contains more than one unique context.
func allSameContext(x []pendingTuple) bool {
	for idx := range x[1:] {
		if x[idx].ctx != x[0].ctx {
			return false
		}
	}
	return true
}

func (b *shard) consumeBatch(ctx context.Context, data any) error {
	var itemCount int
	switch telem := data.(type) {
	case ptrace.Traces:
		itemCount = telem.SpanCount()
	case pmetric.Metrics:
		itemCount = telem.DataPointCount()
	case plog.Logs:
		itemCount = telem.LogRecordCount()
	}

	if itemCount == 0 {
		return nil
	}

	var respCh chan countedError
	if !b.processor.earlyReturn {
		respCh = make(chan countedError, 1)
	}

	item := dataItem{
		data: data,
		pendingItem: pendingItem{
			parentCtx: ctx,
			respCh:    respCh,
			numItems:  itemCount,
		},
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case b.newItem <- item:
		if b.processor.earlyReturn {
			return nil
		}
	}

	return b.waitForItems(ctx, item.numItems, respCh)
}

func (b *shard) waitForItems(ctx context.Context, numItems int, respCh chan countedError) error {
	var err error
	for {
		select {
		case cntErr := <-respCh:
			// nil response might be wrapped as an error.
			if cntErr.err != nil {
				err = errors.Join(err, cntErr)
			}

			numItems -= cntErr.count
			if numItems != 0 {
				continue
			}

			return err
		case <-ctx.Done():
			return errors.Join(err, ctx.Err())
		}
	}
}

// singleShardBatcher is used when metadataKeys is empty, to avoid the
// additional lock and map operations used in multiBatcher.
type singleShardBatcher struct {
	anyShardBatcher
	batcher *shard
}

func (sb *singleShardBatcher) consume(ctx context.Context, data any) error {
	return sb.batcher.consumeBatch(ctx, data)
}

func (sb *singleShardBatcher) currentMetadataCardinality() int {
	return 1
}

func (sb *singleShardBatcher) start(context.Context) error {
	sb.batcher = sb.processor.newShard(nil)
	sb.batcher.start()
	return nil
}

// multiBatcher is used when metadataKeys is not empty.
type multiShardBatcher struct {
	anyShardBatcher
	batchers sync.Map

	// Guards the size and the storing logic to ensure no more than limit items are stored.
	// If we are willing to allow "some" extra items than the limit this can be removed and size can be made atomic.
	lock sync.Mutex
	size int
}

func (sb *multiShardBatcher) start(context.Context) error {
	return nil
}

func (sb *multiShardBatcher) consume(ctx context.Context, data any) error {
	// Get each metadata key value, form the corresponding
	// attribute set for use as a map lookup key.
	info := client.FromContext(ctx)
	md := map[string][]string{}
	var attrs []attribute.KeyValue
	for _, k := range sb.processor.metadataKeys {
		// Lookup the value in the incoming metadata, copy it
		// into the outgoing metadata, and create a unique
		// value for the attributeSet.
		vs := info.Metadata.Get(k)
		md[k] = vs
		if len(vs) == 1 {
			attrs = append(attrs, attribute.String(k, vs[0]))
		} else {
			attrs = append(attrs, attribute.StringSlice(k, vs))
		}
	}
	aset := attribute.NewSet(attrs...)

	b, ok := sb.batchers.Load(aset)
	if !ok {
		sb.lock.Lock()
		if sb.processor.metadataLimit != 0 && sb.size >= sb.processor.metadataLimit {
			sb.lock.Unlock()
			return errTooManyBatchers
		}

		// aset.ToSlice() returns the sorted, deduplicated,
		// and name-downcased list of attributes.
		var loaded bool
		b, loaded = sb.batchers.LoadOrStore(aset, sb.processor.newShard(md))

		if !loaded {
			// This is a new shard
			sb.size++
			b.(*shard).start()

		}
		sb.lock.Unlock()
	}

	return b.(*shard).consumeBatch(ctx, data)
}

func (sb *multiShardBatcher) currentMetadataCardinality() int {
	sb.lock.Lock()
	defer sb.lock.Unlock()
	return sb.size
}

// ConsumeTraces implements TracesProcessor
func (bp *batchProcessor) ConsumeTraces(ctx context.Context, td ptrace.Traces) error {
	return bp.batcher.consume(ctx, td)
}

// ConsumeMetrics implements MetricsProcessor
func (bp *batchProcessor) ConsumeMetrics(ctx context.Context, md pmetric.Metrics) error {
	return bp.batcher.consume(ctx, md)
}

// ConsumeLogs implements LogsProcessor
func (bp *batchProcessor) ConsumeLogs(ctx context.Context, ld plog.Logs) error {
	return bp.batcher.consume(ctx, ld)
}

// newBatchTracesProcessor creates a new batch processor that batches traces by size or with timeout
func newBatchTracesProcessor(set processor.Settings, next consumer.Traces, cfg *Config) (*batchProcessor, error) {
	return newBatchProcessor(set, cfg, func() batch { return newBatchTraces(next) })
}

// newBatchMetricsProcessor creates a new batch processor that batches metrics by size or with timeout
func newBatchMetricsProcessor(set processor.Settings, next consumer.Metrics, cfg *Config) (*batchProcessor, error) {
	return newBatchProcessor(set, cfg, func() batch { return newBatchMetrics(next) })
}

// newBatchLogsProcessor creates a new batch processor that batches logs by size or with timeout
func newBatchLogsProcessor(set processor.Settings, next consumer.Logs, cfg *Config) (*batchProcessor, error) {
	return newBatchProcessor(set, cfg, func() batch { return newBatchLogs(next) })
}

type batchTraces struct {
	nextConsumer consumer.Traces
	traceData    ptrace.Traces
	spanCount    int
	sizer        ptrace.Sizer
}

func newBatchTraces(nextConsumer consumer.Traces) *batchTraces {
	return &batchTraces{nextConsumer: nextConsumer, traceData: ptrace.NewTraces(), sizer: &ptrace.ProtoMarshaler{}}
}

// add updates current batchTraces by adding new TraceData object
func (bt *batchTraces) add(item any) {
	td := item.(ptrace.Traces)

	newSpanCount := td.SpanCount()
	if newSpanCount == 0 {
		return
	}

	bt.spanCount += newSpanCount
	td.ResourceSpans().MoveAndAppendTo(bt.traceData.ResourceSpans())
}

func (bt *batchTraces) sizeBytes(data any) int {
	return bt.sizer.TracesSize(data.(ptrace.Traces))
}

func (bt *batchTraces) export(ctx context.Context, req any) error {
	td := req.(ptrace.Traces)
	return bt.nextConsumer.ConsumeTraces(ctx, td)
}

func (bt *batchTraces) splitBatch(_ context.Context, sendBatchMaxSize int) (int, any) {
	var req ptrace.Traces
	var sent int
	if sendBatchMaxSize > 0 && bt.itemCount() > sendBatchMaxSize {
		req = splitTraces(sendBatchMaxSize, bt.traceData)
		bt.spanCount -= sendBatchMaxSize
		sent = sendBatchMaxSize
	} else {
		req = bt.traceData
		sent = bt.spanCount
		bt.traceData = ptrace.NewTraces()
		bt.spanCount = 0
	}
	return sent, req
}

func (bt *batchTraces) itemCount() int {
	return bt.spanCount
}

type batchMetrics struct {
	nextConsumer   consumer.Metrics
	metricData     pmetric.Metrics
	dataPointCount int
	sizer          pmetric.Sizer
}

func newBatchMetrics(nextConsumer consumer.Metrics) *batchMetrics {
	return &batchMetrics{nextConsumer: nextConsumer, metricData: pmetric.NewMetrics(), sizer: &pmetric.ProtoMarshaler{}}
}

func (bm *batchMetrics) sizeBytes(data any) int {
	return bm.sizer.MetricsSize(data.(pmetric.Metrics))
}

func (bm *batchMetrics) export(ctx context.Context, req any) error {
	md := req.(pmetric.Metrics)
	return bm.nextConsumer.ConsumeMetrics(ctx, md)
}

func (bm *batchMetrics) splitBatch(_ context.Context, sendBatchMaxSize int) (int, any) {
	var req pmetric.Metrics
	var sent int
	if sendBatchMaxSize > 0 && bm.dataPointCount > sendBatchMaxSize {
		req = splitMetrics(sendBatchMaxSize, bm.metricData)
		bm.dataPointCount -= sendBatchMaxSize
		sent = sendBatchMaxSize
	} else {
		req = bm.metricData
		sent = bm.dataPointCount
		bm.metricData = pmetric.NewMetrics()
		bm.dataPointCount = 0
	}

	return sent, req
}

func (bm *batchMetrics) itemCount() int {
	return bm.dataPointCount
}

func (bm *batchMetrics) add(item any) {
	md := item.(pmetric.Metrics)

	newDataPointCount := md.DataPointCount()
	if newDataPointCount == 0 {
		return
	}
	bm.dataPointCount += newDataPointCount
	md.ResourceMetrics().MoveAndAppendTo(bm.metricData.ResourceMetrics())
}

type batchLogs struct {
	nextConsumer consumer.Logs
	logData      plog.Logs
	logCount     int
	sizer        plog.Sizer
}

func newBatchLogs(nextConsumer consumer.Logs) *batchLogs {
	return &batchLogs{nextConsumer: nextConsumer, logData: plog.NewLogs(), sizer: &plog.ProtoMarshaler{}}
}

func (bl *batchLogs) sizeBytes(data any) int {
	return bl.sizer.LogsSize(data.(plog.Logs))
}

func (bl *batchLogs) export(ctx context.Context, req any) error {
	ld := req.(plog.Logs)
	return bl.nextConsumer.ConsumeLogs(ctx, ld)
}

func (bl *batchLogs) splitBatch(_ context.Context, sendBatchMaxSize int) (int, any) {
	var req plog.Logs
	var sent int

	if sendBatchMaxSize > 0 && bl.logCount > sendBatchMaxSize {
		req = splitLogs(sendBatchMaxSize, bl.logData)
		bl.logCount -= sendBatchMaxSize
		sent = sendBatchMaxSize
	} else {
		req = bl.logData
		sent = bl.logCount
		bl.logData = plog.NewLogs()
		bl.logCount = 0
	}
	return sent, req
}

func (bl *batchLogs) itemCount() int {
	return bl.logCount
}

func (bl *batchLogs) add(item any) {
	ld := item.(plog.Logs)

	newLogsCount := ld.LogRecordCount()
	if newLogsCount == 0 {
		return
	}
	bl.logCount += newLogsCount
	ld.ResourceLogs().MoveAndAppendTo(bl.logData.ResourceLogs())
}
