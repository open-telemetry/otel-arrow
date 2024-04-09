// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"fmt"
)

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

// newFifoPrioritizer constructs a channel-based first-available prioritizer.
func newFifoPrioritizer(ctx context.Context, state []*streamWorkState) *fifoPrioritizer {
	return &fifoPrioritizer{
		done:    ctx.Done(),
		channel: make(chan *Stream, len(state)),
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
	case <-fp.done:
		// Shutdown
		return nil, context.Canceled
	case <-ctx.Done():
		// Caller context timeout
		return nil, ctx.Err()
	case stream := <-fp.channel:
		if stream == nil {
			// Downgrade
			return nil, nil
		}
		return &streamSender{
			stream: stream,
		}, nil
	}
}

type streamSender struct {
	stream *Stream
}

var _ streamWriter = &streamSender{}

func (ss *streamSender) sendAndWait(wri writeItem) error {
	return ss.stream.sendAndWait(wri)
}

// setReady marks this stream ready for use.
func (fp *fifoPrioritizer) setReady(stream *Stream) {
	// Note: downgrade() can't be called concurrently.
	fmt.Println("SETREADY")
	fp.channel <- stream
}

// unsetReady removes this stream from the ready set, used in cases
// where the stream has broken unexpectedly.
func (fp *fifoPrioritizer) unsetReady(stream *Stream) {
	// Note: downgrade() can't be called concurrently.
	for {
		fmt.Println("UNSET START")
		// Searching for this stream to get it out of the ready queue.
		select {
		case <-fp.done:
			fmt.Println("UNSET CANCEL")
			// Shutdown case
			return
		case alternate := <-fp.channel:
			fmt.Println("UNSET ALTERNATE")
			if alternate == stream {
				// Success: removed from ready queue.
				return
			}
			fp.channel <- alternate
		case wri := <-stream.workState.toWrite:
			fmt.Println("UNSET WORK")
			// A consumer got us first, means this stream has been removed
			// from the ready queue.
			//
			// Note: the top-level OTLP exporter will retry.
			wri.errCh <- ErrStreamRestarting
			fmt.Println("UNSET BYEEE")
			return
		}
	}
}
