package obfuscationprocessor

import (
	"context"

	"github.com/cyrildever/feistel"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.uber.org/zap"
)

type obfuscation struct {
	// Logger
	logger *zap.Logger
	// Next trace consumer in line
	nextTraces  consumer.Traces
	nextMetrics consumer.Metrics
	nextLogs    consumer.Logs

	encryptAttributes map[string]struct{}
	encrypt           *feistel.FPECipher
	encryptAll        bool
}

// processLogs implements ProcessLogsFunc. It processes the incoming data
// and returns the data to be sent to the next component
func (o *obfuscation) processLogs(ctx context.Context, batch plog.Logs) (plog.Logs, error) {
	for i := 0; i < batch.ResourceLogs().Len(); i++ {
		rm := batch.ResourceLogs().At(i)
		o.processResourceLogs(ctx, rm)
	}
	return batch, nil
}

func (o *obfuscation) processResourceLogs(ctx context.Context, rl plog.ResourceLogs) {
	rlAttrs := rl.Resource().Attributes()
	o.processAttrs(ctx, rlAttrs)

	for j := 0; j < rl.ScopeLogs().Len(); j++ {
		sl := rl.ScopeLogs().At(j)
		slScopeAttrs := sl.Scope().Attributes()
		o.processAttrs(ctx, slScopeAttrs)

		for k := 0; k < sl.LogRecords().Len(); k++ {
			log := sl.LogRecords().At(k)
			o.processAttrs(ctx, log.Attributes())
		}
	}

}

// processMetrics implements ProcessMetricsFunc. It processes the incoming data
// and returns the data to be sent to the next component
func (o *obfuscation) processMetrics(ctx context.Context, batch pmetric.Metrics) (pmetric.Metrics, error) {
	for i := 0; i < batch.ResourceMetrics().Len(); i++ {
		rm := batch.ResourceMetrics().At(i)
		o.processResourceMetrics(ctx, rm)
	}
	return batch, nil
}

func (o *obfuscation) processResourceMetrics(ctx context.Context, rm pmetric.ResourceMetrics) {
	rmAttrs := rm.Resource().Attributes()
	o.processAttrs(ctx, rmAttrs)

	for j := 0; j < rm.ScopeMetrics().Len(); j++ {
		sm := rm.ScopeMetrics().At(j)
		smScopeAttrs := sm.Scope().Attributes()
		o.processAttrs(ctx, smScopeAttrs)

		for k := 0; k < sm.Metrics().Len(); k++ {
			metric := sm.Metrics().At(k)
			o.processMetricAttrs(ctx, metric)
		}
	}

}

func (o *obfuscation) processMetricAttrs(ctx context.Context, metric pmetric.Metric) {
	switch metric.Type() {
	case pmetric.MetricTypeGauge:
		gdp := metric.Gauge().DataPoints()
		for i := 0; i < gdp.Len(); i++ {
			dp := gdp.At(i)
			o.processAttrs(ctx, dp.Attributes())
		}

	case pmetric.MetricTypeSum:
		sdp := metric.Sum().DataPoints()
		for i := 0; i < sdp.Len(); i++ {
			dp := sdp.At(i)
			o.processAttrs(ctx, dp.Attributes())
		}

	case pmetric.MetricTypeHistogram:
		hdp := metric.Histogram().DataPoints()
		for i := 0; i < hdp.Len(); i++ {
			dp := hdp.At(i)
			o.processAttrs(ctx, dp.Attributes())
		}

	case pmetric.MetricTypeExponentialHistogram:
		ehdp := metric.ExponentialHistogram().DataPoints()
		for i := 0; i < ehdp.Len(); i++ {
			dp := ehdp.At(i)
			o.processAttrs(ctx, dp.Attributes())
		}

	case pmetric.MetricTypeSummary:
		smdp := metric.Summary().DataPoints()
		for i := 0; i < smdp.Len(); i++ {
			dp := smdp.At(i)
			o.processAttrs(ctx, dp.Attributes())
		}

	default:
		o.logger.Info("unrecognized metric type")
	}
}

// processTraces implements ProcessTracesFunc. It processes the incoming data
// and returns the data to be sent to the next component
func (o *obfuscation) processTraces(ctx context.Context, batch ptrace.Traces) (ptrace.Traces, error) {
	for i := 0; i < batch.ResourceSpans().Len(); i++ {
		rs := batch.ResourceSpans().At(i)
		o.processResourceSpan(ctx, rs)
	}
	return batch, nil
}

