// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
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

	// state tracks the work being handled by all streams.
	state []*streamWorkState
}

var _ streamPrioritizer = &loadPrioritizer{}

func newLoadPrioritizer(ctx context.Context, state []*streamWorkState) *loadPrioritizer {
	lp := &loadPrioritizer{
		done:  ctx.Done(),
		down:  make(chan struct{}),
		input: make(chan writeItem), // @@@ ?
		state: state,
	}

	// Note that multiple intermediate goroutines are created.
	// One is sufficient, but streams `toWrite` channels are
	// limited size and eventually we have to block.
	for i := 0; i < max(1, len(state)/2); i++ {
		go lp.run()
	}
	return lp
}

func (lp *loadPrioritizer) downgrade() {
	close(lp.down)
}

func (lp *loadPrioritizer) sendOne(item writeItem) {
	writeCh := lp.streamFor(item).toWrite
	select {
	case writeCh <- item:
	case <-lp.done:
		item.errCh <- ErrStreamRestarting
	}
}

func (lp *loadPrioritizer) run() {
	for {
		select {
		case <-lp.done:
			return
		case item := <-lp.input:
			lp.sendOne(item)
		}
	}
}

func (lp *loadPrioritizer) sendAndWait(wri writeItem) error {
	select {
	case <-lp.done:
		return ErrStreamRestarting
	case <-wri.parent.Done():
		return context.Canceled
	case lp.input <- wri:
		return wri.waitForWrite(lp.done)
	}
}

func (lp *loadPrioritizer) nextWriter(ctx context.Context) (streamWriter, error) {
	// @@@ Say why not using <-done case
	select {
	case <-lp.down:
		return nil, nil
	default:
		return lp, nil
	}
}

func (lp *loadPrioritizer) streamFor(wri writeItem) *streamWorkState {
	if len(lp.state) == 1 {
		return lp.state[0]
	}
	var pick [2]*streamWorkState
	cnt := 0
	for _, sws := range lp.state {
		pick[cnt] = sws
		if cnt++; cnt == 2 {
			break
		}
	}
	l0 := pick[0].count()
	l1 := pick[1].count()

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
