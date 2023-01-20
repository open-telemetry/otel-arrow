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

package arrow // import "github.com/f5/otel-arrow-adapter/collector/gen/exporter/otlpexporter/internal/arrow"

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"sync"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrowRecord "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
	"go.uber.org/multierr"
	"go.uber.org/zap"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/status"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer/consumererror"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

// Stream is 1:1 with gRPC stream.
type Stream struct {
	// producer is exclusive to the holder of the stream.
	producer arrowRecord.ProducerAPI

	// prioritizer has a reference to the stream, this allows it to be severed.
	prioritizer *streamPrioritizer

	// perRPCCredentials from the auth extension, or nil.
	perRPCCredentials credentials.PerRPCCredentials

	// telemetry are a copy of the exporter's telemetry settings
	telemetry component.TelemetrySettings

	// client uses the exporter's grpc.ClientConn.  this is
	// initially nil only set when ArrowStream() calls meaning the
	// endpoint recognizes OTLP+Arrow.
	client arrowpb.ArrowStreamService_ArrowStreamClient

	// toWrite is passes a batch from the sender to the stream writer, which
	// includes a dedicated channel for the response.
	toWrite chan writeItem

	// lock protects waiters.
	lock sync.Mutex

	// waiters is the response channel for each active batch.
	waiters map[string]chan error
}

// writeItem is passed from the sender (a pipeline consumer) to the
// stream writer, which is not bound by the sender's context.
type writeItem struct {
	// records is a ptrace.Traces, plog.Logs, or pmetric.Metrics
	records interface{}
	// md is the caller's metadata, derived from its context.
	md map[string]string
	// errCh is used by the stream reader to unblock the sender
	errCh chan error
}

// newStream constructs a stream
func newStream(
	producer arrowRecord.ProducerAPI,
	prioritizer *streamPrioritizer,
	telemetry component.TelemetrySettings,
	perRPCCredentials credentials.PerRPCCredentials,
) *Stream {
	return &Stream{
		producer:          producer,
		prioritizer:       prioritizer,
		perRPCCredentials: perRPCCredentials,
		telemetry:         telemetry,
		toWrite:           make(chan writeItem, 1),
		waiters:           map[string]chan error{},
	}
}

// setBatchChannel places a waiting consumer's batchID into the waiters map, where
// the stream reader may find it.
func (s *Stream) setBatchChannel(batchID string, errCh chan error) {
	s.lock.Lock()
	defer s.lock.Unlock()

	s.waiters[batchID] = errCh
}

// run blocks the calling goroutine while executing stream logic.  run
// will return when the reader and writer are finished.  errors will be logged.
func (s *Stream) run(bgctx context.Context, client arrowpb.ArrowStreamServiceClient, grpcOptions []grpc.CallOption) {
	ctx, cancel := context.WithCancel(bgctx)
	defer cancel()

	sc, err := client.ArrowStream(ctx, grpcOptions...)
	if err != nil {
		// Returning with stream.client == nil signals the
		// lack of an Arrow stream endpoint.  When all the
		// streams return with .client == nil, the ready
		// channel will be closed.
		//
		// Note: These are gRPC server internal errors and
		// will cause downgrade to standard OTLP.  These
		// cannot be simulated by connecting to a gRPC server
		// that does not support the ArrowStream service, with
		// or without the WaitForReady flag set.  In a real
		// gRPC server the first Unimplemented code is
		// generally delivered to the Recv() call below, so
		// this code path is not taken for an ordinary downgrade.
		//
		// TODO: a more graceful recovery strategy?
		s.telemetry.Logger.Error("cannot start arrow stream", zap.Error(err))
		return
	}
	// Setting .client != nil indicates that the endpoint was valid,
	// streaming may start.  When this stream finishes, it will be
	// restarted.
	s.client = sc

	// ww is used to wait for the writer.  Since we wait for the writer,
	// the writer's goroutine is not added to exporter waitgroup (e.wg).
	var ww sync.WaitGroup

	ww.Add(1)
	go func() {
		defer ww.Done()
		defer cancel()
		s.write(ctx)
	}()

	// the result from read() is processed after cancel and wait,
	// so we can set s.client = nil in case of a delayed Unimplemented.
	err = s.read(ctx)

	// Wait for the writer to ensure that all waiters are known.
	cancel()
	ww.Wait()

	if err != nil {
		// This branch is reached with an unimplemented status
		// with or without the WaitForReady flag.
		if status, ok := status.FromError(err); ok && status.Code() == codes.Unimplemented {
			// This (client == nil) signals the controller
			// to downgrade when all streams have returned
			// in that status.
			//
			// TODO: Note there are partial failure modes
			// that will continue to function in a
			// degraded mode, such as when half of the
			// streams are successful and half of streams
			// take this return path.  Design a graceful
			// recovery mechanism?
			s.client = nil
			s.telemetry.Logger.Info("arrow is not supported", zap.Error(err))
		} else if !errors.Is(err, io.EOF) && !errors.Is(err, context.Canceled) {
			// TODO: Should we add debug-level logs for EOF and Canceled?
			s.telemetry.Logger.Error("arrow recv", zap.Error(err))
		}
	}

	// The reader and writer have both finished; respond to any
	// outstanding waiters.
	for _, ch := range s.waiters {
		// Note: the top-level OTLP exporter will retry.
		ch <- ErrStreamRestarting
	}
}

