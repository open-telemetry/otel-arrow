// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver/internal/arrow"

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"runtime"
	"strconv"
	"strings"
	"sync"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/collector/netstats"
	arrowRecord "github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/consumer/consumererror"
	"go.opentelemetry.io/collector/extension/auth"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/receiver"
	"go.opentelemetry.io/collector/receiver/receiverhelper"
	"go.opentelemetry.io/otel"
	otelcodes "go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/multierr"
	"go.uber.org/zap"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	"github.com/open-telemetry/otel-arrow/collector/admission"
)

const (
	streamFormat        = "arrow"
	hpackMaxDynamicSize = 4096
	scopeName           = "github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver"
)

var (
	ErrNoMetricsConsumer   = fmt.Errorf("no metrics consumer")
	ErrNoLogsConsumer      = fmt.Errorf("no logs consumer")
	ErrNoTracesConsumer    = fmt.Errorf("no traces consumer")
	ErrUnrecognizedPayload = fmt.Errorf("unrecognized OTel-Arrow payload")
)

type Consumers interface {
	Traces() consumer.Traces
	Metrics() consumer.Metrics
	Logs() consumer.Logs
}

type Receiver struct {
	Consumers

	arrowpb.UnsafeArrowTracesServiceServer
	arrowpb.UnsafeArrowLogsServiceServer
	arrowpb.UnsafeArrowMetricsServiceServer

	telemetry            component.TelemetrySettings
	tracer               trace.Tracer
	obsrecv              *receiverhelper.ObsReport
	gsettings            configgrpc.ServerConfig
	authServer           auth.Server
	newConsumer          func() arrowRecord.ConsumerAPI
	netReporter          netstats.Interface
	recvInFlightBytes    metric.Int64UpDownCounter
	recvInFlightItems    metric.Int64UpDownCounter
	recvInFlightRequests metric.Int64UpDownCounter
	boundedQueue         *admission.BoundedQueue
	inFlightWG           sync.WaitGroup
}

// New creates a new Receiver reference.
func New(
	cs Consumers,
	set receiver.CreateSettings,
	obsrecv *receiverhelper.ObsReport,
	gsettings configgrpc.ServerConfig,
	authServer auth.Server,
	newConsumer func() arrowRecord.ConsumerAPI,
	bq *admission.BoundedQueue,
	netReporter netstats.Interface,
) (*Receiver, error) {
	tracer := set.TelemetrySettings.TracerProvider.Tracer("otel-arrow-receiver")
	var errors, err error
	recv := &Receiver{
		Consumers:    cs,
		obsrecv:      obsrecv,
		telemetry:    set.TelemetrySettings,
		tracer:       tracer,
		authServer:   authServer,
		newConsumer:  newConsumer,
		gsettings:    gsettings,
		netReporter:  netReporter,
		boundedQueue: bq,
	}

	meter := recv.telemetry.MeterProvider.Meter(scopeName)
	recv.recvInFlightBytes, err = meter.Int64UpDownCounter(
		"otel_arrow_receiver_in_flight_bytes",
		metric.WithDescription("Number of bytes in flight"),
		metric.WithUnit("By"),
	)
	errors = multierr.Append(errors, err)

	recv.recvInFlightItems, err = meter.Int64UpDownCounter(
		"otel_arrow_receiver_in_flight_items",
		metric.WithDescription("Number of items in flight"),
	)
	errors = multierr.Append(errors, err)

	recv.recvInFlightRequests, err = meter.Int64UpDownCounter(
		"otel_arrow_receiver_in_flight_requests",
		metric.WithDescription("Number of requests in flight"),
	)
	errors = multierr.Append(errors, err)

	if errors != nil {
		return nil, errors
	}

	return recv, nil
}

// headerReceiver contains the state necessary to decode per-request metadata
// from an arrow stream.
type headerReceiver struct {
	// decoder maintains state across the stream.
	decoder *hpack.Decoder

	// includeMetadata as configured by gRPC settings.
	includeMetadata bool

	// hasAuthServer indicates that headers must be produced
	// independent of includeMetadata.
	hasAuthServer bool

	// client connection info from the stream context, (optionally
	// if includeMetadata) to be extended with per-request metadata.
	connInfo client.Info

	// streamHdrs was translated from the incoming context, will be
	// merged with per-request metadata.  Note that the contents of
	// this map are equivalent to connInfo.Metadata, however that
	// library does not let us iterate over the map so we recalculate
	// this from the gRPC incoming stream context.
	streamHdrs map[string][]string

	// tmpHdrs is used by the decoder's emit function during Write.
	tmpHdrs map[string][]string
}

