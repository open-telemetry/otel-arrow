/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

import (
	"bytes"
	"sort"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
)

type (
	LogsOptimizer struct {
		sorter LogSorter
	}

	LogsOptimized struct {
		Logs []*FlattenedLog
	}

	FlattenedLog struct {
		// Resource log section.
		ResourceLogsID    string
		Resource          *pcommon.Resource
		ResourceSchemaUrl string

		// Scope log section.
		ScopeLogsID    string
		Scope          *pcommon.InstrumentationScope
		ScopeSchemaUrl string

		// Log section.
		Log *plog.LogRecord
	}

	LogSorter interface {
		Sort(logs []*FlattenedLog)
	}

	LogsByNothing                          struct{}
	LogsByResourceLogsIDScopeLogsIDTraceID struct{}
)

func NewLogsOptimizer(sorter LogSorter) *LogsOptimizer {
	return &LogsOptimizer{
		sorter: sorter,
	}
}

func (t *LogsOptimizer) Optimize(logs plog.Logs) *LogsOptimized {
	logsOptimized := &LogsOptimized{
		Logs: make([]*FlattenedLog, 0),
	}

	resLogsSlice := logs.ResourceLogs()
	for i := 0; i < resLogsSlice.Len(); i++ {
		resLogs := resLogsSlice.At(i)
		resource := resLogs.Resource()
		resourceSchemaUrl := resLogs.SchemaUrl()
		resSpanID := otlp.ResourceID(resource, resourceSchemaUrl)

		scopeLogs := resLogs.ScopeLogs()
		for j := 0; j < scopeLogs.Len(); j++ {
			scopeSpan := scopeLogs.At(j)
			scope := scopeSpan.Scope()
			scopeSchemaUrl := scopeSpan.SchemaUrl()
			scopeSpanID := otlp.ScopeID(scope, scopeSchemaUrl)

			logs := scopeSpan.LogRecords()
			for k := 0; k < logs.Len(); k++ {
				log := logs.At(k)
				logsOptimized.Logs = append(logsOptimized.Logs, &FlattenedLog{
					ResourceLogsID:    resSpanID,
					Resource:          &resource,
					ResourceSchemaUrl: resourceSchemaUrl,
					ScopeLogsID:       scopeSpanID,
					Scope:             &scope,
					ScopeSchemaUrl:    scopeSchemaUrl,
					Log:               &log,
				})
			}
		}
	}

	t.sorter.Sort(logsOptimized.Logs)

	return logsOptimized
}

// No sorting
// ==========

func UnsortedLogs() *LogsByNothing {
	return &LogsByNothing{}
}

func (s *LogsByNothing) Sort(_ []*FlattenedLog) {
}

// Sort logs by resource logs ID, scope logs ID, and trace ID.
func SortLogsByResourceLogsIDScopeLogsIDTraceID() *LogsByResourceLogsIDScopeLogsIDTraceID {
	return &LogsByResourceLogsIDScopeLogsIDTraceID{}
}

func (s *LogsByResourceLogsIDScopeLogsIDTraceID) Sort(logs []*FlattenedLog) {
	sort.Slice(logs, func(i, j int) bool {
		logI := logs[i]
		logJ := logs[j]

		if logI.ResourceLogsID == logJ.ResourceLogsID {
			if logI.ScopeLogsID == logJ.ScopeLogsID {
				traceIdI := logI.Log.TraceID()
				traceIdJ := logJ.Log.TraceID()
				return bytes.Compare(traceIdI[:], traceIdJ[:]) == -1
			} else {
				return logI.ScopeLogsID < logJ.ScopeLogsID
			}
		} else {
			return logI.ResourceLogsID < logJ.ResourceLogsID
		}
	})
}
