// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package netstats // import "github.com/open-telemetry/otel-arrow/collector/netstats"

import (
	"context"
	"fmt"
	"strconv"

	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/stats"
)

type uncompressedSizeContext struct{} // value: string
type netstatsContext struct{}         // value: int

// TagRPC implements grpc/stats.Handler
func (rep *NetworkReporter) TagRPC(ctx context.Context, s *stats.RPCTagInfo) context.Context {
	return context.WithValue(ctx, netstatsContext{}, s.FullMethodName)
}

// UncompressedSizeContext installs a context that knows the true
// uncompressed size of the data.
func UncompressedSizeContext(ctx context.Context, size int) context.Context {
	ctx = internalUncompressedSizeContext(ctx, size)
	return metadata.AppendToOutgoingContext(ctx, "otlp-uncompressed-size", fmt.Sprint(size))
}

func internalUncompressedSizeContext(ctx context.Context, size int) context.Context {
	return context.WithValue(ctx, uncompressedSizeContext{}, size)
}

// HandleRPC implements grpc/stats.Handler
func (rep *NetworkReporter) HandleRPC(ctx context.Context, rs stats.RPCStats) {
	switch rs.(type) {
	case *stats.Begin, *stats.OutHeader, *stats.OutTrailer:
		return
	}
	method := "unknown"
	if name := ctx.Value(netstatsContext{}); name != nil {
		method = name.(string)
	}
	switch s := rs.(type) {
	case *stats.InHeader:
		var ss SizesStruct
		ss.Method = method
		ss.WireLength = int64(s.WireLength)

		// The exporter should have included the uncompressed size,
		// use it here in the receiver.
		if sentSize := s.Header.Get("otlp-uncompressed-size"); len(sentSize) == 1 {
			if uncompSize, err := strconv.ParseUint(sentSize[0], 10, 64); err == nil && uncompSize > 0 {
				ss.Length = int64(uncompSize)
			}
		}
		rep.CountReceive(ctx, ss)

	case *stats.InTrailer:
		var ss SizesStruct
		ss.Method = method
		ss.WireLength = int64(s.WireLength)
		rep.CountReceive(ctx, ss)

	case *stats.InPayload:
		usize := 0
		if uncompSize := ctx.Value(uncompressedSizeContext{}); uncompSize != nil {
			usize = uncompSize.(int)
		}

		var ss SizesStruct
		ss.Method = method
		ss.Length = int64(usize)
		ss.WireLength = int64(s.WireLength)
		ss.WireIsPayload = true
		rep.CountReceive(ctx, ss)

	case *stats.OutPayload:
		usize := 0
		if uncompSize := ctx.Value(uncompressedSizeContext{}); uncompSize != nil {
			usize = uncompSize.(int)
		}
		var ss SizesStruct
		ss.Method = method
		ss.Length = int64(usize)
		ss.WireLength = int64(s.WireLength)
		ss.WireIsPayload = true
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
