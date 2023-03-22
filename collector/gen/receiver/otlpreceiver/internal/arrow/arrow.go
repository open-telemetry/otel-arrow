// Copyright  The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package arrow // import "github.com/f5/otel-arrow-adapter/collector/gen/receiver/otlpreceiver/internal/arrow"

import (
	"context"
	"errors"
	"fmt"
	"io"
	"strings"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrowRecord "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
	"go.uber.org/zap"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/consumer/consumererror"
	"go.opentelemetry.io/collector/extension/auth"
	"go.opentelemetry.io/collector/obsreport"
	"go.opentelemetry.io/collector/receiver"
)

const (
	receiverTransport   = "otlp-arrow"
	hpackMaxDynamicSize = 4096
)

var (
	ErrNoMetricsConsumer   = fmt.Errorf("no metrics consumer")
	ErrNoLogsConsumer      = fmt.Errorf("no logs consumer")
	ErrNoTracesConsumer    = fmt.Errorf("no traces consumer")
	ErrUnrecognizedPayload = fmt.Errorf("unrecognized OTLP payload")
)

type Consumers interface {
	Traces() consumer.Traces
	Metrics() consumer.Metrics
	Logs() consumer.Logs
}

type Receiver struct {
	Consumers
	arrowpb.UnimplementedArrowStreamServiceServer

	telemetry   component.TelemetrySettings
	obsrecv     *obsreport.Receiver
	gsettings   *configgrpc.GRPCServerSettings
	authServer  auth.Server
	newConsumer func() arrowRecord.ConsumerAPI
}

// New creates a new Receiver reference.
func New(
	id component.ID,
	cs Consumers,
	set receiver.CreateSettings,
	gsettings *configgrpc.GRPCServerSettings,
	authServer auth.Server,
	newConsumer func() arrowRecord.ConsumerAPI,
) (*Receiver, error) {
	obs, err := obsreport.NewReceiver(obsreport.ReceiverSettings{
		ReceiverID:             id,
		Transport:              receiverTransport,
		ReceiverCreateSettings: set,
	})
	if err != nil {
		return nil, err
	}
	return &Receiver{
		Consumers:   cs,
		obsrecv:     obs,
		telemetry:   set.TelemetrySettings,
		authServer:  authServer,
		newConsumer: newConsumer,
		gsettings:   gsettings,
	}, nil
}

// headerReceiver contains the state necessary to decode per-request metadata
// from an arrow stream.
type headerReceiver struct {
	// decoder maintains state across the stream.
	decoder *hpack.Decoder

	// includeMetadata as configured by gRPC settings.
	includeMetadata bool

	// hasAuthServer indicates that headers must be produced
	// independent of includeMetadata.
	hasAuthServer bool

	// client connection info from the stream context, (optionally
	// if includeMetadata) to be extended with per-request metadata.
	connInfo client.Info

	// streamHdrs was translated from the incoming context, will be
	// merged with per-request metadata.  Note that the contents of
	// this map are equivalent to connInfo.Metadata, however that
	// library does not let us iterate over the map so we recalculate
	// this from the gRPC incoming stream context.
	streamHdrs map[string][]string

	// tmpHdrs is used by the decoder's emit function during Write.
	tmpHdrs map[string][]string
}

func newHeaderReceiver(streamCtx context.Context, as auth.Server, includeMetadata bool) *headerReceiver {
	hr := &headerReceiver{
		includeMetadata: includeMetadata,
		hasAuthServer:   as != nil,
		connInfo:        client.FromContext(streamCtx),
	}

	// Note that we capture the incoming context if there is an
	// Auth plugin configured or includeMetadata is set.
	if hr.includeMetadata || hr.hasAuthServer {
		if smd, ok := metadata.FromIncomingContext(streamCtx); ok {
			hr.streamHdrs = smd
		}
	}

	// Note the hpack decoder supports additional protections,
	// such as SetMaxStringLength(), but as we already have limits
	// on stream request size, this seems unnecessary.
	hr.decoder = hpack.NewDecoder(hpackMaxDynamicSize, hr.tmpHdrsAppend)

	return hr
}

