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
	"sync/atomic"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/collector/netstats"
	arrowRecord "github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configgrpc"
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
	ErrUnrecognizedPayload = consumererror.NewPermanent(fmt.Errorf("unrecognized OTel-Arrow payload"))
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
func (r *Receiver) logStreamError(err error, where string) {
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
		r.telemetry.Logger.Error("arrow stream error", zap.String("message", msg), zap.Int("code", int(code)), zap.String("where", where))
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
		*retErr = status.Errorf(codes.Internal, "panic in otel-arrow-adapter: %v", err)
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

	// doneCancel allows an error in the sender/receiver to
	// interrupt the corresponding thread.
	doneCtx, doneCancel := context.WithCancel(streamCtx)
	defer doneCancel()

	// streamErrCh returns up to two errors from the sender and
	// receiver threads started below.
	streamErrCh := make(chan error, 2)
	pendingCh := make(chan batchResp, runtime.NumCPU())

	// wg is used to ensure this thread returns after both
	// sender and recevier threads return.
	var wg sync.WaitGroup
	wg.Add(2)

	// The inflightWG is used to wait for all data to send.  The
	// 1-count here is removed after srvReceiveLoop() returns,
	// having this ensures that concurrent calls to Add() in the
	// receiver do not race with Wait() in the sender.
	r.inFlightWG.Add(1)

	go func() {
		var err error
		defer wg.Done()
		defer r.recoverErr(&err)
		defer r.inFlightWG.Done()
		err = r.srvReceiveLoop(doneCtx, serverStream, pendingCh, method, ac)
		streamErrCh <- err
	}()

	go func() {
		var err error
		defer wg.Done()
		defer r.recoverErr(&err)
		err = r.srvSendLoop(doneCtx, serverStream, pendingCh)
		streamErrCh <- err
	}()

	// Wait for sender/receiver threads to return before returning.
	defer wg.Wait()

	select {
	case <-doneCtx.Done():
		return status.Error(codes.Canceled, "server stream shutdown")
	case retErr = <-streamErrCh:
		doneCancel()
		return
	}
}

func (r *Receiver) newInFlightData(ctx context.Context, method string, batchID int64, pendingCh chan<- batchResp) (context.Context, *inFlightData) {
	ctx, span := r.tracer.Start(ctx, "otel_arrow_stream_inflight")

	r.inFlightWG.Add(1)
	r.recvInFlightRequests.Add(ctx, 1)
	id := &inFlightData{
		Receiver:  r,
		method:    method,
		batchID:   batchID,
		pendingCh: pendingCh,
		span:      span,
	}
	id.refs.Add(1)
	return ctx, id
}

// inFlightData is responsible for storing the resources held by one request.
type inFlightData struct {
	// Receiver is the owner of the resources held by this object.
	*Receiver

	method    string
	batchID   int64
	pendingCh chan<- batchResp
	span      trace.Span

	// refs counts the number of goroutines holding this object.
	// initially the recvOne() body, on success the
	// consumeAndRespond() function.
	refs atomic.Int32

	numAcquired int64 // how many bytes held in the semaphore
	numItems    int   // how many items
	uncompSize  int64 // uncompressed data size
}

func (id *inFlightData) recvDone(ctx context.Context, recvErrPtr *error) {
	retErr := *recvErrPtr

	if retErr != nil {
		// logStreamError because this response will break the stream.
		id.logStreamError(retErr, "recv")
		id.span.SetStatus(otelcodes.Error, retErr.Error())
	}

	id.anyDone(ctx)
}

func (id *inFlightData) consumeDone(ctx context.Context, consumeErrPtr *error) {
	retErr := *consumeErrPtr

	if retErr != nil {
		// debug-level because the error was external from the pipeline.
		id.telemetry.Logger.Debug("otel-arrow consume", zap.Error(retErr))
		id.span.SetStatus(otelcodes.Error, retErr.Error())
	}

	id.replyToCaller(retErr)
	id.anyDone(ctx)
}