func newHeaderReceiver(streamCtx context.Context, as auth.Server, includeMetadata bool) *headerReceiver {
	hr := &headerReceiver{
		includeMetadata: includeMetadata,
		hasAuthServer:   as != nil,
		connInfo:        client.FromContext(streamCtx),
	}

	// Note that we capture the incoming context if there is an
	// Auth plugin configured or includeMetadata is set.
	if hr.includeMetadata || hr.hasAuthServer {
		if smd, ok := metadata.FromIncomingContext(streamCtx); ok {
			hr.streamHdrs = smd
		}
	}

	// Note the hpack decoder supports additional protections,
	// such as SetMaxStringLength(), but as we already have limits
	// on stream request size, this seems unnecessary.
	hr.decoder = hpack.NewDecoder(hpackMaxDynamicSize, hr.tmpHdrsAppend)

	return hr
}

// combineHeaders calculates per-request Metadata by combining the stream's
// client.Info with additional key:values associated with the arrow batch.
func (h *headerReceiver) combineHeaders(ctx context.Context, hdrsBytes []byte) (context.Context, map[string][]string, error) {
	if len(hdrsBytes) == 0 && len(h.streamHdrs) == 0 {
		return ctx, nil, nil
	}

	if len(hdrsBytes) == 0 {
		return h.newContext(ctx, h.streamHdrs), h.streamHdrs, nil
	}

	// Note that we will parse the headers even if they are not
	// used, to check for validity and/or trace context.  Also
	// note this code was once optimized to avoid the following
	// map allocation in cases where the return value would not be
	// used.  This logic was "is metadata present" or "is auth
	// server used".  Then we added to this, "is trace propagation
	// in use" and simplified this function to always store the
	// headers into a temporary map.
	h.tmpHdrs = map[string][]string{}

	// Write calls the emitFunc, appending directly into `tmpHdrs`.
	if _, err := h.decoder.Write(hdrsBytes); err != nil {
		return ctx, nil, err
	}

	// Get the global propagator, to extract context.  When there
	// are no fields, it's a no-op propagator implementation and
	// we can skip the allocations inside this block.
	carrier := otel.GetTextMapPropagator()
	if len(carrier.Fields()) != 0 {
		// When there are no fields, it's a no-op
		// implementation and we can skip the allocations.
		flat := map[string]string{}
		for _, key := range carrier.Fields() {
			have := h.tmpHdrs[key]
			if len(have) > 0 {
				flat[key] = have[0]
				delete(h.tmpHdrs, key)
			}
		}

		ctx = carrier.Extract(ctx, propagation.MapCarrier(flat))
	}

	// Add streamHdrs that were not carried in the per-request headers.
	for k, v := range h.streamHdrs {
		// Note: This is done after the per-request metadata is defined
		// in recognition of a potential for duplicated values stemming
		// from the Arrow exporter's independent call to the Auth
		// extension's GetRequestMetadata().  This paired with the
		// headersetter's return of empty-string values means, we would
		// end up with an empty-string element for any headersetter
		// `from_context` rules b/c the stream uses background context.
		// This allows static headers through.
		//
		// See https://github.com/open-telemetry/opentelemetry-collector/issues/6965
		lk := strings.ToLower(k)
		if _, ok := h.tmpHdrs[lk]; !ok {
			h.tmpHdrs[lk] = v
		}
	}

	// Release the temporary copy used in emitFunc().
	newHdrs := h.tmpHdrs
	h.tmpHdrs = nil

	// Note: newHdrs is passed to the Auth plugin.  Whether
	// newHdrs is set in the context depends on h.includeMetadata.
	return h.newContext(ctx, newHdrs), newHdrs, nil
}

// tmpHdrsAppend appends to tmpHdrs, from decoder's emit function.
func (h *headerReceiver) tmpHdrsAppend(hf hpack.HeaderField) {
	if h.tmpHdrs != nil {
		// We force strings.ToLower to ensure consistency.  gRPC itself
		// does this and would do the same.
		hn := strings.ToLower(hf.Name)
		h.tmpHdrs[hn] = append(h.tmpHdrs[hn], hf.Value)
	}
}

