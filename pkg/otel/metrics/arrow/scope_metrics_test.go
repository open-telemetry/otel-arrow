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
	"testing"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

func TestIntersectAttrs(t *testing.T) {
	t.Parallel()

	initMap := pcommon.NewMap()
	initMap.PutStr("a", "1")
	initMap.PutStr("b", "2")
	initMap.PutInt("c", 3)
	initMap.PutDouble("d", 4.0)
	initMap.PutBool("e", true)
	initMap.PutEmptyBytes("f").Append([]byte("6")...)
	initMap.PutEmptyMap("g").PutStr("g1", "7")
	sharedAttrs := common.NewSharedAttributesFrom(initMap)

	// All attributes in common
	attrs := pcommon.NewMap()
	attrs.PutStr("a", "1")
	attrs.PutStr("b", "2")
	attrs.PutInt("c", 3)
	attrs.PutDouble("d", 4.0)
	attrs.PutBool("e", true)
	attrs.PutEmptyBytes("f").Append([]byte("6")...)
	attrs.PutEmptyMap("g").PutStr("g1", "7")
	sharedAttrsCount := sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 7, sharedAttrsCount)

	// 1 attribute is missing from attrs
	attrs = pcommon.NewMap()
	attrs.PutStr("a", "1")
	attrs.PutStr("b", "2")
	attrs.PutInt("c", 3)
	attrs.PutDouble("d", 4.0)
	attrs.PutBool("e", true)
	attrs.PutEmptyBytes("f").Append([]byte("6")...)
	sharedAttrsCount = sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 6, sharedAttrsCount)
	require.False(t, sharedAttrs.Has("g"))

	// 1 attribute has a different
	attrs = pcommon.NewMap()
	attrs.PutStr("a", "1")
	attrs.PutStr("b", "2")
	attrs.PutInt("c", 3)
	attrs.PutDouble("d", 4.0)
	attrs.PutBool("e", false)
	attrs.PutEmptyBytes("f").Append([]byte("6")...)
	sharedAttrsCount = sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 5, sharedAttrsCount)
	require.False(t, sharedAttrs.Has("e"))

	// 1 attribute is new
	attrs = pcommon.NewMap()
	attrs.PutStr("a", "1")
	attrs.PutStr("b", "2")
	attrs.PutInt("c", 3)
	attrs.PutDouble("d", 4.0)
	attrs.PutEmptyBytes("f").Append([]byte("6")...)
	attrs.PutBool("h", false)
	sharedAttrsCount = sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 5, sharedAttrsCount)
	require.False(t, sharedAttrs.Has("h"))

	// 1 attribute is new
	// 1 attribute is missing
	// 1 attribute has a different value
	attrs = pcommon.NewMap()
	attrs.PutStr("a", "1")
	attrs.PutStr("b", "2")
	attrs.PutInt("c", 4)
	attrs.PutEmptyBytes("f").Append([]byte("6")...)
	attrs.PutBool("h", false)
	sharedAttrsCount = sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 3, sharedAttrsCount)
	require.True(t, sharedAttrs.Has("a"))
	require.True(t, sharedAttrs.Has("b"))
	require.True(t, sharedAttrs.Has("f"))

	// No attributes in common
	attrs = pcommon.NewMap()
	attrs.PutStr("x", "1")
	attrs.PutStr("y", "2")
	sharedAttrsCount = sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 0, sharedAttrsCount)

	// Empty attributes
	attrs = pcommon.NewMap()
	sharedAttrsCount = sharedAttrs.IntersectWithMap(attrs)
	require.Equal(t, 0, sharedAttrsCount)
}

func TestScopeMetricsSharedData1(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(0)
	dg := datagen.NewMetricsGeneratorFromEntropy(entropy)
	metrics := dg.GenerateMetricSlice(1, 1)
	sharedData, err := NewMetricsSharedData(metrics)
	require.NoError(t, err)

	require.NotNil(t, sharedData.StartTime)
	require.NotNil(t, sharedData.Time)
	require.NotNil(t, sharedData.Attributes)
	require.Equal(t, 8, sharedData.Attributes.Len()) // cpu attribute

	require.Equal(t, 5, len(sharedData.Metrics))

	require.Nil(t, sharedData.Metrics[0].StartTime)
	require.Nil(t, sharedData.Metrics[0].Time)
	require.Equal(t, 0, len(sharedData.Metrics[0].Attributes.Attributes))

	require.Nil(t, sharedData.Metrics[1].StartTime)
	require.Nil(t, sharedData.Metrics[1].Time)
	require.Equal(t, 0, len(sharedData.Metrics[1].Attributes.Attributes))

	require.Nil(t, sharedData.Metrics[2].StartTime)
	require.Nil(t, sharedData.Metrics[2].Time)
	require.Equal(t, 0, len(sharedData.Metrics[2].Attributes.Attributes))

	require.Nil(t, sharedData.Metrics[3].StartTime)
	require.Nil(t, sharedData.Metrics[3].Time)
	require.Equal(t, 1, len(sharedData.Metrics[3].Attributes.Attributes)) // freq attribute

	require.Nil(t, sharedData.Metrics[4].StartTime)
	require.Nil(t, sharedData.Metrics[4].Time)
	require.Equal(t, 1, len(sharedData.Metrics[4].Attributes.Attributes)) // freq attribute
}

