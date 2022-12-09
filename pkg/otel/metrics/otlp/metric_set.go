package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type MetricSetIds struct {
	Id                 int
	Name               int
	Description        int
	Unit               int
	Data               *UnivariateMetricIds
	SharedAttributeIds *otlp.AttributeIds
	SharedStartTimeID  int
	SharedTimeID       int
}

func NewMetricSetIds(parentDT *arrow.StructType) (*MetricSetIds, error) {
	id, metricSetDT, err := arrowutils.ListOfStructsFieldIDFromStruct(parentDT, constants.UNIVARIATE_METRICS)
	if err != nil {
		return nil, err
	}

	name, _, err := arrowutils.FieldIDFromStruct(metricSetDT, constants.NAME)
	if err != nil {
		return nil, err
	}

	description, _, err := arrowutils.FieldIDFromStruct(metricSetDT, constants.DESCRIPTION)
	if err != nil {
		return nil, err
	}

	unit, _, err := arrowutils.FieldIDFromStruct(metricSetDT, constants.UNIT)
	if err != nil {
		return nil, err
	}

	data, err := NewUnivariateMetricIds(metricSetDT)
	if err != nil {
		return nil, err
	}

	sharedAttrIds := otlp.NewSharedAttributeIds(metricSetDT)
	startTimeID := arrowutils.OptionalFieldIDFromStruct(metricSetDT, constants.SHARED_START_TIME_UNIX_NANO)
	timeID := arrowutils.OptionalFieldIDFromStruct(metricSetDT, constants.SHARED_TIME_UNIX_NANO)

	return &MetricSetIds{
		Id:                 id,
		Name:               name,
		Description:        description,
		Unit:               unit,
		Data:               data,
		SharedAttributeIds: sharedAttrIds,
		SharedStartTimeID:  startTimeID,
		SharedTimeID:       timeID,
	}, nil
}

func AppendMetricSetInto(metrics pmetric.MetricSlice, los *arrowutils.ListOfStructs, row int, ids *MetricSetIds, smdata *SharedData) error {
	metric := metrics.AppendEmpty()

	name, err := los.StringFieldByID(ids.Name, row)
	if err != nil {
		return err
	}
	metric.SetName(name)

	description, err := los.StringFieldByID(ids.Description, row)
	if err != nil {
		return err
	}
	metric.SetDescription(description)

	unit, err := los.StringFieldByID(ids.Unit, row)
	if err != nil {
		return err
	}
	metric.SetUnit(unit)

	mdata := &SharedData{}
	if ids.SharedAttributeIds != nil {
		mdata.Attributes = pcommon.NewMap()
		err = otlp.AppendAttributesInto(mdata.Attributes, los.Array(), row, ids.SharedAttributeIds)
		if err != nil {
			return err
		}
	}
	if ids.SharedStartTimeID != -1 {
		mdata.StartTime = los.OptionalTimestampFieldByID(ids.SharedStartTimeID, row)
	}
	if ids.SharedTimeID != -1 {
		mdata.Time = los.OptionalTimestampFieldByID(ids.SharedTimeID, row)
	}

	return UpdateUnivariateMetricFrom(metric, los, row, ids.Data, smdata, mdata)
}
