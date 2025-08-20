/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package internal

import (
	"github.com/brianvoe/gofakeit/v6"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"golang.org/x/exp/rand"
)

// RandAttrs generates a randon set of attributes with the given names.
// The number of attributes is random between 1 and the number of names.
// The type of each attribute is random.
func RandAttrs(names []string) pcommon.Map {
	attrs := pcommon.NewMap()

	rand.Shuffle(len(names), func(i, j int) {
		names[i], names[j] = names[j], names[i]
	})

	attrCount := rand.Intn(len(names))
	if attrCount == 0 {
		attrCount = 1
	}
	names = names[:attrCount]

	for _, name := range names {
		attrType := rand.Intn(5)
		switch attrType {
		case 0:
			attrs.PutStr(name, gofakeit.AppName())
		case 1:
			attrs.PutInt(name, rand.Int63())
		case 2:
			attrs.PutDouble(name, rand.Float64())
		case 3:
			attrs.PutBool(name, rand.Intn(2) == 1)
		case 4:
			bytes := attrs.PutEmptyBytes(name)
			bytes.Append([]byte(gofakeit.UUID())...)
		}
	}
	return attrs
}

func Attrs1() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutStr("str", "string1")
	attrs.PutInt("int", 1)
	attrs.PutDouble("double", 1.0)
	attrs.PutBool("bool", true)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes1")...)
	return attrs
}

func Attrs2() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutStr("str", "string2")
	attrs.PutInt("int", 2)
	attrs.PutDouble("double", 2.0)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes2")...)
	return attrs
}

func Attrs3() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutStr("str", "string3")
	attrs.PutDouble("double", 3.0)
	attrs.PutBool("bool", false)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes3")...)
	return attrs
}

func Attrs4() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutBool("bool", true)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes4")...)
	return attrs
}

func Attrs5() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutBool("attr1", true)
	bytes := attrs.PutEmptyBytes("attr2")
	bytes.Append([]byte("bytes4")...)
	attrs.PutStr("attr3", "string5")
	attrs.PutInt("attr4", 5)
	attrs.PutDouble("attr5", 5.0)
	attrs.PutBool("attr6", false)
	bytes = attrs.PutEmptyBytes("attr7")
	bytes.Append([]byte("bytes5")...)
	attrs.PutStr("attr8", "string6")
	attrs.PutInt("attr9", 6)
	attrs.PutDouble("attr10", 6.0)
	attrs.PutBool("attr11", true)
	bytes = attrs.PutEmptyBytes("attr12")
	bytes.Append([]byte("bytes6")...)
	attrs.PutStr("attr13", "string7")
	return attrs
}

func Scope1() pcommon.InstrumentationScope {
	scope := pcommon.NewInstrumentationScope()
	scope.SetName("scope1")
	scope.SetVersion("1.0.1")
	scopeAttrs := scope.Attributes()
	Attrs1().CopyTo(scopeAttrs)
	scope.SetDroppedAttributesCount(0)
	return scope
}

func Scope2() pcommon.InstrumentationScope {
	scope := pcommon.NewInstrumentationScope()
	scope.SetName("scope2")
	scope.SetVersion("1.0.2")
	scopeAttrs := scope.Attributes()
	Attrs2().CopyTo(scopeAttrs)
	scope.SetDroppedAttributesCount(1)
	return scope
}

func Resource1() pcommon.Resource {
	resource := pcommon.NewResource()
	resourceAttrs := resource.Attributes()
	Attrs1().CopyTo(resourceAttrs)
	resource.SetDroppedAttributesCount(0)
	return resource
}

func Resource2() pcommon.Resource {
	resource := pcommon.NewResource()
	resourceAttrs := resource.Attributes()
	Attrs2().CopyTo(resourceAttrs)
	resource.SetDroppedAttributesCount(1)
	return resource
}

// IntDP1 returns a pmetric.NumberDataPoint (sample 1).
func IntDP1() pmetric.NumberDataPoint {
	dp := pmetric.NewNumberDataPoint()
	Attrs1().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(1)
	dp.SetTimestamp(2)
	dp.SetIntValue(1)
	exs := dp.Exemplars()
	exs.EnsureCapacity(2)
	Exemplar1().CopyTo(exs.AppendEmpty())
	Exemplar2().CopyTo(exs.AppendEmpty())
	dp.SetFlags(1)
	return dp
}

