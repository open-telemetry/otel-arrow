package obfuscationprocessor

import (
	"context"
	"testing"

	"github.com/cyrildever/feistel"
	"github.com/cyrildever/feistel/common/utils/hash"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
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

	resAttrVal   = "resource-attr-val-1"
	scopeAttrVal = "scope-attr-val-1"

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

func setupSpanWithAttrs() ptrace.Traces {
	td := ptrace.NewTraces()
	rs := td.ResourceSpans().AppendEmpty()

	rs.Resource().Attributes().PutStr("resource-attr", resAttrVal)

	ss := rs.ScopeSpans().AppendEmpty()
	ss.Scope().Attributes().PutStr("scope-attr", scopeAttrVal)

	span := ss.Spans().AppendEmpty()
	span.SetName("operationA")
	span.Attributes().PutStr("span-attr", spanAttrVal)

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
		assert.Equal(t, expected["resource-attr"].val, val.AsString())

		for j := 0; j < rs.ScopeSpans().Len(); j++ {
			// validate scope attributes
			ss := rs.ScopeSpans().At(j)
			scopeVal, ok := ss.Scope().Attributes().Get(expected["scope-attr"].key)
			assert.True(t, ok)
			assert.Equal(t, expected["scope-attr"].val, scopeVal.AsString())

			for k := 0; k < ss.Spans().Len(); k++ {
				// validate span attributes
				span := ss.Spans().At(k)
				val, ok := span.Attributes().Get(expected["span-attr"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["span-attr"].val, val.AsString())

				for h := 0; h < span.Events().Len(); h++ {
					// validate event attributes
					event := span.Events().At(h)
					val, ok := event.Attributes().Get(expected["span-event-attr"].key)
					assert.True(t, ok)
					assert.Equal(t, expected["span-event-attr"].val, val.AsString())
				}

				for h := 0; h < span.Links().Len(); h++ {
					// validate link attributes
					link := span.Links().At(h)
					val, ok := link.Attributes().Get(expected["span-link-attr"].key)
					assert.True(t, ok)
					assert.Equal(t, expected["span-link-attr"].val, val.AsString())
				}
			}
		}
	}
}

type pair struct {
	key string
	val string
}

func cryptPair(o *obfuscation, k, v string) pair {
	ok, _ := o.encrypt.Encrypt(k)
	ov, _ := o.encrypt.Encrypt(v)
	return pair{
		key: ok.String(true),
		val: ov.String(true),
	}
}

func TestProcessTraces(t *testing.T) {
	traces := setupSpanWithAttrs()

	processor := &obfuscation{
		encrypt:    feistel.NewFPECipher(hash.SHA_256, "some-32-byte-long-key-to-be-safe", 128),
		encryptAll: true,
	}

	expected := map[string]pair{
		"resource-attr":   cryptPair(processor, resAttrKey, resAttrVal),
		"scope-attr":      cryptPair(processor, scopeAttrKey, scopeAttrVal),
		"span-attr":       cryptPair(processor, spanAttrKey, spanAttrVal),
		"span-link-attr":  cryptPair(processor, spanLinkAttrKey, linkAttrVal),
		"span-event-attr": cryptPair(processor, spanEventAttrKey, eventAttrVal),
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

	metric = sm.Metrics().AppendEmpty()
	sum := metric.SetEmptySum()
	sdp := sum.DataPoints().AppendEmpty()
	sdp.Attributes().PutStr("sum-attr", sumAttrVal)

	metric = sm.Metrics().AppendEmpty()
	hist := metric.SetEmptyHistogram()
	hdp := hist.DataPoints().AppendEmpty()
	hdp.Attributes().PutStr("histogram-attr", histAttrVal)

	metric = sm.Metrics().AppendEmpty()
	eHist := metric.SetEmptyExponentialHistogram()
	ehdp := eHist.DataPoints().AppendEmpty()
	ehdp.Attributes().PutStr("exp-histogram-attr", eHistAttrVal)

	metric = sm.Metrics().AppendEmpty()
	summary := metric.SetEmptySummary()
	smdp := summary.DataPoints().AppendEmpty()
	smdp.Attributes().PutStr("summary-attr", summaryAttrVal)

	return md
}

func validateMetricsAttrs(t *testing.T, expected map[string]pair, metrics pmetric.Metrics) {
	for i := 0; i < metrics.ResourceMetrics().Len(); i++ {
		// validate resource attributes
		rm := metrics.ResourceMetrics().At(i)
		val, ok := rm.Resource().Attributes().Get(expected["resource-attr"].key)
		assert.True(t, ok)
		assert.Equal(t, expected["resource-attr"].val, val.AsString())

		for j := 0; j < rm.ScopeMetrics().Len(); j++ {
			// validate scope attributes
			sm := rm.ScopeMetrics().At(j)
			scopeVal, ok := sm.Scope().Attributes().Get(expected["scope-attr"].key)
			assert.True(t, ok)
			assert.Equal(t, expected["scope-attr"].val, scopeVal.AsString())

			for k := 0; k < sm.Metrics().Len(); k++ {
				metric := sm.Metrics().At(k)

				switch metric.Type() {
				case pmetric.MetricTypeGauge:
					gdp := metric.Gauge().DataPoints()
					for i := 0; i < gdp.Len(); i++ {
						dp := gdp.At(i)
						val, ok := dp.Attributes().Get(expected["gauge-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["gauge-attr"].val, val.AsString())
					}

				case pmetric.MetricTypeSum:
					sdp := metric.Sum().DataPoints()
					for i := 0; i < sdp.Len(); i++ {
						dp := sdp.At(i)
						val, ok := dp.Attributes().Get(expected["sum-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["sum-attr"].val, val.AsString())
					}

				case pmetric.MetricTypeHistogram:
					hdp := metric.Histogram().DataPoints()
					for i := 0; i < hdp.Len(); i++ {
						dp := hdp.At(i)
						val, ok := dp.Attributes().Get(expected["histogram-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["histogram-attr"].val, val.AsString())
					}

				case pmetric.MetricTypeExponentialHistogram:
					ehdp := metric.ExponentialHistogram().DataPoints()
					for i := 0; i < ehdp.Len(); i++ {
						dp := ehdp.At(i)
						val, ok := dp.Attributes().Get(expected["exp-histogram-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["exp-histogram-attr"].val, val.AsString())
					}

				case pmetric.MetricTypeSummary:
					smdp := metric.Summary().DataPoints()
					for i := 0; i < smdp.Len(); i++ {
						dp := smdp.At(i)
						val, ok := dp.Attributes().Get(expected["summary-attr"].key)
						assert.True(t, ok)
						assert.Equal(t, expected["summary-attr"].val, val.AsString())
					}
				}
			}
		}
	}
}

func TestProcessMetrics(t *testing.T) {
	metrics := setupMetricsWithAttrs()

	processor := &obfuscation{
		encrypt:    feistel.NewFPECipher(hash.SHA_256, "some-32-byte-long-key-to-be-safe", 128),
		encryptAll: true,
	}

	expected := map[string]pair{
		"resource-attr":      cryptPair(processor, resAttrKey, resAttrVal),
		"scope-attr":         cryptPair(processor, scopeAttrKey, scopeAttrVal),
		"gauge-attr":         cryptPair(processor, "gauge-attr", gaugeAttrVal),
		"sum-attr":           cryptPair(processor, "sum-attr", sumAttrVal),
		"histogram-attr":     cryptPair(processor, "histogram-attr", histAttrVal),
		"exp-histogram-attr": cryptPair(processor, "exp-histogram-attr", eHistAttrVal),
		"summary-attr":       cryptPair(processor, "summary-attr", summaryAttrVal),
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

	return ld
}

func validateLogsAttrs(t *testing.T, expected map[string]pair, logs plog.Logs) {
	for i := 0; i < logs.ResourceLogs().Len(); i++ {
		// validate resource attributes
		rl := logs.ResourceLogs().At(i)
		val, ok := rl.Resource().Attributes().Get(expected["resource-attr"].key)
		assert.True(t, ok)
		assert.Equal(t, expected["resource-attr"].val, val.AsString())

		for j := 0; j < rl.ScopeLogs().Len(); j++ {
			// validate scope attributes
			sl := rl.ScopeLogs().At(j)
			scopeVal, ok := sl.Scope().Attributes().Get(expected["scope-attr"].key)
			assert.True(t, ok)
			assert.Equal(t, expected["scope-attr"].val, scopeVal.AsString())

			for k := 0; k < sl.LogRecords().Len(); k++ {
				// validate span attributes
				log := sl.LogRecords().At(k)
				val, ok := log.Attributes().Get(expected["log-attr"].key)
				assert.True(t, ok)
				assert.Equal(t, expected["log-attr"].val, val.AsString())
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

	expected := map[string]pair{
		"resource-attr": cryptPair(processor, resAttrKey, resAttrVal),
		"scope-attr":    cryptPair(processor, scopeAttrKey, scopeAttrVal),
		"log-attr":      cryptPair(processor, "log-attr", logAttrVal),
	}

	processedLogs, err := processor.processLogs(context.Background(), logs)
	require.NoError(t, err)
	validateLogsAttrs(t, expected, processedLogs)
}
