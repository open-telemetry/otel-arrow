// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"errors"
	"math/rand"
	"sync"
	"time"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/collector/netstats"
	arrowRecord "github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
)

// Exporter is 1:1 with exporter, isolates arrow-specific
// functionality.
type Exporter struct {
	// numStreams is the number of streams that will be used.
	numStreams int

	maxStreamLifetime time.Duration

	// disableDowngrade prevents downgrade from occurring, supports
	// forcing Arrow transport.
	disableDowngrade bool

	// telemetry includes logger, tracer, meter.
	telemetry component.TelemetrySettings

	// grpcOptions includes options used by the unary RPC methods,
	// e.g., WaitForReady.
	grpcOptions []grpc.CallOption

	// newProducer returns a real (or mock) Producer.
	newProducer func() arrowRecord.ProducerAPI

	// client is a stream corresponding with the signal's payload
	// type. uses the exporter's gRPC ClientConn (or is a mock, in tests).
	streamClient StreamClientFunc

	// perRPCCredentials derived from the exporter's gRPC auth settings.
	perRPCCredentials credentials.PerRPCCredentials

	// returning is used to pass broken, gracefully-terminated,
	// and otherwise to the stream controller.
	returning chan *Stream

	// ready prioritizes streams that are ready to send
	ready streamPrioritizer

	// cancel cancels the background context of this
	// Exporter, used for shutdown.
	cancel context.CancelFunc

	// wg counts one per active goroutine belonging to all streams
	// of this exporter.  The wait group has Add(1) called before
	// starting goroutines so that they can be properly waited for
	// in shutdown(), so the pattern is:
	//
	//   wg.Add(1)
	//   go func() {
	//     defer wg.Done()
	//     ...
	//   }()
	wg sync.WaitGroup

	// netReporter measures network traffic.
	netReporter netstats.Interface
}

// AnyStreamClient is the interface supported by all Arrow streams.
type AnyStreamClient interface {
	Send(*arrowpb.BatchArrowRecords) error
	Recv() (*arrowpb.BatchStatus, error)
	grpc.ClientStream
}

// streamClientFunc is a constructor for AnyStreamClients.  These return
// the method name to assist with instrumentation, since the gRPC stats
// handler isn't able to see the correct uncompressed size.
type StreamClientFunc func(context.Context, ...grpc.CallOption) (AnyStreamClient, string, error)

// MakeAnyStreamClient accepts any Arrow-like stream and turns it into
// an AnyStreamClient.  The method name is carried through because
// once constructed, gRPC clients will not reveal their service and
// method names.
func MakeAnyStreamClient[T AnyStreamClient](method string, clientFunc func(ctx context.Context, opts ...grpc.CallOption) (T, error)) StreamClientFunc {
	return func(ctx context.Context, opts ...grpc.CallOption) (AnyStreamClient, string, error) {
		client, err := clientFunc(ctx, opts...)
		return client, method, err
	}
}

// NewExporter configures a new Exporter.
func NewExporter(
	maxStreamLifetime time.Duration,
	numStreams int,
	disableDowngrade bool,
	telemetry component.TelemetrySettings,
	grpcOptions []grpc.CallOption,
	newProducer func() arrowRecord.ProducerAPI,
	streamClient StreamClientFunc,
	perRPCCredentials credentials.PerRPCCredentials,
	netReporter netstats.Interface,
) *Exporter {
	return &Exporter{
		maxStreamLifetime: maxStreamLifetime,
		numStreams:        numStreams,
		disableDowngrade:  disableDowngrade,
		telemetry:         telemetry,
		grpcOptions:       grpcOptions,
		newProducer:       newProducer,
		streamClient:      streamClient,
		perRPCCredentials: perRPCCredentials,
		returning:         make(chan *Stream, numStreams),
		netReporter:       netReporter,
	}
}

func newStreamWorkState() *streamWorkState {
	return &streamWorkState{
		waiters: map[int64]chan error{},
		toWrite: make(chan writeItem, 1), // @@@ Can't be zero?
	}
}

// Start creates the background context used by all streams and starts
// a stream controller, which initializes the initial set of streams.
func (e *Exporter) Start(ctx context.Context) error {
	ctx, cancel := context.WithCancel(ctx)

	var sws []*streamWorkState
	for i := 0; i < e.numStreams; i++ {
		sws = append(sws, newStreamWorkState())
	}

	e.ready = newStreamPrioritizer(ctx, sws...)
	e.cancel = cancel
	e.wg.Add(1)
	go e.runStreamController(ctx, sws)

	return nil
}