// processResourceSpan processes the ResourceSpans and all of its spans
func (o *obfuscation) processResourceSpan(ctx context.Context, rs ptrace.ResourceSpans) {
	rsAttrs := rs.Resource().Attributes()
	// Attributes can be part of a resource span
	o.processAttrs(ctx, rsAttrs)

	for j := 0; j < rs.ScopeSpans().Len(); j++ {
		ils := rs.ScopeSpans().At(j)
		ilsScopeAttrs := ils.Scope().Attributes()
		o.processAttrs(ctx, ilsScopeAttrs)

		for k := 0; k < ils.Spans().Len(); k++ {
			span := ils.Spans().At(k)
			span.SetName(o.encryptString(span.Name()))
			spanAttrs := span.Attributes()

			// Attributes can also be part of span
			o.processAttrs(ctx, spanAttrs)
			o.processEventAndLinkAttrs(ctx, span)
		}
	}
}

func (o *obfuscation) processEventAndLinkAttrs(ctx context.Context, span ptrace.Span) {
	for i := 0; i < span.Events().Len(); i++ {
		ev := span.Events().At(i)
		o.processAttrs(ctx, ev.Attributes())
	}

	for j := 0; j < span.Links().Len(); j++ {
		lk := span.Links().At(j)
		o.processAttrs(ctx, lk.Attributes())
	}
}

func (o *obfuscation) processSliceValue(ctx context.Context, slice pcommon.Slice) {
	cpy := pcommon.NewSlice()
	for i := 0; i < slice.Len(); i++ {
		cpyVal := cpy.AppendEmpty()
		val := slice.At(i)

		switch val.Type() {
		case pcommon.ValueTypeStr:
			cpyVal.SetStr(o.encryptString(val.Str()))
		case pcommon.ValueTypeBytes:
			obf := o.encryptStringToBytes(string(val.Bytes().AsRaw()))
			byteSlice := cpyVal.SetEmptyBytes()
			byteSlice.FromRaw(obf)
		case pcommon.ValueTypeSlice:
			cpySlice := cpyVal.SetEmptySlice()
			o.processSliceValue(ctx, val.Slice())
			val.Slice().CopyTo(cpySlice)
		case pcommon.ValueTypeMap:
			cpyMap := cpyVal.SetEmptyMap()
			o.processAttrs(ctx, val.Map())
			val.Map().CopyTo(cpyMap)
		default:
			// no need to obfuscate types that cannot contain a string
			val.CopyTo(cpyVal)
		}

	}

	cpy.CopyTo(slice)
}

// processAttrs obfuscates the attributes of a resource span or a span
func (o *obfuscation) processAttrs(ctx context.Context, attributes pcommon.Map) {
	cpy := pcommon.NewMap()
	attributes.Range(func(k string, value pcommon.Value) bool {
		if !o.encryptAll {
			// check if in encryptList
			_, ok := o.encryptAttributes[k]
			if !ok {
				return true
			}
		}

		switch value.Type() {
		case pcommon.ValueTypeStr:
			cpy.PutStr(o.encryptString(k), o.encryptString(value.Str()))
		case pcommon.ValueTypeBytes:
			byteSlice := cpy.PutEmptyBytes(o.encryptString(k))
			obf := o.encryptStringToBytes(string(value.Bytes().AsRaw()))
			byteSlice.FromRaw(obf)
		case pcommon.ValueTypeSlice:
			slc := cpy.PutEmptySlice(o.encryptString(k))
			o.processSliceValue(ctx, value.Slice())
			value.Slice().CopyTo(slc)
		case pcommon.ValueTypeMap:
			mp := cpy.PutEmptyMap(o.encryptString(k))
			o.processAttrs(ctx, value.Map())
			value.Map().CopyTo(mp)
		default:
			value.CopyTo(cpy.PutEmpty(o.encryptString(k)))
		}
		return true
	})
	cpy.CopyTo(attributes)
}

// Capabilities specifies what this processor does, such as whether it mutates data
func (o *obfuscation) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: true}
}

// Start the obfuscation processor
func (o *obfuscation) Start(_ context.Context, _ component.Host) error {
	return nil
}

// Shutdown the obfuscation processor
func (o *obfuscation) Shutdown(context.Context) error {
	return nil
}

func (o *obfuscation) encryptStringToBytes(source string) []byte {
	obfuscated, err := o.encrypt.Encrypt(source)
	if err != nil {
		o.logger.Error("failed to obfuscate bytes, returning original", zap.Error(err))
		return []byte(source)
	}
	return obfuscated.Bytes()
}

func (o *obfuscation) encryptString(source string) string {
	obfuscated, err := o.encrypt.Encrypt(source)
	if err != nil {
		o.logger.Error("failed to obfuscate string, returning original", zap.Error(err))
		return source
	}
	return obfuscated.String(true)
}
