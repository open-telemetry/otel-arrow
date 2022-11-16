package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type UnivariateEHistogramIds struct {
	DataPoints             *UnivariateEHistogramDataPointIds
	AggregationTemporality int
}

func NewUnivariateEHistogramIds(parentDT *arrow.StructType) (*UnivariateEHistogramIds, error) {
	dataPoints, err := NewUnivariateEHistogramDataPointIds(parentDT)
	if err != nil {
		return nil, err
	}

	aggrTempId, found := parentDT.FieldIdx(constants.AGGREGATION_TEMPORALITY)
	if !found {
		return nil, fmt.Errorf("missing field %q", constants.AGGREGATION_TEMPORALITY)
	}

	return &UnivariateEHistogramIds{
		DataPoints:             dataPoints,
		AggregationTemporality: aggrTempId,
	}, nil
}

func UpdateUnivariateEHistogramFrom(ehistogram pmetric.ExponentialHistogram, arr *array.Struct, row int, ids *UnivariateEHistogramIds) error {
	atArr, ok := arr.Field(ids.AggregationTemporality).(*array.Int32)
	if !ok {
		return fmt.Errorf("field %q is not an int64", constants.AGGREGATION_TEMPORALITY)
	}
	ehistogram.SetAggregationTemporality(pmetric.AggregationTemporality(atArr.Value(row)))

	los, err := arrow_utils.ListOfStructsFromStruct(arr, ids.DataPoints.Id, row)
	if err != nil {
		return err
	}
	return AppendUnivariateEHistogramDataPointInto(ehistogram.DataPoints(), los, ids.DataPoints)
}
