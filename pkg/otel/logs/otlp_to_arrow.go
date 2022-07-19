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

package logs

import (
	"github.com/apache/arrow/go/v9/arrow"
	collogspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
)

// OtlpLogsToArrowRecords converts an OTLP ResourceLogs to one or more Arrow records
func OtlpLogsToArrowRecords(rr *air.RecordRepository, request *collogspb.ExportLogsServiceRequest) ([]arrow.Record, error) {
	for _, resourceLogs := range request.ResourceLogs {
		for _, scopeLogs := range resourceLogs.ScopeLogs {
			for _, log := range scopeLogs.LogRecords {
				record := air.NewRecord()

				if log.TimeUnixNano > 0 {
					record.U64Field(constants.TIME_UNIX_NANO, log.TimeUnixNano)
				}
				if log.ObservedTimeUnixNano > 0 {
					record.U64Field(constants.OBSERVED_TIME_UNIX_NANO, log.ObservedTimeUnixNano)
				}
				common.AddResource(record, resourceLogs.Resource)
				common.AddScope(record, constants.SCOPE_LOGS, scopeLogs.Scope)

				record.I32Field(constants.SEVERITY_NUMBER, int32(log.SeverityNumber))
				record.StringField(constants.SEVERITY_TEXT, log.SeverityText)
				body := common.OtlpAnyValueToValue(log.Body)
				if body != nil {
					record.GenericField(constants.BODY, body)
				}
				attributes := common.NewAttributes(log.Attributes)
				if attributes != nil {
					record.AddField(attributes)
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

				rr.AddRecord(record)
			}
		}
	}

	logsRecords, err := rr.Build()
	if err != nil {
		return nil, err
	}

	result := make([]arrow.Record, 0, len(logsRecords))
	for _, record := range logsRecords {
		result = append(result, record)
	}

	return result, nil
}