// IntDP2 returns a pmetric.NumberDataPoint (sample 1).
func IntDP2() pmetric.NumberDataPoint {
	dp := pmetric.NewNumberDataPoint()
	Attrs1().CopyTo(dp.Attributes())
	dp.SetTimestamp(2)
	dp.SetIntValue(1)
	return dp
}

// NDP1 returns a pmetric.NumberDataPoint (sample 1).
func NDP1() pmetric.NumberDataPoint {
	dp := pmetric.NewNumberDataPoint()
	Attrs1().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(1)
	dp.SetTimestamp(2)
	dp.SetDoubleValue(1.5)
	exs := dp.Exemplars()
	exs.EnsureCapacity(2)
	Exemplar1().CopyTo(exs.AppendEmpty())
	Exemplar2().CopyTo(exs.AppendEmpty())
	dp.SetFlags(1)
	return dp
}

// NDP2 returns a pmetric.NumberDataPoint (sample 1).
func NDP2() pmetric.NumberDataPoint {
	dp := pmetric.NewNumberDataPoint()
	Attrs2().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(2)
	dp.SetTimestamp(3)
	dp.SetIntValue(2)
	exs := dp.Exemplars()
	exs.EnsureCapacity(1)
	Exemplar2().CopyTo(exs.AppendEmpty())
	dp.SetFlags(2)
	return dp
}

// NDP3 returns a pmetric.NumberDataPoint (sample 1).
func NDP3() pmetric.NumberDataPoint {
	dp := pmetric.NewNumberDataPoint()
	Attrs3().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(3)
	dp.SetTimestamp(4)
	dp.SetIntValue(3)
	exs := dp.Exemplars()
	exs.EnsureCapacity(1)
	Exemplar1().CopyTo(exs.AppendEmpty())
	dp.SetFlags(3)
	return dp
}

func Exemplar1() pmetric.Exemplar {
	ex := pmetric.NewExemplar()
	Attrs1().CopyTo(ex.FilteredAttributes())
	ex.SetTimestamp(1)
	ex.SetDoubleValue(1.5)
	ex.SetSpanID([8]byte{0xAA})
	ex.SetTraceID([16]byte{0xAA})
	return ex
}

func Exemplar2() pmetric.Exemplar {
	ex := pmetric.NewExemplar()
	Attrs2().CopyTo(ex.FilteredAttributes())
	ex.SetTimestamp(2)
	ex.SetIntValue(2)
	ex.SetSpanID([8]byte{0xAA})
	ex.SetTraceID([16]byte{0xAA})
	return ex
}

func Gauge1() pmetric.Gauge {
	g := pmetric.NewGauge()
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	NDP2().CopyTo(g.DataPoints().AppendEmpty())
	NDP3().CopyTo(g.DataPoints().AppendEmpty())
	return g
}

func Gauge2() pmetric.Gauge {
	g := pmetric.NewGauge()
	NDP3().CopyTo(g.DataPoints().AppendEmpty())
	return g
}

func Gauge3() pmetric.Gauge {
	g := pmetric.NewGauge()
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	return g
}

func IntSum1() pmetric.Sum {
	g := pmetric.NewSum()
	IntDP1().CopyTo(g.DataPoints().AppendEmpty())
	IntDP2().CopyTo(g.DataPoints().AppendEmpty())
	g.SetAggregationTemporality(pmetric.AggregationTemporalityDelta)
	g.SetIsMonotonic(true)
	return g
}

func IntSum2() pmetric.Sum {
	g := pmetric.NewSum()
	IntDP1().CopyTo(g.DataPoints().AppendEmpty())
	IntDP2().CopyTo(g.DataPoints().AppendEmpty())
	return g
}

