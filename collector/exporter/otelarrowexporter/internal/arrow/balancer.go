// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"math/rand"
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

	avail []*Stream
}

func newLoadPrioritizer(ctx context.Context, numStreams int) *loadPrioritizer {
	lp := &loadPrioritizer{
		done:  ctx.Done(),
		down:  make(chan struct{}),
		input: make(chan writeItem, numStreams),
		avail: make([]*Stream, 0, numStreams),
	}
	lp.cond = sync.NewCond(&lp.lock)
	go lp.run()
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
	num := len(lp.avail)
	if num == 1 {
		return lp.avail[0]
	}

	// Choose two at random, then pick the one with less load.
	a := rand.Intn(num)
	b := rand.Intn(num - 1)
	if b >= a {
		b++
	}
	if lp.avail[a].outstandingRequests.Load() < lp.avail[b].outstandingRequests.Load() {
		return lp.avail[a]
	}
	return lp.avail[b]
}

func (lp *loadPrioritizer) setReady(stream *Stream) {
}

func (lp *loadPrioritizer) unsetReady(stream *Stream) {
}

func (lp *loadPrioritizer) setAvailable(stream *Stream) {
	lp.lock.Lock()
	defer lp.lock.Unlock()

	lp.avail = append(lp.avail, stream)
	lp.cond.Signal()
}

func (lp *loadPrioritizer) unsetAvailable(stream *Stream) {
	lp.lock.Lock()
	defer lp.lock.Unlock()

	num := len(lp.avail)
	for idx, st := range lp.avail {
		if st == stream {
			lp.avail[idx], lp.avail[num-1] = lp.avail[num-1], lp.avail[idx]
			lp.avail = lp.avail[:num-1]
			break
		}
	}

	for len(stream.toWrite) > 0 {

	}
}
