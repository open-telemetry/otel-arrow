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

	cancel context.CancelFunc

	// input from the pipeline, as processed data with headers and
	// a return channel for the result.  This channel is never
	// closed and is buffered.  At shutdown, items of telemetry can
	// be left in this channel, but users are expected to complete
	// their requests before calling shutdown (and the collector's
	// graph package ensures this).
	input chan writeItem

	// state tracks the work being handled by all streams.
	state map[*streamWorkState]struct{}
}

var _ streamPrioritizer = &bestOfTwoPrioritizer{}

func newBestOfTwoPrioritizer(ctx context.Context, numStreams int) (*bestOfTwoPrioritizer, []*streamWorkState) {
	var sws []*streamWorkState
	state := map[*streamWorkState]struct{}{}

	for i := 0; i < numStreams; i++ {
		ws := &streamWorkState{
			waiters: map[int64]chan error{},
			toWrite: make(chan writeItem, 1),
		}

		sws = append(sws, ws)
		state[ws] = struct{}{}
	}

	ctx, cancel := context.WithCancel(ctx)

	lp := &bestOfTwoPrioritizer{
		done:   ctx.Done(),
		cancel: cancel,
		input:  make(chan writeItem, runtime.NumCPU()),
		state:  state,
	}

	for i := 0; i < len(lp.state); i++ {
		go lp.run()
	}

	return lp, sws
}

func (lp *bestOfTwoPrioritizer) downgrade() {
	lp.cancel()

	for ws := range lp.state {
		go drain(ws.toWrite, lp.done)
	}
}

func (lp *bestOfTwoPrioritizer) sendOne(item writeItem) {
	stream := lp.streamFor(item)
	writeCh := stream.toWrite
	select {
	case writeCh <- item:
		return

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
func (lp *bestOfTwoPrioritizer) sendAndWait(ctx context.Context, wri writeItem) error {
	select {
	case <-lp.done:
		return ErrStreamRestarting
	case <-ctx.Done():
		return context.Canceled
	case lp.input <- wri:
		return wri.waitForWrite(ctx, lp.done)
	}
}

func (lp *bestOfTwoPrioritizer) nextWriter(ctx context.Context) (streamWriter, error) {
	select {
	case <-lp.done:
		// In case of downgrade, return nil to return into a
		// non-Arrow code path.
		return nil, nil
	default:
		// Fall through to sendAndWait().
		return lp, nil
	}
}

func (lp *bestOfTwoPrioritizer) streamFor(_ writeItem) *streamWorkState {
	var pick [2]*streamWorkState
	cnt := 0
	for sws := range lp.state {
		pick[cnt] = sws
		if cnt++; cnt == 2 {
			break
		}
	}
	if cnt == 1 {
		return pick[0]
	}
	l0 := pick[0].pendingRequests()
	l1 := pick[1].pendingRequests()

	// Choose two at random, then pick the one with less load.
	if l0 < l1 {
		return pick[0]
	}
	return pick[1]
}
