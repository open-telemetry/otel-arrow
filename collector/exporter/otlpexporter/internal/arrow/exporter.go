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

package arrow // import "github.com/f5/otel-arrow-adapter/collector/exporter/otlpexporter/internal/arrow"

import (
	"context"
	"errors"
	"sync"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrowRecord "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
	"go.uber.org/zap"
	"google.golang.org/grpc"

	"go.opentelemetry.io/collector/component"
)

// High-level TODOs:
// TODO: Use the MAX_CONNECTION_AGE and MAX_CONNECTION_AGE_GRACE settings.

// Exporter is 1:1 with exporter, isolates arrow-specific
// functionality.
type Exporter struct {
	// settings contains Arrow-specific parameters.
	settings Settings

	// newProducer returns a real (or mock) Producer.
	newProducer func() arrowRecord.ProducerAPI

	// telemetry includes logger, tracer, meter.
	telemetry component.TelemetrySettings

	// client uses the exporter's gRPC ClientConn (or is a mock, in tests).
	client arrowpb.ArrowStreamServiceClient

	// grpcOptions includes options used by the unary RPC methods,
	// e.g., WaitForReady.
	grpcOptions []grpc.CallOption

	// ready prioritizes streams that are ready to send
	ready *streamPrioritizer

	// returning is used to pass broken, gracefully-terminated,
	// and otherwise to the stream controller.
	returning chan *Stream

	// cancel cancels the background context of this
	// Exporter, used for shutdown.
	cancel context.CancelFunc

	// wg counts one per active goroutine belonging to all strings
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
}

// NewExporter configures a new Exporter.
func NewExporter(
	settings Settings,
	newProducer func() arrowRecord.ProducerAPI,
	telemetry component.TelemetrySettings,
	client arrowpb.ArrowStreamServiceClient,
	grpcOptions []grpc.CallOption,
) *Exporter {
	return &Exporter{
		settings:    settings,
		newProducer: newProducer,
		telemetry:   telemetry,
		client:      client,
		grpcOptions: grpcOptions,
		returning:   make(chan *Stream, settings.NumStreams),
		ready:       nil,
		cancel:      nil,
	}
}

// Start creates the background context used by all streams and starts
// a stream controller, which initializes the initial set of streams.
func (e *Exporter) Start(ctx context.Context) error {
	ctx, cancel := context.WithCancel(ctx)

	e.cancel = cancel
	e.wg.Add(1)
	e.ready = newStreamPrioritizer(ctx, e.settings)

	go e.runStreamController(ctx)

	return nil
}

// runStreamController starts the initial set of streams, then waits for streams to
// terminate one at a time and restarts them.  If streams come back with a nil
// client (meaning that OTLP+Arrow was not supported by the endpoint), it will
// not be restarted.
func (e *Exporter) runStreamController(bgctx context.Context) {
	defer e.cancel()
	defer e.wg.Done()

	running := e.settings.NumStreams

	// Start the initial number of streams
	for i := 0; i < running; i++ {
		e.wg.Add(1)
		go e.runArrowStream(bgctx)
	}

	for {
		select {
		case stream := <-e.returning:
			if stream.client != nil {
				// The stream closed or broken.  Restart it.
				e.wg.Add(1)
				go e.runArrowStream(bgctx)
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
			}

		case <-bgctx.Done():
			// We are shutting down.
			return
		}
	}
}

// runArrowStream begins one gRPC stream using a child of the background context.
// If the stream connection is successful, this goroutine starts another goroutine
// to call writeStream() and performs readStream() itself.  When the stream shuts
// down this call synchronously waits for and unblocks the consumers.
func (e *Exporter) runArrowStream(ctx context.Context) {
	producer := e.newProducer()
	stream := newStream(producer, e.ready, e.telemetry)

	defer func() {
		if err := producer.Close(); err != nil {
			e.telemetry.Logger.Error("arrow producer close:", zap.Error(err))
		}
		e.wg.Done()
		e.returning <- stream
	}()

	stream.run(ctx, e.client, e.grpcOptions)
}

// SendAndWait tries to send using an Arrow stream.  The results are:
//
// (true, nil):      Arrow send: success at consumer
// (false, nil):     Arrow is not supported by the server, caller expected to fallback.
// (true, non-nil):  Arrow send: server response may be permanent or allow retry.
// (false, non-nil): Context timeout prevents retry.
//
// consumer should fall back to standard OTLP, (true, nil)
func (e *Exporter) SendAndWait(ctx context.Context, data interface{}) (bool, error) {
	for {
		var stream *Stream
		var err error
		select {
		case <-ctx.Done():
			err = ctx.Err()
		case stream = <-e.ready.readyChannel():
		}

		if err != nil {
			return false, err // a Context error
		}
		if stream == nil {
			return false, nil // a downgraded connection
		}

		err = stream.SendAndWait(ctx, data)
		if err != nil && errors.Is(err, ErrStreamRestarting) {
			continue // an internal retry

		}
		// result from arrow server (may be nil, may be
		// permanent, etc.)
		return true, err
	}
}

// Shutdown returns when all Arrow-associated goroutines have returned.
func (e *Exporter) Shutdown(ctx context.Context) error {
	e.cancel()
	e.wg.Wait()
	return nil
}
