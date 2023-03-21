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

package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	ametric "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type UnivariateMetricIds struct {
	Id                      int
	UnivariateGaugeIds      *UnivariateGaugeIds
	UnivariateSumIds        *UnivariateSumIds
	UnivariateSummaryIds    *UnivariateSummaryIds
	UnivariateHistogramIds  *UnivariateHistogramIds
	UnivariateEHistogramIds *UnivariateEHistogramIds
}

func NewUnivariateMetricIds(parentDT *arrow.StructType) (*UnivariateMetricIds, error) {
	id, _ := arrowutils.FieldIDFromStruct(parentDT, constants.Data)

	if id == -1 {
		return &UnivariateMetricIds{
			Id:                      id,
			UnivariateGaugeIds:      nil,
			UnivariateSumIds:        nil,
			UnivariateSummaryIds:    nil,
			UnivariateHistogramIds:  nil,
			UnivariateEHistogramIds: nil,
		}, nil
	}

	dataDT, ok := parentDT.Field(id).Type.(*arrow.SparseUnionType)
	if !ok {
		return nil, werror.Wrap(ErrNotArraySparseUnion)
	}

	gaugeDT := arrowutils.StructFromSparseUnion(dataDT, ametric.GaugeCode)
	gaugeIds, err := NewUnivariateGaugeIds(gaugeDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	sumDT := arrowutils.StructFromSparseUnion(dataDT, ametric.SumCode)
	sumIds, err := NewUnivariateSumIds(sumDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	summaryDT := arrowutils.StructFromSparseUnion(dataDT, ametric.SummaryCode)
	summaryIds, err := NewUnivariateSummaryIds(summaryDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	histogramDT := arrowutils.StructFromSparseUnion(dataDT, ametric.HistogramCode)
	histogramIds, err := NewUnivariateHistogramIds(histogramDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	ehistogramDT := arrowutils.StructFromSparseUnion(dataDT, ametric.ExpHistogramCode)
	ehistogramIds, err := NewUnivariateEHistogramIds(ehistogramDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &UnivariateMetricIds{
		Id:                      id,
		UnivariateGaugeIds:      gaugeIds,
		UnivariateSumIds:        sumIds,
		UnivariateSummaryIds:    summaryIds,
		UnivariateHistogramIds:  histogramIds,
		UnivariateEHistogramIds: ehistogramIds,
	}, nil
}

func UpdateUnivariateMetricFrom(metric pmetric.Metric, arr *array.SparseUnion, row int, ids *UnivariateMetricIds, smdata *SharedData, mdata *SharedData) (err error) {
	tcode := arr.TypeCode(row)
	fieldID := arr.ChildID(row)

	switch tcode {
	case ametric.GaugeCode:
		err = UpdateUnivariateGaugeFrom(metric.SetEmptyGauge(), arr.Field(fieldID).(*array.Struct), row, ids.UnivariateGaugeIds, smdata, mdata)
	case ametric.SumCode:
		err = UpdateUnivariateSumFrom(metric.SetEmptySum(), arr.Field(fieldID).(*array.Struct), row, ids.UnivariateSumIds, smdata, mdata)
	case ametric.SummaryCode:
		err = UpdateUnivariateSummaryFrom(metric.SetEmptySummary(), arr.Field(fieldID).(*array.Struct), row, ids.UnivariateSummaryIds, smdata, mdata)
	case ametric.HistogramCode:
		err = UpdateUnivariateHistogramFrom(metric.SetEmptyHistogram(), arr.Field(fieldID).(*array.Struct), row, ids.UnivariateHistogramIds, smdata, mdata)
	case ametric.ExpHistogramCode:
		err = UpdateUnivariateEHistogramFrom(metric.SetEmptyExponentialHistogram(), arr.Field(fieldID).(*array.Struct), row, ids.UnivariateEHistogramIds, smdata, mdata)
	default:
		err = werror.WrapWithContext(ErrUnknownTypeCode, map[string]interface{}{"type_code": tcode, "row": row})
	}
	if err != nil {
		err = werror.WrapWithContext(err, map[string]interface{}{"row": row})
	}
	return
}