func (id *inFlightData) replyToCaller(callerErr error) {
	id.pendingCh <- batchResp{
		id:  id.batchID,
		err: callerErr,
	}
}

func (id *inFlightData) anyDone(ctx context.Context) {
	// check if there are still refs, in which case leave the in-flight
	// counts where they are.
	if id.refs.Add(-1) != 0 {
		return
	}

	id.span.End()

	if id.numAcquired != 0 {
		if err := id.boundedQueue.Release(id.numAcquired); err != nil {
			id.telemetry.Logger.Error("release error", zap.Error(err))
		}
	}

	if id.uncompSize != 0 {
		id.recvInFlightBytes.Add(ctx, -id.uncompSize)
	}
	if id.numItems != 0 {
		id.recvInFlightItems.Add(ctx, int64(-id.numItems))
	}

	// The netstats code knows that uncompressed size is
	// unreliable for arrow transport, so we instrument it
	// directly here.  Only the primary direction of transport
	// is instrumented this way.
	var sized netstats.SizesStruct
	sized.Method = id.method
	sized.Length = id.uncompSize
	id.netReporter.CountReceive(ctx, sized)

	id.recvInFlightRequests.Add(ctx, -1)
	id.inFlightWG.Done()
}

// recvOne begins processing a single Arrow batch.
//
// If an error is encountered before Arrow data is successfully consumed,
// the stream will break and the error will be returned immediately.
//
// If the error is due to authorization, the stream remains unbroken
// and the request fails.
//
// If not enough resources are available, the stream will block (if
// waiting permitted) or break (insufficient waiters).
//
// Assuming success, a new goroutine is created to handle consuming the
// data.
//
// This handles constructing an inFlightData object, which itself
// tracks everything that needs to be used by instrumention when the
// batch finishes.
func (r *Receiver) recvOne(streamCtx context.Context, serverStream anyStreamServer, hrcv *headerReceiver, pendingCh chan<- batchResp, method string, ac arrowRecord.ConsumerAPI) (retErr error) {

	// Receive a batch corresponding with one ptrace.Traces, pmetric.Metrics,
	// or plog.Logs item.
	req, err := serverStream.Recv()

	// inflightCtx is carried through into consumeAndProcess on the success path.
	inflightCtx, flight := r.newInFlightData(streamCtx, method, req.GetBatchId(), pendingCh)
	defer flight.recvDone(inflightCtx, &retErr)

	// this span is a child of the inflight, covering the Arrow decode, Auth, etc.
	_, span := r.tracer.Start(inflightCtx, "otel_arrow_stream_recv")
	defer span.End()

	if err != nil {
		if errors.Is(err, io.EOF) {
			return status.Error(codes.Canceled, "client stream shutdown")
		} else if errors.Is(err, context.Canceled) {
			return status.Error(codes.Canceled, "server stream shutdown")
		}
		// Note: err is directly from gRPC, should already have status.
		return err
	}

	// Check for optional headers and set the incoming context.
	inflightCtx, authHdrs, err := hrcv.combineHeaders(inflightCtx, req.GetHeaders())
	if err != nil {
		// Failing to parse the incoming headers breaks the stream.
		return status.Errorf(codes.Internal, "arrow metadata error: %v", err)
	}

	// Authorize the request, if configured, prior to acquiring resources.
	if r.authServer != nil {
		var authErr error
		inflightCtx, authErr = r.authServer.Authenticate(inflightCtx, authHdrs)
		if authErr != nil {
			flight.replyToCaller(status.Error(codes.Unauthenticated, authErr.Error()))
			return nil
		}
	}

	var prevAcquiredBytes int64
	uncompSizeHeaderStr, uncompSizeHeaderFound := authHdrs["otlp-pdata-size"]
	if !uncompSizeHeaderFound || len(uncompSizeHeaderStr) == 0 {
		// This is a compressed size so make sure to acquire the difference when request is decompressed.
		prevAcquiredBytes = int64(proto.Size(req))
	} else {
		prevAcquiredBytes, err = strconv.ParseInt(uncompSizeHeaderStr[0], 10, 64)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to convert string to request size: %v", err)
		}
	}

	// Use the bounded queue to memory limit based on incoming
	// uncompressed request size and waiters.  Acquire will fail
	// immediately if there are too many waiters, or will
	// otherwise block until timeout or enough memory becomes
	// available.
	err = r.boundedQueue.Acquire(inflightCtx, prevAcquiredBytes)
	if err != nil {
		return status.Errorf(codes.ResourceExhausted, "otel-arrow bounded queue: %v", err)
	}
	flight.numAcquired = prevAcquiredBytes

	err, data, numItems, uncompSize := r.consumeBatch(ac, req)

	if err != nil {
		if errors.Is(err, arrowRecord.ErrConsumerMemoryLimit) {
			return status.Errorf(codes.ResourceExhausted, "otel-arrow decode: %v", err)
		} else {
			return status.Errorf(codes.Internal, "otel-arrow decode: %v", err)
		}
	}

	flight.uncompSize = uncompSize
	flight.numItems = numItems

	r.recvInFlightBytes.Add(inflightCtx, uncompSize)
	r.recvInFlightItems.Add(inflightCtx, int64(numItems))

	numAcquired, err := r.acquireAdditionalBytes(inflightCtx, prevAcquiredBytes, uncompSize, hrcv.connInfo.Addr, uncompSizeHeaderFound)

	flight.numAcquired = numAcquired
	if err != nil {
		return status.Errorf(codes.ResourceExhausted, "otel-arrow bounded queue re-acquire: %v", err)
	}

	// Recognize that the request is still in-flight via consumeAndRespond()
	flight.refs.Add(1)

	// consumeAndRespond consumes the data and returns control to the sender loop.
	go r.consumeAndRespond(inflightCtx, data, flight)

	return nil
}

