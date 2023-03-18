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
	"math"
	"testing"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	cfg "github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/config"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/internal"
)

var DefaultDictConfig = &cfg.Dictionary{
	MaxCard: math.MaxUint16,
}

func TestValue(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.MetricValue, Type: MetricValueDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		mvb := MetricValueBuilderFrom(rBuilder.SparseUnionBuilder(constants.MetricValue))

		err := mvb.AppendNumberDataPointValue(internal.NDP1())
		require.NoError(t, err)
		err = mvb.AppendNumberDataPointValue(internal.NDP2())
		require.NoError(t, err)
		err = mvb.AppendNumberDataPointValue(internal.NDP3())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"value":[1,1.5]}
,{"value":[0,2]}
,{"value":[0,3]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExemplar(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.Exemplars, Type: ExemplarDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		exb := ExemplarBuilderFrom(rBuilder.StructBuilder(constants.Exemplars))

		err := exb.Append(internal.Exemplar1())
		require.NoError(t, err)
		err = exb.Append(internal.Exemplar2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"exemplars":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}}
,{"exemplars":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateNDP(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.DataPoints, Type: UnivariateNumberDataPointDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		exb := NumberDataPointBuilderFrom(rBuilder.StructBuilder(constants.DataPoints))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := exb.Append(internal.NDP1(), smdata, mdata)
		require.NoError(t, err)

		err = exb.Append(internal.NDP2(), smdata, mdata)
		require.NoError(t, err)

		err = exb.Append(internal.NDP3(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"data_points":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]}}
,{"data_points":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]}}
,{"data_points":{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateGauge(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.GaugeMetrics, Type: UnivariateGaugeDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		gb := UnivariateGaugeBuilderFrom(rBuilder.StructBuilder(constants.GaugeMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := gb.Append(internal.Gauge1(), smdata, mdata)
		require.NoError(t, err)
		err = gb.Append(internal.Gauge2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"gauge":{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}}
,{"gauge":{"data_points":[{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateSum(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.SumMetrics, Type: UnivariateSumDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := UnivariateSumBuilderFrom(rBuilder.StructBuilder(constants.SumMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := sb.Append(internal.Sum1(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.Sum2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"sum":{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}}
,{"sum":{"aggregation_temporality":2,"data_points":[{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":null}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestQuantileValue(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.SummaryQuantileValues, Type: QuantileValueDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := QuantileValueBuilderFrom(rBuilder.StructBuilder(constants.SummaryQuantileValues))

		err := sb.Append(internal.QuantileValue1())
		require.NoError(t, err)
		err = sb.Append(internal.QuantileValue2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"quantile":{"quantile":0.1,"value":1.5}}
,{"quantile":{"quantile":0.2,"value":2.5}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateSummaryDataPoint(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.DataPoints, Type: UnivariateSummaryDataPointDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := UnivariateSummaryDataPointBuilderFrom(rBuilder.StructBuilder(constants.DataPoints))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := sb.Append(internal.SummaryDataPoint1(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.SummaryDataPoint2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"data_points":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"}}
,{"data_points":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateSummary(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.SummaryMetrics, Type: UnivariateSummaryDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := UnivariateSummaryBuilderFrom(rBuilder.StructBuilder(constants.SummaryMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := sb.Append(internal.Summary1(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.Summary2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"summary":{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}}
,{"summary":{"data_points":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestUnivariateMetric(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.UnivariateMetrics, Type: UnivariateMetricDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := UnivariateMetricBuilderFrom(rBuilder.SparseUnionBuilder(constants.UnivariateMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := sb.Append(internal.Metric1(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.Metric2(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.Metric3(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"univariate_metrics":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}]}
,{"univariate_metrics":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}]}
,{"univariate_metrics":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestMetricSet(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.UnivariateMetrics, Type: UnivariateMetricSetDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := MetricSetBuilderFrom(rBuilder.StructBuilder(constants.UnivariateMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := sb.Append(internal.Metric1(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.Metric2(), smdata, mdata)
		require.NoError(t, err)
		err = sb.Append(internal.Metric3(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"univariate_metrics":{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"}}
,{"univariate_metrics":{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"}}
,{"univariate_metrics":{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestScopeMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ScopeMetrics, Type: ScopeMetricsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := ScopeMetricsBuilderFrom(rBuilder.StructBuilder(constants.ScopeMetrics))

		err := sb.Append(internal.ScopeMetrics1())
		require.NoError(t, err)
		err = sb.Append(internal.ScopeMetrics2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"scope_metrics":{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}}
,{"scope_metrics":{"schema_url":"schema-2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"univariate_metrics":[{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestResourceMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ResourceMetrics, Type: ResourceMetricsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb := ResourceMetricsBuilderFrom(rBuilder.StructBuilder(constants.ResourceMetrics))

		err := sb.Append(internal.ResourceMetrics1())
		require.NoError(t, err)
		err = sb.Append(internal.ResourceMetrics2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"resource_metrics":{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema-1","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]},{"schema_url":"schema-2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"univariate_metrics":[{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}]}}
,{"resource_metrics":{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema-2","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestMetrics(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	rBuilder := builder.NewRecordBuilderExt(pool, Schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		sb, err := NewMetricsBuilder(rBuilder)
		require.NoError(t, err)
		defer sb.Release()

		err = sb.Append(internal.Metrics1())
		require.NoError(t, err)

		err = sb.Append(internal.Metrics2())
		require.NoError(t, err)

		record, err = sb.Build()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	require.NoError(t, err)

	record.Release()

	expected := `[{"resource_metrics":[{"resource":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null},"schema_url":"schema-1","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]},{"schema_url":"schema-2","scope":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1,"name":"scope2","version":"1.0.2"},"univariate_metrics":[{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}]},{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema-2","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}]}]}
,{"resource_metrics":[{"resource":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"dropped_attributes_count":1},"schema_url":"schema-2","scope_metrics":[{"schema_url":"schema-1","scope":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"dropped_attributes_count":null,"name":"scope1","version":"1.0.1"},"univariate_metrics":[{"data":[0,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}]}],"description":"gauge-1-desc","name":"gauge-1","unit":"gauge-1-unit"},{"data":[1,{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","time_unix_nano":"1970-01-01 00:00:00.000000002","value":[1,1.5]},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000002","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[0,2]}],"flags":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","time_unix_nano":"1970-01-01 00:00:00.000000003","value":[0,2]},{"attributes":[{"key":"str","value":[0,"string3"]},{"key":"double","value":[2,3]},{"key":"bool","value":[3,false]},{"key":"bytes","value":[4,"Ynl0ZXMz"]}],"exemplars":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"span_id":"qgAAAAAAAAA=","time_unix_nano":"1970-01-01 00:00:00.000000001","trace_id":"qgAAAAAAAAAAAAAAAAAAAA==","value":[1,1.5]}],"flags":3,"start_time_unix_nano":"1970-01-01 00:00:00.000000003","time_unix_nano":"1970-01-01 00:00:00.000000004","value":[0,3]}],"is_monotonic":true}],"description":"sum-2-desc","name":"sum-2","unit":"sum-2-unit"},{"data":[2,{"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"quantile":[{"quantile":0.1,"value":1.5},{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"quantile":[{"quantile":0.2,"value":2.5}],"start_time_unix_nano":"1970-01-01 00:00:00.000000003","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000004"}]}],"description":"summary-3-desc","name":"summary-3","unit":"summary-3-unit"}]}]}]}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExponentialHistogramDataPointBuckets(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ExpHistogramPositive, Type: EHistogramDataPointBucketsDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		b := EHistogramDataPointBucketsBuilderFrom(rBuilder.StructBuilder(constants.ExpHistogramPositive))

		err := b.Append(internal.ExponentialHistogramDataPointBuckets1())
		require.NoError(t, err)
		err = b.Append(internal.ExponentialHistogramDataPointBuckets2())
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"positive":{"bucket_counts":[1,2],"offset":1}}
,{"positive":{"bucket_counts":[3,4],"offset":2}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExponentialHistogramDataPoint(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.DataPoints, Type: UnivariateEHistogramDataPointDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		b := EHistogramDataPointBuilderFrom(rBuilder.StructBuilder(constants.DataPoints))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := b.Append(internal.ExponentialHistogramDataPoint1(), smdata, mdata)
		require.NoError(t, err)
		err = b.Append(internal.ExponentialHistogramDataPoint2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"data_points":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"max":2.5,"min":1.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002","zero_count":1}}
,{"data_points":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"max":3.5,"min":2.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003","zero_count":2}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestHistogramDataPoint(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.DataPoints, Type: UnivariateHistogramDataPointDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		b := HistogramDataPointBuilderFrom(rBuilder.StructBuilder(constants.DataPoints))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := b.Append(internal.HistogramDataPoint1(), smdata, mdata)
		require.NoError(t, err)
		err = b.Append(internal.HistogramDataPoint2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"data_points":{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"bucket_counts":[1,2],"count":1,"explicit_bounds":[1.5,2.5],"flags":1,"max":2.5,"min":1.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"}}
,{"data_points":{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"bucket_counts":[3,4],"count":2,"explicit_bounds":[2.5,3.5],"flags":2,"max":3.5,"min":2.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003"}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestHistogram(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.HistogramMetrics, Type: UnivariateHistogramDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		b := UnivariateHistogramBuilderFrom(rBuilder.StructBuilder(constants.HistogramMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := b.Append(internal.Histogram1(), smdata, mdata)
		require.NoError(t, err)
		err = b.Append(internal.Histogram2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"histogram":{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"bucket_counts":[1,2],"count":1,"explicit_bounds":[1.5,2.5],"flags":1,"max":2.5,"min":1.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002"},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"bucket_counts":[3,4],"count":2,"explicit_bounds":[2.5,3.5],"flags":2,"max":3.5,"min":2.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003"}]}}
,{"histogram":{"aggregation_temporality":2,"data_points":[{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"bucket_counts":[3,4],"count":2,"explicit_bounds":[2.5,3.5],"flags":2,"max":3.5,"min":2.5,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003"}]}}
]`

	require.JSONEq(t, expected, string(json))
}

func TestExponentialHistogram(t *testing.T) {
	t.Parallel()

	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ExpHistogramMetrics, Type: UnivariateEHistogramDT, Metadata: acommon.Metadata(acommon.Optional)},
	}, nil)
	rBuilder := builder.NewRecordBuilderExt(pool, schema, DefaultDictConfig)
	defer rBuilder.Release()

	var record arrow.Record

	for {
		b := UnivariateEHistogramBuilderFrom(rBuilder.StructBuilder(constants.ExpHistogramMetrics))

		smdata := &ScopeMetricsSharedData{Attributes: &common.SharedAttributes{}}
		mdata := &MetricSharedData{Attributes: &common.SharedAttributes{}}

		err := b.Append(internal.ExpHistogram1(), smdata, mdata)
		require.NoError(t, err)
		err = b.Append(internal.ExpHistogram2(), smdata, mdata)
		require.NoError(t, err)

		record, err = rBuilder.NewRecord()
		if err == nil {
			break
		}
		assert.Error(t, acommon.ErrSchemaNotUpToDate)
	}

	json, err := record.MarshalJSON()
	if err != nil {
		t.Fatal(err)
	}

	record.Release()

	expected := `[{"exp_histogram":{"aggregation_temporality":1,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"max":2.5,"min":1.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002","zero_count":1},{"attributes":[{"key":"str","value":[0,"string2"]},{"key":"int","value":[1,2]},{"key":"double","value":[2,2]},{"key":"bytes","value":[4,"Ynl0ZXMy"]}],"count":2,"flags":2,"max":3.5,"min":2.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":2,"start_time_unix_nano":"1970-01-01 00:00:00.000000002","sum":2.5,"time_unix_nano":"1970-01-01 00:00:00.000000003","zero_count":2}]}}
,{"exp_histogram":{"aggregation_temporality":2,"data_points":[{"attributes":[{"key":"str","value":[0,"string1"]},{"key":"int","value":[1,1]},{"key":"double","value":[2,1]},{"key":"bool","value":[3,true]},{"key":"bytes","value":[4,"Ynl0ZXMx"]}],"count":1,"flags":1,"max":2.5,"min":1.5,"negative":{"bucket_counts":[3,4],"offset":2},"positive":{"bucket_counts":[1,2],"offset":1},"scale":1,"start_time_unix_nano":"1970-01-01 00:00:00.000000001","sum":1.5,"time_unix_nano":"1970-01-01 00:00:00.000000002","zero_count":1}]}}
]`

	require.JSONEq(t, expected, string(json))
}
