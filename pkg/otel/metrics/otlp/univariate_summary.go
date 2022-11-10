package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	arrow_utils "github.com/f5/otel-arrow-adapter/pkg/arrow"
)

type UnivariateSummaryIds struct {
	DataPoints *UnivariateSdpIds
}

func NewUnivariateSummaryIds(parentDT *arrow.StructType) (*UnivariateSummaryIds, error) {
	dataPoints, err := NewUnivariateSdpIds(parentDT)
	if err != nil {
		return nil, err
	}

	return &UnivariateSummaryIds{
		DataPoints: dataPoints,
	}, nil
}

func UpdateUnivariateSummaryFrom(summary pmetric.Summary, arr *array.Struct, row int, ids *UnivariateSummaryIds) error {
	los, err := arrow_utils.ListOfStructsFromStruct(arr, ids.DataPoints.Id, row)
	if err != nil {
		return err
	}
	return AppendUnivariateSdpInto(summary.DataPoints(), los, ids.DataPoints)
}
