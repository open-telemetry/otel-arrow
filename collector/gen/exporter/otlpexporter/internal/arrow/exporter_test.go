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
	"encoding/json"
	"errors"
	"sync"
	"testing"
	"time"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrowRecord "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
	arrowRecordMock "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record/mock"
	otelAssert "github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/atomic"

	"github.com/f5/otel-arrow-adapter/collector/gen/internal/testdata"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
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

type exporterTestCase struct {
	*commonTestCase
	exporter *Exporter
}

func newExporterTestCase(t *testing.T, noisy noisyTest, arrowset Settings) *exporterTestCase {
	ctc := newCommonTestCase(t, noisy)
	exp := NewExporter(arrowset, func() arrowRecord.ProducerAPI {
		// Mock the close function, use a real producer for testing dataflow.
		prod := arrowRecordMock.NewMockProducerAPI(ctc.ctrl)
		real := arrowRecord.NewProducer()

		prod.EXPECT().BatchArrowRecordsFromTraces(gomock.Any()).AnyTimes().DoAndReturn(
			real.BatchArrowRecordsFromTraces)
		prod.EXPECT().BatchArrowRecordsFromLogs(gomock.Any()).AnyTimes().DoAndReturn(
			real.BatchArrowRecordsFromLogs)
		prod.EXPECT().BatchArrowRecordsFromMetrics(gomock.Any()).AnyTimes().DoAndReturn(
			real.BatchArrowRecordsFromMetrics)
		prod.EXPECT().Close().Times(1).Return(nil)
		return prod
	}, ctc.telset, ctc.serviceClient, nil)

	return &exporterTestCase{
		commonTestCase: ctc,
		exporter:       exp,
	}
}

func statusOKFor(id string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:    id,
				StatusCode: arrowpb.StatusCode_OK,
			},
		},
	}
}

func statusUnavailableFor(id string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:      id,
				StatusCode:   arrowpb.StatusCode_ERROR,
				ErrorCode:    arrowpb.ErrorCode_UNAVAILABLE,
				ErrorMessage: "test unavailable",
			},
		},
	}
}

func statusInvalidFor(id string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:      id,
				StatusCode:   arrowpb.StatusCode_ERROR,
				ErrorCode:    arrowpb.ErrorCode_INVALID_ARGUMENT,
				ErrorMessage: "test invalid",
			},
		},
	}
}

func statusUnrecognizedFor(id string) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		Statuses: []*arrowpb.StatusMessage{
			{
				BatchId:      id,
				StatusCode:   arrowpb.StatusCode_ERROR,
				ErrorCode:    1 << 20,
				ErrorMessage: "test unrecognized",
			},
		},
	}
}

// TestArrowExporterSuccess tests a single Send through a healthy channel.
func TestArrowExporterSuccess(t *testing.T) {
	for _, inputData := range []interface{}{twoTraces, twoMetrics, twoLogs} {
		tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
		channel := newHealthyTestChannel()

		tc.streamCall.Times(1).DoAndReturn(tc.returnNewStream(channel))

		ctx := context.Background()
		require.NoError(t, tc.exporter.Start(ctx))

		var wg sync.WaitGroup
		var outputData *arrowpb.BatchArrowRecords
		wg.Add(1)
		go func() {
			defer wg.Done()
			outputData = <-channel.sent
			channel.recv <- statusOKFor(outputData.BatchId)
		}()

		sent, err := tc.exporter.SendAndWait(ctx, inputData)
		require.NoError(t, err)
		require.True(t, sent)

		wg.Wait()

		testCon := arrowRecord.NewConsumer()
		switch testData := inputData.(type) {
		case ptrace.Traces:
			traces, err := testCon.TracesFrom(outputData)
			require.NoError(t, err)
			require.Equal(t, 1, len(traces))
			otelAssert.Equiv(t, []json.Marshaler{
				compareJSONTraces{testData},
			}, []json.Marshaler{
				compareJSONTraces{traces[0]},
			})
		case plog.Logs:
			logs, err := testCon.LogsFrom(outputData)
			require.NoError(t, err)
			require.Equal(t, 1, len(logs))
			otelAssert.Equiv(t, []json.Marshaler{
				compareJSONLogs{testData},
			}, []json.Marshaler{
				compareJSONLogs{logs[0]},
			})
		case pmetric.Metrics:
			metrics, err := testCon.MetricsFrom(outputData)
			require.NoError(t, err)
			require.Equal(t, 1, len(metrics))
			otelAssert.Equiv(t, []json.Marshaler{
				compareJSONMetrics{testData},
			}, []json.Marshaler{
				compareJSONMetrics{metrics[0]},
			})
		}

		require.NoError(t, tc.exporter.Shutdown(ctx))
	}
}

