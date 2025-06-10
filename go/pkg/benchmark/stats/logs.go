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

package stats

import (
	"fmt"

	"go.opentelemetry.io/collector/pdata/plog"
)

type LogsStats struct {
	resourceLogs       *Metric
	resourceAttributes *Metric
	scopeLogs          *Metric
	scopeAttributes    *Metric
	logs               *Metric
	logAttributes      *Metric
}

func NewLogsStats() *LogsStats {
	return &LogsStats{
		resourceLogs:       NewMetric(),
		resourceAttributes: NewMetric(),
		scopeLogs:          NewMetric(),
		scopeAttributes:    NewMetric(),
		logs:               NewMetric(),
		logAttributes:      NewMetric(),
	}
}

func (ls *LogsStats) Analyze(logs plog.Logs) {
	resLogsSlice := logs.ResourceLogs()
	ls.resourceLogs.Record(float64(resLogsSlice.Len()))

	for i := 0; i < resLogsSlice.Len(); i++ {
		resLogs := resLogsSlice.At(i)
		scopeLogsSlice := resLogs.ScopeLogs()

		ls.scopeLogs.Record(float64(scopeLogsSlice.Len()))
		ls.resourceAttributes.Record(float64(resLogs.Resource().Attributes().Len()))

		for j := 0; j < scopeLogsSlice.Len(); j++ {
			scopeLogs := scopeLogsSlice.At(j)
			logRecordsSlice := scopeLogs.LogRecords()

			ls.logs.Record(float64(logRecordsSlice.Len()))
			ls.scopeAttributes.Record(float64(scopeLogs.Scope().Attributes().Len()))

			for k := 0; k < logRecordsSlice.Len(); k++ {
				logRecord := logRecordsSlice.At(k)
				attrs := logRecord.Attributes()

				ls.logAttributes.Record(float64(attrs.Len()))
			}
		}
	}
}

func (ls *LogsStats) ShowStats() {
	fmt.Printf("\t- ResourceLogs           => %s\n", ls.resourceLogs.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Resource    => %s\n", ls.resourceAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- ScopeLogs/ResourceLogs => %s\n", ls.scopeLogs.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/Scope       => %s\n", ls.scopeAttributes.ComputeSummary().ToString())
	fmt.Printf("\t- LogRecord/ScopeLogs    => %s\n", ls.logs.ComputeSummary().ToString())
	fmt.Printf("\t- Attributes/LogRecord   => %s\n", ls.logAttributes.ComputeSummary().ToString())
}