// write repeatedly places this stream into the next-available queue, then
// performs a blocking send().  This returns when the data is in the write buffer,
// the caller waiting on its error channel.
func (s *Stream) write(ctx context.Context) {
	// headers are encoding using hpack, reusing a buffer on each call.
	var hdrsBuf bytes.Buffer
	hdrsEnc := hpack.NewEncoder(&hdrsBuf)

	for {
		// Note: this can't block b/c stream has capacity &
		// individual streams shut down synchronously.
		s.prioritizer.setReady(s)

		// this can block, and if the context is canceled we
		// wait for the reader to find this stream.
		var wri writeItem
		select {
		case wri = <-s.toWrite:
		case <-ctx.Done():
			// Because we did not <-stream.toWrite, there
			// is a potential sender race since the stream
			// is currently in the ready set.
			s.prioritizer.removeReady(s)
			return
		}
		// Note: For the two return statements below there is no potential
		// sender race because the stream is not available, as indicated by
		// the successful <-stream.toWrite.

		batch, err := s.encode(wri.records)
		if err != nil {
			// This is some kind of internal error.  We will restart the
			// stream and mark this record as a permanent one.
			wri.errCh <- consumererror.NewPermanent(err)
			s.telemetry.Logger.Error("arrow encode", zap.Error(err))
			return
		}

		// Optionally include outgoing metadata, if present.
		if len(wri.md) != 0 {
			hdrsBuf.Reset()
			for key, val := range wri.md {
				err := hdrsEnc.WriteField(hpack.HeaderField{
					Name:  key,
					Value: val,
				})
				if err != nil {
					// This case is like the encode-failure case
					// above, we will restart the stream but consider
					// this a permenent error.
					wri.errCh <- consumererror.NewPermanent(err)
					s.telemetry.Logger.Error("hpack encode", zap.Error(err))
					return
				}
			}
			batch.Headers = hdrsBuf.Bytes()
		}

		// Let the receiver knows what to look for.
		s.setBatchChannel(batch.BatchId, wri.errCh)

		if err := s.client.Send(batch); err != nil {
			// The error will be sent to errCh during cleanup for this stream.
			// Note there are common cases like EOF and Canceled that we may
			// wish to suppress in the logs if they become a problem.
			s.telemetry.Logger.Error("arrow send", zap.Error(err))
			return
		}
	}
}

