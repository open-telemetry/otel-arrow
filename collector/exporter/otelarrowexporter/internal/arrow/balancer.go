// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"sync"
)

// loadPrioritizer is a prioritizer that selects a less-loaded stream to write.
// https://smallrye.io/smallrye-stork/1.1.1/load-balancer/power-of-two-choices/
type loadPrioritizer struct {
	// done corresponds with the background context Done channel.
	done <-chan struct{}

	// down is closed to indicate a downgrade.
	down chan struct{}

	// input from the exporter; never closed.  callers should
	// finish before shutting down the exporter, otherwise they
	// will block indefinitely, however this is handled
	// automatically by the graph package, which shuts down in
	// topological order.
	input chan writeItem

	// lock protects avail
	lock sync.Mutex

	// cond is used to wait while available streams is empty.
	cond *sync.Cond

	// avail tracks the streams that are available.
	avail map[*Stream]struct{}
}

func newLoadPrioritizer(ctx context.Context, numStreams int) *loadPrioritizer {
	lp := &loadPrioritizer{
		done:  ctx.Done(),
		down:  make(chan struct{}),
		input: make(chan writeItem, numStreams),
		avail: map[*Stream]struct{}{},
	}
	lp.cond = sync.NewCond(&lp.lock)

	// Note that multiple intermediate goroutines are created.
	// One is sufficient, but streams `toWrite` channels are
	// limited size and eventually we have to block.
	for i := 0; i < max(1, numStreams/2); i++ {
		go lp.run()
	}
	return lp
}

func (lp *loadPrioritizer) run() {
	for {
		select {
		case <-lp.done:
			return
		case item := <-lp.input:
			lp.streamFor(item).toWrite <- item
		}
	}
}

func (lp *loadPrioritizer) downgrade() {
	close(lp.down)
}

func (lp *loadPrioritizer) streamWrite(wri writeItem) error {
	lp.input <- wri
	return nil
}

func (lp *loadPrioritizer) nextWriter(ctx context.Context) (streamWriter, error) {
	select {
	case <-lp.down:
		return nil, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
		return lp, nil
	}
}

func (lp *loadPrioritizer) streamFor(wri writeItem) *Stream {
	lp.lock.Lock()
	defer lp.lock.Unlock()

	for len(lp.avail) == 0 {
		lp.cond.Wait()
	}
	if len(lp.avail) == 1 {
		for stream := range lp.avail {
			return stream
		}
	}
	var pick [2]*Stream
	cnt := 0
	for stream := range lp.avail {
		pick[cnt] = stream
		if cnt++; cnt == 2 {
			break
		}
	}
	l0 := pick[0].outstandingRequests.Load() + uint32(len(pick[0].toWrite))
	l1 := pick[1].outstandingRequests.Load() + uint32(len(pick[1].toWrite))

	// Choose two at random, then pick the one with less load.
	if l0 < l1 {
		return pick[0]
	}
	return pick[1]
}

func (lp *loadPrioritizer) setReady(stream *Stream) {
}

func (lp *loadPrioritizer) unsetReady(stream *Stream) {
}

func (lp *loadPrioritizer) setAvailable(stream *Stream) {
	lp.lock.Lock()
	defer lp.lock.Unlock()

	lp.avail[stream] = struct{}{}
	lp.cond.Broadcast()
}

func (lp *loadPrioritizer) unsetAvailable(stream *Stream) {
	lp.lock.Lock()
	defer lp.lock.Unlock()

	// Note that when we unset availability, there may still
	// be one or more items pending on the `toWrite` channel.
	// The toWrite channel is transferred to the replacement
	// stream for this reason.

	delete(lp.avail, stream)
}
