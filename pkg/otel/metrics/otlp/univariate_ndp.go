package otlp

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type UnivariateNdpIds struct {
	Id                int
	Attributes        *otlp.AttributeIds
	StartTimeUnixNano int
	TimeUnixNano      int
	MetricValue       int
	Exemplars         *ExemplarIds
	Flags             int
}

func NewUnivariateNdpIds(parentDT *arrow.StructType) (*UnivariateNdpIds, error) {
	id, univariateNdpDT, err := arrow_utils.ListOfStructsFieldIdFromStruct(parentDT, constants.DATA_POINTS)
	if err != nil {
		return nil, err
	}

	attributes, err := otlp.NewAttributeIds(univariateNdpDT)
	if err != nil {
		return nil, err
	}

	startTimeUnixNanoId, found := univariateNdpDT.FieldIdx(constants.START_TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.START_TIME_UNIX_NANO)
	}

	timeUnixNanoId, found := univariateNdpDT.FieldIdx(constants.TIME_UNIX_NANO)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.TIME_UNIX_NANO)
	}

	metricValueId, found := univariateNdpDT.FieldIdx(constants.METRIC_VALUE)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.METRIC_VALUE)
	}

	exemplars, err := NewExemplarIds(univariateNdpDT)
	if err != nil {
		return nil, err
	}

	flagsId, found := univariateNdpDT.FieldIdx(constants.FLAGS)
	if !found {
		return nil, fmt.Errorf("field %q not found", constants.FLAGS)
	}

	return &UnivariateNdpIds{
		Id:                id,
		Attributes:        attributes,
		StartTimeUnixNano: startTimeUnixNanoId,
		TimeUnixNano:      timeUnixNanoId,
		MetricValue:       metricValueId,
		Exemplars:         exemplars,
		Flags:             flagsId,
	}, nil
}

func AppendUnivariateNdpInto(ndpSlice pmetric.NumberDataPointSlice, ndp *arrow_utils.ListOfStructs, ids *UnivariateNdpIds) error {
	if ndp == nil {
		return nil
	}

	for ndpIdx := ndp.Start(); ndpIdx < ndp.End(); ndpIdx++ {
		ndpValue := ndpSlice.AppendEmpty()

		if ndp.IsNull(ndpIdx) {
			continue
		}

		if err := otlp.AppendAttributesInto(ndpValue.Attributes(), ndp.Array(), ndpIdx, ids.Attributes); err != nil {
			return err
		}

		startTimeUnixNano, err := ndp.U64FieldById(ids.StartTimeUnixNano, ndpIdx)
		if err != nil {
			return err
		}
		ndpValue.SetStartTimestamp(pcommon.Timestamp(startTimeUnixNano))
		timeUnixNano, err := ndp.U64FieldById(ids.TimeUnixNano, ndpIdx)
		if err != nil {
			return err
		}
		ndpValue.SetTimestamp(pcommon.Timestamp(timeUnixNano))

		value := ndp.FieldById(ids.MetricValue)
		if valueArr, ok := value.(*array.DenseUnion); ok {
			if err := UpdateValueFromNumberDataPoint(ndpValue, valueArr, ndpIdx); err != nil {
				return err
			}
		} else {
			return fmt.Errorf("value field shound be a DenseUnion")
		}

		flags, err := ndp.U32FieldById(ids.Flags, ndpIdx)
		if err != nil {
			return err
		}
		ndpValue.SetFlags(pmetric.DataPointFlags(flags))

		exemplars, err := ndp.ListOfStructsById(ndpIdx, ids.Exemplars.Id)
		if exemplars != nil && err == nil {
			if err := AppendExemplarsInto(ndpValue.Exemplars(), exemplars, ndpIdx, ids.Exemplars); err != nil {
				return err
			}
		} else if err != nil {
			return err
		}
	}

	return nil
}
