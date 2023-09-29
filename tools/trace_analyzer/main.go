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
	"flag"
	"os"

	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"
	"github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
)

var help = flag.Bool("help", false, "Show help")

func main() {
	// Supported flags
	schemaStats := flag.Bool("schema-stats", false, "Display Arrow schema statistics")
	recordStats := flag.Bool("record-stats", false, "Display Arrow record statistics")
	schemaUpdates := flag.Bool("schema-updates", false, "Display Arrow schema updates")
	producerStats := flag.Bool("producer-stats", false, "Display OTel Arrow producer statistics")
	compressionRatio := flag.Bool("compression-ratio", false, "Display compression ratio per record type")
	all := flag.Bool("all", false, "Display all statistics and updates")

	defaultBatchSize := flag.Int("batch-size", 1000, "Batch size")

	// supports "proto" and "json" formats
	format := flag.String("format", "proto", "file format")

	// Parse the flag
	flag.Parse()

	// Usage Demo
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	inputFiles := flag.Args()

	var options []config.Option

	if all != nil && *all {
		schemaStats = all
		recordStats = all
		schemaUpdates = all
		producerStats = all
		compressionRatio = all
	}

	if schemaStats != nil && *schemaStats {
		options = append(options, config.WithSchemaStats())
	}
	if recordStats != nil && *recordStats {
		options = append(options, config.WithRecordStats())
	}
	if schemaUpdates != nil && *schemaUpdates {
		options = append(options, config.WithSchemaUpdates())
	}
	if producerStats != nil && *producerStats {
		options = append(options, config.WithProducerStats())
	}

	var producerWithoutCompression *arrow_record.Producer

	if compressionRatio != nil && *compressionRatio {
		producerWithoutCompression = arrow_record.NewProducerWithOptions(config.WithCompressionRatioStats(), config.WithNoZstd())
		options = append(options, config.WithCompressionRatioStats())
	}
	producerWithCompression := arrow_record.NewProducerWithOptions(append(options, config.WithZstd())...)

	// Analyze all files and report statistics
	for i := range inputFiles {
		ds := dataset.NewRealTraceDataset(inputFiles[i], benchmark.CompressionTypeZstd, *format, []string{"trace_id"})
		ds.Resize(4000)

		startAt := 0

		for startAt < ds.Len() {
			batchSize := min(ds.Len()-startAt, *defaultBatchSize)
			traces := ds.Traces(startAt, batchSize)
			for _, trace := range traces {
				if producerWithoutCompression != nil {
					if _, err := producerWithoutCompression.BatchArrowRecordsFromTraces(trace); err != nil {
						panic(err)
					}
				}
				if _, err := producerWithCompression.BatchArrowRecordsFromTraces(trace); err != nil {
					panic(err)
				}
			}
			startAt += batchSize
		}
	}

	producerWithCompression.ShowStats()

	if producerWithoutCompression != nil {
		withoutCompressionStats := producerWithoutCompression.RecordSizeStats()
		withCompressionStats := producerWithCompression.RecordSizeStats()
		println()
		stats.CompareRecordSizeStats(withCompressionStats, withoutCompressionStats)
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}

	return b
}
