package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type MetricSetIds struct {
	Id          int
	Name        int
	Description int
	Unit        int
	Data        *UnivariateMetricIds
}

func NewMetricSetIds(parentDT *arrow.StructType) (*MetricSetIds, error) {
	id, metricSetDT, err := arrow_utils.ListOfStructsFieldIdFromStruct(parentDT, constants.METRICS)
	if err != nil {
		return nil, err
	}

	name, _, err := arrow_utils.FieldIdFromStruct(metricSetDT, constants.NAME)
	if err != nil {
		return nil, err
	}

	description, _, err := arrow_utils.FieldIdFromStruct(metricSetDT, constants.DESCRIPTION)
	if err != nil {
		return nil, err
	}

	unit, _, err := arrow_utils.FieldIdFromStruct(metricSetDT, constants.UNIT)
	if err != nil {
		return nil, err
	}

	data, err := NewUnivariateMetricIds(metricSetDT)
	if err != nil {
		return nil, err
	}

	return &MetricSetIds{
		Id:          id,
		Name:        name,
		Description: description,
		Unit:        unit,
		Data:        data,
	}, nil
}

func AppendMetricSetInto(metrics pmetric.MetricSlice, los *arrow_utils.ListOfStructs, row int, ids *MetricSetIds) error {
	metric := metrics.AppendEmpty()

	name, err := los.StringFieldById(ids.Name, row)
	if err != nil {
		return err
	}
	metric.SetName(name)

	description, err := los.StringFieldById(ids.Description, row)
	if err != nil {
		return err
	}
	metric.SetDescription(description)

	unit, err := los.StringFieldById(ids.Unit, row)
	if err != nil {
		return err
	}
	metric.SetUnit(unit)

	return UpdateUnivariateMetricFrom(metric, los, row, ids.Data)
}