func (h *headerReceiver) newContext(ctx context.Context, hdrs map[string][]string) context.Context {
	// Retain the Addr/Auth of the stream connection, update the
	// per-request metadata from the Arrow batch.
	var md client.Metadata
	if h.includeMetadata && hdrs != nil {
		md = client.NewMetadata(hdrs)
	}
	return client.NewContext(ctx, client.Info{
		Addr:     h.connInfo.Addr,
		Auth:     h.connInfo.Auth,
		Metadata: md,
	})
}

// logStreamError decides how to log an error.
func (r *Receiver) logStreamError(err error) {
	var code codes.Code
	var msg string
	// gRPC tends to supply status-wrapped errors, so we always
	// unpack them.  A wrapped Canceled code indicates intentional
	// shutdown, which can be due to normal causes (EOF, e.g.,
	// max-stream-lifetime reached) or unusual causes (Canceled,
	// e.g., because the other stream direction reached an error).
	if status, ok := status.FromError(err); ok {
		code = status.Code()
		msg = status.Message()
	} else if errors.Is(err, io.EOF) || errors.Is(err, context.Canceled) {
		code = codes.Canceled
		msg = err.Error()
	} else {
		code = codes.Internal
		msg = err.Error()
	}

	if code == codes.Canceled {
		r.telemetry.Logger.Debug("arrow stream shutdown", zap.String("message", msg))
	} else {
		r.telemetry.Logger.Error("arrow stream error", zap.String("message", msg), zap.Int("code", int(code)))
	}
}

func gRPCName(desc grpc.ServiceDesc) string {
	return netstats.GRPCStreamMethodName(desc, desc.Streams[0])
}

var (
	arrowTracesMethod  = gRPCName(arrowpb.ArrowTracesService_ServiceDesc)
	arrowMetricsMethod = gRPCName(arrowpb.ArrowMetricsService_ServiceDesc)
	arrowLogsMethod    = gRPCName(arrowpb.ArrowLogsService_ServiceDesc)
)

func (r *Receiver) ArrowTraces(serverStream arrowpb.ArrowTracesService_ArrowTracesServer) error {
	return r.anyStream(serverStream, arrowTracesMethod)
}

func (r *Receiver) ArrowLogs(serverStream arrowpb.ArrowLogsService_ArrowLogsServer) error {
	return r.anyStream(serverStream, arrowLogsMethod)
}

func (r *Receiver) ArrowMetrics(serverStream arrowpb.ArrowMetricsService_ArrowMetricsServer) error {
	return r.anyStream(serverStream, arrowMetricsMethod)
}

type anyStreamServer interface {
	Send(*arrowpb.BatchStatus) error
	Recv() (*arrowpb.BatchArrowRecords, error)
	grpc.ServerStream
}

type batchResp struct {
	id  int64
	err error
}

func (r *Receiver) recoverErr(retErr *error) {
	if err := recover(); err != nil {
		// When this happens, the stacktrace is
		// important and lost if we don't capture it
		// here.
		r.telemetry.Logger.Error("panic detail in otel-arrow-adapter",
			zap.Reflect("recovered", err),
			zap.Stack("stacktrace"),
		)
		*retErr = fmt.Errorf("panic in otel-arrow-adapter: %v", err)
	}
}

func (r *Receiver) anyStream(serverStream anyStreamServer, method string) (retErr error) {
	streamCtx := serverStream.Context()
	ac := r.newConsumer()

	defer func() {
		if err := ac.Close(); err != nil {
			r.telemetry.Logger.Error("arrow stream close", zap.Error(err))
		}
	}()
	defer r.recoverErr(&retErr)

	doneCtx, doneCancel := context.WithCancel(streamCtx)
	defer doneCancel()
	streamErrCh := make(chan error, 2)
	pendingCh := make(chan batchResp, runtime.NumCPU())

	go func() {
		var err error
		defer r.recoverErr(&err)
		err = r.srvReceiveLoop(doneCtx, serverStream, pendingCh, method, ac)
		streamErrCh <- err
	}()

	// WG is used to ensure main thread only returns once sender is finished sending responses for all requests.
	var senderWG sync.WaitGroup
	senderWG.Add(1)
	go func() {
		var err error
		defer r.recoverErr(&err)
		err = r.srvSendLoop(doneCtx, serverStream, pendingCh)
		streamErrCh <- err
		senderWG.Done()
	}()

	select {
	case <-doneCtx.Done():
		senderWG.Wait()
		return status.Error(codes.Canceled, "server stream shutdown")
	case retErr = <-streamErrCh:
		doneCancel()
		senderWG.Wait()
		return
	}
}

