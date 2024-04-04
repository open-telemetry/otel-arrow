// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var ErrStreamRestarting = status.Error(codes.Aborted, "stream is restarting")

// streamPrioritizer is an interface for prioritizing multiple
// streams.
type streamPrioritizer interface {
	nextWriter(context.Context) (streamWriter, error)
	downgrade()

	// On every stream
	setAvailable(*Stream)
	unsetAvailable(*Stream)

	// On every send/recv pair
	setReady(*Stream)
	unsetReady(*Stream)
}

// streamWriter is the caller's interface to a stream.
type streamWriter interface {
	// streamWrite is called to begin a write.  After completing
	// the call, wait on writeItem.errCh for the response.
	streamWrite(writeItem) error
}

// fifoPrioritizer is a prioritizer that selects the next stream to write.
// It is the simplest prioritizer implementation.
type fifoPrioritizer struct {
	// done corresponds with the background context Done channel.
	done <-chan struct{}

	// channel will be closed to downgrade to standard OTLP,
	// otherwise it returns the first-available.
	channel chan *Stream
}

var _ streamPrioritizer = &fifoPrioritizer{}

func newStreamPrioritizer(bgctx context.Context, numStreams int) streamPrioritizer {
	// TODO: More options @@@
	//return newFifoPrioritizer(bgctx, numStreams)
	return newLoadPrioritizer(bgctx, numStreams)
}

// newFifoPrioritizer constructs a channel-based first-available prioritizer.
func newFifoPrioritizer(bgctx context.Context, numStreams int) *fifoPrioritizer {
	return &fifoPrioritizer{
		done:    bgctx.Done(),
		channel: make(chan *Stream, numStreams),
	}
}

// downgrade indicates that streams are never going to be ready.  Note
// the caller is required to ensure that setReady() and unsetReady()
// cannot be called concurrently; this is done by waiting for
// Stream.writeStream() calls to return before downgrading.
func (fp *fifoPrioritizer) downgrade() {
	close(fp.channel)
}

// nextWriter returns the first-available stream.
func (fp *fifoPrioritizer) nextWriter(ctx context.Context) (streamWriter, error) {
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case stream := <-fp.channel:
		if stream == nil {
			return nil, nil
		}
		return stream, nil
	}
}

// setAvailable is a no-op for fifoPrioritizer, see setReady.
func (fp *fifoPrioritizer) setAvailable(stream *Stream) {
}

// unsetAvailable is a no-op for fifoPrioritizer, see unsetReady.
func (fp *fifoPrioritizer) unsetAvailable(stream *Stream) {
}

// setReady marks this stream ready for use.
func (fp *fifoPrioritizer) setReady(stream *Stream) {
	// Note: downgrade() can't be called concurrently.
	fp.channel <- stream
}

// unsetReady removes this stream from the ready set, used in cases
// where the stream has broken unexpectedly.
func (fp *fifoPrioritizer) unsetReady(stream *Stream) {
	// Note: downgrade() can't be called concurrently.
	for {
		// Searching for this stream to get it out of the ready queue.
		select {
		case <-fp.done:
			// Shutdown case
			return
		case alternate := <-fp.channel:
			if alternate == stream {
				// Success: removed from ready queue.
				return
			}
			fp.channel <- alternate
		case wri := <-stream.toWrite:
			// A consumer got us first, means this stream has been removed
			// from the ready queue.
			//
			// Note: the top-level OTLP exporter will retry.
			wri.errCh <- ErrStreamRestarting
			return
		}
	}
}
