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
	"fmt"

	"github.com/apache/arrow/go/v9/arrow"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/otel/common"
	"otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
)

func ArrowRecordsToOtlpLogs(record arrow.Record) (plog.Logs, error) {
	request := plog.NewLogs()

	resourceLogs := map[string]plog.ResourceLogs{}
	scopeLogs := map[string]plog.ScopeLogs{}

	numRows := int(record.NumRows())
	for i := 0; i < numRows; i++ {
		resource, err := common.NewResourceFrom(record, i)
		if err != nil {
			return request, err
		}
		resId := common.ResourceId(resource)
		rl, ok := resourceLogs[resId]
		if !ok {
			rl = request.ResourceLogs().AppendEmpty()
			resource.CopyTo(rl.Resource())
			// TODO: SchemaURL
			resourceLogs[resId] = rl
		}

		// TODO: Note we could avoid the copy below by using
		// the New/Set methods like for LogRecord.
		scope, err := common.NewInstrumentationScopeFrom(record, i, constants.SCOPE_LOGS)
		if err != nil {
			return request, err
		}
		scopeLogId := resId + "|" + common.ScopeId(scope)
		sl, ok := scopeLogs[scopeLogId]
		if !ok {
			sl = rl.ScopeLogs().AppendEmpty()
			scope.CopyTo(sl.Scope())
			// TODO: SchemaURL
			scopeLogs[scopeLogId] = sl
		}

		logRecord := sl.LogRecords().AppendEmpty()
		err = SetLogRecordFrom(logRecord, record, i)
		if err != nil {
			return request, err
		}
	}

	return request, nil
}

func SetLogRecordFrom(logRecord plog.LogRecord, record arrow.Record, row int) error {
	lr := plog.NewLogRecord()
	if ts, err := air.U64FromRecord(record, row, constants.TIME_UNIX_NANO); err != nil {
		return err
	} else {
		lr.SetTimestamp(pcommon.Timestamp(ts))
	}
	if observedTs, err := air.U64FromRecord(record, row, constants.OBSERVED_TIME_UNIX_NANO); err != nil {
		return err
	} else {
		lr.SetObservedTimestamp(pcommon.Timestamp(observedTs))
	}
	if severityNumber, err := air.I32FromRecord(record, row, constants.SEVERITY_NUMBER); err != nil {
		return err
	} else {
		lr.SetSeverityNumber(plog.SeverityNumber(severityNumber))
	}
	if severityText, err := air.StringFromRecord(record, row, constants.SEVERITY_TEXT); err != nil {
		return err
	} else {
		lr.SetSeverityText(severityText)
	}
	if bodyField, bodyArray := air.FieldArray(record, constants.BODY); bodyArray != nil {
		if err := common.CopyValueFrom(lr.Body(), bodyField.Type, bodyArray, row); err != nil {
			return err
		}
	}
	if attrField, attrColumn := air.FieldArray(record, constants.ATTRIBUTES); attrColumn != nil {
		if err := common.CopyAttributesFrom(lr.Attributes(), attrField.Type, attrColumn, row); err != nil {
			return err
		}
	}
	if dc, err := air.U32FromRecord(record, row, constants.DROPPED_ATTRIBUTES_COUNT); err != nil {
		return err
	} else {
		lr.SetDroppedAttributesCount(uint32(dc))
	}
	if flags, err := air.U32FromRecord(record, row, constants.FLAGS); err != nil {
		return err
	} else {
		lr.SetFlags(plog.LogRecordFlags(flags))
	}
	if traceId, err := air.BinaryFromRecord(record, row, constants.TRACE_ID); err != nil {
		return err
	} else if len(traceId) != 16 {
		return fmt.Errorf("invalid log TraceID len: %d", len(traceId))
	} else {
		var tid pcommon.TraceID
		copy(tid[:], traceId)
		lr.SetTraceID(tid)
	}
	if spanId, err := air.BinaryFromRecord(record, row, constants.SPAN_ID); err != nil {
		return err
	} else if len(spanId) != 8 {
		return fmt.Errorf("invalid log SpanID len: %d", len(spanId))
	} else {
		var sid pcommon.SpanID
		copy(sid[:], spanId)
		lr.SetSpanID(sid)
	}
	return nil
}
