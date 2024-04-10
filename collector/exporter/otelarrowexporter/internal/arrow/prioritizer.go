// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"

import (
	"context"
	"fmt"

	"go.opentelemetry.io/collector/component"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var ErrStreamRestarting = status.Error(codes.Aborted, "stream is restarting")

type PrioritizerName string

var _ component.ConfigValidator = PrioritizerName("")

const (
	FifoPrioritizer      PrioritizerName = "fifo"
	BestOfTwoPrioritizer PrioritizerName = "bestoftwo"
	DefaultPrioritizer   PrioritizerName = FifoPrioritizer
	unsetPrioritizer     PrioritizerName = ""
)

// streamPrioritizer is an interface for prioritizing multiple
// streams.
type streamPrioritizer interface {
	nextWriter(context.Context) (streamWriter, error)
	downgrade()
}

// streamWriter is the caller's interface to a stream.
type streamWriter interface {
	// sendAndWait is called to begin a write.  After completing
	// the call, wait on writeItem.errCh for the response.
	sendAndWait(writeItem) error
}

func newStreamPrioritizer(ctx context.Context, name PrioritizerName, numStreams int) (streamPrioritizer, []*streamWorkState) {
	switch name {
	case BestOfTwoPrioritizer:
		return newBestOfTwoPrioritizer(ctx, numStreams)
	default:
		return newFifoPrioritizer(ctx, numStreams)
	}
}

func (p PrioritizerName) Validate() error {
	switch p {
	case FifoPrioritizer, BestOfTwoPrioritizer, unsetPrioritizer:
		return nil
	}
	return fmt.Errorf("unrecognized prioritizer: %q", string(p))
}