// combineHeaders calculates per-request Metadata by combining the stream's
// client.Info with additional key:values associated with the arrow batch.
func (h *headerReceiver) combineHeaders(ctx context.Context, hdrsBytes []byte) (context.Context, map[string][]string, error) {
	if len(hdrsBytes) == 0 && len(h.streamHdrs) == 0 {
		return ctx, nil, nil
	}

	if len(hdrsBytes) == 0 {
		return h.newContext(ctx, h.streamHdrs), h.streamHdrs, nil
	}

	// Note that we will parse the headers even if they are not
	// used, to check for validity.  tmpHdrsAppend() will skip
	// modifying tmpHdrs if it is nil.
	h.tmpHdrs = nil

	needMergedHeaders := h.includeMetadata || h.hasAuthServer

	// If headers are being merged, allocate a new map.
	if needMergedHeaders {
		h.tmpHdrs = map[string][]string{}
	}

	// Write calls the emitFunc, appending directly into `tmpHdrs`.
	if _, err := h.decoder.Write(hdrsBytes); err != nil {
		return ctx, nil, err
	}

	if needMergedHeaders {
		// Add streamHdrs that were not carried in the per-request headers.
		for k, v := range h.streamHdrs {
			// Note: This is done after the per-request metadata is defined
			// in recognition of a potential for duplicated values stemming
			// from the Arrow exporter's independent call to the Auth
			// extension's GetRequestMetadata().  This paired with the
			// headersetter's return of empty-string values means, we would
			// end up with an empty-string element for any headersetter
			// `from_context` rules b/c the stream uses background context.
			// This allows static headers through.
			//
			// See https://github.com/open-telemetry/opentelemetry-collector/issues/6965
			lk := strings.ToLower(k)
			if _, ok := h.tmpHdrs[lk]; !ok {
				h.tmpHdrs[lk] = v
			}
		}
	}

	// Release the temporary copy used in emitFunc().
	newHdrs := h.tmpHdrs
	h.tmpHdrs = nil

	// Note: newHdrs is passed to the Auth plugin.  Whether
	// newHdrs is set in the context depends on h.includeMetadata.
	return h.newContext(ctx, newHdrs), newHdrs, nil
}

// tmpHdrsAppend appends to tmpHdrs, from decoder's emit function.
func (h *headerReceiver) tmpHdrsAppend(hf hpack.HeaderField) {
	if h.tmpHdrs != nil {
		// We force strings.ToLower to ensure consistency.  gRPC itself
		// does this and would do the same.
		hn := strings.ToLower(hf.Name)
		h.tmpHdrs[hn] = append(h.tmpHdrs[hn], hf.Value)
	}
}

func (h *headerReceiver) newContext(ctx context.Context, hdrs map[string][]string) context.Context {
	// Retain the Addr/Auth of the stream connection, update the
	// per-request metadata from the Arrow batch.
	var md client.Metadata
	if h.includeMetadata && hdrs != nil {
		md = client.NewMetadata(hdrs)
	}
	return client.NewContext(ctx, client.Info{
		Addr:     h.connInfo.Addr,
		Auth:     h.connInfo.Auth,
		Metadata: md,
	})
}

// logStreamError decides whether to log an error to the console.
func (r *Receiver) logStreamError(err error) {
	status, ok := status.FromError(err)

	if ok {
		switch status.Code() {
		case codes.Canceled:
			r.telemetry.Logger.Debug("arrow stream canceled")
		default:
			r.telemetry.Logger.Error("arrow stream error",
				zap.Uint32("code", uint32(status.Code())),
				zap.String("message", status.Message()),
			)
		}
		return
	}

	isEOF := errors.Is(err, io.EOF)
	isCanceled := errors.Is(err, context.Canceled)

	if !isEOF && !isCanceled {
		r.telemetry.Logger.Error("arrow stream error", zap.Error(err))
	} else if isEOF {
		r.telemetry.Logger.Debug("arrow stream end")
	} else {
		r.telemetry.Logger.Debug("arrow stream canceled")
	}
}

