package otlp

import (
	"github.com/apache/arrow/go/v10/arrow"
	"github.com/apache/arrow/go/v10/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
)

type UnivariateGaugeIds struct {
	DataPoints *UnivariateNdpIds
}

func NewUnivariateGaugeIds(parentDT *arrow.StructType) (*UnivariateGaugeIds, error) {
	dataPoints, err := NewUnivariateNdpIds(parentDT)
	if err != nil {
		return nil, err
	}

	return &UnivariateGaugeIds{
		DataPoints: dataPoints,
	}, nil
}

func UpdateUnivariateGaugeFrom(gauge pmetric.Gauge, arr *array.Struct, row int, ids *UnivariateGaugeIds, smdata *SharedData, mdata *SharedData) error {
	los, err := arrowutils.ListOfStructsFromStruct(arr, ids.DataPoints.Id, row)
	if err != nil {
		return err
	}
	return AppendUnivariateNdpInto(gauge.DataPoints(), los, ids.DataPoints, smdata, mdata)
}
