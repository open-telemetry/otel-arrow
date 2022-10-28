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

package arrow

import (
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/logs"

	"go.opentelemetry.io/collector/pdata/plog"
)

// A TopLevelWrapper wraps a [plog.Logs] to expose methods of the [common.TopLevelEntities] interface.
type TopLevelWrapper struct {
	logs.Constants
	logs plog.Logs
}

// A ResourceLogsSliceWrapper wraps a [plog.ResourceLogsSlice] to expose methods of the [common.TopLevelEntitiesSlice]
// interface.
type ResourceLogsSliceWrapper struct {
	rls plog.ResourceLogsSlice
}

// A ResourceLogsWrapper wraps a [plog.ResourceLogs] to expose methods of the [common.ResourceEntities] interface.
type ResourceLogsWrapper struct {
	rl plog.ResourceLogs
}

// Wrap wraps a [plog.Logs] to expose methods of the [common.TopLevelEntities] interface.
func Wrap(logs plog.Logs) TopLevelWrapper {
	return TopLevelWrapper{logs: logs}
}

// ResourceSlice returns a [plog.ResourceLogsSlice].
func (t TopLevelWrapper) ResourceSlice() arrow.TopLevelEntitiesSlice[plog.ScopeLogs] {
	return ResourceLogsSliceWrapper{rls: t.logs.ResourceLogs()}
}

// EntityGrouper converts [plog.LogRecord]s of a [plog.ScopeLogs] into their AIR representation and groups them based
// on a given configuration. A log signature is based on the log attributes when the attribute encoding configuration is
// AttributesAsStructs, otherwise it is an empty string.
func (t TopLevelWrapper) EntityGrouper(scopeLogs plog.ScopeLogs, cfg *config.Config) map[string][]rfield.Value {
	logRecordList := scopeLogs.LogRecords()
	logsPerSig := make(map[string][]rfield.Value, logRecordList.Len())

	for k := 0; k < logRecordList.Len(); k++ {
		var logSig strings.Builder
		log := logRecordList.At(k)

		fields := make([]*rfield.Field, 0, 10)
		if ts := log.Timestamp(); ts > 0 {
			fields = append(fields, rfield.NewU64Field(constants.TIME_UNIX_NANO, uint64(ts)))
		}
		if ts := log.ObservedTimestamp(); ts > 0 {
			fields = append(fields, rfield.NewU64Field(constants.OBSERVED_TIME_UNIX_NANO, uint64(ts)))
		}
		if tid := log.TraceID(); !tid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.TRACE_ID, tid[:]))
		}
		if sid := log.SpanID(); !sid.IsEmpty() {
			fields = append(fields, rfield.NewBinaryField(constants.SPAN_ID, sid[:]))
		}
		fields = append(fields, rfield.NewI32Field(constants.SEVERITY_NUMBER, int32(log.SeverityNumber())))
		if log.SeverityText() != "" {
			fields = append(fields, rfield.NewStringField(constants.SEVERITY_TEXT, log.SeverityText()))
		}
		body := common.OtlpAnyValueToValue(log.Body())
		if body != nil {
			fields = append(fields, rfield.NewField(constants.BODY, body))
		}

		attributes := common.NewAttributes(log.Attributes(), cfg)
		if attributes != nil {
			fields = append(fields, attributes)
			if cfg.Attribute.Encoding == config.AttributesAsStructs {
				attributes.Normalize()
				attributes.WriteSigType(&logSig)
			}
		}

		if dc := log.DroppedAttributesCount(); dc > 0 {
			fields = append(fields, rfield.NewU32Field(constants.DROPPED_ATTRIBUTES_COUNT, uint32(dc)))
		}

		if log.Flags() > 0 {
			fields = append(fields, rfield.NewU32Field(constants.FLAGS, uint32(log.Flags())))
		}

		logValue := rfield.NewStruct(fields)

		lsig := logSig.String()
		logsPerSig[lsig] = append(logsPerSig[lsig], logValue)
	}
	return logsPerSig
}

// Len returns the number of elements in the slice.
func (t ResourceLogsSliceWrapper) Len() int {
	return t.rls.Len()
}

// At returns the element at the given index.
func (t ResourceLogsSliceWrapper) At(i int) arrow.ResourceEntities[plog.ScopeLogs] {
	return ResourceLogsWrapper{rl: t.rls.At(i)}
}

// Resource returns the resource associated with the resource logs.
func (t ResourceLogsWrapper) Resource() pcommon.Resource {
	return t.rl.Resource()
}

// SchemaUrl returns the schema URL associated with the resource logs.
func (t ResourceLogsWrapper) SchemaUrl() string {
	return t.rl.SchemaUrl()
}

// ScopeEntities returns the scope logs associated with the resource logs.
func (t ResourceLogsWrapper) ScopeEntities() arrow.ScopeEntitiesSlice[plog.ScopeLogs] {
	return t.rl.ScopeLogs()
}
