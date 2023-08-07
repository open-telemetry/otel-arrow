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

package traces_test

import (
	"encoding/hex"
	"encoding/json"
	"math"
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/stats"
	tracesarrow "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
	tracesotlp "github.com/f5/otel-arrow-adapter/pkg/otel/traces/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
)

var DefaultDictConfig = cfg.NewDictionary(math.MaxUint16)
var ProducerStats = stats.NewProducerStats()

// TestTracesEncodingDecoding tests the conversion of OTLP traces to OTel Arrow traces
// and back to OTLP. The initial OTLP traces are generated from a synthetic
// dataset.
//
// The validation process is based on the JSON comparison the OTLP traces generated
// and the OTLP traces decoded from the OTel Arrow traces. This comparison is strict
// and accept differences in the order of the fields.
func TestTracesEncodingDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	tracesGen := datagen.NewTracesGenerator(
		entropy,
		entropy.NewStandardResourceAttributes(),
		entropy.NewStandardInstrumentationScopes(),
	)

	expectedRequest := ptraceotlp.NewExportRequestFromTraces(
		tracesGen.Generate(100, 100))
	CheckEncodeDecode(t, expectedRequest)
}

func TestRandomTracesEncodingDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	tracesGen := datagen.NewTracesGenerator(
		entropy,
		entropy.NewRandomResourceAttributes(10),
		entropy.NewRandomInstrumentationScopes(10),
	)

	for i := 0; i < 100; i++ {
		expectedRequest := ptraceotlp.NewExportRequestFromTraces(
			tracesGen.GenerateRandomTraces(1, 100))
		CheckEncodeDecode(t, expectedRequest)
	}
}

func TestCustom1TracesEncodingDecoding(t *testing.T) {
	t.Parallel()

	expected := ptrace.NewTraces()
	rs := expected.ResourceSpans().AppendEmpty()
	rs.Resource().Attributes().PutStr("hostname", "host3.mydomain.com")
	rs.Resource().Attributes().PutStr("unique3", "uv3")
	rs.Resource().Attributes().PutStr("ip", "192.168.0.3")
	rs.Resource().Attributes().PutDouble("version", 1.5)
	rs.Resource().Attributes().PutInt("status", 500)
	rs.Resource().Attributes().PutBool("up", false)
	rs.SetSchemaUrl("https://opentelemetry.io/schemas/1.0.0")

	ss := rs.ScopeSpans().AppendEmpty()
	scope := ss.Scope()
	scope.SetName("fake_generator")
	scope.SetVersion("1.0.1")

	span := ss.Spans().AppendEmpty()
	span.SetTraceID(traceID("6d759c9c5e1a049927ca069a497b0508"))
	span.SetSpanID(spanID("90d5ead3745935bd"))
	span.TraceState().FromRaw("maiores")
	span.SetKind(2)
	span.SetDroppedAttributesCount(9)
	span.SetDroppedEventsCount(9)
	span.SetDroppedLinksCount(6)
	span.Status().SetMessage("OK")

	span = ss.Spans().AppendEmpty()
	span.SetTraceID(traceID("72e8551d2f079f29231aa57088384785"))
	span.SetSpanID(spanID("35ce5d0711df60f2"))
	span.SetParentSpanID(spanID("35ce5d0711df60f2"))
	span.SetName("GET /user-info")
	span.SetStartTimestamp(1668124800000010667)
	span.SetEndTimestamp(1668124800000010668)
	span.SetDroppedAttributesCount(8)

	event := span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010674)
	event.SetName("odit")
	event.SetDroppedAttributesCount(2)
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event.SetName("velit")
	event.Attributes().PutStr("attr_0", "est")
	event.Attributes().PutDouble("attr_1", 0.017895097521176077)
	event.Attributes().PutStr("attr_2", "consectetur")
	event.SetDroppedAttributesCount(9)
	event = span.Events().AppendEmpty()
	event.SetName("exercitationem")
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event.SetName("soluta")
	event.SetDroppedAttributesCount(9)
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event.SetDroppedAttributesCount(7)
	event = span.Events().AppendEmpty()

	link := span.Links().AppendEmpty()
	link.SetTraceID(traceID("72e8551d2f079f29231aa57088384785"))
	link.TraceState().FromRaw("ut")
	link.Attributes().PutInt("attr_0", 4055508854307121380)
	link.Attributes().PutInt("attr_1", 2603754219448080514)
	link.Attributes().PutStr("attr_2", "ut")
	link.Attributes().PutInt("attr_3", 542986775976848616)
	link.Attributes().PutInt("attr_4", 5562030613432072994)
	link.SetDroppedAttributesCount(8)
	link = span.Links().AppendEmpty()
	link.TraceState().FromRaw("vel")
	link.SetDroppedAttributesCount(6)

	span.Status().SetCode(1)

	expectedRequest := ptraceotlp.NewExportRequestFromTraces(expected)
	CheckEncodeDecode(t, expectedRequest)
}

