// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"runtime"
)

// bestOfTwoPrioritizer is a prioritizer that selects a less-loaded stream to write.
// https://smallrye.io/smallrye-stork/1.1.1/load-balancer/power-of-two-choices/
type bestOfTwoPrioritizer struct {
	// done corresponds with the background context Done channel.
	// the prioritizer will stop background activity when this
	// channel is closed.
	done <-chan struct{}

	// down is closed to indicate a downgrade.  Note that `done` and
	// `down` have similar effect, but that one signal is internal and
	// one signal is external.
	down chan struct{}

	// input from the pipeline, as processed data with headers and
	// a return channel for the result.  This channel is never
	// closed and is buffered.  At shutdown, items of telemetry can
	// be left in this channel, but users are expected to complete
	// their requests before calling shutdown (and the collector's
	// graph package ensures this).
	input chan writeItem

	// state tracks the work being handled by all streams.
	state []*streamWorkState
}

var _ streamPrioritizer = &bestOfTwoPrioritizer{}

func newBestOfTwoPrioritizer(ctx context.Context, numStreams int) (*bestOfTwoPrioritizer, []*streamWorkState) {
	var state []*streamWorkState

	for i := 0; i < numStreams; i++ {
		ws := &streamWorkState{
			waiters: map[int64]chan error{},

			// Note: toWrite must be unbuffered.
			toWrite: make(chan writeItem),
		}

		state = append(state, ws)
	}

	lp := &bestOfTwoPrioritizer{
		done:  ctx.Done(),
		down:  make(chan struct{}),
		input: make(chan writeItem, runtime.NumCPU()),
		state: state,
	}

	for i := 0; i < max(1, len(lp.state)/2); i++ {
		go lp.run()
	}

	return lp, state
}

func (lp *bestOfTwoPrioritizer) downgrade() {
	close(lp.down)
}

func (lp *bestOfTwoPrioritizer) sendOne(item writeItem) {
	stream := lp.streamFor(item)
	writeCh := stream.toWrite
	select {
	case writeCh <- item:
		return

	case <-lp.down:
	case <-lp.done:
		// All other cases: signal restart.
	}
	item.errCh <- ErrStreamRestarting
}

func (lp *bestOfTwoPrioritizer) run() {
	for {
		select {
		case <-lp.done:
			return
		case item := <-lp.input:
			lp.sendOne(item)
		}
	}
}

// sendAndWait implements streamWriter
func (lp *bestOfTwoPrioritizer) sendAndWait(wri writeItem) error {
	select {
	case <-lp.down:
		return ErrStreamRestarting
	case <-lp.done:
		return ErrStreamRestarting
	case <-wri.parent.Done():
		return context.Canceled
	case lp.input <- wri:
		return wri.waitForWrite(lp.down)
	}
}

func (lp *bestOfTwoPrioritizer) nextWriter(ctx context.Context) (streamWriter, error) {
	select {
	case <-lp.down:
		// In case of downgrade, return nil to return into a
		// non-Arrow code path.
		return nil, nil
	default:
		// Fall through to sendAndWait().
		return lp, nil
	}
}

func (lp *bestOfTwoPrioritizer) streamFor(_ writeItem) *streamWorkState {
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
	l0 := pick[0].pendingRequests()
	l1 := pick[1].pendingRequests()

	// Choose two at random, then pick the one with less load.
	if l0 < l1 {
		return pick[0]
	}
	return pick[1]
}