// runStreamController starts the initial set of streams, then waits for streams to
// terminate one at a time and restarts them.  If streams come back with a nil
// client (meaning that OTel-Arrow was not supported by the endpoint), it will
// not be restarted.
func (e *Exporter) runStreamController(bgctx context.Context, state []*streamWorkState) {
	defer e.cancel()
	defer e.wg.Done()

	running := e.numStreams

	// Start the initial number of streams
	for i := 0; i < running; i++ {
		e.wg.Add(1)
		go e.runArrowStream(bgctx, state[i])
	}

	for {
		select {
		case stream := <-e.returning:
			if stream.client != nil || e.disableDowngrade {
				// The stream closed or broken.  Restart it.
				e.wg.Add(1)
				go e.runArrowStream(bgctx, stream.workState)
				continue
			}
			// Otherwise, the stream never got started.  It was
			// downgraded and senders will use the standard OTLP path.
			running--

			// None of the streams were able to connect to
			// an Arrow endpoint.
			if running == 0 {
				e.telemetry.Logger.Info("could not establish arrow streams, downgrading to standard OTLP export")
				e.ready.downgrade()
				return
			}

		case <-bgctx.Done():
			// We are shutting down.
			return
		}
	}
}

// addJitter is used to subtract 0-5% from max_stream_lifetime.  Since
// the max_stream_lifetime value is expected to be close to the
// receiver's max_connection_age_grace setting, we do not add jitter,
// only subtract.
func addJitter(v time.Duration) time.Duration {
	if v == 0 {
		return 0
	}
	return v - time.Duration(rand.Int63n(int64(v/20)))
}

// runArrowStream begins one gRPC stream using a child of the background context.
// If the stream connection is successful, this goroutine starts another goroutine
// to call writeStream() and performs readStream() itself.  When the stream shuts
// down this call synchronously waits for and unblocks the consumers.
func (e *Exporter) runArrowStream(ctx context.Context, state *streamWorkState) {
	producer := e.newProducer()

	stream := newStream(producer, e.ready, e.telemetry, e.netReporter, state)
	stream.maxStreamLifetime = addJitter(e.maxStreamLifetime)

	defer func() {
		if err := producer.Close(); err != nil {
			e.telemetry.Logger.Error("arrow producer close:", zap.Error(err))
		}
		e.wg.Done()
		e.returning <- stream
	}()

	stream.run(ctx, e.streamClient, e.grpcOptions)
}

// SendAndWait tries to send using an Arrow stream.  The results are:
//
// (true, nil):      Arrow send: success at consumer
// (false, nil):     Arrow is not supported by the server, caller expected to fallback.
// (true, non-nil):  Arrow send: server response may be permanent or allow retry.
// (false, non-nil): Context timeout prevents retry.
//
// consumer should fall back to standard OTLP, (true, nil)
func (e *Exporter) SendAndWait(ctx context.Context, data any) (bool, error) {
	errCh := make(chan error, 1)

	// Note that if the OTLP exporter's gRPC Headers field was
	// set, those (static) headers were used to establish the
	// stream.  The caller's context was returned by
	// baseExporter.enhanceContext() includes the static headers
	// plus optional client metadata.  Here, get whatever
	// headers that gRPC would have transmitted for a unary RPC
	// and convey them via the Arrow batch.

	// Note that the "uri" parameter to GetRequestMetadata is
	// not used by the headersetter extension and is not well
	// documented.  Since it's an optional list, we omit it.
	var md map[string]string
	if e.perRPCCredentials != nil {
		var err error
		md, err = e.perRPCCredentials.GetRequestMetadata(ctx)
		if err != nil {
			return false, err
		}
	}

	// Note that the uncompressed size as measured by the receiver
	// will be different than uncompressed size as measured by the
	// exporter, because of the optimization phase performed in the
	// conversion to Arrow.
	var uncompSize int
	if e.telemetry.MetricsLevel > configtelemetry.LevelNormal {
		switch data := data.(type) {
		case ptrace.Traces:
			var sizer ptrace.ProtoMarshaler
			uncompSize = sizer.TracesSize(data)
		case plog.Logs:
			var sizer plog.ProtoMarshaler
			uncompSize = sizer.LogsSize(data)
		case pmetric.Metrics:
			var sizer pmetric.ProtoMarshaler
			uncompSize = sizer.MetricsSize(data)
		}
	}

	wri := writeItem{
		records:    data,
		md:         md,
		uncompSize: uncompSize,
		errCh:      errCh,
		parent:     ctx,
	}

	for {
		stream, err := e.ready.nextWriter(ctx)

		if err != nil {
			return false, err // a Context error
		}
		if stream == nil {
			return false, nil // a downgraded connection
		}

		err = stream.sendAndWait(wri)
		if err != nil && errors.Is(err, ErrStreamRestarting) {
			continue // an internal retry

		}
		// result from arrow server (may be nil, may be
		// permanent, etc.)
		return true, err
	}
}

// Shutdown returns when all Arrow-associated goroutines have returned.
func (e *Exporter) Shutdown(_ context.Context) error {
	e.cancel()
	e.wg.Wait()
	return nil
}

func (wri writeItem) waitForWrite(done <-chan struct{}) error {
	select {
	case <-done:
		return ErrStreamRestarting
	case <-wri.parent.Done():
		// This caller's context timed out.
		return wri.parent.Err()
	case err := <-wri.errCh:
		// Note: includes err == nil and err != nil cases.
		return err
	}
}