// consumeAndRespond finishes the span started in recvOne and logs the
// result after invoking the pipeline to consume the data.
func (r *Receiver) consumeAndRespond(ctx context.Context, data any, flight *inFlightData) {
	var err error
	defer flight.consumeDone(ctx, &err)

	// recoverErr is a special function because it recovers panics, so we
	// keep it in a separate defer than the processing above, which will
	// run after the panic is recovered into an ordinary error.
	defer r.recoverErr(&err)

	err = r.consumeData(ctx, data, flight)
}

// srvReceiveLoop repeatedly receives one batch of data.
func (r *Receiver) srvReceiveLoop(ctx context.Context, serverStream anyStreamServer, pendingCh chan<- batchResp, method string, ac arrowRecord.ConsumerAPI) (retErr error) {
	hrcv := newHeaderReceiver(ctx, r.authServer, r.gsettings.IncludeMetadata)
	for {
		select {
		case <-ctx.Done():
			return status.Error(codes.Canceled, "server stream shutdown")
		default:
			if err := r.recvOne(ctx, serverStream, hrcv, pendingCh, method, ac); err != nil {
				return err
			}
		}
	}
}

// srvReceiveLoop repeatedly sends one batch data response.
func (r *Receiver) sendOne(serverStream anyStreamServer, resp batchResp) error {
	// Note: Statuses can be batched, but we do not take
	// advantage of this feature.
	bs := &arrowpb.BatchStatus{
		BatchId: resp.id,
	}
	if resp.err == nil {
		bs.StatusCode = arrowpb.StatusCode_OK
	} else {
		// Generally, code in the receiver should use
		// status.Errorf(codes.XXX, ...)  so that we take the
		// first branch.
		if gsc, ok := status.FromError(resp.err); ok {
			bs.StatusCode = arrowpb.StatusCode(gsc.Code())
			bs.StatusMessage = gsc.Message()
		} else {
			// Ideally, we don't take this branch because all code uses
			// gRPC status constructors and we've taken the branch above.
			//
			// This is a fallback for several broad categories of error.
			bs.StatusMessage = resp.err.Error()

			switch {
			case consumererror.IsPermanent(resp.err):
				// Some kind of pipeline error, somewhere downstream.
				r.telemetry.Logger.Error("arrow data error", zap.Error(resp.err))
				bs.StatusCode = arrowpb.StatusCode_INVALID_ARGUMENT
			default:
				// Probably a pipeline error, retryable.
				r.telemetry.Logger.Debug("arrow consumer error", zap.Error(resp.err))
				bs.StatusCode = arrowpb.StatusCode_UNAVAILABLE
			}
		}
	}

	if err := serverStream.Send(bs); err != nil {
		// logStreamError because this response will break the stream.
		r.logStreamError(err, "send")
		return err
	}

	return nil
}

