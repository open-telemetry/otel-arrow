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

package otlp

import (
	"fmt"

	"github.com/f5/otel-arrow-adapter/pkg/air"
	common_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/otel/logs"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
)

// TopLevelWrapper is a wrapper around [plog.Logs] that implements the [otlp.TopLevelEntities] interface.
type TopLevelWrapper struct {
	logs plog.Logs
}

// ResourceEntities returns a wrapper around the [plog.ResourceLogsSlice].
func (w TopLevelWrapper) ResourceEntities() otlp.ResourceEntitiesSlice[plog.LogRecord] {
	return ResourceLogsSliceWrapper{slice: w.logs.ResourceLogs()}
}

// Unwrap returns the underlying [plog.Logs].
func (w TopLevelWrapper) Unwrap() plog.Logs {
	return w.logs
}

// ResourceLogsSliceWrapper is a wrapper around [plog.ResourceLogsSlice] that implements the
// [otlp.ResourceEntitiesSlice] interface.
type ResourceLogsSliceWrapper struct {
	slice plog.ResourceLogsSlice
}

// EnsureCapacity ensures that the underlying [plog.ResourceLogsSlice] has enough capacity to hold the given number of
// [plog.ResourceLogs].
func (s ResourceLogsSliceWrapper) EnsureCapacity(newCap int) {
	s.slice.EnsureCapacity(newCap)
}

// AppendEmpty creates a new [plog.ResourceLogs] and appends it to the underlying [plog.ResourceLogsSlice].
func (s ResourceLogsSliceWrapper) AppendEmpty() otlp.ResourceEntities[plog.LogRecord] {
	return ResourceLogsWrapper{resourceLogs: s.slice.AppendEmpty()}
}

// ResourceLogsWrapper is a wrapper around [plog.ResourceLogs] that implements the [otlp.ResourceEntities] interface.
type ResourceLogsWrapper struct {
	resourceLogs plog.ResourceLogs
}

// Resource returns the [pcommon.Resource] of the underlying [plog.ResourceLogs].
func (w ResourceLogsWrapper) Resource() pcommon.Resource {
	return w.resourceLogs.Resource()
}

// SetSchemaUrl sets the schema URL of the underlying [plog.ResourceLogs].
func (w ResourceLogsWrapper) SetSchemaUrl(schemaUrl string) {
	w.resourceLogs.SetSchemaUrl(schemaUrl)
}

// ScopeEntities returns a wrapper around [plog.ScopeLogs] that implements the [otlp.ScopeEntities] interface.
func (w ResourceLogsWrapper) ScopeEntities() otlp.ScopeEntities[plog.LogRecord] {
	return ScopeLogsWrapper{scopeLogs: w.resourceLogs.ScopeLogs().AppendEmpty()}
}

// ScopeLogsWrapper is a wrapper around [plog.ScopeLogs] that implements the [otlp.ScopeEntities] interface.
type ScopeLogsWrapper struct {
	scopeLogs plog.ScopeLogs
}

// Scope returns the [plog.Scope] of the underlying [plog.ScopeLogs].
func (w ScopeLogsWrapper) Scope() pcommon.InstrumentationScope {
	return w.scopeLogs.Scope()
}

// SetSchemaUrl sets the schema URL of the underlying [plog.ScopeLogs].
func (w ScopeLogsWrapper) SetSchemaUrl(schemaUrl string) {
	w.scopeLogs.SetSchemaUrl(schemaUrl)
}

// Entity returns a [plog.LogRecord] and add it to the underlying [plog.ScopeLogs].
func (w ScopeLogsWrapper) Entity() plog.LogRecord {
	return w.scopeLogs.LogRecords().AppendEmpty()
}

// LogsProducer is a [otlp.EntityProducer] that produces [plog.Logs].
type LogsProducer struct {
	logs.Constants
}

// NewTopLevelEntities creates a new [plog.Logs] and returns a wrapper around it that implements the
// [otlp.TopLevelEntities] interface.
func (p LogsProducer) NewTopLevelEntities() otlp.TopLevelEntities[plog.Logs, plog.LogRecord] {
	return TopLevelWrapper{plog.NewLogs()}
}

// EntityProducer creates [plog.LogRecord]s from their AIR representation.
func (p LogsProducer) EntityProducer(scopeLog otlp.ScopeEntities[plog.LogRecord], los *air.ListOfStructs, row int) error {
	log := scopeLog.Entity()
	timeUnixNano, err := los.U64FieldByName(constants.TIME_UNIX_NANO, row)
	if err != nil {
		return err
	}
	observedTimeUnixNano, err := los.U64FieldByName(constants.OBSERVED_TIME_UNIX_NANO, row)
	if err != nil {
		return err
	}
	traceId, err := los.BinaryFieldByName(constants.TRACE_ID, row)
	if err != nil {
		return err
	}
	if len(traceId) != 16 {
		return fmt.Errorf("trace_id field should be 16 bytes")
	}
	spanId, err := los.BinaryFieldByName(constants.SPAN_ID, row)
	if err != nil {
		return err
	}
	severityNumber, err := los.I32FieldByName(constants.SEVERITY_NUMBER, row)
	if err != nil {
		return err
	}
	severityText, err := los.StringFieldByName(constants.SEVERITY_TEXT, row)
	if err != nil {
		return err
	}
	bodyArray, ok := los.Field(constants.BODY)
	if ok {
		if err := common_arrow.CopyValueFrom(log.Body(), bodyArray.DataType(), bodyArray, row); err != nil {
			return err
		}
	}
	droppedAttributesCount, err := los.U32FieldByName(constants.DROPPED_ATTRIBUTES_COUNT, row)
	if err != nil {
		return err
	}
	attrs, err := los.ListOfStructsByName(constants.ATTRIBUTES, row)
	if err != nil {
		return err
	}
	if attrs != nil {
		err = attrs.CopyAttributesFrom(log.Attributes())
	}
	flags, err := los.U32FieldByName(constants.FLAGS, row)
	if err != nil {
		return err
	}

	var tid pcommon.TraceID
	var sid pcommon.SpanID
	copy(tid[:], traceId)
	copy(sid[:], spanId)

	log.SetTimestamp(pcommon.Timestamp(timeUnixNano))
	log.SetObservedTimestamp(pcommon.Timestamp(observedTimeUnixNano))
	log.SetTraceID(tid)
	log.SetSpanID(sid)
	log.SetSeverityNumber(plog.SeverityNumber(severityNumber))
	log.SetSeverityText(severityText)
	log.SetFlags(plog.LogRecordFlags(flags))
	log.SetDroppedAttributesCount(droppedAttributesCount)

	return nil
}
