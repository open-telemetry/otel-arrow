// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package netstats // import "github.com/open-telemetry/otel-arrow/collector/internal/netstats"

import (
	"context"

	"google.golang.org/grpc/stats"
)

type netstatsContext struct{}

// TagRPC implements grpc/stats.Handler
func (rep *NetworkReporter) TagRPC(ctx context.Context, s *stats.RPCTagInfo) context.Context {
	return context.WithValue(ctx, netstatsContext{}, s.FullMethodName)
}

// HandleRPC implements grpc/stats.Handler
func (rep *NetworkReporter) HandleRPC(ctx context.Context, rs stats.RPCStats) {
	method := "unknown"
	if name := ctx.Value(netstatsContext{}); name != nil {
		method = name.(string)
	}
	switch s := rs.(type) {
	case *stats.Begin, *stats.OutHeader, *stats.OutTrailer, *stats.InHeader, *stats.InTrailer:
		// Note we have some info about header WireLength,
		// but intentionally not counting.

	case *stats.InPayload:
		var ss SizesStruct
		ss.Method = method
		ss.Length = int64(s.Length)
		ss.WireLength = int64(s.WireLength)
		rep.CountReceive(ctx, ss)

	case *stats.OutPayload:
		var ss SizesStruct
		ss.Method = method
		ss.Length = int64(s.Length)
		ss.WireLength = int64(s.WireLength)
		rep.CountSend(ctx, ss)
	}
}

// TagConn implements grpc/stats.Handler
func (rep *NetworkReporter) TagConn(ctx context.Context, _ *stats.ConnTagInfo) context.Context {
	return ctx
}

// HandleConn implements grpc/stats.Handler
func (rep *NetworkReporter) HandleConn(_ context.Context, _ stats.ConnStats) {
	// Note: ConnBegin and ConnEnd
}