func (r *Receiver) flushSender(serverStream anyStreamServer, pendingCh <-chan batchResp) error {
	var err error
	// wait for all in flight requests to be successfully
	// processed or fail.  this implies waiting for the receiver
	// loop to exit, as it holds one additional wait count to
	// avoid a race with Add() here.
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
func (r *Receiver) consumeBatch(arrowConsumer arrowRecord.ConsumerAPI, records *arrowpb.BatchArrowRecords) (retErr error, retData any, numItems int, uncompSize int64) {

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
		if err == nil {
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
		if err == nil {
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
		if err == nil {
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

// consumeData invokes the next pipeline consumer for a received batch of data.
// it uses the standard OTel collector instrumentation (receiverhelper.ObsReport).
//
// if any errors are permanent, returns a permanent error.
func (r *Receiver) consumeData(ctx context.Context, data any, flight *inFlightData) (retErr error) {
	oneOp := func(err error) {
		retErr = multierr.Append(retErr, err)
	}
	var final func(context.Context, string, int, error)

	switch items := data.(type) {
	case []pmetric.Metrics:
		ctx = r.obsrecv.StartMetricsOp(ctx)
		for _, metrics := range items {
			oneOp(r.Metrics().ConsumeMetrics(ctx, metrics))
		}
		final = r.obsrecv.EndMetricsOp

	case []plog.Logs:
		ctx = r.obsrecv.StartLogsOp(ctx)
		for _, logs := range items {
			oneOp(r.Logs().ConsumeLogs(ctx, logs))
		}
		final = r.obsrecv.EndLogsOp

	case []ptrace.Traces:
		ctx = r.obsrecv.StartTracesOp(ctx)
		for _, traces := range items {
			oneOp(r.Traces().ConsumeTraces(ctx, traces))
		}
		final = r.obsrecv.EndTracesOp

	default:
		retErr = ErrUnrecognizedPayload
	}
	if final != nil {
		final(ctx, streamFormat, flight.numItems, retErr)
	}
	return retErr
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
	} else if diff < 0 {
		// proto.Size() on compressed request was greater than pdata uncompressed size.
		r.telemetry.Logger.Debug("uncompressed size is less than compressed size",
			zap.Int("uncompressed", int(uncompSize)),
			zap.Int("compressed", int(prevAcquired)),
		)
	}

	if diff < 0 {
		// If the difference is negative, release the overage.
		if err := r.boundedQueue.Release(-diff); err != nil {
			return 0, err
		}
	} else {
		// Release previously acquired bytes to prevent deadlock and
		// reacquire the uncompressed size we just calculated.
		if err := r.boundedQueue.Release(prevAcquired); err != nil {
			return 0, err
		}
		if err := r.boundedQueue.Acquire(ctx, uncompSize); err != nil {
			return 0, err
		}
	}
	return uncompSize, nil
}