// TestArrowExporterTimeout tests that single slow Send leads to context canceled.
func TestArrowExporterTimeout(t *testing.T) {
	tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
	channel := newUnresponsiveTestChannel()

	tc.streamCall.Times(1).DoAndReturn(tc.returnNewStream(channel))

	ctx, cancel := context.WithCancel(context.Background())
	require.NoError(t, tc.exporter.Start(ctx))

	go func() {
		time.Sleep(200 * time.Millisecond)
		cancel()
	}()
	_, err := tc.exporter.SendAndWait(ctx, twoTraces)
	require.Error(t, err)
	require.True(t, errors.Is(err, context.Canceled))

	require.NoError(t, tc.exporter.Shutdown(ctx))
}

// TestConnectError tests that if the connetions fail fast the
// stream object for some reason is nil.  This causes downgrade.
func TestArrowExporterStreamConnectError(t *testing.T) {
	tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
	channel := newConnectErrorTestChannel()

	tc.streamCall.AnyTimes().DoAndReturn(tc.returnNewStream(channel))

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	sent, err := tc.exporter.SendAndWait(bg, twoTraces)
	require.False(t, sent)
	require.NoError(t, err)

	require.NoError(t, tc.exporter.Shutdown(bg))

	require.Less(t, 0, len(tc.observedLogs.All()), "should have at least one log: %v", tc.observedLogs.All())
	require.Equal(t, tc.observedLogs.All()[0].Message, "cannot start arrow stream")
}

// TestArrowExporterDowngrade tests that if the Recv() returns an
// Unimplemented code (as gRPC does) that the connection is downgraded
// without error.
func TestArrowExporterDowngrade(t *testing.T) {
	tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
	channel := newArrowUnsupportedTestChannel()

	tc.streamCall.AnyTimes().DoAndReturn(tc.returnNewStream(channel))

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	sent, err := tc.exporter.SendAndWait(bg, twoTraces)
	require.False(t, sent)
	require.NoError(t, err)

	require.NoError(t, tc.exporter.Shutdown(bg))

	require.Less(t, 1, len(tc.observedLogs.All()), "should have at least two logs: %v", tc.observedLogs.All())
	require.Equal(t, tc.observedLogs.All()[0].Message, "arrow is not supported")
	require.Contains(t, tc.observedLogs.All()[1].Message, "downgrading")
}

// TestArrowExporterConnectTimeout tests that an error is returned to
// the caller if the response does not arrive in time.
func TestArrowExporterConnectTimeout(t *testing.T) {
	tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
	channel := newDisconnectedTestChannel()

	tc.streamCall.AnyTimes().DoAndReturn(tc.returnNewStream(channel))

	bg := context.Background()
	ctx, cancel := context.WithCancel(bg)
	require.NoError(t, tc.exporter.Start(bg))

	go func() {
		time.Sleep(200 * time.Millisecond)
		cancel()
	}()
	_, err := tc.exporter.SendAndWait(ctx, twoTraces)
	require.Error(t, err)
	require.True(t, errors.Is(err, context.Canceled))

	require.NoError(t, tc.exporter.Shutdown(bg))
}

