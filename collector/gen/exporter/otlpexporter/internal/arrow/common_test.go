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

package arrow

import (
	"context"
	"fmt"
	"io"
	"testing"

	arrowpb "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
	arrowCollectorMock "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1/mock"
	"github.com/golang/mock/gomock"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"go.uber.org/zap/zaptest"
	"go.uber.org/zap/zaptest/observer"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/status"

	"github.com/f5/otel-arrow-adapter/collector/gen/exporter/otlpexporter/internal/arrow/grpcmock"
	"github.com/f5/otel-arrow-adapter/collector/gen/internal/testdata"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/component/componenttest"
)

var (
	twoTraces  = testdata.GenerateTraces(2)
	twoMetrics = testdata.GenerateMetrics(2)
	twoLogs    = testdata.GenerateLogs(2)
)

type testChannel interface {
	onRecv(context.Context) func() (*arrowpb.BatchStatus, error)
	onSend(context.Context) func(*arrowpb.BatchArrowRecords) error
	onConnect(context.Context) error
}

type commonTestCase struct {
	ctrl                *gomock.Controller
	telset              component.TelemetrySettings
	observedLogs        *observer.ObservedLogs
	streamClient        streamClientFunc
	streamCall          *gomock.Call
	perRPCCredentials   credentials.PerRPCCredentials
	requestMetadataCall *gomock.Call
}

type noisyTest bool

const Noisy noisyTest = true
const NotNoisy noisyTest = false

func newTestTelemetry(t *testing.T, noisy noisyTest) (component.TelemetrySettings, *observer.ObservedLogs) {
	telset := componenttest.NewNopTelemetrySettings()
	if noisy {
		return telset, nil
	}
	core, obslogs := observer.New(zapcore.InfoLevel)
	telset.Logger = zap.New(zapcore.NewTee(core, zaptest.NewLogger(t).Core()))
	return telset, obslogs
}

func newCommonTestCase(t *testing.T, noisy noisyTest) *commonTestCase {
	ctrl := gomock.NewController(t)
	telset, obslogs := newTestTelemetry(t, noisy)

	creds := grpcmock.NewMockPerRPCCredentials(ctrl)
	creds.EXPECT().RequireTransportSecurity().Times(0) // unused interface method
	requestMetadataCall := creds.EXPECT().GetRequestMetadata(
		gomock.Any(), // context.Context
		gomock.Any(), // ...string (unused `uri` parameter)
	).Times(0)

	client := arrowCollectorMock.NewMockArrowStreamServiceClient(ctrl)

	streamCall := client.EXPECT().ArrowStream(
		gomock.Any(), // context.Context
		gomock.Any(), // ...grpc.CallOption
	).Times(0)
	return &commonTestCase{
		ctrl:                ctrl,
		telset:              telset,
		observedLogs:        obslogs,
		streamClient:        MakeAnyStreamClient(client.ArrowStream),
		streamCall:          streamCall,
		perRPCCredentials:   creds,
		requestMetadataCall: requestMetadataCall,
	}
}

type commonTestStream struct {
	anyStreamClient AnyStreamClient
	ctxCall         *gomock.Call
	sendCall        *gomock.Call
	recvCall        *gomock.Call
}

func (ctc *commonTestCase) newMockStream(ctx context.Context) *commonTestStream {
	client := arrowCollectorMock.NewMockArrowStreamService_ArrowStreamClient(ctc.ctrl)

	testStream := &commonTestStream{
		anyStreamClient: client,
		ctxCall:         client.EXPECT().Context().AnyTimes().Return(ctx),
		sendCall: client.EXPECT().Send(
			gomock.Any(), // *arrowpb.BatchArrowRecords
		).Times(0),
		recvCall: client.EXPECT().Recv().Times(0),
	}
	return testStream
}

// returnNewStream applies the list of test channels in order to
// construct new streams.  The final entry is re-used for new streams
// when it is reached.
func (ctc *commonTestCase) returnNewStream(hs ...testChannel) func(context.Context, ...grpc.CallOption) (
	arrowpb.ArrowStreamService_ArrowStreamClient,
	error,
) {
	var pos int
	return func(ctx context.Context, _ ...grpc.CallOption) (
		arrowpb.ArrowStreamService_ArrowStreamClient,
		error,
	) {
		h := hs[pos]
		if pos < len(hs) {
			pos++
		}
		if err := h.onConnect(ctx); err != nil {
			return nil, err
		}
		str := ctc.newMockStream(ctx)
		str.sendCall.AnyTimes().DoAndReturn(h.onSend(ctx))
		str.recvCall.AnyTimes().DoAndReturn(h.onRecv(ctx))
		return str.anyStreamClient, nil
	}
}

// repeatedNewStream returns a stream configured with a new test
// channel on every ArrowStream() request.
func (ctc *commonTestCase) repeatedNewStream(nc func() testChannel) func(context.Context, ...grpc.CallOption) (
	arrowpb.ArrowStreamService_ArrowStreamClient,
	error,
) {
	return func(ctx context.Context, opts ...grpc.CallOption) (
		arrowpb.ArrowStreamService_ArrowStreamClient,
		error,
	) {
		h := nc()
		if err := h.onConnect(ctx); err != nil {
			return nil, err
		}
		str := ctc.newMockStream(ctx)
		str.sendCall.AnyTimes().DoAndReturn(h.onSend(ctx))
		str.recvCall.AnyTimes().DoAndReturn(h.onRecv(ctx))
		return str.anyStreamClient, nil
	}
}

// healthyTestChannel accepts the connection and returns an OK status immediately.
type healthyTestChannel struct {
	sent chan *arrowpb.BatchArrowRecords
	recv chan *arrowpb.BatchStatus
}

