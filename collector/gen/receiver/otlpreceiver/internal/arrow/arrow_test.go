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

package arrow

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"testing"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrowCollectorMock "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1/mock"
	arrowRecord "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
	arrowRecordMock "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record/mock"
	otelAssert "github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/zap/zaptest"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc/metadata"

	"go.opentelemetry.io/collector/client"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/component/componenttest"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/extension/auth"
	"github.com/f5/otel-arrow-adapter/collector/gen/internal/testdata"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/receiver"
	"github.com/f5/otel-arrow-adapter/collector/gen/receiver/otlpreceiver/internal/arrow/mock"
)

type compareJSONTraces struct{ ptrace.Traces }
type compareJSONMetrics struct{ pmetric.Metrics }
type compareJSONLogs struct{ plog.Logs }

func (c compareJSONTraces) MarshalJSON() ([]byte, error) {
	var m ptrace.JSONMarshaler
	return m.MarshalTraces(c.Traces)
}

func (c compareJSONMetrics) MarshalJSON() ([]byte, error) {
	var m pmetric.JSONMarshaler
	return m.MarshalMetrics(c.Metrics)
}

func (c compareJSONLogs) MarshalJSON() ([]byte, error) {
	var m plog.JSONMarshaler
	return m.MarshalLogs(c.Logs)
}

type consumeResult struct {
	Ctx  context.Context
	Data interface{}
}

type commonTestCase struct {
	ctrl      *gomock.Controller
	cancel    context.CancelFunc
	telset    component.TelemetrySettings
	consumers mockConsumers
	stream    *arrowCollectorMock.MockArrowStreamService_ArrowStreamServer
	receive   chan recvResult
	consume   chan consumeResult
	streamErr chan error

	testProducer *arrowRecord.Producer

	ctxCall  *gomock.Call
	recvCall *gomock.Call
}

type testChannel interface {
	onConsume() error
}

type healthyTestChannel struct{}

func (healthyTestChannel) onConsume() error {
	return nil
}

type unhealthyTestChannel struct{}

func (unhealthyTestChannel) onConsume() error {
	return fmt.Errorf("consumer unhealthy")
}

type recvResult struct {
	payload *arrowpb.BatchArrowRecords
	err     error
}

type mockConsumers struct {
	traces  *mock.MockTraces
	logs    *mock.MockLogs
	metrics *mock.MockMetrics

	tracesCall  *gomock.Call
	logsCall    *gomock.Call
	metricsCall *gomock.Call
}

func newTestTelemetry(t *testing.T) component.TelemetrySettings {
	telset := componenttest.NewNopTelemetrySettings()
	telset.Logger = zaptest.NewLogger(t)
	return telset
}

func (ctc *commonTestCase) putBatch(payload *arrowpb.BatchArrowRecords, err error) {
	ctc.receive <- recvResult{
		payload: payload,
		err:     err,
	}
}

func (ctc *commonTestCase) doAndReturnGetBatch(ctx context.Context) func() (*arrowpb.BatchArrowRecords, error) {
	return func() (*arrowpb.BatchArrowRecords, error) {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case r, ok := <-ctc.receive:
			if !ok {
				return nil, io.EOF
			}
			return r.payload, r.err
		}
	}
}