func Sum1() pmetric.Sum {
	g := pmetric.NewSum()
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	NDP2().CopyTo(g.DataPoints().AppendEmpty())
	NDP3().CopyTo(g.DataPoints().AppendEmpty())
	g.SetAggregationTemporality(pmetric.AggregationTemporalityDelta)
	g.SetIsMonotonic(true)
	return g
}

func Sum3() pmetric.Sum {
	g := pmetric.NewSum()
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	g.SetAggregationTemporality(pmetric.AggregationTemporalityDelta)
	g.SetIsMonotonic(true)
	return g
}

func Sum2() pmetric.Sum {
	g := pmetric.NewSum()
	NDP3().CopyTo(g.DataPoints().AppendEmpty())
	g.SetAggregationTemporality(pmetric.AggregationTemporalityCumulative)
	g.SetIsMonotonic(false)
	return g
}

func Summary1() pmetric.Summary {
	s := pmetric.NewSummary()
	SummaryDataPoint1().CopyTo(s.DataPoints().AppendEmpty())
	SummaryDataPoint2().CopyTo(s.DataPoints().AppendEmpty())
	return s
}

func Summary2() pmetric.Summary {
	s := pmetric.NewSummary()
	SummaryDataPoint2().CopyTo(s.DataPoints().AppendEmpty())
	return s
}

func SummaryDataPoint1() pmetric.SummaryDataPoint {
	dp := pmetric.NewSummaryDataPoint()
	Attrs1().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(1)
	dp.SetTimestamp(2)
	dp.SetCount(1)
	dp.SetSum(1.5)
	qvs := dp.QuantileValues()
	qvs.EnsureCapacity(2)
	QuantileValue1().CopyTo(qvs.AppendEmpty())
	QuantileValue2().CopyTo(qvs.AppendEmpty())
	dp.SetFlags(1)
	return dp
}

func SummaryDataPoint2() pmetric.SummaryDataPoint {
	dp := pmetric.NewSummaryDataPoint()
	Attrs2().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(3)
	dp.SetTimestamp(4)
	dp.SetCount(2)
	dp.SetSum(2.5)
	qvs := dp.QuantileValues()
	qvs.EnsureCapacity(1)
	QuantileValue2().CopyTo(qvs.AppendEmpty())
	dp.SetFlags(2)
	return dp
}

func QuantileValue1() pmetric.SummaryDataPointValueAtQuantile {
	qv := pmetric.NewSummaryDataPointValueAtQuantile()
	qv.SetQuantile(0.1)
	qv.SetValue(1.5)
	return qv
}

func QuantileValue2() pmetric.SummaryDataPointValueAtQuantile {
	qv := pmetric.NewSummaryDataPointValueAtQuantile()
	qv.SetQuantile(0.2)
	qv.SetValue(2.5)
	return qv
}

func ExponentialHistogramDataPointBuckets1() pmetric.ExponentialHistogramDataPointBuckets {
	b := pmetric.NewExponentialHistogramDataPointBuckets()
	b.SetOffset(1)
	bcs := b.BucketCounts()
	bcs.EnsureCapacity(2)
	bcs.Append(1, 2)
	return b
}

func ExponentialHistogramDataPointBuckets2() pmetric.ExponentialHistogramDataPointBuckets {
	b := pmetric.NewExponentialHistogramDataPointBuckets()
	b.SetOffset(2)
	bcs := b.BucketCounts()
	bcs.EnsureCapacity(2)
	bcs.Append(3, 4)
	return b
}

func ExponentialHistogramDataPoint1() pmetric.ExponentialHistogramDataPoint {
	dp := pmetric.NewExponentialHistogramDataPoint()
	Attrs1().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(1)
	dp.SetTimestamp(2)
	dp.SetCount(1)
	dp.SetSum(1.5)
	ExponentialHistogramDataPointBuckets1().CopyTo(dp.Positive())
	ExponentialHistogramDataPointBuckets2().CopyTo(dp.Negative())
	dp.SetFlags(1)
	dp.SetScale(1)
	dp.SetZeroCount(1)
	dp.SetMin(1.5)
	dp.SetMax(2.5)
	return dp
}

