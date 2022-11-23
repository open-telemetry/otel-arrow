package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type QuantileValueIds struct {
	Id       int
	Quantile int
	Value    int
}

func NewQuantileValueIds(parentDT *arrow.StructType) (*QuantileValueIds, error) {
	id, quantileValueDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.SUMMARY_QUANTILE_VALUES)
	if err != nil {
		return nil, err
	}

	quantile, quantileFound := quantileValueDT.FieldIdx(constants.SUMMARY_QUANTILE)
	if !quantileFound {
		return nil, fmt.Errorf("field %q not found", constants.SUMMARY_QUANTILE)
	}

	value, valueFound := quantileValueDT.FieldIdx(constants.SUMMARY_VALUE)
	if !valueFound {
		return nil, fmt.Errorf("field %q not found", constants.SUMMARY_VALUE)
	}

	return &QuantileValueIds{
		Id:       id,
		Quantile: quantile,
		Value:    value,
	}, nil
}

func AppendQuantileValuesInto(quantileSlice pmetric.SummaryDataPointValueAtQuantileSlice, ndp *arrowutils.ListOfStructs, ndpIdx int, ids *QuantileValueIds) error {
	quantileValues, err := ndp.ListOfStructsById(ndpIdx, ids.Id)
	if err != nil {
		return err
	}
	if quantileValues == nil {
		return nil
	}

	for quantileIdx := quantileValues.Start(); quantileIdx < quantileValues.End(); quantileIdx++ {
		quantileValue := quantileSlice.AppendEmpty()

		if quantileValues.IsNull(quantileIdx) {
			continue
		}

		quantile, err := quantileValues.F64FieldByID(ids.Quantile, quantileIdx)
		if err != nil {
			return err
		}
		quantileValue.SetQuantile(quantile)

		value, err := quantileValues.F64FieldByID(ids.Value, quantileIdx)
		if err != nil {
			return err
		}
		quantileValue.SetValue(value)
	}
	return nil
}
