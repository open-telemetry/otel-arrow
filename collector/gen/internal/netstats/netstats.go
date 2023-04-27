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

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/metric/instrument"
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

	scopeName = "github.com/f5/otel-arrow-adapter/collector/netstats"
)

// NetworkReporter is a helper to add network-level observability to
// an exporter or receiver.
type NetworkReporter struct {
	attrs         []attribute.KeyValue
	sentBytes     instrument.Int64Counter
	sentWireBytes instrument.Int64Counter
	recvBytes     instrument.Int64Counter
	recvWireBytes instrument.Int64Counter
}

// SizesStruct is used to pass uncompressed on-wire message lengths to
// the CountSend() and CountReceive() methods.
type SizesStruct struct {
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
func makeSentMetrics(prefix string, meter metric.Meter) (sent, sentWire instrument.Int64Counter, _ error) {
	sentBytes, err1 := meter.Int64Counter(prefix+"_"+SentBytes, instrument.WithDescription(sentDescription), instrument.WithUnit(bytesUnit))
	sentWireBytes, err2 := meter.Int64Counter(prefix+"_"+SentWireBytes, instrument.WithDescription(sentWireDescription), instrument.WithUnit(bytesUnit))
	return sentBytes, sentWireBytes, multierr.Append(err1, err2)
}

// makeRecvMetrics builds the received and received-wire metric instruments
// for an exporter or receiver using the corresponding `prefix`.
func makeRecvMetrics(prefix string, meter metric.Meter) (recv, recvWire instrument.Int64Counter, _ error) {
	recvBytes, err1 := meter.Int64Counter(prefix+"_"+RecvBytes, instrument.WithDescription(recvDescription), instrument.WithUnit(bytesUnit))
	recvWireBytes, err2 := meter.Int64Counter(prefix+"_"+RecvWireBytes, instrument.WithDescription(recvWireDescription), instrument.WithUnit(bytesUnit))
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
		attrs: []attribute.KeyValue{
			attribute.String(ExporterKey, settings.ID.String()),
		},
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
		attrs: []attribute.KeyValue{
			attribute.String(ReceiverKey, settings.ID.String()),
		},
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
		rep.sentBytes.Add(ctx, ss.Length, rep.attrs...)
	}
	if rep.sentWireBytes != nil && ss.WireLength > 0 {
		rep.sentWireBytes.Add(ctx, ss.WireLength, rep.attrs...)
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
		rep.recvBytes.Add(ctx, ss.Length, rep.attrs...)
	}
	if rep.recvWireBytes != nil && ss.WireLength > 0 {
		rep.recvWireBytes.Add(ctx, ss.WireLength, rep.attrs...)
	}
}