func (r *Receiver) recvOne(ctx context.Context, serverStream anyStreamServer, hrcv *headerReceiver, pendingCh chan<- batchResp, method string, ac arrowRecord.ConsumerAPI) (retErr error) {

	ctx, span := r.tracer.Start(ctx, "otel_arrow_stream_recv")
	defer span.End()

	r.inFlightWG.Add(1)
	r.recvInFlightRequests.Add(ctx, 1)
	defer func() {
		if retErr != nil {
			// Set span status if an error is returned.
			span := trace.SpanFromContext(ctx)
			span.SetStatus(otelcodes.Error, retErr.Error())

			r.logStreamError(retErr)

			// Release the wait-count and in-flight count
			// because of the error.  These are kept when
			// retErr == nil.
			r.inFlightWG.Done() // not processed
			r.recvInFlightRequests.Add(ctx, -1)
		}
	}()

	// Receive a batch corresponding with one ptrace.Traces, pmetric.Metrics,
	// or plog.Logs item.
	req, err := serverStream.Recv()

	if err != nil {
		if errors.Is(err, io.EOF) {
			return status.Error(codes.Canceled, "client stream shutdown")
		} else if errors.Is(err, context.Canceled) {
			return status.Error(codes.Canceled, "server stream shutdown")
		}
		return err
	}

	// Check for optional headers and set the incoming context.
	thisCtx, authHdrs, err := hrcv.combineHeaders(ctx, req.GetHeaders())
	if err != nil {
		// Failing to parse the incoming headers breaks the stream.
		return fmt.Errorf("arrow metadata error: %w", err)
	}
	var prevAcquiredBytes int64
	uncompSizeHeaderStr, uncompSizeHeaderFound := authHdrs["otlp-pdata-size"]
	if !uncompSizeHeaderFound || len(uncompSizeHeaderStr) == 0 {
		// This is a compressed size so make sure to acquire the difference when request is decompressed.
		prevAcquiredBytes = int64(proto.Size(req))
	} else {
		prevAcquiredBytes, err = strconv.ParseInt(uncompSizeHeaderStr[0], 10, 64)
		if err != nil {
			return fmt.Errorf("failed to convert string to request size: %w", err)
		}
	}
	// bounded queue to memory limit based on incoming uncompressed request size and waiters.
	// Acquire will fail immediately if there are too many waiters,
	// or will otherwise block until timeout or enough memory becomes available.
	err = r.boundedQueue.Acquire(ctx, int64(prevAcquiredBytes))
	if err != nil {
		return fmt.Errorf("otel-arrow bounded queue: %w", err)
	}

	if r.authServer != nil {
		var newCtx context.Context
		if newCtx, err = r.authServer.Authenticate(thisCtx, authHdrs); err != nil {
			return err
		}
		thisCtx = newCtx
	}

	err, data, numItems, uncompSize := r.consumeBatch(ctx, ac, req)

	if err != nil {
		return fmt.Errorf("otel-arrow consume: %w", err)
	}

	numAcquired, err := r.acquireAdditionalBytes(ctx, prevAcquiredBytes, uncompSize, hrcv.connInfo.Addr, uncompSizeHeaderFound)
	if err != nil {
		return fmt.Errorf("otel-arrow re-acquire additional bytes: %d: %w", uncompSize, err)
	}

	// Note: uncompSize is used as opposed to numAcquired, which may be >= uncompSize
	// in case the user makes an error in our favor.
	r.recvInFlightBytes.Add(ctx, uncompSize)
	r.recvInFlightItems.Add(ctx, int64(numItems))

	// processRecords will process and send an error to the sender loop
	go func(batchId int64) {
		ctx, span := r.tracer.Start(ctx, "otel_arrow_stream_consume")
		defer span.End()

		var err error
		defer func() {
			data = nil
			if err = r.boundedQueue.Release(numAcquired); err != nil {
				// @@@ logger
			}
			pendingCh <- batchResp{
				id:  batchId,
				err: err,
			}

			r.recvInFlightBytes.Add(ctx, -uncompSize)
			r.recvInFlightItems.Add(ctx, -int64(numItems))
			r.recvInFlightRequests.Add(ctx, -1)

			if r.telemetry.MetricsLevel > configtelemetry.LevelNormal {
				// The netstats code knows that uncompressed size is
				// unreliable for arrow transport, so we instrument it
				// directly here.  Only the primary direction of transport
				// is instrumented this way.
				var sized netstats.SizesStruct
				sized.Method = method
				sized.Length = uncompSize
				r.netReporter.CountReceive(ctx, sized)

				// @@@@ what is this really doing w/ sized only partially filled?
				r.netReporter.SetSpanSizeAttributes(ctx, sized)
			}

			r.inFlightWG.Done() // done processing
		}()

		// recoverErr is a special function because it recovers panics, so we
		// keep it in a separate defer than the processing above, which will
		// run after the panic is recovered into an ordinary error.
		defer r.recoverErr(&err)

		err = r.consumeData(ctx, method, data, numItems)
	}(req.GetBatchId())

	return nil
}

