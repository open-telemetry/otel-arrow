// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package netstats // import "github.com/open-telemetry/otel-arrow/collector/gen/internal/netstats"

import (
	"context"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
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

	scopeName = "github.com/open-telemetry/otel-arrow/collector/netstats"
)

// NetworkReporter is a helper to add network-level observability to
// an exporter or receiver.
type NetworkReporter struct {
	staticAttr    attribute.KeyValue
	sentBytes     metric.Int64Counter
	sentWireBytes metric.Int64Counter
	recvBytes     metric.Int64Counter
	recvWireBytes metric.Int64Counter
}

// SizesStruct is used to pass uncompressed on-wire message lengths to
// the CountSend() and CountReceive() methods.
type SizesStruct struct {
	Method     string
	Length     int64
	WireLength int64
}

const (
	bytesUnit           = "bytes"
	sentDescription     = "Number of bytes sent by the component."
	sentWireDescription = "Number of bytes sent on the wire by the component."
	recvDescription     = "Number of bytes received by the component."
	recvWireDescription = "Number of bytes received on the wire by the component."
)

// makeSentMetrics builds the sent and sent-wire metric instruments
// for an exporter or receiver using the corresponding `prefix`.
func makeSentMetrics(prefix string, meter metric.Meter) (sent, sentWire metric.Int64Counter, _ error) {
	sentBytes, err1 := meter.Int64Counter(prefix+"_"+SentBytes, metric.WithDescription(sentDescription), metric.WithUnit(bytesUnit))
	sentWireBytes, err2 := meter.Int64Counter(prefix+"_"+SentWireBytes, metric.WithDescription(sentWireDescription), metric.WithUnit(bytesUnit))
	return sentBytes, sentWireBytes, multierr.Append(err1, err2)
}

// makeRecvMetrics builds the received and received-wire metric instruments
// for an exporter or receiver using the corresponding `prefix`.
func makeRecvMetrics(prefix string, meter metric.Meter) (recv, recvWire metric.Int64Counter, _ error) {
	recvBytes, err1 := meter.Int64Counter(prefix+"_"+RecvBytes, metric.WithDescription(recvDescription), metric.WithUnit(bytesUnit))
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
		staticAttr: attribute.String(ExporterKey, settings.ID.String()),
	}

	var errors, err error
	rep.sentBytes, rep.sentWireBytes, err = makeSentMetrics(ExporterKey, meter)
	errors = multierr.Append(errors, err)

	// Normally, an exporter counts sent bytes, and skips received
	// bytes.  LevelDetailed will reveal exporter-received bytes.
	if level > configtelemetry.LevelNormal {
		rep.recvBytes, rep.recvWireBytes, err = makeRecvMetrics(ExporterKey, meter)
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
		staticAttr: attribute.String(ReceiverKey, settings.ID.String()),
	}

	var errors, err error
	rep.recvBytes, rep.recvWireBytes, err = makeRecvMetrics(ReceiverKey, meter)
	errors = multierr.Append(errors, err)

	// Normally, a receiver counts received bytes, and skips sent
	// bytes.  LevelDetailed will reveal receiver-sent bytes.
	if level > configtelemetry.LevelNormal {
		rep.sentBytes, rep.sentWireBytes, err = makeSentMetrics(ReceiverKey, meter)
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

	if rep.sentBytes != nil && ss.Length > 0 {
		rep.sentBytes.Add(ctx, ss.Length, metric.WithAttributes(rep.staticAttr, attribute.String("method", ss.Method)))
	}
	if rep.sentWireBytes != nil && ss.WireLength > 0 {
		rep.sentWireBytes.Add(ctx, ss.WireLength, metric.WithAttributes(rep.staticAttr, attribute.String("method", ss.Method)))
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

	if rep.recvBytes != nil && ss.Length > 0 {
		rep.recvBytes.Add(ctx, ss.Length, metric.WithAttributes(rep.staticAttr, attribute.String("method", ss.Method)))
	}
	if rep.recvWireBytes != nil && ss.WireLength > 0 {
		rep.recvWireBytes.Add(ctx, ss.WireLength, metric.WithAttributes(rep.staticAttr, attribute.String("method", ss.Method)))
	}
}
