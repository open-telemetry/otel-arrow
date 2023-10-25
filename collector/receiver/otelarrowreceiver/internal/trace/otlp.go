// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package trace // import "github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver/internal/trace"

import (
	"context"

	"github.com/open-telemetry/otel-arrow/collector/netstats"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
	"go.opentelemetry.io/collector/receiver/receiverhelper"
	tracepb "go.opentelemetry.io/proto/otlp/collector/trace/v1"
)

const dataFormatProtobuf = "protobuf"

// Receiver is the type used to handle spans from OpenTelemetry exporters.
type Receiver struct {
	ptraceotlp.UnimplementedGRPCServer
	nextConsumer consumer.Traces
	obsrecv      *receiverhelper.ObsReport
}

// New creates a new Receiver reference.
func New(nextConsumer consumer.Traces, obsrecv *receiverhelper.ObsReport) *Receiver {
	return &Receiver{
		nextConsumer: nextConsumer,
		obsrecv:      obsrecv,
	}
}

var methodName = netstats.GRPCUnaryMethodName(tracepb.TraceService_ServiceDesc, tracepb.TraceService_ServiceDesc.Methods[0])

// Export implements the service Export traces func.
func (r *Receiver) Export(ctx context.Context, req ptraceotlp.ExportRequest) (ptraceotlp.ExportResponse, error) {
	td := req.Traces()

	// We need to ensure that it propagates the receiver name as a tag
	numSpans := td.SpanCount()
	if numSpans == 0 {
		return ptraceotlp.NewExportResponse(), nil
	}

	ctx = r.obsrecv.StartTracesOp(ctx)
	err := r.nextConsumer.ConsumeTraces(ctx, td)
	r.obsrecv.EndTracesOp(ctx, dataFormatProtobuf, numSpans, err)

	// Note: no call to CountSend() here because we know the uncompressed
	// size of this object is zero until any fields are set, and they're not.
	return ptraceotlp.NewExportResponse(), err
}

func (r *Receiver) Consumer() consumer.Traces {
	return r.nextConsumer
}
