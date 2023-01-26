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

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

func TestValue(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	mvb := NewMetricValueBuilder(pool)

	if err := mvb.AppendNumberDataPointValue(NDP1()); err != nil {
		t.Fatal(err)
	}
	if err := mvb.AppendNumberDataPointValue(NDP2()); err != nil {
		t.Fatal(err)
	}
	if err := mvb.AppendNumberDataPointValue(NDP3()); err != nil {
		t.Fatal(err)
	}
	arr, err := mvb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[[1,1.5]
,[0,2]
,[0,3]
]`

	require.JSONEq(t, expected, string(json))
}

func TestExemplar(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	exb := NewExemplarBuilder(pool)

	if err := exb.Append(Exemplar1()); err != nil {
		t.Fatal(err)
	}
	if err := exb.Append(Exemplar2()); err != nil {
		t.Fatal(err)
	}
	arr, err := exb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateNDP(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	exb := NewNumberDataPointBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := exb.Append(NDP1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := exb.Append(NDP2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := exb.Append(NDP3(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := exb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]}
,{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateGauge(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	gb := NewUnivariateGaugeBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := gb.Append(Gauge1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := gb.Append(Gauge2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := gb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}
,{"data_points":[{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateSum(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewUnivariateSumBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := sb.Append(Sum1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Sum2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}
,{"aggregation_temporality":2,"data_points":[{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":null}
]`

	require.JSONEq(t, expected, string(json))
}