func (ctc *commonTestCase) doAndReturnConsumeTraces(tc testChannel) func(ctx context.Context, traces ptrace.Traces) error {
	return func(ctx context.Context, traces ptrace.Traces) error {
		select {
		case ctc.consume <- consumeResult{
			Ctx:  ctx,
			Data: traces,
		}:
			return tc.onConsume()
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

func (ctc *commonTestCase) doAndReturnConsumeMetrics(tc testChannel) func(ctx context.Context, metrics pmetric.Metrics) error {
	return func(ctx context.Context, metrics pmetric.Metrics) error {
		select {
		case ctc.consume <- consumeResult{
			Ctx:  ctx,
			Data: metrics,
		}:
			return tc.onConsume()
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

func (ctc *commonTestCase) doAndReturnConsumeLogs(tc testChannel) func(ctx context.Context, logs plog.Logs) error {
	return func(ctx context.Context, logs plog.Logs) error {
		select {
		case ctc.consume <- consumeResult{
			Ctx:  ctx,
			Data: logs,
		}:
			return tc.onConsume()
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

func newMockConsumers(ctrl *gomock.Controller) mockConsumers {
	mc := mockConsumers{
		traces:  mock.NewMockTraces(ctrl),
		logs:    mock.NewMockLogs(ctrl),
		metrics: mock.NewMockMetrics(ctrl),
	}
	mc.traces.EXPECT().Capabilities().Times(0)
	mc.tracesCall = mc.traces.EXPECT().ConsumeTraces(
		gomock.Any(),
		gomock.Any(),
	).Times(0)
	mc.logs.EXPECT().Capabilities().Times(0)
	mc.logsCall = mc.logs.EXPECT().ConsumeLogs(
		gomock.Any(),
		gomock.Any(),
	).Times(0)
	mc.metrics.EXPECT().Capabilities().Times(0)
	mc.metricsCall = mc.metrics.EXPECT().ConsumeMetrics(
		gomock.Any(),
		gomock.Any(),
	).Times(0)
	return mc
}

func (m mockConsumers) Traces() consumer.Traces {
	return m.traces
}

func (m mockConsumers) Logs() consumer.Logs {
	return m.logs
}
func (m mockConsumers) Metrics() consumer.Metrics {
	return m.metrics
}

var _ Consumers = mockConsumers{}

func newCommonTestCase(t *testing.T, tc testChannel) *commonTestCase {
	ctrl := gomock.NewController(t)
	stream := arrowCollectorMock.NewMockArrowStreamService_ArrowStreamServer(ctrl)

	ctx, cancel := context.WithCancel(context.Background())
	ctx = metadata.NewIncomingContext(ctx, metadata.MD{
		"stream_ctx": []string{"per-request"},
	})

	ctc := &commonTestCase{
		ctrl:         ctrl,
		cancel:       cancel,
		telset:       newTestTelemetry(t),
		consumers:    newMockConsumers(ctrl),
		stream:       stream,
		receive:      make(chan recvResult),
		consume:      make(chan consumeResult),
		streamErr:    make(chan error),
		testProducer: arrowRecord.NewProducer(),
		ctxCall:      stream.EXPECT().Context().Times(0),
		recvCall:     stream.EXPECT().Recv().Times(0),
	}

	ctc.ctxCall.AnyTimes().Return(ctx)
	ctc.recvCall.AnyTimes().DoAndReturn(ctc.doAndReturnGetBatch(ctx))
	ctc.consumers.tracesCall.AnyTimes().DoAndReturn(ctc.doAndReturnConsumeTraces(tc))
	ctc.consumers.logsCall.AnyTimes().DoAndReturn(ctc.doAndReturnConsumeLogs(tc))
	ctc.consumers.metricsCall.AnyTimes().DoAndReturn(ctc.doAndReturnConsumeMetrics(tc))
	return ctc
}

func (ctc *commonTestCase) cancelAndWait() error {
	ctc.cancel()
	return ctc.wait()
}

func (ctc *commonTestCase) wait() error {
	return <-ctc.streamErr
}

func statusOKFor(batchID string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:    batchID,
				StatusCode: arrowpb.StatusCode_OK,
			},
		},
	}
}

func statusUnavailableFor(batchID string, msg string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:      batchID,
				StatusCode:   arrowpb.StatusCode_ERROR,
				ErrorCode:    arrowpb.ErrorCode_UNAVAILABLE,
				ErrorMessage: msg,
			},
		},
	}
}

func statusInvalidFor(batchID string, msg string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:      batchID,
				StatusCode:   arrowpb.StatusCode_ERROR,
				ErrorCode:    arrowpb.ErrorCode_INVALID_ARGUMENT,
				ErrorMessage: msg,
			},
		},
	}
}

