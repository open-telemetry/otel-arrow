package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type UnivariateHistogramIds struct {
	DataPoints             *UnivariateHistogramDataPointIds
	AggregationTemporality int
}

func NewUnivariateHistogramIds(parentDT *arrow.StructType) (*UnivariateHistogramIds, error) {
	dataPoints, err := NewUnivariateHistogramDataPointIds(parentDT)
	if err != nil {
		return nil, err
	}

	aggrTempId, found := parentDT.FieldIdx(constants.AGGREGATION_TEMPORALITY)
	if !found {
		return nil, fmt.Errorf("missing field %q", constants.AGGREGATION_TEMPORALITY)
	}

	return &UnivariateHistogramIds{
		DataPoints:             dataPoints,
		AggregationTemporality: aggrTempId,
	}, nil
}

func UpdateUnivariateHistogramFrom(histogram pmetric.Histogram, arr *array.Struct, row int, ids *UnivariateHistogramIds, smdata *SharedData, mdata *SharedData) error {
	atArr, ok := arr.Field(ids.AggregationTemporality).(*array.Int32)
	if !ok {
		return fmt.Errorf("field %q is not an int64", constants.AGGREGATION_TEMPORALITY)
	}
	histogram.SetAggregationTemporality(pmetric.AggregationTemporality(atArr.Value(row)))

	los, err := arrowutils.ListOfStructsFromStruct(arr, ids.DataPoints.Id, row)
	if err != nil {
		return err
	}
	return AppendUnivariateHistogramDataPointInto(histogram.DataPoints(), los, ids.DataPoints, smdata, mdata)
}
