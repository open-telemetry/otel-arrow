package logs

import (
	"github.com/apache/arrow/go/v9/arrow"
	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	logspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/logs/v1"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/field_value"
)

func OtlpLogsToArrowEvents(rbr *rbb.RecordBatchRepository, request *collogspb.ExportLogsServiceRequest) ([]arrow.Record, error) {
	for _, resourceLogs := range request.ResourceLogs {
		for _, scopeLogs := range resourceLogs.ScopeLogs {
			for _, log := range scopeLogs.LogRecords {
				record := rbb.NewRecord()

				if log.TimeUnixNano > 0 {
					record.U64Field(constants.TIME_UNIX_NANO, log.TimeUnixNano)
				}
				if log.ObservedTimeUnixNano > 0 {
					record.U64Field(constants.OBSERVED_TIME_UNIX_NANO, log.ObservedTimeUnixNano)
				}
				common.AddResource(record, resourceLogs.Resource)
				AddScopeLogs(record, scopeLogs)

				record.I32Field(constants.SEVERITY_NUMBER, int32(log.SeverityNumber))
				record.StringField(constants.SEVERITY_TEXT, log.SeverityText)
				body := common.OtlpAnyValueToValue(log.Body)
				if body != nil {
					record.GenericField(constants.BODY, body)
				}
				attributes := common.NewAttributes(log.Attributes)
				if attributes != nil {
					record.AddField(*attributes)
				}

				if log.DroppedAttributesCount > 0 {
					record.U32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(log.DroppedAttributesCount))
				}
				if log.Flags > 0 {
					record.U32Field(constants.FLAGS, uint32(log.Flags))
				}
				if log.TraceId != nil && len(log.TraceId) > 0 {
					record.BinaryField(constants.TRACE_ID, log.TraceId)
				}
				if log.SpanId != nil && len(log.SpanId) > 0 {
					record.BinaryField(constants.SPAN_ID, log.SpanId)
				}

				rbr.AddRecord(record)
			}
		}
	}

	logsRecords, err := rbr.Build()
	if err != nil {
		return nil, err
	}

	result := make([]arrow.Record, 0, len(logsRecords))
	for _, record := range logsRecords {
		result = append(result, record)
	}

	return result, nil
}

func AddScopeLogs(record *rbb.Record, scopeLogs *logspb.ScopeLogs) {
	record.StructField(constants.SCOPE_LOGS, field_value.Struct{
		Fields: []field_value.Field{
			field_value.MakeStringField(constants.NAME, scopeLogs.Scope.Name),
			field_value.MakeStringField(constants.VERSION, scopeLogs.Scope.Version),
		},
	})
}
