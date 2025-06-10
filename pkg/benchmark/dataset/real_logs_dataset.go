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

package dataset

import (
	"bufio"
	"errors"
	"io"
	"log"
	"os"
	"path/filepath"

	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/klauspost/compress/zstd"

	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/stats"
)

// RealLogsDataset represents a dataset of real logs read from a Logs serialized to a binary file.
type RealLogsDataset struct {
	logs        []logUnit
	sizeInBytes int
	logsStats   *stats.LogsStats
}

type logUnit struct {
	logRecord plog.LogRecord
	resource  plog.ResourceLogs
	scope     plog.ScopeLogs
}

type logReader struct {
	stringReader *bufio.Reader
	unmarshaler  *plog.JSONUnmarshaler
	bytesRead    int
}

func (lr *logReader) readAllLogs() (plog.Logs, error) {
	logs := plog.NewLogs()

	for {
		if line, err := lr.stringReader.ReadString('\n'); err == nil {
			ll, err := lr.unmarshaler.UnmarshalLogs([]byte(line))
			if err != nil {
				return logs, err
			}
			for i := 0; i < ll.ResourceLogs().Len(); i++ {
				rl := logs.ResourceLogs().AppendEmpty()
				ll.ResourceLogs().At(i).CopyTo(rl)
			}
			lr.bytesRead += len(line)
		} else { // failed to read line
			if err != nil {
				if errors.Is(err, io.EOF) {
					return logs, nil
				}
				return logs, err
			}
		}
	}
}

func logsFromJSON(path string, compression string) (plog.Logs, int) {
	file, err := os.Open(filepath.Clean(path))
	if err != nil {
		log.Fatal("open file:", err)
	}

	lr := &logReader{
		unmarshaler: &plog.JSONUnmarshaler{},
		bytesRead:   0,
	}

	if compression == benchmark.CompressionTypeZstd {
		cr, err := zstd.NewReader(file)
		if err != nil {
			log.Fatal("Failed to create compressed reader: ", err)
		}
		lr.stringReader = bufio.NewReader(cr)
	} else { // no compression
		lr.stringReader = bufio.NewReader(file)
	}

	logs, err := lr.readAllLogs()
	if err != nil {
		if lr.bytesRead == 0 {
			log.Fatal("Read zero bytes from file: ", err)
		}
		log.Print("Found error when reading file: ", err)
		log.Print("Bytes read: ", lr.bytesRead)
	}

	return logs, lr.bytesRead
}

func logsFromProto(path string) (plog.Logs, int) {
	data, err := os.ReadFile(filepath.Clean(path))
	if err != nil {
		log.Fatal("read file:", err)
	}
	otlp := plogotlp.NewExportRequest()

	if err := otlp.UnmarshalProto(data); err != nil {
		log.Fatal("unmarshal:", err)
	}
	logs := otlp.Logs()

	return logs, len(data)
}

// NewRealLogsDataset creates a new RealLogsDataset from a binary file
// which is either formatted as otlp protobuf or compressed otlp json.
func NewRealLogsDataset(path string, compression string, format string) *RealLogsDataset {
	var logs plog.Logs
	var size int

	if format == "json" {
		logs, size = logsFromJSON(path, compression)
	} else {
		logs, size = logsFromProto(path)
	}

	ds := &RealLogsDataset{
		logs:        []logUnit{},
		sizeInBytes: size,
		logsStats:   stats.NewLogsStats(),
	}
	ds.logsStats.Analyze(logs)

	for ri := 0; ri < logs.ResourceLogs().Len(); ri++ {
		rl := logs.ResourceLogs().At(ri)
		for si := 0; si < rl.ScopeLogs().Len(); si++ {
			sl := rl.ScopeLogs().At(si)
			for li := 0; li < sl.LogRecords().Len(); li++ {
				lr := sl.LogRecords().At(li)
				ds.logs = append(ds.logs, logUnit{logRecord: lr, resource: rl, scope: sl})
			}
		}
	}

	return ds
}

func (d *RealLogsDataset) SizeInBytes() int {
	return d.sizeInBytes
}

// Len returns the number of log records in the dataset.
func (d *RealLogsDataset) Len() int {
	return len(d.logs)
}

func (d *RealLogsDataset) ShowStats() {
	println()
	println("Logs stats:")
	d.logsStats.ShowStats()
}

// Resize resizes the dataset to the specified size.
func (d *RealLogsDataset) Resize(size int) {
	d.logs = d.logs[:size]
}

// Logs returns a subset of log records from the original dataset.
func (d *RealLogsDataset) Logs(offset, size int) []plog.Logs {
	resourceLogs := map[plog.ResourceLogs]map[plog.ScopeLogs][]plog.LogRecord{}

	for _, logRecord := range d.logs[offset : offset+size] {
		if _, ok := resourceLogs[logRecord.resource]; !ok {
			resourceLogs[logRecord.resource] = map[plog.ScopeLogs][]plog.LogRecord{}
		}

		resourceLogs[logRecord.resource][logRecord.scope] =
			append(resourceLogs[logRecord.resource][logRecord.scope], logRecord.logRecord)
	}

	request := plog.NewLogs()

	for rl, sl := range resourceLogs {
		outRl := request.ResourceLogs().AppendEmpty()
		rl.Resource().CopyTo(outRl.Resource())

		for sl, lrs := range sl {
			outSl := outRl.ScopeLogs().AppendEmpty()
			sl.Scope().CopyTo(outSl.Scope())

			for _, lr := range lrs {
				outLr := outSl.LogRecords().AppendEmpty()
				lr.CopyTo(outLr)
			}
		}
	}

	return []plog.Logs{request}
}
