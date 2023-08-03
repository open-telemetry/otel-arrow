package obfuscationprocessor

import (
	"context"
	"testing"

	"github.com/cyrildever/feistel"
	"github.com/cyrildever/feistel/common/utils/hash"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

var (
	resAttrKey       = "resource-attr"
	scopeAttrKey     = "scope-attr"
	spanAttrKey      = "span-attr"
	spanEventAttrKey = "span-event-attr"
	spanLinkAttrKey  = "span-link-attr"
	byteIDKey        = "byte-id"

	resAttrVal   = "resource-attr-val-1"
	scopeAttrVal = "scope-attr-val-1"
	byteIDVal = []byte("abcdefg")

	// span specific attrs
	spanAttrVal  = "span-attr-val-1"
	eventAttrVal = "event-attr-val-1"
	linkAttrVal  = "link-attr-val-1"

	// metric specific attrs
	gaugeAttrVal   = "gauge-attr-val-1"
	sumAttrVal     = "sum-attr-val-1"
	histAttrVal    = "hist-attr-val-1"
	eHistAttrVal   = "exp-hist-attr-val-1"
	summaryAttrVal = "summary-attr-val-1"

	// log specific attrs
	logAttrVal = "log-attr-val-1"
)

// returns a map that has a more complicated structure to obfuscate
func newFilledMap() pcommon.Map {
	kv := pcommon.NewMap()

	mp := kv.PutEmptyMap("baz")
	slc1 := mp.PutEmptySlice("foo")
	elt1 := slc1.AppendEmpty()
	elt1.SetStr("fooval1")
	elt2 := slc1.AppendEmpty()
	elt2.SetEmptyBytes().FromRaw([]byte("fooval2"))

	slc2 := mp.PutEmptySlice("bar")
	elt3 := slc2.AppendEmpty()
	elt3.SetStr("barval1")
	elt4 := slc2.AppendEmpty()
	elt4.SetEmptyBytes().FromRaw([]byte("fooval2"))

	return kv
}

func setupSpanWithAttrs() ptrace.Traces {
	td := ptrace.NewTraces()
	rs := td.ResourceSpans().AppendEmpty()

	rs.Resource().Attributes().PutStr("resource-attr", resAttrVal)

	ss := rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("scope-attr", scopeAttrVal)

	span := ss.Spans().AppendEmpty()
	span.SetName("operationA")
	span.Attributes().PutStr("span-attr", spanAttrVal)
	span.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp := span.Attributes().PutEmptyMap("complex-span-attr")
	newFilledMap().CopyTo(mp)

	link0 := span.Links().AppendEmpty()
	link0.Attributes().PutStr("span-link-attr", linkAttrVal)
	ev0 := span.Events().AppendEmpty()
	ev0.Attributes().PutStr("span-event-attr", eventAttrVal)

	return td
}

func validateTraceAttrs(t *testing.T, expected map[string]pair, traces ptrace.Traces) {
	for i := 0; i < traces.ResourceSpans().Len(); i++ {
		// validate resource attributes
		rs := traces.ResourceSpans().At(i)
		val, ok := rs.Resource().Attributes().Get(expected["resource-attr"].key)
		assert.True(t, ok)
		assert.Equal(t, expected["resource-attr"].val.AsString(), val.AsString())

		for j := 0; j < rs.ScopeSpans().Len(); j++ {
			// validate scope attributes
			ss := rs.ScopeSpans().At(j)
			scopeVal, ok := ss.Scope().Attributes().Get(expected["scope-attr"].key)
			assert.True(t, ok)
			assert.Equal(t, expected["scope-attr"].val.AsString(), scopeVal.AsString())

			for k := 0; k < ss.Spans().Len(); k++ {
				// validate span attributes
				span := ss.Spans().At(k)
				val, ok := span.Attributes().Get(expected["span-attr"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["span-attr"].val.AsString(), val.AsString())
				val2, ok := span.Attributes().Get(expected["complex-span-attr"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["complex-span-attr"].val.AsString(), val2.AsString())
				val3, ok := span.Attributes().Get(expected["byte-id"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["byte-id"].val.AsString(), val3.AsString())

				for h := 0; h < span.Events().Len(); h++ {
					// validate event attributes
					event := span.Events().At(h)
					val, ok := event.Attributes().Get(expected["span-event-attr"].key)
					assert.True(t, ok)
					assert.Equal(t, expected["span-event-attr"].val.AsString(), val.AsString())
				}

				for h := 0; h < span.Links().Len(); h++ {
					// validate link attributes
					link := span.Links().At(h)
					val, ok := link.Attributes().Get(expected["span-link-attr"].key)
					assert.True(t, ok)
					assert.Equal(t, expected["span-link-attr"].val.AsString(), val.AsString())
				}
			}
		}
	}
}

type pair struct {
	key string
	val pcommon.Value
}

func cryptPair(o *obfuscation, k string, v pcommon.Value) pair {
	ok, _ := o.encrypt.Encrypt(k)
	switch v.Type() {
	case pcommon.ValueTypeStr:
		ov, _ := o.encrypt.Encrypt(v.Str())
		v.SetStr(ov.String(true))
	case pcommon.ValueTypeBytes:
		obf := o.encryptStringToBytes(string(v.Bytes().AsRaw()))
		byteSlice := v.SetEmptyBytes()
		byteSlice.FromRaw(obf)
	case pcommon.ValueTypeSlice:
		o.processSliceValue(context.Background(), v.Slice())
	case pcommon.ValueTypeMap:
		o.processAttrs(context.Background(), v.Map())
	default:
	}
	return pair{
		key: ok.String(true),
		val: v,
	}
}

func TestProcessTraces(t *testing.T) {
	traces := setupSpanWithAttrs()

	processor := &obfuscation{
		encrypt:    feistel.NewFPECipher(hash.SHA_256, "some-32-byte-long-key-to-be-safe", 128),
		encryptAll: true,
	}
	csVal := pcommon.NewValueMap()
	newFilledMap().CopyTo(csVal.SetEmptyMap())
	bVal := pcommon.NewValueBytes()
	bVal.SetEmptyBytes().FromRaw(byteIDVal)

	expected := map[string]pair{
		"resource-attr":     cryptPair(processor, resAttrKey, pcommon.NewValueStr(resAttrVal)),
		"scope-attr":        cryptPair(processor, scopeAttrKey, pcommon.NewValueStr(scopeAttrVal)),
		"span-attr":         cryptPair(processor, spanAttrKey, pcommon.NewValueStr(spanAttrVal)),
		"span-link-attr":    cryptPair(processor, spanLinkAttrKey, pcommon.NewValueStr(linkAttrVal)),
		"span-event-attr":   cryptPair(processor, spanEventAttrKey, pcommon.NewValueStr(eventAttrVal)),
		"complex-span-attr": cryptPair(processor, "complex-span-attr", csVal),
		"byte-id":           cryptPair(processor, "byte-id", bVal),
	}

	processedTraces, err := processor.processTraces(context.Background(), traces)
	require.NoError(t, err)
	validateTraceAttrs(t, expected, processedTraces)
}

func setupMetricsWithAttrs() pmetric.Metrics {
	md := pmetric.NewMetrics()
	rm := md.ResourceMetrics().AppendEmpty()

	rm.Resource().Attributes().PutStr("resource-attr", resAttrVal)

	sm := rm.ScopeMetrics().AppendEmpty()
	sm.Scope().Attributes().PutStr("scope-attr", scopeAttrVal)

	metric := sm.Metrics().AppendEmpty()
	gauge := metric.SetEmptyGauge()
	gdp := gauge.DataPoints().AppendEmpty()
	gdp.Attributes().PutStr("gauge-attr", gaugeAttrVal)
	gdp.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp1 := gdp.Attributes().PutEmptyMap("complex-metric-attr")
	newFilledMap().CopyTo(mp1)

	metric = sm.Metrics().AppendEmpty()
	sum := metric.SetEmptySum()
	sdp := sum.DataPoints().AppendEmpty()
	sdp.Attributes().PutStr("sum-attr", sumAttrVal)
	sdp.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp2 := sdp.Attributes().PutEmptyMap("complex-metric-attr")
	newFilledMap().CopyTo(mp2)

	metric = sm.Metrics().AppendEmpty()
	hist := metric.SetEmptyHistogram()
	hdp := hist.DataPoints().AppendEmpty()
	hdp.Attributes().PutStr("histogram-attr", histAttrVal)
	hdp.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp3 := hdp.Attributes().PutEmptyMap("complex-metric-attr")
	newFilledMap().CopyTo(mp3)

	metric = sm.Metrics().AppendEmpty()
	eHist := metric.SetEmptyExponentialHistogram()
	ehdp := eHist.DataPoints().AppendEmpty()
	ehdp.Attributes().PutStr("exp-histogram-attr", eHistAttrVal)
	ehdp.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp4 := ehdp.Attributes().PutEmptyMap("complex-metric-attr")
	newFilledMap().CopyTo(mp4)

	metric = sm.Metrics().AppendEmpty()
	summary := metric.SetEmptySummary()
	smdp := summary.DataPoints().AppendEmpty()
	smdp.Attributes().PutStr("summary-attr", summaryAttrVal)
	smdp.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp5 := smdp.Attributes().PutEmptyMap("complex-metric-attr")
	newFilledMap().CopyTo(mp5)

	return md
}

func validateMetricsAttrs(t *testing.T, expected map[string]pair, metrics pmetric.Metrics) {
	for i := 0; i < metrics.ResourceMetrics().Len(); i++ {
		// validate resource attributes
		rm := metrics.ResourceMetrics().At(i)
		val, ok := rm.Resource().Attributes().Get(expected["resource-attr"].key)
		assert.True(t, ok)
		assert.Equal(t, expected["resource-attr"].val.AsString(), val.AsString())

		for j := 0; j < rm.ScopeMetrics().Len(); j++ {
			// validate scope attributes
			sm := rm.ScopeMetrics().At(j)
			scopeVal, ok := sm.Scope().Attributes().Get(expected["scope-attr"].key)
			assert.True(t, ok)
			assert.Equal(t, expected["scope-attr"].val.AsString(), scopeVal.AsString())

			for k := 0; k < sm.Metrics().Len(); k++ {
				metric := sm.Metrics().At(k)

				switch metric.Type() {
				case pmetric.MetricTypeGauge:
					gdp := metric.Gauge().DataPoints()
					for i := 0; i < gdp.Len(); i++ {
						dp := gdp.At(i)
						val, ok := dp.Attributes().Get(expected["gauge-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["gauge-attr"].val.AsString(), val.AsString())
						assertCommonMetricAttrs(t, expected, dp.Attributes())
					}

				case pmetric.MetricTypeSum:
					sdp := metric.Sum().DataPoints()
					for i := 0; i < sdp.Len(); i++ {
						dp := sdp.At(i)
						val, ok := dp.Attributes().Get(expected["sum-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["sum-attr"].val.AsString(), val.AsString())
						assertCommonMetricAttrs(t, expected, dp.Attributes())
					}

				case pmetric.MetricTypeHistogram:
					hdp := metric.Histogram().DataPoints()
					for i := 0; i < hdp.Len(); i++ {
						dp := hdp.At(i)
						val, ok := dp.Attributes().Get(expected["histogram-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["histogram-attr"].val.AsString(), val.AsString())
						assertCommonMetricAttrs(t, expected, dp.Attributes())
					}

				case pmetric.MetricTypeExponentialHistogram:
					ehdp := metric.ExponentialHistogram().DataPoints()
					for i := 0; i < ehdp.Len(); i++ {
						dp := ehdp.At(i)
						val, ok := dp.Attributes().Get(expected["exp-histogram-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["exp-histogram-attr"].val.AsString(), val.AsString())
						assertCommonMetricAttrs(t, expected, dp.Attributes())
					}

				case pmetric.MetricTypeSummary:
					smdp := metric.Summary().DataPoints()
					for i := 0; i < smdp.Len(); i++ {
						dp := smdp.At(i)
						val, ok := dp.Attributes().Get(expected["summary-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["summary-attr"].val.AsString(), val.AsString())
						assertCommonMetricAttrs(t, expected, dp.Attributes())
					}
				}
			}
		}
	}
}

func assertCommonMetricAttrs(t *testing.T, expected map[string]pair, attrs pcommon.Map) {
	val2, ok := attrs.Get(expected["complex-metric-attr"].key)
	assert.True(t, ok)
	assert.Equal(t, expected["complex-metric-attr"].val.AsString(), val2.AsString())
	val3, ok := attrs.Get(expected["byte-id"].key)
	assert.True(t, ok)
	assert.Equal(t, expected["byte-id"].val.AsString(), val3.AsString())
}

func TestProcessMetrics(t *testing.T) {
	metrics := setupMetricsWithAttrs()

	processor := &obfuscation{
		encrypt:    feistel.NewFPECipher(hash.SHA_256, "some-32-byte-long-key-to-be-safe", 128),
		encryptAll: true,
	}
	cmVal := pcommon.NewValueMap()
	newFilledMap().CopyTo(cmVal.SetEmptyMap())
	bVal := pcommon.NewValueBytes()
	bVal.SetEmptyBytes().FromRaw(byteIDVal)

	expected := map[string]pair{
		"resource-attr":       cryptPair(processor, resAttrKey, pcommon.NewValueStr(resAttrVal)),
		"scope-attr":          cryptPair(processor, scopeAttrKey, pcommon.NewValueStr(scopeAttrVal)),
		"gauge-attr":          cryptPair(processor, "gauge-attr", pcommon.NewValueStr(gaugeAttrVal)),
		"sum-attr":            cryptPair(processor, "sum-attr", pcommon.NewValueStr(sumAttrVal)),
		"histogram-attr":      cryptPair(processor, "histogram-attr", pcommon.NewValueStr(histAttrVal)),
		"exp-histogram-attr":  cryptPair(processor, "exp-histogram-attr", pcommon.NewValueStr(eHistAttrVal)),
		"summary-attr":        cryptPair(processor, "summary-attr", pcommon.NewValueStr(summaryAttrVal)),
		"complex-metric-attr": cryptPair(processor, "complex-metric-attr", cmVal),
		"byte-id":             cryptPair(processor, "byte-id", bVal),
	}

	processedMetrics, err := processor.processMetrics(context.Background(), metrics)
	require.NoError(t, err)
	validateMetricsAttrs(t, expected, processedMetrics)
}

func setupLogsWithAttrs() plog.Logs {
	ld := plog.NewLogs()
	rl := ld.ResourceLogs().AppendEmpty()

	rl.Resource().Attributes().PutStr("resource-attr", resAttrVal)

	sl := rl.ScopeLogs().AppendEmpty()
	sl.Scope().Attributes().PutStr("scope-attr", scopeAttrVal)

	log := sl.LogRecords().AppendEmpty()
	log.Attributes().PutStr("log-attr", logAttrVal)
	log.Attributes().PutEmptyBytes(byteIDKey).FromRaw(byteIDVal)
	mp := log.Attributes().PutEmptyMap("complex-log-attr")
	newFilledMap().CopyTo(mp)

	return ld
}

func validateLogsAttrs(t *testing.T, expected map[string]pair, logs plog.Logs) {
	for i := 0; i < logs.ResourceLogs().Len(); i++ {
		// validate resource attributes
		rl := logs.ResourceLogs().At(i)
		val, ok := rl.Resource().Attributes().Get(expected["resource-attr"].key)
		assert.True(t, ok)
		assert.Equal(t, expected["resource-attr"].val.AsString(), val.AsString())

		for j := 0; j < rl.ScopeLogs().Len(); j++ {
			// validate scope attributes
			sl := rl.ScopeLogs().At(j)
			scopeVal, ok := sl.Scope().Attributes().Get(expected["scope-attr"].key)
			assert.True(t, ok)
			assert.Equal(t, expected["scope-attr"].val.AsString(), scopeVal.AsString())

			for k := 0; k < sl.LogRecords().Len(); k++ {
				// validate span attributes
				log := sl.LogRecords().At(k)
				val, ok := log.Attributes().Get(expected["log-attr"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["log-attr"].val.AsString(), val.AsString())

				val2, ok := log.Attributes().Get(expected["complex-log-attr"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["complex-log-attr"].val.AsString(), val2.AsString())

				val3, ok := log.Attributes().Get(expected["byte-id"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["byte-id"].val.AsString(), val3.AsString())
			}
		}
	}
}

func TestProcessLogs(t *testing.T) {
	logs := setupLogsWithAttrs()

	processor := &obfuscation{
		encrypt:    feistel.NewFPECipher(hash.SHA_256, "some-32-byte-long-key-to-be-safe", 128),
		encryptAll: true,
	}
	clVal := pcommon.NewValueMap()
	newFilledMap().CopyTo(clVal.SetEmptyMap())
	bVal := pcommon.NewValueBytes()
	bVal.SetEmptyBytes().FromRaw(byteIDVal)

	expected := map[string]pair{
		"resource-attr":    cryptPair(processor, resAttrKey, pcommon.NewValueStr(resAttrVal)),
		"scope-attr":       cryptPair(processor, scopeAttrKey, pcommon.NewValueStr(scopeAttrVal)),
		"log-attr":         cryptPair(processor, "log-attr", pcommon.NewValueStr(logAttrVal)),
		"complex-log-attr": cryptPair(processor, "complex-log-attr", clVal),
		"byte-id":          cryptPair(processor, "byte-id", bVal),
	}

	processedLogs, err := processor.processLogs(context.Background(), logs)
	require.NoError(t, err)
	validateLogsAttrs(t, expected, processedLogs)
}
