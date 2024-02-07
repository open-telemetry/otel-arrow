// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package netstats // import "github.com/open-telemetry/otel-arrow/collector/netstats"

import (
	"context"

	"go.opentelemetry.io/otel/attribute"
	otelcodes "go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/trace"
	noopmetric "go.opentelemetry.io/otel/metric/noop"
	"go.uber.org/multierr"

	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/receiver"
)

const (
	// ExporterKey is an attribute name that identifies an
	// exporter component that produces internal metrics, logs,
	// and traces.
	ExporterKey = "exporter"

	// ReceiverKey is an attribute name that identifies an
	// receiver component that produces internal metrics, logs,
	// and traces.
	ReceiverKey = "receiver"

	// SentBytes is used to track bytes sent by exporters and receivers.
	SentBytes = "sent"

	// SentWireBytes is used to track bytes sent on the wire
	// (includes compression) by exporters and receivers.
	SentWireBytes = "sent_wire"

	// RecvBytes is used to track bytes received by exporters and receivers.
	RecvBytes = "recv"

	// RecvWireBytes is used to track bytes received on the wire
	// (includes compression) by exporters and receivers.
	RecvWireBytes = "recv_wire"

	// CompSize is used for compressed size histogram metrics.
	CompSize = "compressed_size"

	scopeName = "github.com/open-telemetry/otel-arrow/collector/netstats"
)

// NetworkReporter is a helper to add network-level observability to
// an exporter or receiver.
type NetworkReporter struct {
	isExporter    bool
	staticAttr    attribute.KeyValue
	sentBytes     metric.Int64Counter
	sentWireBytes metric.Int64Counter
	recvBytes     metric.Int64Counter
	recvWireBytes metric.Int64Counter
	compSizeHisto metric.Int64Histogram
}

// SizesStruct is used to pass uncompressed on-wire message lengths to
// the CountSend() and CountReceive() methods.
type SizesStruct struct {
	// Method refers to the gRPC method name
	Method string
	// Length is the uncompressed size
	Length int64
	// WireLength is compressed size
	WireLength int64
}

// Interface describes a *NetworkReporter or a Noop.
type Interface interface {
	// CountSend reports outbound bytes.
	CountSend(ctx context.Context, ss SizesStruct)

	// CountSend reports inbound bytes.
	CountReceive(ctx context.Context, ss SizesStruct)

	// SetSpanAttributes takes a context and adds attributes to the associated span.
	SetSpanSizeAttributes(ctx context.Context, ss SizesStruct)

	// SetSpanError set span status explicitly, if there is an non-nil error provided.
	SetSpanError(ctx context.Context, err error)
}

// Noop is a no-op implementation of Interface.
type Noop struct{}

var _ Interface = Noop{}

func (Noop) CountSend(ctx context.Context, ss SizesStruct)    {}
func (Noop) CountReceive(ctx context.Context, ss SizesStruct) {}
func (Noop) SetSpanSizeAttributes(ctx context.Context, ss SizesStruct) {}
func (Noop) SetSpanError(ctx context.Context, err error) {}

const (
	bytesUnit           = "bytes"
	sentDescription     = "Number of bytes sent by the component."
	sentWireDescription = "Number of bytes sent on the wire by the component."
	recvDescription     = "Number of bytes received by the component."
	recvWireDescription = "Number of bytes received on the wire by the component."
	compSizeDescription = "Size of compressed payload"
)

// makeSentMetrics builds the sent and sent-wire metric instruments
// for an exporter or receiver using the corresponding `prefix`.
// major` indicates the major direction of the pipeline,
// which is true when sending for exporters, receiving for receivers.
func makeSentMetrics(prefix string, meter metric.Meter, major bool) (sent, sentWire metric.Int64Counter, _ error) {
	var sentBytes metric.Int64Counter = noopmetric.Int64Counter{}
	var err1 error
	if major {
		sentBytes, err1 = meter.Int64Counter(prefix+"_"+SentBytes, metric.WithDescription(sentDescription), metric.WithUnit(bytesUnit))
	}
	sentWireBytes, err2 := meter.Int64Counter(prefix+"_"+SentWireBytes, metric.WithDescription(sentWireDescription), metric.WithUnit(bytesUnit))
	return sentBytes, sentWireBytes, multierr.Append(err1, err2)
}

// makeRecvMetrics builds the received and received-wire metric
// instruments for an exporter or receiver using the corresponding
// `prefix`.  `major` indicates the major direction of the pipeline,
// which is true when sending for exporters, receiving for receivers.
func makeRecvMetrics(prefix string, meter metric.Meter, major bool) (recv, recvWire metric.Int64Counter, _ error) {
	var recvBytes metric.Int64Counter = noopmetric.Int64Counter{}
	var err1 error
	if major {
		recvBytes, err1 = meter.Int64Counter(prefix+"_"+RecvBytes, metric.WithDescription(recvDescription), metric.WithUnit(bytesUnit))
	}
	recvWireBytes, err2 := meter.Int64Counter(prefix+"_"+RecvWireBytes, metric.WithDescription(recvWireDescription), metric.WithUnit(bytesUnit))
	return recvBytes, recvWireBytes, multierr.Append(err1, err2)
}

