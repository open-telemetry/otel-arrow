// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package arrow

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	arrowRecord "github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
	arrowRecordMock "github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record/mock"
	otelAssert "github.com/open-telemetry/otel-arrow/pkg/otel/assert"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"

	"github.com/open-telemetry/otel-arrow/collector/internal/testdata"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

const defaultMaxStreamLifetime = 11 * time.Second

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

func newSingleStreamTestCase(t *testing.T) *exporterTestCase {
	return newExporterTestCaseCommon(t, NotNoisy, 1, false, nil)
}

func newSingleStreamDowngradeDisabledTestCase(t *testing.T) *exporterTestCase {
	return newExporterTestCaseCommon(t, NotNoisy, 1, true, nil)
}

func newSingleStreamMetadataTestCase(t *testing.T) *exporterTestCase {
	var count int
	return newExporterTestCaseCommon(t, NotNoisy, 1, false, func(ctx context.Context) (map[string]string, error) {
		defer func() { count++ }()
		if count%2 == 0 {
			return nil, nil
		}
		return map[string]string{
			"expected1": "metadata1",
			"expected2": fmt.Sprint(count),
		}, nil
	})
}

func newExporterNoisyTestCase(t *testing.T, numStreams int) *exporterTestCase {
	return newExporterTestCaseCommon(t, Noisy, numStreams, false, nil)
}

func copyBatch[T any](real func(T) (*arrowpb.BatchArrowRecords, error)) func(T) (*arrowpb.BatchArrowRecords, error) {
	// Because Arrow-IPC uses zero copy, we have to copy inside the test
	// instead of sharing pointers to BatchArrowRecords.
	return func(data T) (*arrowpb.BatchArrowRecords, error) {
		in, err := real(data)
		if err != nil {
			return nil, err
		}

		hcpy := make([]byte, len(in.Headers))
		copy(hcpy, in.Headers)

		pays := make([]*arrowpb.ArrowPayload, len(in.ArrowPayloads))

		for i, inp := range in.ArrowPayloads {
			rcpy := make([]byte, len(inp.Record))
			copy(rcpy, inp.Record)
			pays[i] = &arrowpb.ArrowPayload{
				SchemaId: inp.SchemaId,
				Type:     inp.Type,
				Record:   rcpy,
			}
		}

		return &arrowpb.BatchArrowRecords{
			BatchId:       in.BatchId,
			Headers:       hcpy,
			ArrowPayloads: pays,
		}, nil
	}
}

func newExporterTestCaseCommon(t *testing.T, noisy noisyTest, numStreams int, disableDowngrade bool, metadataFunc func(ctx context.Context) (map[string]string, error)) *exporterTestCase {
	ctc := newCommonTestCase(t, noisy)

	if metadataFunc == nil {
		ctc.requestMetadataCall.AnyTimes().Return(nil, nil)
	} else {
		ctc.requestMetadataCall.AnyTimes().DoAndReturn(func(ctx context.Context, _ ...string) (map[string]string, error) {
			return metadataFunc(ctx)
		})
	}

	exp := NewExporter(defaultMaxStreamLifetime, numStreams, disableDowngrade, ctc.telset, nil, func() arrowRecord.ProducerAPI {
		// Mock the close function, use a real producer for testing dataflow.
		mock := arrowRecordMock.NewMockProducerAPI(ctc.ctrl)
		prod := arrowRecord.NewProducer()

		mock.EXPECT().BatchArrowRecordsFromTraces(gomock.Any()).AnyTimes().DoAndReturn(
			copyBatch(prod.BatchArrowRecordsFromTraces))
		mock.EXPECT().BatchArrowRecordsFromLogs(gomock.Any()).AnyTimes().DoAndReturn(
			copyBatch(prod.BatchArrowRecordsFromLogs))
		mock.EXPECT().BatchArrowRecordsFromMetrics(gomock.Any()).AnyTimes().DoAndReturn(
			copyBatch(prod.BatchArrowRecordsFromMetrics))
		mock.EXPECT().Close().Times(1).Return(nil)
		return mock
	}, ctc.streamClient, ctc.perRPCCredentials)

	return &exporterTestCase{
		commonTestCase: ctc,
		exporter:       exp,
	}
}

func statusOKFor(id int64) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		BatchId:    id,
		StatusCode: arrowpb.StatusCode_OK,
	}
}

func statusStreamShutdownFor(id int64) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		BatchId:    id,
		StatusCode: arrowpb.StatusCode_STREAM_SHUTDOWN,
	}
}

func statusUnavailableFor(id int64) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		BatchId:       id,
		StatusCode:    arrowpb.StatusCode_UNAVAILABLE,
		StatusMessage: "test unavailable",
	}
}

func statusInvalidFor(id int64) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		BatchId:       id,
		StatusCode:    arrowpb.StatusCode_INVALID_ARGUMENT,
		StatusMessage: "test invalid",
	}
}

func statusUnrecognizedFor(id int64) *arrowpb.BatchStatus {
	return &arrowpb.BatchStatus{
		BatchId:       id,
		StatusCode:    1 << 20,
		StatusMessage: "test unrecognized",
	}
}

