/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
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