func (ctc *commonTestCase) newRealConsumer() arrowRecord.ConsumerAPI {
	cons := arrowRecordMock.NewMockConsumerAPI(ctc.ctrl)
	real := arrowRecord.NewConsumer()

	cons.EXPECT().Close().Times(1).Return(nil)
	cons.EXPECT().TracesFrom(gomock.Any()).AnyTimes().DoAndReturn(real.TracesFrom)
	cons.EXPECT().MetricsFrom(gomock.Any()).AnyTimes().DoAndReturn(real.MetricsFrom)
	cons.EXPECT().LogsFrom(gomock.Any()).AnyTimes().DoAndReturn(real.LogsFrom)

	return cons
}

func (ctc *commonTestCase) newErrorConsumer() arrowRecord.ConsumerAPI {
	cons := arrowRecordMock.NewMockConsumerAPI(ctc.ctrl)

	cons.EXPECT().Close().Times(1).Return(nil)
	cons.EXPECT().TracesFrom(gomock.Any()).AnyTimes().Return(nil, fmt.Errorf("test invalid error"))
	cons.EXPECT().MetricsFrom(gomock.Any()).AnyTimes().Return(nil, fmt.Errorf("test invalid error"))
	cons.EXPECT().LogsFrom(gomock.Any()).AnyTimes().Return(nil, fmt.Errorf("test invalid error"))

	return cons
}

func (ctc *commonTestCase) start(newConsumer func() arrowRecord.ConsumerAPI, opts ...func(*configgrpc.GRPCServerSettings, *auth.Server)) {
	var authServer auth.Server
	gsettings := &configgrpc.GRPCServerSettings{}
	for _, gf := range opts {
		gf(gsettings, &authServer)
	}
	rcvr, err := New(
		component.NewID("arrowtest"),
		ctc.consumers,
		receiver.CreateSettings{
			TelemetrySettings: ctc.telset,
			BuildInfo:         component.NewDefaultBuildInfo(),
		},
		gsettings,
		authServer,
		newConsumer,
	)
	if err != nil {
		// it would be because obsreport.NewReceiver failed, this is
		// not tested here.
		panic("new failure not tested")
	}
	go func() {
		ctc.streamErr <- rcvr.ArrowStream(ctc.stream)
	}()
}

func TestReceiverTraces(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	td := testdata.GenerateTraces(2)
	batch, err := ctc.testProducer.BatchArrowRecordsFromTraces(td)
	require.NoError(t, err)

	ctc.stream.EXPECT().Send(statusOKFor(batch.BatchId)).Times(1).Return(nil)

	ctc.start(ctc.newRealConsumer)
	ctc.putBatch(batch, nil)

	assert.EqualValues(t, td, (<-ctc.consume).Data)

	err = ctc.cancelAndWait()
	require.Error(t, err)
	require.True(t, errors.Is(err, context.Canceled))
}

func TestReceiverLogs(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	ld := testdata.GenerateLogs(2)
	batch, err := ctc.testProducer.BatchArrowRecordsFromLogs(ld)
	require.NoError(t, err)

	ctc.stream.EXPECT().Send(statusOKFor(batch.BatchId)).Times(1).Return(nil)

	ctc.start(ctc.newRealConsumer)
	ctc.putBatch(batch, nil)

	assert.EqualValues(t, []json.Marshaler{compareJSONLogs{ld}}, []json.Marshaler{compareJSONLogs{(<-ctc.consume).Data.(plog.Logs)}})

	err = ctc.cancelAndWait()
	require.Error(t, err)
	require.True(t, errors.Is(err, context.Canceled), "for %v", err)
}

func TestReceiverMetrics(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	md := testdata.GenerateMetrics(2)
	batch, err := ctc.testProducer.BatchArrowRecordsFromMetrics(md)
	require.NoError(t, err)

	ctc.stream.EXPECT().Send(statusOKFor(batch.BatchId)).Times(1).Return(nil)

	ctc.start(ctc.newRealConsumer)
	ctc.putBatch(batch, nil)

	otelAssert.Equiv(t, []json.Marshaler{
		compareJSONMetrics{md},
	}, []json.Marshaler{
		compareJSONMetrics{(<-ctc.consume).Data.(pmetric.Metrics)},
	})

	err = ctc.cancelAndWait()
	require.Error(t, err)
	require.True(t, errors.Is(err, context.Canceled), "for %v", err)
}