func (r *Receiver) srvReceiveLoop(ctx context.Context, serverStream anyStreamServer, pendingCh chan<- batchResp, method string, ac arrowRecord.ConsumerAPI) (retErr error) {
	hrcv := newHeaderReceiver(ctx, r.authServer, r.gsettings.IncludeMetadata)
	for {
		select {
		case <-ctx.Done():
			return status.Error(codes.Canceled, "server stream shutdown")
		default:
			err := r.recvOne(ctx, serverStream, hrcv, pendingCh, method, ac)
			if err != nil {
				return err
			}
		}
	}
}

func (r *Receiver) sendOne(serverStream anyStreamServer, resp batchResp) error {
	// Note: Statuses can be batched, but we do not take
	// advantage of this feature.
	status := &arrowpb.BatchStatus{
		BatchId: resp.id,
	}
	if resp.err == nil {
		status.StatusCode = arrowpb.StatusCode_OK
	} else {
		status.StatusMessage = resp.err.Error()
		switch {
		case errors.Is(resp.err, arrowRecord.ErrConsumerMemoryLimit):
			r.telemetry.Logger.Error("arrow resource exhausted", zap.Error(resp.err))
			status.StatusCode = arrowpb.StatusCode_RESOURCE_EXHAUSTED
		case consumererror.IsPermanent(resp.err):
			r.telemetry.Logger.Error("arrow data error", zap.Error(resp.err))
			status.StatusCode = arrowpb.StatusCode_INVALID_ARGUMENT
		default:
			r.telemetry.Logger.Debug("arrow consumer error", zap.Error(resp.err))
			status.StatusCode = arrowpb.StatusCode_UNAVAILABLE
		}
	}

	if err := serverStream.Send(status); err != nil {
		r.logStreamError(err)
		return err
	}

	return nil
}

func (r *Receiver) flushSender(serverStream anyStreamServer, pendingCh <-chan batchResp) error {
	var err error
	// wait for all in flight requests to successfully be processed or fail.
	r.inFlightWG.Wait()
	for {
		select {
		case resp := <-pendingCh:
			err = r.sendOne(serverStream, resp)
			if err != nil {
				return err
			}
		default:
			// Currently nothing left in pendingCh.
			return nil
		}
	}
}

func (r *Receiver) srvSendLoop(ctx context.Context, serverStream anyStreamServer, pendingCh <-chan batchResp) error {
	for {
		select {
		case <-ctx.Done():
			return r.flushSender(serverStream, pendingCh)
		case resp := <-pendingCh:
			if err := r.sendOne(serverStream, resp); err != nil {
				return err
			}
		}
	}
}