func TestCustom2TracesEncodingDecoding(t *testing.T) {
	t.Parallel()

	expected := ptrace.NewTraces()
	rs := expected.ResourceSpans().AppendEmpty()
	rs.SetSchemaUrl("https://opentelemetry.io/schemas/1.0.0")

	// First scope span with a scope
	ss := rs.ScopeSpans().AppendEmpty()
	scope := ss.Scope()
	scope.SetName("fake_generator")
	scope.SetVersion("1.0.1")

	span := ss.Spans().AppendEmpty()
	span.SetTraceID(traceID("6d759c9c5e1a049927ca069a497b0508"))
	span.SetSpanID(spanID("90d5ead3745935bd"))
	span.TraceState().FromRaw("maiores")
	span.SetKind(2)
	span.SetDroppedAttributesCount(9)
	span.SetDroppedEventsCount(9)
	span.SetDroppedLinksCount(6)
	span.Status().SetMessage("OK")

	span = ss.Spans().AppendEmpty()
	span.SetTraceID(traceID("72e8551d2f079f29231aa57088384785"))
	span.SetSpanID(spanID("35ce5d0711df60f2"))
	span.SetParentSpanID(spanID("35ce5d0711df60f2"))
	span.SetName("GET /user-info")
	span.SetStartTimestamp(1668124800000010667)
	span.SetEndTimestamp(1668124800000010668)
	span.SetDroppedAttributesCount(8)

	event := span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010674)
	event.SetName("odit")
	event.SetDroppedAttributesCount(2)
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event.SetName("velit")
	event.Attributes().PutStr("attr_0", "est")
	event.Attributes().PutDouble("attr_1", 0.017895097521176077)
	event.Attributes().PutStr("attr_2", "consectetur")
	event.SetDroppedAttributesCount(9)
	event = span.Events().AppendEmpty()
	event.SetName("exercitationem")
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event.SetName("soluta")
	event.SetDroppedAttributesCount(9)
	event = span.Events().AppendEmpty()
	event.SetTimestamp(1668124800000010672)
	event.SetDroppedAttributesCount(7)
	event = span.Events().AppendEmpty()

	link := span.Links().AppendEmpty()
	link.SetTraceID(traceID("72e8551d2f079f29231aa57088384785"))
	link.TraceState().FromRaw("ut")
	link.Attributes().PutInt("attr_0", 4055508854307121380)
	link.Attributes().PutInt("attr_1", 2603754219448080514)
	link.Attributes().PutStr("attr_2", "ut")
	link.Attributes().PutInt("attr_3", 542986775976848616)
	link.Attributes().PutInt("attr_4", 5562030613432072994)
	link.SetDroppedAttributesCount(8)
	link = span.Links().AppendEmpty()
	link.TraceState().FromRaw("vel")
	link.SetDroppedAttributesCount(6)

	span.Status().SetCode(1)

	// Second scope span with an empty scope
	ss = rs.ScopeSpans().AppendEmpty()
	scope = ss.Scope()

	span = ss.Spans().AppendEmpty()
	span.SetTraceID(traceID("6d759c9c5e1a049927ca069a497b0666"))
	span.SetSpanID(spanID("90d5ead3745923ab"))
	span.TraceState().FromRaw("maiores2")
	span.SetKind(1)
	span.SetDroppedAttributesCount(0)
	span.SetDroppedEventsCount(0)
	span.SetDroppedLinksCount(0)
	span.Status().SetMessage("OK")

	span = ss.Spans().AppendEmpty()
	span.SetTraceID(traceID("72e8551d2f079f29231aa57088384999"))
	span.SetSpanID(spanID("35ce5d0711df61f3"))
	span.SetParentSpanID(spanID("35ce5d0711df61f3"))
	span.SetName("POST /user-info")
	span.SetStartTimestamp(1668124800000010456)
	span.SetEndTimestamp(1668124800000010123)
	span.SetDroppedAttributesCount(0)

	expectedRequest := ptraceotlp.NewExportRequestFromTraces(expected)
	CheckEncodeDecode(t, expectedRequest)
}

// TestInvalidTracesDecoding is similar to TestLogsEncodingDecoding but introduces
// some random modification of the Arrow Records used to represent OTel traces.
// These modifications should be handled gracefully by the decoding process and
// generate an error but should never panic.
func TestInvalidTracesDecoding(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	tracesGen := datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	// Generate a random OTLP traces request.
	expectedRequest := ptraceotlp.NewExportRequestFromTraces(tracesGen.GenerateRandomTraces(2000, 100))

	MultiRoundOfCheckEncodeMessUpDecode(t, expectedRequest)
}