func TestReceiverRecvError(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	ctc.start(ctc.newRealConsumer)

	ctc.putBatch(nil, fmt.Errorf("test recv error"))

	err := ctc.wait()
	require.Error(t, err)
	require.Contains(t, err.Error(), "test recv error")
}

func TestReceiverSendError(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	ld := testdata.GenerateLogs(2)
	batch, err := ctc.testProducer.BatchArrowRecordsFromLogs(ld)
	require.NoError(t, err)

	ctc.stream.EXPECT().Send(statusOKFor(batch.BatchId)).Times(1).Return(fmt.Errorf("test send error"))

	ctc.start(ctc.newRealConsumer)
	ctc.putBatch(batch, nil)

	assert.EqualValues(t, ld, (<-ctc.consume).Data)

	err = ctc.wait()
	require.Error(t, err)
	require.Contains(t, err.Error(), "test send error")
}

func TestReceiverConsumeError(t *testing.T) {
	data := []interface{}{
		testdata.GenerateTraces(2),
		testdata.GenerateMetrics(2),
		testdata.GenerateLogs(2),
	}

	for _, item := range data {
		tc := unhealthyTestChannel{}
		ctc := newCommonTestCase(t, tc)

		var batch *arrowpb.BatchArrowRecords
		var err error
		switch input := item.(type) {
		case ptrace.Traces:
			batch, err = ctc.testProducer.BatchArrowRecordsFromTraces(input)
		case plog.Logs:
			batch, err = ctc.testProducer.BatchArrowRecordsFromLogs(input)
		case pmetric.Metrics:
			batch, err = ctc.testProducer.BatchArrowRecordsFromMetrics(input)
		default:
			panic(input)
		}
		require.NoError(t, err)

		ctc.stream.EXPECT().Send(statusUnavailableFor(batch.BatchId, "consumer unhealthy")).Times(1).Return(nil)

		ctc.start(ctc.newRealConsumer)

		ctc.putBatch(batch, nil)

		switch input := item.(type) {
		case ptrace.Traces:
			otelAssert.Equiv(t, []json.Marshaler{
				compareJSONTraces{input},
			}, []json.Marshaler{
				compareJSONTraces{(<-ctc.consume).Data.(ptrace.Traces)},
			})
		case plog.Logs:
			otelAssert.Equiv(t, []json.Marshaler{
				compareJSONLogs{input},
			}, []json.Marshaler{
				compareJSONLogs{(<-ctc.consume).Data.(plog.Logs)},
			})
		case pmetric.Metrics:
			otelAssert.Equiv(t, []json.Marshaler{
				compareJSONMetrics{input},
			}, []json.Marshaler{
				compareJSONMetrics{(<-ctc.consume).Data.(pmetric.Metrics)},
			})
		}

		err = ctc.cancelAndWait()
		require.Error(t, err)
		require.True(t, errors.Is(err, context.Canceled), "for %v", err)
	}
}

func TestReceiverInvalidData(t *testing.T) {
	data := []interface{}{
		testdata.GenerateTraces(2),
		testdata.GenerateMetrics(2),
		testdata.GenerateLogs(2),
	}

	for _, item := range data {
		tc := unhealthyTestChannel{}
		ctc := newCommonTestCase(t, tc)

		var batch *arrowpb.BatchArrowRecords
		var err error
		switch input := item.(type) {
		case ptrace.Traces:
			batch, err = ctc.testProducer.BatchArrowRecordsFromTraces(input)
		case plog.Logs:
			batch, err = ctc.testProducer.BatchArrowRecordsFromLogs(input)
		case pmetric.Metrics:
			batch, err = ctc.testProducer.BatchArrowRecordsFromMetrics(input)
		default:
			panic(input)
		}
		require.NoError(t, err)

		ctc.stream.EXPECT().Send(statusInvalidFor(batch.BatchId, "Permanent error: test invalid error")).Times(1).Return(nil)

		ctc.start(ctc.newErrorConsumer)
		ctc.putBatch(batch, nil)

		err = ctc.cancelAndWait()
		require.Error(t, err)
		require.True(t, errors.Is(err, context.Canceled), "for %v", err)
	}
}