func (r *Receiver) ArrowStream(serverStream arrowpb.ArrowStreamService_ArrowStreamServer) error {
	streamCtx := serverStream.Context()
	ac := r.newConsumer()
	hrcv := newHeaderReceiver(serverStream.Context(), r.authServer, r.gsettings.IncludeMetadata)

	defer func() {
		if err := ac.Close(); err != nil {
			r.telemetry.Logger.Error("arrow stream close", zap.Error(err))
		}
	}()

	for {
		// Receive a batch corresponding with one ptrace.Traces, pmetric.Metrics,
		// or plog.Logs item.
		req, err := serverStream.Recv()

		if err != nil {
			r.logStreamError(err)
			return err
		}

		// Check for optional headers and set the incoming context.
		thisCtx, authHdrs, err := hrcv.combineHeaders(streamCtx, req.GetHeaders())
		if err != nil {
			// Failing to parse the incoming headers breaks the stream.
			r.telemetry.Logger.Error("arrow metadata error", zap.Error(err))
			return err
		}

		var authErr error
		if r.authServer != nil {
			var newCtx context.Context
			if newCtx, err = r.authServer.Authenticate(thisCtx, authHdrs); err != nil {
				authErr = err
			} else {
				thisCtx = newCtx
			}
		}

		// Process records: an error in this code path does
		// not necessarily break the stream.
		if authErr != nil {
			err = authErr
		} else {
			err = r.processRecords(thisCtx, ac, req)
		}

		// Note: Statuses can be batched: TODO: should we?
		resp := &arrowpb.BatchStatus{}
		status := &arrowpb.StatusMessage{
			BatchId: req.GetBatchId(),
		}
		if err == nil {
			status.StatusCode = arrowpb.StatusCode_OK
		} else {
			status.StatusCode = arrowpb.StatusCode_ERROR
			status.ErrorMessage = err.Error()

			if consumererror.IsPermanent(err) {
				r.telemetry.Logger.Error("arrow data error", zap.Error(err))
				status.ErrorCode = arrowpb.ErrorCode_INVALID_ARGUMENT
			} else {
				r.telemetry.Logger.Debug("arrow consumer error", zap.Error(err))
				status.ErrorCode = arrowpb.ErrorCode_UNAVAILABLE
			}
		}
		resp.Statuses = append(resp.Statuses, status)

		err = serverStream.Send(resp)
		if err != nil {
			r.logStreamError(err)
			return err
		}
	}
}

// processRecords returns an error and a boolean indicating whether
// the error (true) was from processing the data (i.e., invalid
// argument) or (false) from the consuming pipeline.  The boolean is
// not used when success (nil error) is returned.
func (r *Receiver) processRecords(ctx context.Context, arrowConsumer arrowRecord.ConsumerAPI, records *arrowpb.BatchArrowRecords) error {
	payloads := records.GetOtlpArrowPayloads()
	if len(payloads) == 0 {
		return nil
	}
	// TODO: Use the obsreport object to instrument (somehow)
	switch payloads[0].Type {
	case arrowpb.OtlpArrowPayloadType_METRICS:
		otlp, err := arrowConsumer.MetricsFrom(records)
		if err != nil {
			return consumererror.NewPermanent(err)
		}
		for _, metrics := range otlp {
			err = r.Metrics().ConsumeMetrics(ctx, metrics)
			if err != nil {
				return err
			}
		}

	case arrowpb.OtlpArrowPayloadType_LOGS:
		otlp, err := arrowConsumer.LogsFrom(records)
		if err != nil {
			return consumererror.NewPermanent(err)
		}

		for _, logs := range otlp {
			err = r.Logs().ConsumeLogs(ctx, logs)
			if err != nil {
				return err
			}
		}

	case arrowpb.OtlpArrowPayloadType_SPANS:
		otlp, err := arrowConsumer.TracesFrom(records)
		if err != nil {
			return consumererror.NewPermanent(err)
		}

		for _, traces := range otlp {
			err = r.Traces().ConsumeTraces(ctx, traces)
			if err != nil {
				return err
			}
		}

	default:
		return ErrUnrecognizedPayload
	}
	return nil
}
