// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"runtime"
	"sort"
)

// bestOfNPrioritizer is a prioritizer that selects a less-loaded stream to write.
// https://smallrye.io/smallrye-stork/1.1.1/load-balancer/power-of-two-choices/
type bestOfNPrioritizer struct {
	doneCancel

	// input from the pipeline, as processed data with headers and
	// a return channel for the result.  This channel is never
	// closed and is buffered.  At shutdown, items of telemetry can
	// be left in this channel, but users are expected to complete
	// their requests before calling shutdown (and the collector's
	// graph package ensures this).
	input chan writeItem

	// state tracks the work being handled by all streams.
	state map[*streamWorkState]struct{}

	// N is the number of streams to consder in each decision.
	N int

	// loadFunc is the load function.
	loadFunc loadFunc
}

type loadFunc func(*streamWorkState) float64

type streamSorter struct {
	work *streamWorkState
	load float64
}

var _ streamPrioritizer = &bestOfNPrioritizer{}

func newBestOfNPrioritizer(dc doneCancel, N, numStreams int, lf loadFunc) (*bestOfNPrioritizer, []*streamWorkState) {
	var sws []*streamWorkState
	state := map[*streamWorkState]struct{}{}

	for i := 0; i < numStreams; i++ {
		ws := &streamWorkState{
			waiters: map[int64]chan<- error{},
			toWrite: make(chan writeItem, 1),
		}

		sws = append(sws, ws)
		state[ws] = struct{}{}
	}

	lp := &bestOfNPrioritizer{
		doneCancel: dc,
		input:      make(chan writeItem, runtime.NumCPU()),
		state:      state,
		N:          N,
		loadFunc:   lf,
	}

	for i := 0; i < len(lp.state); i++ {
		// TODO It's not clear if/when the the prioritizer can
		// become a bottleneck.
		go lp.run()
	}

	return lp, sws
}

func (lp *bestOfNPrioritizer) downgrade(ctx context.Context) {
	for ws := range lp.state {
		go drain(ws.toWrite, ctx.Done())
	}
}

func (lp *bestOfNPrioritizer) sendOne(item writeItem, tmp []streamSorter) {
	stream := lp.streamFor(item, tmp)
	writeCh := stream.toWrite
	select {
	case writeCh <- item:
		return

	case <-lp.done:
		// All other cases: signal restart.
	}
	item.errCh <- ErrStreamRestarting
}

func (lp *bestOfNPrioritizer) run() {
	tmp := make([]streamSorter, lp.N)
	for {
		select {
		case <-lp.done:
			return
		case item := <-lp.input:
			lp.sendOne(item, tmp)
		}
	}
}

// sendAndWait implements streamWriter
func (lp *bestOfNPrioritizer) sendAndWait(ctx context.Context, errCh <-chan error, wri writeItem) error {
	select {
	case <-lp.done:
		return ErrStreamRestarting
	case <-ctx.Done():
		return context.Canceled
	case lp.input <- wri:
		return waitForWrite(ctx, errCh, lp.done)
	}
}

func (lp *bestOfNPrioritizer) nextWriter(ctx context.Context) (streamWriter, error) {
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

func (lp *bestOfNPrioritizer) streamFor(_ writeItem, tmp []streamSorter) *streamWorkState {
	cnt := 0
	for sws := range lp.state {
		// TODO: skip channels w/ a pending item (maybe)
		tmp[cnt].work = sws
		tmp[cnt].load = lp.loadFunc(sws)
		if cnt++; cnt == lp.N {
			break
		}
	}
	sort.Slice(tmp, func(i, j int) bool {
		return tmp[i].load < tmp[j].load
	})
	return tmp[0].work
}
