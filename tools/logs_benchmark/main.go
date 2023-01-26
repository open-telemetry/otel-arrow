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

package main

import (
	"encoding/csv"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/dustin/go-humanize"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/dataset"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/profileable/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/benchmark/profileable/otlp"
)

var help = flag.Bool("help", false, "Show help")

// Command line to generate a benchmark report on a log dataset. The benchmark consists in comparing the performance of
// the standard OTLP representation and the OTLP Arrow representation for the same dataset.
//
// The dataset can be generated from a CSV file (ext .csv) or from a OTLP protobuf file (ext .pb).

func main() {
	// Parse the flag
	flag.Parse()

	// Usage Demo
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	// Define default input file
	inputFiles := flag.Args()
	if len(inputFiles) == 0 {
		println("\nNo input file specified, using default file ./data/otlp_logs.pb")
		println("CSV and OTLP protobuf files are supported as input files (ext .csv or .pb)")
		inputFiles = append(inputFiles, "./data/otlp_logs.pb")
	}

	// Compare the performance for each input file
	for i := range inputFiles {
		var ds dataset.LogsDataset

		inputFile := inputFiles[i]
		compressionAlgo := benchmark.Zstd()
		maxIter := uint64(3)

		// Compare the performance between the standard OTLP representation and the OTLP Arrow representation.
		profiler := benchmark.NewProfiler([]int{ /*10, 100,*/ 1000, 2000, 5000, 10000}, "output/logs_benchmark.log", 2)

		// Build dataset from CSV file or from OTLP protobuf file
		if strings.HasSuffix(inputFile, ".csv") {
			ds = CsvToLogsDataset(inputFile)
		} else if strings.HasSuffix(inputFile, ".pb") {
			ds = dataset.NewRealLogsDataset(inputFiles[i])
		} else {
			log.Fatal("Unsupported input file format (only .csv and .pb are supported)")
		}
		profiler.Printf("Dataset '%s' (%s) loaded\n", inputFiles[i], humanize.Bytes(uint64(ds.SizeInBytes())))

		otlpLogs := otlp.NewLogsProfileable(ds, compressionAlgo)
		otlpArrowLogs := arrow.NewLogsProfileable([]string{}, ds, &benchmark.Config{})

		if err := profiler.Profile(otlpLogs, maxIter); err != nil {
			panic(fmt.Errorf("expected no error, got %v", err))
		}
		if err := profiler.Profile(otlpArrowLogs, maxIter); err != nil {
			panic(fmt.Errorf("expected no error, got %v", err))
		}

		profiler.CheckProcessingResults()

		// Configure the profile output
		benchmark.OtlpArrowConversionSection.CustomColumnFor(otlpLogs).
			MetricNotApplicable()

		profiler.Printf("\nLogs dataset summary:\n")
		profiler.Printf("- #logs: %d\n", ds.Len())

		profiler.PrintResults(maxIter)

		profiler.ExportMetricsTimesCSV(fmt.Sprintf("%d_logs_benchmark_results", i))
		profiler.ExportMetricsBytesCSV(fmt.Sprintf("%d_logs_benchmark_results", i))
	}
}

type AttrIndex struct {
	name  string
	index int
}

type CsvRowSchema struct {
	// List of source attributes (order by name)
	sourceAttrs []AttrIndex
	// The text message/body of the log record (for unstructured logs)
	body int
	// List of log attributes
	logAttrs []AttrIndex
}

type LogsPerSource struct {
	sourceAttrs *pcommon.Map
	logs        []*plog.LogRecord
}

type CsvLogsDataset struct {
	logs        []plog.Logs
	sizeInBytes int
}

func (ds *CsvLogsDataset) Len() int {
	return len(ds.logs)
}

func (ds *CsvLogsDataset) Logs(start, size int) []plog.Logs {
	return ds.logs[start : start+size]
}

// CsvToLogsDataset converts a CSV file to a log dataset directly usable by this benchmark utility.
func CsvToLogsDataset(file string) dataset.LogsDataset {
	// Open the CSV file
	csvFile, err := os.Open(file)
	if err != nil {
		log.Fatal(err)
	}
	defer func() {
		err = csvFile.Close()
		if err != nil {
			log.Fatal(err)
		}
	}()

	// Create CsvReader to read the CSV file line by line
	csvReader := csv.NewReader(csvFile)

	// Read first line to get the column names
	firstLine, err := csvReader.Read()
	if err == io.EOF {
		log.Fatal("Empty CSV file")
	}
	if err != nil {
		log.Fatal(err)
	}

	csvSchema := ReadCsvRowSchema(firstLine)
	sources := make(map[string]*LogsPerSource)

	// Read the rest of the file
	for {
		rec, err := csvReader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Fatal(err)
		}

		sourceID, sourceAttrs, logRecord := ReadCsvRow(csvSchema, rec)
		if _, ok := sources[sourceID]; !ok {
			lps := LogsPerSource{
				sourceAttrs: sourceAttrs,
				logs:        make([]*plog.LogRecord, 0, 100),
			}
			sources[sourceID] = &lps
		}
		sources[sourceID].logs = append(sources[sourceID].logs, logRecord)
	}

	logs := plog.NewLogs()
	rls := logs.ResourceLogs()
	rls.EnsureCapacity(len(sources))

	for _, source := range sources {
		rl := rls.AppendEmpty()
		source.sourceAttrs.CopyTo(rl.Resource().Attributes())
		sls := rl.ScopeLogs()
		sl := sls.AppendEmpty()
		lrs := sl.LogRecords()
		lrs.EnsureCapacity(len(source.logs))
		for _, l := range source.logs {
			l.CopyTo(lrs.AppendEmpty())
		}
	}

	ds := CsvLogsDataset{
		logs:        []plog.Logs{logs},
		sizeInBytes: 0,
	}

	return &ds
}

