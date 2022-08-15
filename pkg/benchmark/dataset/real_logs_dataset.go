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

package dataset

import (
	"log"
	"os"

	"google.golang.org/protobuf/proto"

	collogs "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
	logspb "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/logs/v1"
)

// RealLogsDataset represents a dataset of real logs read from an ExportLogsServiceRequest serialized to a binary file.
type RealLogsDataset struct {
	logs []logUnit
}

type logUnit struct {
	logRecord *logspb.LogRecord
	resource  *logspb.ResourceLogs
	scope     *logspb.ScopeLogs
}

// NewRealLogsDataset creates a new RealLogsDataset from a binary file.
func NewRealLogsDataset(path string) *RealLogsDataset {
	data, err := os.ReadFile(path)
	if err != nil {
		log.Fatal("read file:", err)
	}
	var otlp collogs.ExportLogsServiceRequest
	if err := proto.Unmarshal(data, &otlp); err != nil {
		log.Fatal("unmarshal:", err)
	}

	ds := &RealLogsDataset{logs: []logUnit{}}

	for _, rl := range otlp.ResourceLogs {
		for _, sl := range rl.ScopeLogs {
			for _, lr := range sl.LogRecords {
				ds.logs = append(ds.logs, logUnit{logRecord: lr, resource: rl, scope: sl})
			}
		}
	}

	return ds
}

// Len returns the number of log records in the dataset.
func (d *RealLogsDataset) Len() int {
	return len(d.logs)
}

// Logs returns a subset of log records from the original dataset.
func (d *RealLogsDataset) Logs(offset, size int) []*collogs.ExportLogsServiceRequest {
	resourceLogs := map[*logspb.ResourceLogs]map[*logspb.ScopeLogs][]*logspb.LogRecord{}

	for _, logRecord := range d.logs[offset : offset+size] {
		if rl, ok := resourceLogs[logRecord.resource]; !ok {
			resourceLogs[logRecord.resource] = map[*logspb.ScopeLogs][]*logspb.LogRecord{}
		} else if _, ok := rl[logRecord.scope]; !ok {
			rl[logRecord.scope] = []*logspb.LogRecord{}
		}

		logs := resourceLogs[logRecord.resource][logRecord.scope]
		logs = append(logs, logRecord.logRecord)
	}

	request := collogs.ExportLogsServiceRequest{
		ResourceLogs: make([]*logspb.ResourceLogs, 0, len(resourceLogs)),
	}

	for rl, sl := range resourceLogs {

		scopeLogs := make([]*logspb.ScopeLogs, 0, len(sl))
		for sl, lrs := range sl {
			scopeLogs = append(scopeLogs, &logspb.ScopeLogs{
				Scope:      sl.Scope,
				LogRecords: lrs,
				SchemaUrl:  sl.SchemaUrl,
			})
		}

		request.ResourceLogs = append(request.ResourceLogs, &logspb.ResourceLogs{
			Resource:  rl.Resource,
			ScopeLogs: scopeLogs,
			SchemaUrl: rl.SchemaUrl,
		})
	}

	return []*collogs.ExportLogsServiceRequest{&request}
}