// read repeatedly reads a batch status and releases the consumers waiting for
// a response.
func (s *Stream) read(_ context.Context) error {
	// Note we do not use the context, the stream context might
	// cancel a call to Recv() but the call to processBatchStatus
	// is non-blocking.
	for {
		resp, err := s.client.Recv()
		if err != nil {
			return err
		}

		if err = s.processBatchStatus(resp.Statuses); err != nil {
			return err
		}
	}
}

// getSenderChannels takes the stream lock and removes the
// corresonding sender channel for each BatchId.  They are returned
// with the same index as the original status, for correlation.  Nil
// channels will be returned when there are errors locating the
// sender channel.
func (s *Stream) getSenderChannels(statuses []*arrowpb.StatusMessage) ([]chan error, error) {
	var err error

	fin := make([]chan error, len(statuses))

	s.lock.Lock()
	defer s.lock.Unlock()

	for idx, status := range statuses {
		ch, ok := s.waiters[status.BatchId]
		if !ok {
			// Will break the stream.
			err = multierr.Append(err, fmt.Errorf("unrecognized batch ID: %s", status.BatchId))
			continue
		}
		delete(s.waiters, status.BatchId)
		fin[idx] = ch
	}

	return fin, err
}

// processBatchStatus processes a single response from the server and unblocks the
// associated senders.
func (s *Stream) processBatchStatus(statuses []*arrowpb.StatusMessage) error {
	fin, ret := s.getSenderChannels(statuses)

	for idx, ch := range fin {
		if ch == nil {
			// In case getSenderChannels encounters a problem, the
			// channel is nil.
			continue
		}
		status := statuses[idx]

		if status.StatusCode == arrowpb.StatusCode_OK {
			ch <- nil
			continue
		}
		var err error
		switch status.ErrorCode {
		case arrowpb.ErrorCode_UNAVAILABLE:
			// TODO: translate retry information into the form
			// exporterhelper recognizes.
			err = fmt.Errorf("destination unavailable: %s: %s", status.BatchId, status.ErrorMessage)
		case arrowpb.ErrorCode_INVALID_ARGUMENT:
			err = consumererror.NewPermanent(
				fmt.Errorf("invalid argument: %s: %s", status.BatchId, status.ErrorMessage))
		default:
			base := fmt.Errorf("unexpected stream response: %s: %s", status.BatchId, status.ErrorMessage)
			err = consumererror.NewPermanent(base)

			// Will break the stream.
			ret = multierr.Append(ret, base)
		}
		ch <- err
	}
	return ret
}

// SendAndWait submits a batch of records to be encoded and sent.  Meanwhile, this
// goroutine waits on the incoming context or for the asynchronous response to be
// received by the stream reader.
func (s *Stream) SendAndWait(ctx context.Context, records interface{}) error {
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
	if s.perRPCCredentials != nil {
		var err error
		md, err = s.perRPCCredentials.GetRequestMetadata(ctx)
		if err != nil {
			return err
		}
	}

	s.toWrite <- writeItem{
		records: records,
		md:      md,
		errCh:   errCh,
	}

	// Note this ensures the caller's timeout is respected.
	select {
	case <-ctx.Done():
		return ctx.Err()
	case err := <-errCh:
		return err
	}
}

// encode produces the next batch of Arrow records.
func (s *Stream) encode(records interface{}) (_ *arrowpb.BatchArrowRecords, retErr error) {
	// Defensively, protect against panics in the Arrow producer function.
	defer func() {
		if err := recover(); err != nil {
			retErr = fmt.Errorf("panic in otel-arrow-adapter: %v", err)
		}
	}()
	var batch *arrowpb.BatchArrowRecords
	var err error
	switch data := records.(type) {
	case ptrace.Traces:
		batch, err = s.producer.BatchArrowRecordsFromTraces(data)
	case plog.Logs:
		batch, err = s.producer.BatchArrowRecordsFromLogs(data)
	case pmetric.Metrics:
		batch, err = s.producer.BatchArrowRecordsFromMetrics(data)
	default:
		return nil, fmt.Errorf("unsupported OTLP type: %T", records)
	}
	return batch, err
}