func (d *CsvLogsDataset) SizeInBytes() int {
	return d.sizeInBytes
}

// ReadCsvRowSchema returns the schema of a CSV file based on the first line of the file.
func ReadCsvRowSchema(colNames []string) CsvRowSchema {
	if len(colNames) < 2 {
		log.Fatal("Invalid CSV file format: ts and level are mandatory columns")
	}
	if colNames[0] != "ts" {
		log.Fatal("Invalid CSV file format: first column must be named 'ts'")
	}
	if colNames[1] != "level" {
		log.Fatal("Invalid CSV file format: second column must be named 'level'")
	}

	idx := 2
	csvSchema := CsvRowSchema{
		sourceAttrs: make([]AttrIndex, 0, len(colNames)),
		body:        -1,
		logAttrs:    make([]AttrIndex, 0, len(colNames)),
	}

	for idx < len(colNames) {
		colName := colNames[idx]
		if strings.HasPrefix(colName, "source-") {
			csvSchema.sourceAttrs = append(csvSchema.sourceAttrs, AttrIndex{colName[7:], idx})
			idx++
		} else if colName == "body" {
			csvSchema.body = idx
			idx++
		} else if strings.HasPrefix(colName, "log-") {
			csvSchema.logAttrs = append(csvSchema.logAttrs, AttrIndex{colName[4:], idx})
			idx++
		} else {
			log.Fatalf("Invalid CSV file format: invalid column name %q", colName)
		}
	}
	// Sort by name to build canonical representation of source id.
	sort.Slice(csvSchema.sourceAttrs, func(i, j int) bool {
		return csvSchema.sourceAttrs[i].name < csvSchema.sourceAttrs[j].name
	})

	return csvSchema
}

// ReadCsvRow returns the source id, source attributes and log record for a CSV row.
func ReadCsvRow(csvSchema CsvRowSchema, row []string) (string, *pcommon.Map, *plog.LogRecord) {
	logRecord := plog.NewLogRecord()

	// Read timestamp
	ts, err := strconv.ParseInt(row[0], 10, 64)
	if err != nil {
		log.Fatal(err)
	}

	// Read level
	level := row[1]

	// Read source attributes
	// sourceID is the concatenation of all source attributes in alphabetical order.
	var sourceID strings.Builder

	sourceAttrs := pcommon.NewMap()

	for _, attr := range csvSchema.sourceAttrs {
		attrName := attr.name
		attrValue := row[attr.index]

		sourceID.WriteString(attrName)
		sourceID.WriteString("=")
		sourceID.WriteString(attrValue)
		sourceID.WriteString(",")

		PutAttr(&sourceAttrs, attrName, attrValue)
	}

	// Read body
	if csvSchema.body >= 0 {
		logRecord.Body().SetStr(row[csvSchema.body])
	}

	// Read log attributes
	bodyAttrs := logRecord.Attributes()

	for _, attr := range csvSchema.logAttrs {
		attrName := attr.name
		attrValue := row[attr.index]

		PutAttr(&bodyAttrs, attrName, attrValue)
	}

	logRecord.SetTimestamp(pcommon.NewTimestampFromTime(time.Unix(0, ts)))
	logRecord.SetSeverityNumber(ToSeverityNumber(level))
	logRecord.SetSeverityText(level)

	return sourceID.String(), &sourceAttrs, &logRecord
}

// ToSeverityNumber converts a string to a severity number.
func ToSeverityNumber(level string) plog.SeverityNumber {
	switch level {
	case "trace":
		return plog.SeverityNumberTrace
	case "debug":
		return plog.SeverityNumberDebug
	case "info":
		return plog.SeverityNumberInfo
	case "warn":
		return plog.SeverityNumberWarn
	case "error":
		return plog.SeverityNumberError
	case "fatal":
		return plog.SeverityNumberFatal
	default:
		return plog.SeverityNumberUnspecified
	}
}

// PutAttr puts an attribute in a map.
// Conversion rules:
// If the attribute is a number, it is converted to a int64 or float64 attribute.
// If the attribute is a boolean, it is converted to a bool attribute.
// If the attribute is a string, it is converted to a string attribute.
func PutAttr(attrs *pcommon.Map, key string, value string) {
	if s, err := strconv.ParseInt(value, 10, 64); err == nil {
		attrs.PutInt(key, s)
	} else if s, err := strconv.ParseFloat(value, 64); err == nil {
		attrs.PutDouble(key, s)
	} else if s, err := strconv.ParseBool(value); err == nil {
		attrs.PutBool(key, s)
	} else {
		attrs.PutStr(key, value)
	}
}