func TestReceiverEOF(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	// send a sequence of data then simulate closing the connection.
	const times = 10

	var actualData []ptrace.Traces
	var expectData []ptrace.Traces

	ctc.stream.EXPECT().Send(gomock.Any()).Times(times).Return(nil)

	ctc.start(ctc.newRealConsumer)

	go func() {
		for i := 0; i < times; i++ {
			td := testdata.GenerateTraces(2)
			expectData = append(expectData, td)

			batch, err := ctc.testProducer.BatchArrowRecordsFromTraces(td)
			require.NoError(t, err)

			ctc.putBatch(batch, nil)
		}
		close(ctc.receive)
	}()

	for i := 0; i < times; i++ {
		actualData = append(actualData, (<-ctc.consume).Data.(ptrace.Traces))
	}

	assert.EqualValues(t, expectData, actualData)

	err := ctc.wait()
	require.Error(t, err)
	require.True(t, errors.Is(err, io.EOF))
}

func TestReceiverHeadersNoAuth(t *testing.T) {
	t.Run("include", func(t *testing.T) { testReceiverHeaders(t, true) })
	t.Run("noinclude", func(t *testing.T) { testReceiverHeaders(t, false) })
}

func testReceiverHeaders(t *testing.T, includeMeta bool) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	expectData := []map[string][]string{
		{"k1": []string{"v1"}},
		nil,
		{"k2": []string{"v2"}, "k3": []string{"v3"}},
		nil,
		{"k1": []string{"v5"}},
		{"k1": []string{"v1"}, "k3": []string{"v2", "v3", "v4"}},
		nil,
	}

	ctc.stream.EXPECT().Send(gomock.Any()).Times(len(expectData)).Return(nil)

	ctc.start(ctc.newRealConsumer, func(gsettings *configgrpc.GRPCServerSettings, _ *auth.Server) {
		gsettings.IncludeMetadata = includeMeta
	})

	go func() {
		var hpb bytes.Buffer
		hpe := hpack.NewEncoder(&hpb)

		for _, md := range expectData {
			td := testdata.GenerateTraces(2)
			batch, err := ctc.testProducer.BatchArrowRecordsFromTraces(td)
			require.NoError(t, err)

			if len(md) != 0 {
				hpb.Reset()
				for key, vals := range md {
					for _, val := range vals {
						err := hpe.WriteField(hpack.HeaderField{
							Name:  key,
							Value: val,
						})
						require.NoError(t, err)
					}
				}

				batch.Headers = make([]byte, hpb.Len())
				copy(batch.Headers, hpb.Bytes())
			}
			ctc.putBatch(batch, nil)
		}
		close(ctc.receive)
	}()

	for _, expect := range expectData {
		info := client.FromContext((<-ctc.consume).Ctx)

		// The static stream context contains one extra variable.
		if expect == nil {
			expect = map[string][]string{}
		}
		expect["stream_ctx"] = []string{"per-request"}

		for key, vals := range expect {
			if includeMeta {
				require.Equal(t, vals, info.Metadata.Get(key))
			} else {
				require.Equal(t, []string(nil), info.Metadata.Get(key))
			}
		}
	}

	err := ctc.wait()
	require.Error(t, err)
	require.True(t, errors.Is(err, io.EOF))
}

func TestReceiverCancel(t *testing.T) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	ctc.cancel()
	ctc.start(ctc.newRealConsumer)

	err := ctc.wait()
	require.Error(t, err)
	require.True(t, errors.Is(err, context.Canceled))
}

func requireContainsAll(t *testing.T, md client.Metadata, exp map[string][]string) {
	for key, vals := range exp {
		require.Equal(t, vals, md.Get(key))
	}
}

func requireContainsNone(t *testing.T, md client.Metadata, exp map[string][]string) {
	for key := range exp {
		require.Equal(t, []string(nil), md.Get(key))
	}
}

func TestHeaderReceiverStreamContextOnly(t *testing.T) {
	expect := map[string][]string{
		"K": {"k1", "k2"},
		"L": {"l1"},
	}

	ctx := metadata.NewIncomingContext(context.Background(), metadata.MD(expect))

	h := newHeaderReceiver(ctx, true)

	for i := 0; i < 3; i++ {
		cc, _, err := h.combineHeaders(ctx, nil)

		require.NoError(t, err)
		requireContainsAll(t, client.FromContext(cc).Metadata, expect)
	}
}