func newHealthyTestChannel() *healthyTestChannel {
	return &healthyTestChannel{
		sent: make(chan *arrowpb.BatchArrowRecords),
		recv: make(chan *arrowpb.BatchStatus),
	}
}

func (tc *healthyTestChannel) onConnect(_ context.Context) error {
	return nil
}

func (tc *healthyTestChannel) onSend(ctx context.Context) func(*arrowpb.BatchArrowRecords) error {
	return func(req *arrowpb.BatchArrowRecords) error {
		select {
		case tc.sent <- req:
			return nil
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

func (tc *healthyTestChannel) onRecv(ctx context.Context) func() (*arrowpb.BatchStatus, error) {
	return func() (*arrowpb.BatchStatus, error) {
		select {
		case recv, ok := <-tc.recv:
			if !ok {
				return nil, io.EOF
			}

			return recv, nil
		case <-ctx.Done():
			return &arrowpb.BatchStatus{}, ctx.Err()
		}
	}
}

// unresponsiveTestChannel accepts the connection and receives data,
// but never responds with status OK.
type unresponsiveTestChannel struct {
	ch chan struct{}
}

func newUnresponsiveTestChannel() *unresponsiveTestChannel {
	return &unresponsiveTestChannel{
		ch: make(chan struct{}),
	}
}

func (tc *unresponsiveTestChannel) onConnect(_ context.Context) error {
	return nil
}

func (tc *unresponsiveTestChannel) onSend(ctx context.Context) func(*arrowpb.BatchArrowRecords) error {
	return func(req *arrowpb.BatchArrowRecords) error {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			return nil
		}
	}
}

func (tc *unresponsiveTestChannel) onRecv(ctx context.Context) func() (*arrowpb.BatchStatus, error) {
	return func() (*arrowpb.BatchStatus, error) {
		select {
		case <-tc.ch:
			return nil, io.EOF
		case <-ctx.Done():
			return &arrowpb.BatchStatus{}, ctx.Err()
		}
	}
}

func (tc *unresponsiveTestChannel) unblock() {
	close(tc.ch)
}

// unsupportedTestChannel mimics gRPC's behavior when there is no
// arrow stream service registered with the server.
type arrowUnsupportedTestChannel struct {
}

func newArrowUnsupportedTestChannel() *arrowUnsupportedTestChannel {
	return &arrowUnsupportedTestChannel{}
}

func (tc *arrowUnsupportedTestChannel) onConnect(_ context.Context) error {
	// Note: this matches gRPC's apparent behavior. the stream
	// connection succeeds and the unsupported code is returned to
	// the Recv() call.
	return nil
}

func (tc *arrowUnsupportedTestChannel) onSend(ctx context.Context) func(*arrowpb.BatchArrowRecords) error {
	return func(req *arrowpb.BatchArrowRecords) error {
		<-ctx.Done()
		return ctx.Err()
	}
}

func (tc *arrowUnsupportedTestChannel) onRecv(ctx context.Context) func() (*arrowpb.BatchStatus, error) {
	return func() (*arrowpb.BatchStatus, error) {
		err := status.Error(codes.Unimplemented, "arrow will not be served")
		return &arrowpb.BatchStatus{}, err
	}
}

// disconnectedTestChannel allows the connection to time out.
type disconnectedTestChannel struct {
}

func newDisconnectedTestChannel() *disconnectedTestChannel {
	return &disconnectedTestChannel{}
}

func (tc *disconnectedTestChannel) onConnect(ctx context.Context) error {
	<-ctx.Done()
	return ctx.Err()
}

func (tc *disconnectedTestChannel) onSend(ctx context.Context) func(*arrowpb.BatchArrowRecords) error {
	return func(req *arrowpb.BatchArrowRecords) error {
		panic("unreachable")
	}
}

func (tc *disconnectedTestChannel) onRecv(ctx context.Context) func() (*arrowpb.BatchStatus, error) {
	return func() (*arrowpb.BatchStatus, error) {
		panic("unreachable")
	}
}

// sendErrorTestChannel returns an error in Send()
type sendErrorTestChannel struct {
	release chan struct{}
}

func newSendErrorTestChannel() *sendErrorTestChannel {
	return &sendErrorTestChannel{
		release: make(chan struct{}),
	}
}

func (tc *sendErrorTestChannel) onConnect(ctx context.Context) error {
	return nil
}

func (tc *sendErrorTestChannel) onSend(ctx context.Context) func(*arrowpb.BatchArrowRecords) error {
	return func(*arrowpb.BatchArrowRecords) error {
		return io.EOF
	}
}

func (tc *sendErrorTestChannel) unblock() {
	close(tc.release)
}

func (tc *sendErrorTestChannel) onRecv(ctx context.Context) func() (*arrowpb.BatchStatus, error) {
	return func() (*arrowpb.BatchStatus, error) {
		<-tc.release
		return &arrowpb.BatchStatus{}, io.EOF
	}
}

// connectErrorTestChannel returns an error from the ArrowStream() call
type connectErrorTestChannel struct {
}

func newConnectErrorTestChannel() *connectErrorTestChannel {
	return &connectErrorTestChannel{}
}

func (tc *connectErrorTestChannel) onConnect(ctx context.Context) error {
	return fmt.Errorf("test connect error")
}

func (tc *connectErrorTestChannel) onSend(ctx context.Context) func(*arrowpb.BatchArrowRecords) error {
	return func(*arrowpb.BatchArrowRecords) error {
		panic("not reached")
	}
}

func (tc *connectErrorTestChannel) onRecv(ctx context.Context) func() (*arrowpb.BatchStatus, error) {
	return func() (*arrowpb.BatchStatus, error) {
		panic("not reached")
	}
}