// TestArrowExporterSuccess tests a single Send through a healthy channel.
func TestArrowExporterSuccess(t *testing.T) {
	stdTesting := otelAssert.NewStdUnitTest(t)
	for _, inputData := range []interface{}{twoTraces, twoMetrics, twoLogs} {
		tc := newSingleStreamTestCase(t)
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
			otelAssert.Equiv(stdTesting, []json.Marshaler{
				compareJSONTraces{testData},
			}, []json.Marshaler{
				compareJSONTraces{traces[0]},
			})
		case plog.Logs:
			logs, err := testCon.LogsFrom(outputData)
			require.NoError(t, err)
			require.Equal(t, 1, len(logs))
			otelAssert.Equiv(stdTesting, []json.Marshaler{
				compareJSONLogs{testData},
			}, []json.Marshaler{
				compareJSONLogs{logs[0]},
			})
		case pmetric.Metrics:
			metrics, err := testCon.MetricsFrom(outputData)
			require.NoError(t, err)
			require.Equal(t, 1, len(metrics))
			otelAssert.Equiv(stdTesting, []json.Marshaler{
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
	tc := newSingleStreamTestCase(t)
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
	tc := newSingleStreamTestCase(t)
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
	tc := newSingleStreamTestCase(t)
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

// TestArrowExporterDisableDowngrade tests that if the Recv() returns
// any error downgrade still does not occur amd that the connection is
// retried without error.
func TestArrowExporterDisableDowngrade(t *testing.T) {
	tc := newSingleStreamDowngradeDisabledTestCase(t)
	badChannel := newArrowUnsupportedTestChannel()
	goodChannel := newHealthyTestChannel()

	fails := 0
	tc.streamCall.AnyTimes().DoAndReturn(func(ctx context.Context, opts ...grpc.CallOption) (
		arrowpb.ArrowStreamService_ArrowStreamClient,
		error,
	) {
		defer func() { fails++ }()

		if fails < 3 {
			return tc.returnNewStream(badChannel)(ctx, opts...)
		}
		return tc.returnNewStream(goodChannel)(ctx, opts...)
	})

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		outputData := <-goodChannel.sent
		goodChannel.recv <- statusOKFor(outputData.BatchId)
	}()

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	sent, err := tc.exporter.SendAndWait(bg, twoTraces)
	require.True(t, sent)
	require.NoError(t, err)

	wg.Wait()

	require.NoError(t, tc.exporter.Shutdown(bg))

	require.Less(t, 1, len(tc.observedLogs.All()), "should have at least two logs: %v", tc.observedLogs.All())
	require.Equal(t, tc.observedLogs.All()[0].Message, "arrow is not supported")
	require.NotContains(t, tc.observedLogs.All()[1].Message, "downgrading")
}

// TestArrowExporterConnectTimeout tests that an error is returned to
// the caller if the response does not arrive in time.
func TestArrowExporterConnectTimeout(t *testing.T) {
	tc := newSingleStreamTestCase(t)
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
	tc := newSingleStreamTestCase(t)
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
	// This creates the conditions likely to produce a
	// stream race in prioritizer.go.
	tc := newExporterNoisyTestCase(t, 20)

	var tries atomic.Int32

	tc.streamCall.AnyTimes().DoAndReturn(tc.repeatedNewStream(func() testChannel {
		noResponse := newUnresponsiveTestChannel()
		// Immediately unblock to return the EOF to the stream
		// receiver and shut down the stream.
		go noResponse.unblock()
		tries.Add(1)
		return noResponse
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
	tc := newSingleStreamTestCase(t)
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

	// As this equality check doesn't support out of order slices,
	// we sort the slices directly in the GenerateTraces function.
	require.Equal(t, expectOutput, actualOutput)
	require.NoError(t, tc.exporter.Shutdown(bg))
}

// TestArrowExporterHeaders tests a mix of outgoing context headers.
func TestArrowExporterHeaders(t *testing.T) {
	tc := newSingleStreamMetadataTestCase(t)
	channel := newHealthyTestChannel()

	tc.streamCall.AnyTimes().DoAndReturn(tc.returnNewStream(channel))

	bg := context.Background()
	require.NoError(t, tc.exporter.Start(bg))

	var expectOutput []metadata.MD
	var actualOutput []metadata.MD

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		md := metadata.MD{}
		hpd := hpack.NewDecoder(4096, func(f hpack.HeaderField) {
			md[f.Name] = append(md[f.Name], f.Value)
		})
		for data := range channel.sent {
			if len(data.Headers) == 0 {
				actualOutput = append(actualOutput, nil)
			} else {
				_, err := hpd.Write(data.Headers)
				require.NoError(t, err)
				actualOutput = append(actualOutput, md)
				md = metadata.MD{}
			}
			channel.recv <- statusOKFor(data.BatchId)
		}
	}()

	for times := 0; times < 10; times++ {
		input := testdata.GenerateTraces(2)
		ctx := context.Background()

		if times%2 == 1 {
			md := metadata.MD{
				"expected1": []string{"metadata1"},
				"expected2": []string{fmt.Sprint(times)},
			}
			expectOutput = append(expectOutput, md)
		} else {
			expectOutput = append(expectOutput, nil)
		}

		sent, err := tc.exporter.SendAndWait(ctx, input)
		require.NoError(t, err)
		require.True(t, sent)
	}
	// Stop the test conduit started above.  If the sender were
	// still sending, it would panic on a closed channel.
	close(channel.sent)
	wg.Wait()

	require.Equal(t, expectOutput, actualOutput)
	require.NoError(t, tc.exporter.Shutdown(bg))
}

func TestAddJitter(t *testing.T) {
	require.Equal(t, time.Duration(0), addJitter(0))

	// Expect no more than 5% less in each trial.
	for i := 0; i < 100; i++ {
		x := addJitter(20 * time.Minute)
		require.LessOrEqual(t, 19*time.Minute, x)
		require.Less(t, x, 20*time.Minute)
	}
}