func TestHeaderReceiverNoIncludeMetadata(t *testing.T) {
	noExpect := map[string][]string{
		"K": {"k1", "k2"},
		"L": {"l1"},
	}

	ctx := metadata.NewIncomingContext(context.Background(), metadata.MD(noExpect))

	h := newHeaderReceiver(ctx, false)

	for i := 0; i < 3; i++ {
		cc, _, err := h.combineHeaders(ctx, nil)

		require.NoError(t, err)
		requireContainsNone(t, client.FromContext(cc).Metadata, noExpect)
	}
}

func TestHeaderReceiverRequestNoStreamMetadata(t *testing.T) {
	expect := map[string][]string{
		"K": {"k1", "k2"},
		"L": {"l1"},
	}

	var hpb bytes.Buffer

	hpe := hpack.NewEncoder(&hpb)

	ctx := context.Background()

	h := newHeaderReceiver(ctx, true)

	for i := 0; i < 3; i++ {
		hpb.Reset()

		for key, vals := range expect {
			for _, val := range vals {
				err := hpe.WriteField(hpack.HeaderField{
					Name:  key,
					Value: val,
				})
				require.NoError(t, err)
			}
		}

		cc, _, err := h.combineHeaders(ctx, hpb.Bytes())

		require.NoError(t, err)
		requireContainsAll(t, client.FromContext(cc).Metadata, expect)
	}
}

func TestHeaderReceiverBothMetadata(t *testing.T) {
	expectK := map[string][]string{
		"K": {"k1", "k2"},
	}
	expectL := map[string][]string{
		"L": {"l1"},
		"M": {"m1", "m2"},
	}
	expect := map[string][]string{
		"K": {"k1", "k2"},
		"L": {"l1"},
		"M": {"m1", "m2"},
	}

	var hpb bytes.Buffer

	hpe := hpack.NewEncoder(&hpb)

	ctx := metadata.NewIncomingContext(context.Background(), metadata.MD(expectK))

	h := newHeaderReceiver(ctx, true)

	for i := 0; i < 3; i++ {
		hpb.Reset()

		for key, vals := range expectL {
			for _, val := range vals {
				err := hpe.WriteField(hpack.HeaderField{
					Name:  key,
					Value: val,
				})
				require.NoError(t, err)
			}
		}

		cc, _, err := h.combineHeaders(ctx, hpb.Bytes())

		require.NoError(t, err)
		requireContainsAll(t, client.FromContext(cc).Metadata, expect)
	}
}

func TestHeaderReceiverDuplicateMetadata(t *testing.T) {
	expectStream := map[string][]string{
		"K": {"k1", "k2"},

		// "M" value does not appear b/c the same header
		// appears in per-request metadata.
		"M": {""},
	}
	expectRequest := map[string][]string{
		"L": {"l1"},
		"M": {"m1", "m2"},
	}
	expectCombined := map[string][]string{
		"K": {"k1", "k2"},
		"L": {"l1"},
		"M": {"m1", "m2"},
	}

	var hpb bytes.Buffer

	hpe := hpack.NewEncoder(&hpb)

	ctx := metadata.NewIncomingContext(context.Background(), metadata.MD(expectStream))

	h := newHeaderReceiver(ctx, true)

	for i := 0; i < 3; i++ {
		hpb.Reset()

		for key, vals := range expectRequest {
			for _, val := range vals {
				err := hpe.WriteField(hpack.HeaderField{
					Name:  key,
					Value: val,
				})
				require.NoError(t, err)
			}
		}

		cc, _, err := h.combineHeaders(ctx, hpb.Bytes())

		require.NoError(t, err)
		requireContainsAll(t, client.FromContext(cc).Metadata, expectCombined)
	}
}

func TestReceiverAuthHeadersStream(t *testing.T) {
	t.Run("no-metadata", func(t *testing.T) { testReceiverAuthHeaders(t, false, false) })
	t.Run("per-stream", func(t *testing.T) { testReceiverAuthHeaders(t, true, false) })
	t.Run("per-data", func(t *testing.T) { testReceiverAuthHeaders(t, true, true) })
}