func TestScopeMetricsSharedData2(t *testing.T) {
	t.Parallel()

	metrics := internal.ScopeMetrics3().Metrics()
	sharedData, err := NewMetricsSharedData(metrics)
	require.NoError(t, err)

	require.NotNil(t, sharedData.StartTime)
	require.NotNil(t, sharedData.Time)
	require.NotNil(t, sharedData.Attributes)
	require.Equal(t, 5, sharedData.Attributes.Len())

	startTime := pcommon.Timestamp(1)
	require.Equal(t, &startTime, sharedData.StartTime)
	time := pcommon.Timestamp(2)
	require.Equal(t, &time, sharedData.Time)

	require.Equal(t, 1, len(sharedData.Metrics))

	require.Nil(t, sharedData.Metrics[0].StartTime)
	require.Nil(t, sharedData.Metrics[0].Time)
	require.Equal(t, 0, len(sharedData.Metrics[0].Attributes.Attributes))
}

func TestScopeMetricsSharedData3(t *testing.T) {
	t.Parallel()

	metrics := internal.ScopeMetrics4().Metrics()
	sharedData, err := NewMetricsSharedData(metrics)
	require.NoError(t, err)

	require.NotNil(t, sharedData.StartTime)
	require.NotNil(t, sharedData.Time)
	require.NotNil(t, sharedData.Attributes)
	require.Equal(t, 5, sharedData.Attributes.Len())

	startTime := pcommon.Timestamp(1)
	require.Equal(t, &startTime, sharedData.StartTime)
	time := pcommon.Timestamp(2)
	require.Equal(t, &time, sharedData.Time)

	require.Equal(t, 2, len(sharedData.Metrics))

	require.Nil(t, sharedData.Metrics[0].StartTime)
	require.Nil(t, sharedData.Metrics[0].Time)
	require.Equal(t, 0, len(sharedData.Metrics[0].Attributes.Attributes))

	require.Nil(t, sharedData.Metrics[1].StartTime)
	require.Nil(t, sharedData.Metrics[1].Time)
	require.Equal(t, 0, len(sharedData.Metrics[1].Attributes.Attributes))
}

func TestMetricSharedData(t *testing.T) {
	t.Parallel()

	metric := SingleSystemMemoryUsage(0, 10)
	sharedData, err := NewMetricSharedData(metric)
	require.NoError(t, err)
	require.Equal(t, 1, sharedData.NumDP)
	require.NotNil(t, sharedData.StartTime)
	require.NotNil(t, sharedData.Time)
	require.NotNil(t, sharedData.Attributes)
	require.Equal(t, 2, sharedData.Attributes.Len())

	metric = MultiSystemMemoryUsage(0, 10)
	sharedData, err = NewMetricSharedData(metric)
	require.NoError(t, err)
	require.Equal(t, 3, sharedData.NumDP)
	require.NotNil(t, sharedData.StartTime)
	require.NotNil(t, sharedData.Time)
	require.NotNil(t, sharedData.Attributes)
	require.Equal(t, 1, sharedData.Attributes.Len())
}

func SingleSystemMemoryUsage(startTs, currentTs pcommon.Timestamp) pmetric.Metric {
	metric := pmetric.NewMetric()
	metric.SetName("system.memory.usage")
	metric.SetDescription("Bytes of memory in use.")
	metric.SetUnit("By")

	sum := metric.SetEmptySum()
	sum.SetAggregationTemporality(pmetric.AggregationTemporalityCumulative)
	sum.SetIsMonotonic(false)

	points := sum.DataPoints()

	p1 := points.AppendEmpty()
	p1.Attributes().PutStr("host", "my-host")
	p1.Attributes().PutStr("state", "used")
	p1.SetStartTimestamp(startTs)
	p1.SetTimestamp(currentTs)
	p1.SetIntValue(10)

	return metric
}

func MultiSystemMemoryUsage(startTs, currentTs pcommon.Timestamp) pmetric.Metric {
	metric := pmetric.NewMetric()
	metric.SetName("system.memory.usage")
	metric.SetDescription("Bytes of memory in use.")
	metric.SetUnit("By")

	sum := metric.SetEmptySum()
	sum.SetAggregationTemporality(pmetric.AggregationTemporalityCumulative)
	sum.SetIsMonotonic(false)

	points := sum.DataPoints()

	p1 := points.AppendEmpty()
	p1.Attributes().PutStr("host", "my-host")
	p1.Attributes().PutStr("state", "used")
	p1.SetStartTimestamp(startTs)
	p1.SetTimestamp(currentTs)
	p1.SetIntValue(10)

	p2 := points.AppendEmpty()
	p2.Attributes().PutStr("host", "my-host")
	p2.Attributes().PutStr("state", "free")
	p2.SetStartTimestamp(startTs)
	p2.SetTimestamp(currentTs)
	p2.SetIntValue(20)

	p3 := points.AppendEmpty()
	p3.Attributes().PutStr("host", "my-host")
	p3.Attributes().PutStr("state", "inactive")
	p3.SetStartTimestamp(startTs)
	p3.SetTimestamp(currentTs)
	p3.SetIntValue(30)

	return metric
}