// NewExporterNetworkReporter creates a new NetworkReporter configured for an exporter.
func NewExporterNetworkReporter(settings exporter.CreateSettings) (*NetworkReporter, error) {
	level := settings.TelemetrySettings.MetricsLevel

	if level <= configtelemetry.LevelBasic {
		// Note: NetworkReporter implements nil a check.
		return nil, nil
	}

	meter := settings.TelemetrySettings.MeterProvider.Meter(scopeName)
	rep := &NetworkReporter{
		isExporter:    true,
		staticAttr:    attribute.String(ExporterKey, settings.ID.String()),
		compSizeHisto: noopmetric.Int64Histogram{},
	}

	var errors, err error
	if level > configtelemetry.LevelNormal {
		rep.compSizeHisto, err = meter.Int64Histogram(ExporterKey+"_"+CompSize, metric.WithDescription(compSizeDescription), metric.WithUnit(bytesUnit))
		errors = multierr.Append(errors, err)
	}

	rep.sentBytes, rep.sentWireBytes, err = makeSentMetrics(ExporterKey, meter, true)
	errors = multierr.Append(errors, err)

	// Normally, an exporter counts sent bytes, and skips received
	// bytes.  LevelDetailed will reveal exporter-received bytes.
	if level > configtelemetry.LevelNormal {
		rep.recvBytes, rep.recvWireBytes, err = makeRecvMetrics(ExporterKey, meter, false)
		errors = multierr.Append(errors, err)
	}

	return rep, errors
}

// NewReceiverNetworkReporter creates a new NetworkReporter configured for an exporter.
func NewReceiverNetworkReporter(settings receiver.CreateSettings) (*NetworkReporter, error) {
	level := settings.TelemetrySettings.MetricsLevel

	if level <= configtelemetry.LevelBasic {
		// Note: NetworkReporter implements nil a check.
		return nil, nil
	}

	meter := settings.MeterProvider.Meter(scopeName)
	rep := &NetworkReporter{
		isExporter:    false,
		staticAttr:    attribute.String(ReceiverKey, settings.ID.String()),
		compSizeHisto: noopmetric.Int64Histogram{},
	}

	var errors, err error
	if level > configtelemetry.LevelNormal {
		rep.compSizeHisto, err = meter.Int64Histogram(ReceiverKey+"_"+CompSize, metric.WithDescription(compSizeDescription), metric.WithUnit(bytesUnit))
		errors = multierr.Append(errors, err)
	}

	rep.recvBytes, rep.recvWireBytes, err = makeRecvMetrics(ReceiverKey, meter, true)
	errors = multierr.Append(errors, err)

	// Normally, a receiver counts received bytes, and skips sent
	// bytes.  LevelDetailed will reveal receiver-sent bytes.
	if level > configtelemetry.LevelNormal {
		rep.sentBytes, rep.sentWireBytes, err = makeSentMetrics(ReceiverKey, meter, false)
		errors = multierr.Append(errors, err)
	}

	return rep, errors
}

// CountSend is used to report a message sent by the component.  For
// exporters, SizesStruct indicates the size of a request.  For
// receivers, SizesStruct indicates the size of a response.
func (rep *NetworkReporter) CountSend(ctx context.Context, ss SizesStruct) {
	// Indicates basic level telemetry, not counting bytes.
	if rep == nil {
		return
	}

	attrs := metric.WithAttributes(rep.staticAttr, attribute.String("method", ss.Method))

	if rep.isExporter && ss.WireLength > 0 {
		rep.compSizeHisto.Record(ctx, ss.WireLength, attrs)
	}
	if rep.sentBytes != nil && ss.Length > 0 {
		rep.sentBytes.Add(ctx, ss.Length, attrs)
	}
	if rep.sentWireBytes != nil && ss.WireLength > 0 {
		rep.sentWireBytes.Add(ctx, ss.WireLength, attrs)
	}
}

// CountReceive is used to report a message received by the component.  For
// exporters, SizesStruct indicates the size of a response.  For
// receivers, SizesStruct indicates the size of a request.
func (rep *NetworkReporter) CountReceive(ctx context.Context, ss SizesStruct) {
	// Indicates basic level telemetry, not counting bytes.
	if rep == nil {
		return
	}

	attrs := metric.WithAttributes(rep.staticAttr, attribute.String("method", ss.Method))
	if !rep.isExporter && ss.WireLength > 0 {
		rep.compSizeHisto.Record(ctx, ss.WireLength, attrs)
	}
	if rep.recvBytes != nil && ss.Length > 0 {
		rep.recvBytes.Add(ctx, ss.Length, attrs)
	}
	if rep.recvWireBytes != nil && ss.WireLength > 0 {
		rep.recvWireBytes.Add(ctx, ss.WireLength, attrs)
	}
}

func (rep *NetworkReporter) SetSpanSizeAttributes(ctx context.Context, ss SizesStruct) {
	if rep == nil {
		return
	}

	span := trace.SpanFromContext(ctx)

	var compressedName string
	var uncompressedName string
	// set attribute name based on exporter vs receiver
	if rep.isExporter {
		compressedName = "stream_client_compressed_bytes_sent"
		uncompressedName = "stream_client_uncompressed_bytes_sent"
	} else { // receiver attributes
		compressedName = "stream_server_compressed_bytes_recv"
		uncompressedName = "stream_server_uncompressed_bytes_recv"
	}

	if ss.Length > 0 {
		span.SetAttributes(attribute.Int(uncompressedName, int(ss.Length)))
	}

	if ss.WireLength > 0 {
		span.SetAttributes(attribute.Int(compressedName, int(ss.WireLength)))
	}
}

func (rep *NetworkReporter) SetSpanError(ctx context.Context, err error) {
	if err == nil {
		return
	}

	span := trace.SpanFromContext(ctx)
	span.SetStatus(otelcodes.Error, err.Error())
}