func testReceiverAuthHeaders(t *testing.T, includeMeta bool, dataAuth bool) {
	tc := healthyTestChannel{}
	ctc := newCommonTestCase(t, tc)

	expectData := []map[string][]string{
		{"auth": []string{"true"}},
		nil,
		{"auth": []string{"false"}},
		nil,
	}

	var recvBatches []*arrowpb.BatchStatus

	ctc.stream.EXPECT().Send(gomock.Any()).Times(len(expectData)).DoAndReturn(func(batch *arrowpb.BatchStatus) error {
		recvBatches = append(recvBatches, batch)
		return nil
	})

	var authCall *gomock.Call
	ctc.start(ctc.newRealConsumer, func(gsettings *configgrpc.GRPCServerSettings, authPtr *auth.Server) {
		gsettings.IncludeMetadata = includeMeta

		as := mock.NewMockServer(ctc.ctrl)
		*authPtr = as

		authCall = as.EXPECT().Authenticate(gomock.Any(), gomock.Any()).AnyTimes()
	})

	dataCount := 0

	authCall.DoAndReturn(func(ctx context.Context, hdrs map[string][]string) (context.Context, error) {
		dataCount++
		if !dataAuth {
			return ctx, nil
		}

		ok := false
		for _, val := range hdrs["auth"] {
			ok = ok || (val == "true")
		}

		if ok {
			newmd := map[string][]string{}
			for k, v := range hdrs {
				newmd[k] = v
			}
			newmd["has_auth"] = []string{":+1:", ":100:"}
			return client.NewContext(ctx, client.Info{
				Metadata: client.NewMetadata(newmd),
			}), nil
		}
		return ctx, fmt.Errorf("not authorized")
	})

	go func() {
		var hpb bytes.Buffer
		hpe := hpack.NewEncoder(&hpb)

		for _, md := range expectData {
			td := testdata.GenerateTraces(2)
			batch, err := ctc.testProducer.BatchArrowRecordsFromTraces(td)
			require.NoError(t, err)

			if len(md) != 0 {

				hpb.Reset()
				for key, vals := range md {
					for _, val := range vals {
						err := hpe.WriteField(hpack.HeaderField{
							Name:  key,
							Value: val,
						})
						require.NoError(t, err)
					}
				}

				batch.Headers = make([]byte, hpb.Len())
				copy(batch.Headers, hpb.Bytes())
			}
			ctc.putBatch(batch, nil)
		}
		close(ctc.receive)
	}()

	var expectErrs []bool

	for _, expect := range expectData {
		// The static stream context contains one extra variable.
		if expect == nil {
			expect = map[string][]string{}
		}
		expect["stream_ctx"] = []string{"per-request"}

		expectErr := false
		if dataAuth {
			hasAuth := false
			for _, val := range expect["auth"] {
				hasAuth = hasAuth || (val == "true")
			}
			if hasAuth {
				expect["has_auth"] = []string{":+1:", ":100:"}
			} else {
				expectErr = true
			}
		}

		expectErrs = append(expectErrs, expectErr)

		if expectErr {
			continue
		}

		info := client.FromContext((<-ctc.consume).Ctx)

		for key, vals := range expect {
			if includeMeta {
				require.Equal(t, vals, info.Metadata.Get(key))
			} else {
				require.Equal(t, []string(nil), info.Metadata.Get(key))
			}
		}
	}

	err := ctc.wait()
	require.Error(t, err)
	require.True(t, errors.Is(err, io.EOF))

	require.Equal(t, len(expectData), dataCount)

	require.Equal(t, len(recvBatches), dataCount)

	for idx, batch := range recvBatches {
		if expectErrs[idx] {
			require.Equal(t, 1, len(batch.Statuses))
			require.Equal(t, arrowpb.StatusCode_ERROR, batch.Statuses[0].StatusCode)
		} else {
			require.Equal(t, 1, len(batch.Statuses))
			require.Equal(t, arrowpb.StatusCode_OK, batch.Statuses[0].StatusCode)
		}
	}
}
