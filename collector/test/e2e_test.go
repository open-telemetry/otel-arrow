// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package test

import (
	"context"
	"encoding/json"
	"fmt"
	"math/rand"
	"sync"
	"testing"
	"time"

	"github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter"
	"github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver"
	"github.com/open-telemetry/otel-arrow/collector/testutil"
	"github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/component/componenttest"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/consumer/consumertest"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
	"go.opentelemetry.io/collector/receiver"
	"go.uber.org/zap"
)

type testConsumer struct {
	sink consumertest.TracesSink
}

var _ consumer.Traces = &testConsumer{}

func (*testConsumer) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{}
}

func (tc *testConsumer) ConsumeTraces(ctx context.Context, td ptrace.Traces) error {
	time.Sleep(time.Duration(float64(time.Millisecond) * (1 + rand.Float64())))
	return tc.sink.ConsumeTraces(ctx, td)
}

func TestIntegrationSimpleTraces(t *testing.T) {
	const (
		threadCount  = 10
		requestCount = 100
	)

	efact := otelarrowexporter.NewFactory()
	rfact := otelarrowreceiver.NewFactory()

	ecfg := efact.CreateDefaultConfig()
	rcfg := rfact.CreateDefaultConfig()

	receiverCfg := rcfg.(*otelarrowreceiver.Config)
	exporterCfg := ecfg.(*otelarrowexporter.Config)

	addr := testutil.GetAvailableLocalAddress(t)

	receiverCfg.Protocols.GRPC.NetAddr.Endpoint = addr
	exporterCfg.ClientConfig.Endpoint = addr
	exporterCfg.ClientConfig.WaitForReady = true
	exporterCfg.ClientConfig.TLSSetting.Insecure = true
	exporterCfg.TimeoutSettings.Timeout = time.Minute
	exporterCfg.QueueSettings.Enabled = false
	exporterCfg.RetryConfig.Enabled = false
	exporterCfg.Arrow.NumStreams = 1

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	tset := componenttest.NewNopTelemetrySettings()
	tset.Logger, _ = zap.NewDevelopment()

	host := componenttest.NewNopHost()

	testCon := &testConsumer{}

	receiver, err := rfact.CreateTracesReceiver(ctx, receiver.CreateSettings{
		ID:                component.MustNewID("otelarrowreceiver"),
		TelemetrySettings: tset,
	}, receiverCfg, testCon)
	require.NoError(t, err)

	exporter, err := efact.CreateTracesExporter(ctx, exporter.CreateSettings{
		ID:                component.MustNewID("otelarrowexporter"),
		TelemetrySettings: tset,
	}, exporterCfg)
	require.NoError(t, err)

	var rwg sync.WaitGroup
	var ewg sync.WaitGroup
	var swg sync.WaitGroup

	rwg.Add(1)
	ewg.Add(1)
	swg.Add(1)

	go func() {
		defer rwg.Done()
		require.NoError(t, receiver.Start(ctx, host))
		ewg.Wait()
		require.NoError(t, receiver.Shutdown(ctx))
	}()

	go func() {
		defer ewg.Done()
		require.NoError(t, exporter.Start(ctx, host))
		swg.Done()
		<-ctx.Done()
		require.NoError(t, exporter.Shutdown(ctx))
	}()

	// wait for the exporter to start
	var cwg sync.WaitGroup
	swg.Wait()

	var expect [threadCount][]ptrace.Traces

	for num := 0; num < threadCount; num++ {
		cwg.Add(1)
		go func() {
			defer cwg.Done()
			for i := 0; i < requestCount; i++ {
				td := ptrace.NewTraces()
				td.ResourceSpans().AppendEmpty().Resource().Attributes().PutStr("resource-attr", fmt.Sprint("resource-attr-val-", i))

				ss := td.ResourceSpans().At(0).ScopeSpans().AppendEmpty().Spans()
				span := ss.AppendEmpty()

				span.SetName("operationA")
				span.SetStartTimestamp(pcommon.NewTimestampFromTime(time.Now()))
				span.SetEndTimestamp(pcommon.NewTimestampFromTime(time.Now()))

				span.SetTraceID(testutil.UInt64ToTraceID(rand.Uint64(), rand.Uint64()))
				span.SetSpanID(testutil.UInt64ToSpanID(rand.Uint64()))
				evs := span.Events()
				ev0 := evs.AppendEmpty()
				ev0.SetTimestamp(pcommon.NewTimestampFromTime(time.Now()))
				ev0.SetName("event-with-attr")
				ev0.Attributes().PutStr("span-event-attr", "span-event-attr-val")
				ev0.SetDroppedAttributesCount(2)
				ev1 := evs.AppendEmpty()
				ev1.SetTimestamp(pcommon.NewTimestampFromTime(time.Now()))
				ev1.SetName("event")
				ev1.SetDroppedAttributesCount(2)
				span.SetDroppedEventsCount(1)
				status := span.Status()
				status.SetCode(ptrace.StatusCodeError)
				status.SetMessage("status-cancelled")

				require.NoError(t, exporter.ConsumeTraces(ctx, td))
				expect[num] = append(expect[num], td)
			}
		}()
	}

	// wait til senders finish
	cwg.Wait()

	// shut down exporter; it triggers receiver to shut down
	cancel()

	// wait for receiver to shut down
	rwg.Wait()

	// Check for matching request count and data
	require.Equal(t, requestCount*threadCount, testCon.sink.SpanCount())

	var expectJSON []json.Marshaler
	for _, tdn := range expect {
		for _, td := range tdn {
			expectJSON = append(expectJSON, ptraceotlp.NewExportRequestFromTraces(td))
		}
	}
	var receivedJSON []json.Marshaler

	for _, td := range testCon.sink.AllTraces() {
		receivedJSON = append(receivedJSON, ptraceotlp.NewExportRequestFromTraces(td))
	}
	asserter := assert.NewStdUnitTest(t)
	assert.Equiv(asserter, expectJSON, receivedJSON)
}
