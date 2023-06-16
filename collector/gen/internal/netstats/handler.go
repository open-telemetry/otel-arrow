// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package netstats // import "github.com/f5/otel-arrow-adapter/collector/gen/internal/netstats"

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