func ExponentialHistogramDataPoint2() pmetric.ExponentialHistogramDataPoint {
	dp := pmetric.NewExponentialHistogramDataPoint()
	Attrs2().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(2)
	dp.SetTimestamp(3)
	dp.SetCount(2)
	dp.SetSum(2.5)
	ExponentialHistogramDataPointBuckets1().CopyTo(dp.Positive())
	ExponentialHistogramDataPointBuckets2().CopyTo(dp.Negative())
	dp.SetFlags(2)
	dp.SetScale(2)
	dp.SetZeroCount(2)
	dp.SetMin(2.5)
	dp.SetMax(3.5)
	return dp
}

func HistogramDataPoint1() pmetric.HistogramDataPoint {
	dp := pmetric.NewHistogramDataPoint()
	Attrs1().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(1)
	dp.SetTimestamp(2)
	dp.SetCount(1)
	dp.SetSum(1.5)
	bcs := dp.BucketCounts()
	bcs.EnsureCapacity(2)
	bcs.Append(1, 2)
	ebs := dp.ExplicitBounds()
	ebs.EnsureCapacity(2)
	ebs.Append(1.5, 2.5)
	dp.SetFlags(1)
	dp.SetMin(1.5)
	dp.SetMax(2.5)
	return dp
}

func HistogramDataPoint2() pmetric.HistogramDataPoint {
	dp := pmetric.NewHistogramDataPoint()
	Attrs2().CopyTo(dp.Attributes())
	dp.SetStartTimestamp(2)
	dp.SetTimestamp(3)
	dp.SetCount(2)
	dp.SetSum(2.5)
	bcs := dp.BucketCounts()
	bcs.EnsureCapacity(2)
	bcs.Append(3, 4)
	ebs := dp.ExplicitBounds()
	ebs.EnsureCapacity(2)
	ebs.Append(2.5, 3.5)
	dp.SetFlags(2)
	dp.SetMin(2.5)
	dp.SetMax(3.5)
	return dp
}

func Histogram1() pmetric.Histogram {
	h := pmetric.NewHistogram()
	h.SetAggregationTemporality(1)
	dps := h.DataPoints()
	dps.EnsureCapacity(2)
	HistogramDataPoint1().CopyTo(dps.AppendEmpty())
	HistogramDataPoint2().CopyTo(dps.AppendEmpty())
	return h
}

func Histogram2() pmetric.Histogram {
	h := pmetric.NewHistogram()
	h.SetAggregationTemporality(2)
	dps := h.DataPoints()
	dps.EnsureCapacity(1)
	HistogramDataPoint2().CopyTo(dps.AppendEmpty())
	return h
}

func ExpHistogram1() pmetric.ExponentialHistogram {
	h := pmetric.NewExponentialHistogram()
	h.SetAggregationTemporality(1)
	dps := h.DataPoints()
	dps.EnsureCapacity(2)
	ExponentialHistogramDataPoint1().CopyTo(dps.AppendEmpty())
	ExponentialHistogramDataPoint2().CopyTo(dps.AppendEmpty())
	return h
}

func ExpHistogram2() pmetric.ExponentialHistogram {
	h := pmetric.NewExponentialHistogram()
	h.SetAggregationTemporality(2)
	dps := h.DataPoints()
	dps.EnsureCapacity(1)
	ExponentialHistogramDataPoint1().CopyTo(dps.AppendEmpty())
	return h
}

func Metric1() pmetric.Metric {
	m := pmetric.NewMetric()
	m.SetName("gauge-1")
	m.SetDescription("gauge-1-desc")
	m.SetUnit("gauge-1-unit")
	Gauge1().CopyTo(m.SetEmptyGauge())
	return m
}

func Metric2() pmetric.Metric {
	m := pmetric.NewMetric()
	m.SetName("sum-2")
	m.SetDescription("sum-2-desc")
	m.SetUnit("sum-2-unit")
	Sum1().CopyTo(m.SetEmptySum())
	return m
}

func Metric3() pmetric.Metric {
	m := pmetric.NewMetric()
	m.SetName("summary-3")
	m.SetDescription("summary-3-desc")
	m.SetUnit("summary-3-unit")
	Summary1().CopyTo(m.SetEmptySummary())
	return m
}

