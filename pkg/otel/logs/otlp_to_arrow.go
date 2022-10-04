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

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/plog"
)

// OtlpLogsToArrowRecords converts an OTLP ResourceLogs to one or more Arrow records
func OtlpLogsToArrowRecords(rr *air.RecordRepository, request plog.Logs, cfg *config.Config) ([]arrow.Record, error) {
	for i := 0; i < request.ResourceLogs().Len(); i++ {
		resourceLogs := request.ResourceLogs().At(i)

		for j := 0; j < resourceLogs.ScopeLogs().Len(); j++ {
			scopeLogs := resourceLogs.ScopeLogs().At(j)

			for k := 0; k < scopeLogs.LogRecords().Len(); k++ {
				record := air.NewRecord()
				log := scopeLogs.LogRecords().At(k)

				if ts := log.Timestamp(); ts > 0 {
					record.U64Field(constants.TIME_UNIX_NANO, uint64(ts))
				}
				if ots := log.ObservedTimestamp(); ots > 0 {
					record.U64Field(constants.OBSERVED_TIME_UNIX_NANO, uint64(ots))
				}
				common.AddResource(record, resourceLogs.Resource(), cfg)
				common.AddScope(record, constants.SCOPE_LOGS, scopeLogs.Scope(), cfg)

				record.I32Field(constants.SEVERITY_NUMBER, int32(log.SeverityNumber()))
				if log.SeverityText() != "" {
					record.StringField(constants.SEVERITY_TEXT, log.SeverityText())
				}
				body := common.OtlpAnyValueToValue(log.Body())
				if body != nil {
					record.GenericField(constants.BODY, body)
				}
				attributes := common.NewAttributes(log.Attributes(), cfg)
				if attributes != nil {
					record.AddField(attributes)
				}

				if dc := log.DroppedAttributesCount(); dc > 0 {
					record.U32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc))
				}
				if log.Flags() > 0 {
					record.U32Field(constants.FLAGS, uint32(log.Flags()))
				}
				if tid := log.TraceID(); !tid.IsEmpty() {
					record.BinaryField(constants.TRACE_ID, tid[:])
				}
				if sid := log.SpanID(); !sid.IsEmpty() {
					record.BinaryField(constants.SPAN_ID, sid[:])
				}

				rr.AddRecord(record)
			}
		}
	}

	logsRecords, err := rr.BuildRecords()
	if err != nil {
		return nil, err
	}

	result := make([]arrow.Record, 0, len(logsRecords))
	for _, record := range logsRecords {
		result = append(result, record)
	}

	return result, nil
}
