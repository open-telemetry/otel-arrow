/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package logs

import (
	"github.com/apache/arrow/go/v9/arrow"

	collogs "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	commonpb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/common/v1"
	logspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/logs/v1"
	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"
)

func ArrowRecordsToOtlpLogs(record arrow.Record) (*collogs.ExportLogsServiceRequest, error) {
	request := collogs.ExportLogsServiceRequest{
		ResourceLogs: []*logspb.ResourceLogs{},
	}

	resourceLogs := map[string]*logspb.ResourceLogs{}
	scopeLogs := map[string]*logspb.ScopeLogs{}

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resource, err := common.NewResourceFrom(record, i)
		if err != nil {
			return nil, err
		}
		resId := common.ResourceId(resource)
		if _, ok := resourceLogs[resId]; !ok {
			rs := &logspb.ResourceLogs{
				Resource:  resource,
				ScopeLogs: []*logspb.ScopeLogs{},
				SchemaUrl: "",
			}
			resourceLogs[resId] = rs
		}
		rs := resourceLogs[resId]

		scope, err := common.NewInstrumentationScopeFrom(record, i, constants.SCOPE_LOGS)
		if err != nil {
			return nil, err
		}
		scopeSpanId := resId + "|" + common.ScopeId(scope)
		if _, ok := scopeLogs[scopeSpanId]; !ok {
			ss := &logspb.ScopeLogs{
				Scope:      scope,
				LogRecords: []*logspb.LogRecord{},
				SchemaUrl:  "",
			}
			scopeLogs[scopeSpanId] = ss
			rs.ScopeLogs = append(rs.ScopeLogs, ss)
		}
		ss := scopeLogs[scopeSpanId]

		logRecord, err := NewLogRecordFrom(record, i)
		if err != nil {
			return nil, err
		}
		ss.LogRecords = append(ss.LogRecords, logRecord)
	}

	for _, resLog := range resourceLogs {
		request.ResourceLogs = append(request.ResourceLogs, resLog)
	}

	return &request, nil
}

func NewLogRecordFrom(record arrow.Record, row int) (*logspb.LogRecord, error) {
	timeUnixNano, err := air.U64FromRecord(record, row, constants.TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}
	observedTimeUnixNano, err := air.U64FromRecord(record, row, constants.OBSERVED_TIME_UNIX_NANO)
	if err != nil {
		return nil, err
	}
	severityNumer, err := air.I32FromRecord(record, row, constants.SEVERITY_NUMBER)
	if err != nil {
		return nil, err
	}
	severityText, err := air.StringFromRecord(record, row, constants.SEVERITY_TEXT)
	if err != nil {
		return nil, err
	}
	bodyField, bodyArray := air.FieldArray(record, constants.BODY)
	var body *commonpb.AnyValue
	if bodyArray != nil {
		body, err = common.AnyValueFrom(bodyField.Type, bodyArray, row)
		if err != nil {
			return nil, err
		}
	}
	attrField, attrColumn := air.FieldArray(record, constants.ATTRIBUTES)
	attributes := []*commonpb.KeyValue(nil)
	if attrColumn != nil {
		attributes, err = common.AttributesFrom(attrField.Type, attrColumn, row)
		if err != nil {
			return nil, err
		}
	}
	droppedAttributesCount, err := air.U32FromRecord(record, row, constants.DROPPED_ATTRIBUTES_COUNT)
	if err != nil {
		return nil, err
	}
	flags, err := air.U32FromRecord(record, row, constants.FLAGS)
	if err != nil {
		return nil, err
	}
	traceId, err := air.BinaryFromRecord(record, row, constants.TRACE_ID)
	if err != nil {
		return nil, err
	}
	spanId, err := air.BinaryFromRecord(record, row, constants.SPAN_ID)
	if err != nil {
		return nil, err
	}

	return &logspb.LogRecord{
		TimeUnixNano:           timeUnixNano,
		ObservedTimeUnixNano:   observedTimeUnixNano,
		SeverityNumber:         logspb.SeverityNumber(severityNumer),
		SeverityText:           severityText,
		Body:                   body,
		Attributes:             attributes,
		DroppedAttributesCount: droppedAttributesCount,
		Flags:                  flags,
		TraceId:                traceId,
		SpanId:                 spanId,
	}, nil
}