func Metric4() pmetric.Metric {
	m := pmetric.NewMetric()
	m.SetName("sum-4")
	m.SetDescription("sum-4-desc")
	m.SetUnit("sum-4-unit")
	Sum3().CopyTo(m.SetEmptySum())
	return m
}

func Metric5() pmetric.Metric {
	m := pmetric.NewMetric()
	m.SetName("gauge-1")
	m.SetDescription("gauge-1-desc")
	m.SetUnit("gauge-1-unit")
	Gauge3().CopyTo(m.SetEmptyGauge())
	return m
}

func ScopeMetrics1() pmetric.ScopeMetrics {
	sm := pmetric.NewScopeMetrics()
	sm.SetSchemaUrl("schema-1")
	Scope1().CopyTo(sm.Scope())
	ms := sm.Metrics()
	ms.EnsureCapacity(3)
	Metric1().CopyTo(ms.AppendEmpty())
	Metric2().CopyTo(ms.AppendEmpty())
	Metric3().CopyTo(ms.AppendEmpty())
	return sm
}

func ScopeMetrics2() pmetric.ScopeMetrics {
	sm := pmetric.NewScopeMetrics()
	sm.SetSchemaUrl("schema-2")
	Scope2().CopyTo(sm.Scope())
	ms := sm.Metrics()
	ms.EnsureCapacity(2)
	Metric2().CopyTo(ms.AppendEmpty())
	Metric3().CopyTo(ms.AppendEmpty())
	return sm
}

func ScopeMetrics3() pmetric.ScopeMetrics {
	sm := pmetric.NewScopeMetrics()
	sm.SetSchemaUrl("schema-3")
	Scope2().CopyTo(sm.Scope())
	ms := sm.Metrics()
	ms.EnsureCapacity(1)
	Metric4().CopyTo(ms.AppendEmpty())
	return sm
}

func ScopeMetrics4() pmetric.ScopeMetrics {
	sm := pmetric.NewScopeMetrics()
	sm.SetSchemaUrl("schema-3")
	Scope2().CopyTo(sm.Scope())
	ms := sm.Metrics()
	ms.EnsureCapacity(1)
	Metric5().CopyTo(ms.AppendEmpty())
	Metric4().CopyTo(ms.AppendEmpty())
	return sm
}

func ResourceMetrics1() pmetric.ResourceMetrics {
	rm := pmetric.NewResourceMetrics()
	Resource1().CopyTo(rm.Resource())
	rm.SetSchemaUrl("schema-1")
	sms := rm.ScopeMetrics()
	sms.EnsureCapacity(2)
	ScopeMetrics1().CopyTo(sms.AppendEmpty())
	ScopeMetrics2().CopyTo(sms.AppendEmpty())
	return rm
}

func ResourceMetrics2() pmetric.ResourceMetrics {
	rm := pmetric.NewResourceMetrics()
	Resource2().CopyTo(rm.Resource())
	rm.SetSchemaUrl("schema-2")
	sms := rm.ScopeMetrics()
	sms.EnsureCapacity(1)
	ScopeMetrics1().CopyTo(sms.AppendEmpty())
	return rm
}

func Metrics1() pmetric.Metrics {
	m := pmetric.NewMetrics()
	rms := m.ResourceMetrics()
	rms.EnsureCapacity(2)
	ResourceMetrics1().CopyTo(rms.AppendEmpty())
	ResourceMetrics2().CopyTo(rms.AppendEmpty())
	return m
}

func Metrics2() pmetric.Metrics {
	m := pmetric.NewMetrics()
	rms := m.ResourceMetrics()
	rms.EnsureCapacity(1)
	ResourceMetrics2().CopyTo(rms.AppendEmpty())
	return m
}

func Metrics(iter int) pmetric.Metrics {
	m := pmetric.NewMetrics()
	rms := m.ResourceMetrics()
	rms.EnsureCapacity(iter * 2)
	for i := 0; i < iter; i++ {
		ResourceMetrics1().CopyTo(rms.AppendEmpty())
		ResourceMetrics2().CopyTo(rms.AppendEmpty())
	}
	return m
}