func TestQuantileValue(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewQuantileValueBuilder(pool)

	if err := sb.Append(QuantileValue1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(QuantileValue2()); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"quantile":0.1,"value":1.5}
,{"quantile":0.2,"value":2.5}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateSummaryDataPoint(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewUnivariateSummaryDataPointBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := sb.Append(SummaryDataPoint1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(SummaryDataPoint2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateSummary(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewUnivariateSummaryBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := sb.Append(Summary1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Summary2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}
,{"data_points":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateMetric(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewUnivariateMetricBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := sb.Append(Metric1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Metric2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Metric3(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}]
,[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}]
,[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}]
]`

	require.JSONEq(t, expected, string(json))
}

func TestMetricSet(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewMetricSetBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := sb.Append(Metric1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Metric2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(Metric3(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"}
,{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"}
,{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestScopeMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewScopeMetricsBuilder(pool)

	if err := sb.Append(ScopeMetrics1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(ScopeMetrics2()); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}
,{"schema_url":"schema-2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestResourceMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	sb := NewResourceMetricsBuilder(pool)

	if err := sb.Append(ResourceMetrics1()); err != nil {
		t.Fatal(err)
	}
	if err := sb.Append(ResourceMetrics2()); err != nil {
		t.Fatal(err)
	}
	arr, err := sb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema-1","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]},{"schema_url":"schema-2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}]}
,{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema-2","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	metricsSchema := acommon.NewAdaptiveSchema(pool, Schema)
	defer metricsSchema.Release()
	sb, err := NewMetricsBuilder(metricsSchema)
	require.NoError(t, err)
	defer sb.Release()

	err = sb.Append(Metrics1())
	require.NoError(t, err)

	err = sb.Append(Metrics2())
	require.NoError(t, err)

	var arr arrow.Record
	arr, err = sb.Build()
	require.NoError(t, err)
	defer arr.Release()

	json, err := arr.MarshalJSON()
	require.NoError(t, err)

	expected := `[{"resource_metrics":[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema-1","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]},{"schema_url":"schema-2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}]},{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema-2","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}]}]}
,{"resource_metrics":[{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema-2","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","shared_attributes":null,"shared_start_time_unix_nano":null,"shared_time_unix_nano":null,"unit":"summary-3-unit"}]}]}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExponentialHistogramDataPointBuckets(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	b := NewEHistogramDataPointBucketsBuilder(pool)

	if err := b.Append(ExponentialHistogramDataPointBuckets1()); err != nil {
		t.Fatal(err)
	}
	if err := b.Append(ExponentialHistogramDataPointBuckets2()); err != nil {
		t.Fatal(err)
	}
	arr, err := b.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"bucket_counts":[1,2],"offset":1}
,{"bucket_counts":[3,4],"offset":2}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExponentialHistogramDataPoint(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	b := NewEHistogramDataPointBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := b.Append(ExponentialHistogramDataPoint1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := b.Append(ExponentialHistogramDataPoint2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := b.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"exemplars":null,"flags":1,"max":2.5,"min":1.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002","zero_count":1}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"exemplars":null,"flags":2,"max":3.5,"min":2.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003","zero_count":2}
]`

	require.JSONEq(t, expected, string(json))
}

func TestHistogramDataPoint(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	b := NewHistogramDataPointBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := b.Append(HistogramDataPoint1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := b.Append(HistogramDataPoint2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := b.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"bucket_counts":[1,2],"count":1,"exemplars":null,"explicit_bounds":[1.5,2.5],"flags":1,"max":2.5,"min":1.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"}
,{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"bucket_counts":[3,4],"count":2,"exemplars":null,"explicit_bounds":[2.5,3.5],"flags":2,"max":3.5,"min":2.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003"}
]`

	require.JSONEq(t, expected, string(json))
}

func TestHistogram(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	b := NewUnivariateHistogramBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := b.Append(Histogram1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := b.Append(Histogram2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := b.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"bucket_counts":[1,2],"count":1,"exemplars":null,"explicit_bounds":[1.5,2.5],"flags":1,"max":2.5,"min":1.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"bucket_counts":[3,4],"count":2,"exemplars":null,"explicit_bounds":[2.5,3.5],"flags":2,"max":3.5,"min":2.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003"}]}
,{"aggregation_temporality":2,"data_points":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"bucket_counts":[3,4],"count":2,"exemplars":null,"explicit_bounds":[2.5,3.5],"flags":2,"max":3.5,"min":2.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003"}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExponentialHistogram(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	b := NewUnivariateEHistogramBuilder(pool)

	smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
	mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

	if err := b.Append(ExpHistogram1(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	if err := b.Append(ExpHistogram2(), smdata, mdata); err != nil {
		t.Fatal(err)
	}
	arr, err := b.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer arr.Release()

	json, err := arr.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	expected := `[{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"exemplars":null,"flags":1,"max":2.5,"min":1.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002","zero_count":1},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"exemplars":null,"flags":2,"max":3.5,"min":2.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003","zero_count":2}]}
,{"aggregation_temporality":2,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"exemplars":null,"flags":1,"max":2.5,"min":1.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002","zero_count":1}]}
]`

	require.JSONEq(t, expected, string(json))
}

// NDP1 returns a pmetric.NumberDataPoint (sample 1).
func NDP1() pmetric.NumberDataPoint {
	dp := pmetric.NewNumberDataPoint()
	internal.Attrs1().CopyTo(dp.Attributes())
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
	internal.Attrs2().CopyTo(dp.Attributes())
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
	internal.Attrs3().CopyTo(dp.Attributes())
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
	internal.Attrs1().CopyTo(ex.FilteredAttributes())
	ex.SetTimestamp(1)
	ex.SetDoubleValue(1.5)
	ex.SetSpanID([8]byte{0xAA})
	ex.SetTraceID([16]byte{0xAA})
	return ex
}

func Exemplar2() pmetric.Exemplar {
	ex := pmetric.NewExemplar()
	internal.Attrs2().CopyTo(ex.FilteredAttributes())
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

func Sum1() pmetric.Sum {
	g := pmetric.NewSum()
	NDP1().CopyTo(g.DataPoints().AppendEmpty())
	NDP2().CopyTo(g.DataPoints().AppendEmpty())
	NDP3().CopyTo(g.DataPoints().AppendEmpty())
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

func SummaryDataPoint1() pmetric.SummaryDataPoint {
	dp := pmetric.NewSummaryDataPoint()
	internal.Attrs1().CopyTo(dp.Attributes())
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
	internal.Attrs2().CopyTo(dp.Attributes())
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

func ScopeMetrics1() pmetric.ScopeMetrics {
	sm := pmetric.NewScopeMetrics()
	sm.SetSchemaUrl("schema-1")
	internal.Scope1().CopyTo(sm.Scope())
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
	internal.Scope2().CopyTo(sm.Scope())
	ms := sm.Metrics()
	ms.EnsureCapacity(2)
	Metric2().CopyTo(ms.AppendEmpty())
	Metric3().CopyTo(ms.AppendEmpty())
	return sm
}

func ResourceMetrics1() pmetric.ResourceMetrics {
	rm := pmetric.NewResourceMetrics()
	internal.Resource1().CopyTo(rm.Resource())
	rm.SetSchemaUrl("schema-1")
	sms := rm.ScopeMetrics()
	sms.EnsureCapacity(2)
	ScopeMetrics1().CopyTo(sms.AppendEmpty())
	ScopeMetrics2().CopyTo(sms.AppendEmpty())
	return rm
}

func ResourceMetrics2() pmetric.ResourceMetrics {
	rm := pmetric.NewResourceMetrics()
	internal.Resource2().CopyTo(rm.Resource())
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
	internal.Attrs1().CopyTo(dp.Attributes())
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
	internal.Attrs2().CopyTo(dp.Attributes())
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
	internal.Attrs1().CopyTo(dp.Attributes())
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
	internal.Attrs2().CopyTo(dp.Attributes())
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