// consumeBatch applies the batch to the Arrow Consumer, returns a
// slice of pdata objects of the corresponding data type as `any`.
// along with the number of items and true uncompressed size.
func (r *Receiver) consumeBatch(ctx context.Context, arrowConsumer arrowRecord.ConsumerAPI, records *arrowpb.BatchArrowRecords) (retErr error, retData any, numItems int, uncompSize int64) {

	payloads := records.GetArrowPayloads()
	if len(payloads) == 0 {
		return nil, nil, 0, 0
	}

	switch payloads[0].Type {
	case arrowpb.ArrowPayloadType_UNIVARIATE_METRICS:
		if r.Metrics() == nil {
			return status.Error(codes.Unimplemented, "metrics service not available"), nil, 0, 0
		}
		var sizer pmetric.ProtoMarshaler

		data, err := arrowConsumer.MetricsFrom(records)
		if err != nil {
			err = consumererror.NewPermanent(err)
		} else {
			for _, metrics := range data {
				numItems += metrics.DataPointCount()
				uncompSize += int64(sizer.MetricsSize(metrics))
			}
		}
		retData = data
		retErr = err

	case arrowpb.ArrowPayloadType_LOGS:
		if r.Logs() == nil {
			return status.Error(codes.Unimplemented, "logs service not available"), nil, 0, 0
		}
		var sizer plog.ProtoMarshaler

		data, err := arrowConsumer.LogsFrom(records)
		if err != nil {
			err = consumererror.NewPermanent(err)
		} else {
			for _, logs := range data {
				numItems += logs.LogRecordCount()
				uncompSize += int64(sizer.LogsSize(logs))
			}
		}
		retData = data
		retErr = err

	case arrowpb.ArrowPayloadType_SPANS:
		if r.Traces() == nil {
			return status.Error(codes.Unimplemented, "traces service not available"), nil, 0, 0
		}
		var sizer ptrace.ProtoMarshaler

		data, err := arrowConsumer.TracesFrom(records)
		if err != nil {
			err = consumererror.NewPermanent(err)
		} else {
			for _, traces := range data {
				numItems += traces.SpanCount()
				uncompSize += int64(sizer.TracesSize(traces))
			}
		}
		retData = data
		retErr = err

	default:
		retErr = ErrUnrecognizedPayload
	}

	return retErr, retData, numItems, uncompSize
}

func (r *Receiver) consumeData(ctx context.Context, method string, data any, numItems int) error {
	var err error
	switch items := data.(type) {
	case []pmetric.Metrics:
		ctx = r.obsrecv.StartMetricsOp(ctx)
		for _, metrics := range items {
			err = multierr.Append(err,
				r.Metrics().ConsumeMetrics(ctx, metrics),
			)
		}

		r.obsrecv.EndMetricsOp(ctx, streamFormat, numItems, err)

	case []plog.Logs:
		ctx = r.obsrecv.StartLogsOp(ctx)
		for _, logs := range items {
			err = multierr.Append(err,
				r.Logs().ConsumeLogs(ctx, logs),
			)
		}
		r.obsrecv.EndLogsOp(ctx, streamFormat, numItems, err)

	case []ptrace.Traces:
		ctx = r.obsrecv.StartTracesOp(ctx)
		for _, traces := range items {
			err = multierr.Append(err,
				r.Traces().ConsumeTraces(ctx, traces),
			)
		}
		r.obsrecv.EndTracesOp(ctx, streamFormat, numItems, err)

	default:
		err = ErrUnrecognizedPayload
	}
	return err
}

func (r *Receiver) acquireAdditionalBytes(ctx context.Context, prevAcquired, uncompSize int64, addr net.Addr, uncompSizeHeaderFound bool) (int64, error) {
	diff := uncompSize - prevAcquired

	if diff == 0 {
		return uncompSize, nil
	}

	if uncompSizeHeaderFound {
		var clientAddr string
		if addr != nil {
			clientAddr = addr.String()
		}
		// a mismatch between header set by exporter and the uncompSize just calculated.
		r.telemetry.Logger.Debug("mismatch between uncompressed size in receiver and otlp-pdata-size header",
			zap.String("client-address", clientAddr),
			zap.Int("uncompsize", int(uncompSize)),
			zap.Int("otlp-pdata-size", int(prevAcquired)),
		)
		return prevAcquired, nil
	}

	if diff < 0 {
		// proto.Size() on compressed request was greater than pdata uncompressed size.
		r.telemetry.Logger.Debug("uncompressed size is less than compressed size",
			zap.Int("uncompressed", int(uncompSize)),
			zap.Int("compressed", int(prevAcquired)),
		)
		return prevAcquired, nil
	}

	// diff > 0 means we previously acquired too few bytes initially and we need to correct this. Release previously
	// acquired bytes to prevent deadlock and reacquire the uncompressed size we just calculated.
	// Note: No need to release and reacquire bytes if diff < 0 because this has less impact and no reason to potentially block
	// a request that is in flight by reacquiring the correct size.
	if err := r.boundedQueue.Release(prevAcquired); err != nil {
		return 0, err
	}
	if err := r.boundedQueue.Acquire(ctx, uncompSize); err != nil {
		return 0, err
	}
	return uncompSize, nil
}