func CheckEncodeDecode(
	t *testing.T,
	expectedRequest ptraceotlp.ExportRequest,
) {
	stdTesting := assert.NewStdUnitTest(t)

	// Convert the OTLP traces request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, tracesarrow.TracesSchema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		tb, err := tracesarrow.NewTracesBuilder(rBuilder, tracesarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(expectedRequest.Traces())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = tb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	relatedData, _, err := tracesotlp.RelatedDataFrom(relatedRecords, tracesarrow.NewConfig(conf))
	require.NoError(t, err)

	// Convert the Arrow record back to OTLP.
	traces, err := tracesotlp.TracesFrom(record, relatedData)
	require.NoError(t, err)

	record.Release()

	assert.Equiv(stdTesting, []json.Marshaler{expectedRequest}, []json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)})
}

func MultiRoundOfCheckEncodeMessUpDecode(
	t *testing.T,
	expectedRequest ptraceotlp.ExportRequest,
) {
	rng := rand.New(rand.NewSource(int64(rand.Uint64())))

	for i := 0; i < 100; i++ {
		CheckEncodeMessUpDecode(t, expectedRequest, rng)
	}
}

func CheckEncodeMessUpDecode(
	t *testing.T,
	expectedRequest ptraceotlp.ExportRequest,
	rng *rand.Rand,
) {
	// Convert the OTLP traces request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, tracesarrow.TracesSchema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		tb, err := tracesarrow.NewTracesBuilder(rBuilder, tracesarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		defer tb.Release()

		err = tb.Append(expectedRequest.Traces())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = tb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	// Mix up the Arrow records in such a way as to make decoding impossible.
	mainRecordChanged, record, relatedRecords := common.MixUpArrowRecords(rng, record, relatedRecords)

	relatedData, _, err := tracesotlp.RelatedDataFrom(relatedRecords, tracesarrow.NewConfig(conf))

	// Convert the Arrow record back to OTLP.
	_, err = tracesotlp.TracesFrom(record, relatedData)

	if mainRecordChanged || relatedData == nil {
		require.Error(t, err)
	} else {
		require.NoError(t, err)
	}

	record.Release()
}

// TestConversionFromRealData tests the conversion of OTLP traces to Arrow and back to OTLP.
// The initial OTLP traces are generated from a real dataset (anonymized).
// This test is based on the JSON serialization of the initial generated OTLP traces compared to the JSON serialization
// of the OTLP traces generated from the Arrow records.
func TestConversionFromRealData(t *testing.T) {
	t.Parallel()
	t.Skip("Testing based on production data that is not stored in the")

	// Load a real OTLP traces request.
	ds := dataset.NewRealTraceDataset("../../../data/nth_first_otlp_traces.pb", "", "proto", []string{"trace_id"})

	batchSizes := []int{1, 10, 100, 1000, 5000, 10000}
	for _, batchSize := range batchSizes {
		tracesList := ds.Traces(0, batchSize)
		expectedRequest := ptraceotlp.NewExportRequestFromTraces(tracesList[0])

		// Convert the OTLP traces request to Arrow.
		checkTracesConversion(t, expectedRequest)
	}
}

func checkTracesConversion(t *testing.T, expectedRequest ptraceotlp.ExportRequest) { //nolint:unused // only used for testing
	t.Helper()

	stdTesting := assert.NewStdUnitTest(t)
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, tracesarrow.TracesSchema, DefaultDictConfig, ProducerStats)
	defer rBuilder.Release()

	var record arrow.Record
	var relatedRecords []*record_message.RecordMessage

	conf := config.DefaultConfig()

	for {
		tb, err := tracesarrow.NewTracesBuilder(rBuilder, tracesarrow.NewConfig(conf), stats.NewProducerStats())
		require.NoError(t, err)
		err = tb.Append(expectedRequest.Traces())
		require.NoError(t, err)
		record, err = rBuilder.NewRecord()
		if err == nil {
			relatedRecords, err = tb.RelatedData().BuildRecordMessages()
			require.NoError(t, err)
			break
		}
		require.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	relatedData, _, err := tracesotlp.RelatedDataFrom(relatedRecords, tracesarrow.NewConfig(conf))
	require.NoError(t, err)

	// Convert the Arrow records back to OTLP.
	traces, err := tracesotlp.TracesFrom(record, relatedData)
	if err != nil {
		t.Fatal(err)
	}

	defer record.Release()

	assert.Equiv(stdTesting, []json.Marshaler{expectedRequest}, []json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)})
}

func traceID(id string) [16]byte {
	data, err := hex.DecodeString(id)
	if err != nil {
		panic(err)
	}
	var traceID [16]byte
	copy(traceID[:], data[:16])
	return traceID
}

func spanID(id string) [8]byte {
	data, err := hex.DecodeString(id)
	if err != nil {
		panic(err)
	}
	var spanID [8]byte
	copy(spanID[:], data[:8])
	return spanID
}