// TestArrowExporterStreamFailure tests that a single stream failure
// followed by a healthy stream.
func TestArrowExporterStreamFailure(t *testing.T) {
	tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
	channel0 := newUnresponsiveTestChannel()
	channel1 := newHealthyTestChannel()

	tc.streamCall.AnyTimes().DoAndReturn(tc.returnNewStream(channel0, channel1))

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	go func() {
		time.Sleep(200 * time.Millisecond)
		channel0.unblock()
	}()

	var wg sync.WaitGroup
	var outputData *arrowpb.BatchArrowRecords
	wg.Add(1)
	go func() {
		defer wg.Done()
		outputData = <-channel1.sent
		channel1.recv <- statusOKFor(outputData.BatchId)
	}()

	sent, err := tc.exporter.SendAndWait(bg, twoTraces)
	require.NoError(t, err)
	require.True(t, sent)

	wg.Wait()

	require.NoError(t, tc.exporter.Shutdown(bg))
}

// TestArrowExporterStreamRace reproduces the situation needed for a
// race between stream send and stream cancel, causing it to fully
// exercise the removeReady() code path.
func TestArrowExporterStreamRace(t *testing.T) {
	// Two streams ensures every possibility.
	tc := newExporterTestCase(t, Noisy, Settings{
		Enabled: true,

		// This creates the conditions likely to produce a
		// stream race in prioritizer.go.
		NumStreams: 20,
	})

	var tries atomic.Int32

	tc.streamCall.AnyTimes().DoAndReturn(tc.repeatedNewStream(func() testChannel {
		tc := newUnresponsiveTestChannel()
		// Immediately unblock to return the EOF to the stream
		// receiver and shut down the stream.
		go tc.unblock()
		tries.Add(1)
		return tc
	}))

	var wg sync.WaitGroup

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	callctx, cancel := context.WithCancel(bg)

	// These goroutines will repeatedly try for an available
	// stream, but none will become available.  Eventually the
	// context will be canceled and cause these goroutines to
	// return.
	for i := 0; i < 5; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			// This blocks until the cancelation.
			_, err := tc.exporter.SendAndWait(callctx, twoTraces)
			require.Error(t, err)
			require.True(t, errors.Is(err, context.Canceled))
		}()
	}

	// Wait until 1000 streams have started.
	assert.Eventually(t, func() bool {
		return tries.Load() >= 1000
	}, 10*time.Second, 5*time.Millisecond)

	cancel()
	wg.Wait()
	require.NoError(t, tc.exporter.Shutdown(bg))
}

// TestArrowExporterStreaming tests 10 sends in a row.
func TestArrowExporterStreaming(t *testing.T) {
	tc := newExporterTestCase(t, NotNoisy, singleStreamSettings)
	channel := newHealthyTestChannel()

	tc.streamCall.AnyTimes().DoAndReturn(tc.returnNewStream(channel))

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	var expectOutput []ptrace.Traces
	var actualOutput []ptrace.Traces
	testCon := arrowRecord.NewConsumer()

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		for data := range channel.sent {
			traces, err := testCon.TracesFrom(data)
			require.NoError(t, err)
			require.Equal(t, 1, len(traces))
			actualOutput = append(actualOutput, traces[0])
			channel.recv <- statusOKFor(data.BatchId)
		}
	}()

	for times := 0; times < 10; times++ {
		input := testdata.GenerateTraces(2)
		ctx := context.Background()

		sent, err := tc.exporter.SendAndWait(ctx, input)
		require.NoError(t, err)
		require.True(t, sent)

		expectOutput = append(expectOutput, input)
	}
	// Stop the test conduit started above.  If the sender were
	// still sending, it would panic on a closed channel.
	close(channel.sent)
	wg.Wait()

	require.Equal(t, expectOutput, actualOutput)
	require.NoError(t, tc.exporter.Shutdown(bg))
}
