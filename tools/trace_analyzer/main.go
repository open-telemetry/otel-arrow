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

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
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

	// Record type #rows to display
	spans := flag.Int("spans", 0, "Number of spans to display per Arrow record")
	resourceAttrs := flag.Int("resource-attrs", 0, "Number of resource attributes to display per Arrow record")
	spanAttrs := flag.Int("span-attrs", 0, "Number of span attributes to display per Arrow record")
	spanEvents := flag.Int("span-events", 0, "Number of span events to display per Arrow record")
	spanLinks := flag.Int("span-links", 0, "Number of span links to display per Arrow record")
	spanEventAttrs := flag.Int("span-event-attrs", 0, "Number of span event attributes to display per Arrow record")
	spanLinkAttrs := flag.Int("span-link-attrs", 0, "Number of span link attributes to display per Arrow record")

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

	// Set flags
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

	// Set number of rows to display (per payload type)
	if spans != nil && *spans > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPANS.String(), *spans))
	}
	if resourceAttrs != nil && *resourceAttrs > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_RESOURCE_ATTRS.String(), *resourceAttrs))
	}
	if spanAttrs != nil && *spanAttrs > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_ATTRS.String(), *spanAttrs))
	}
	if spanEvents != nil && *spanEvents > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_EVENTS.String(), *spanEvents))
	}
	if spanLinks != nil && *spanLinks > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_LINKS.String(), *spanLinks))
	}
	if spanEventAttrs != nil && *spanEventAttrs > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_EVENT_ATTRS.String(), *spanEventAttrs))
	}
	if spanLinkAttrs != nil && *spanLinkAttrs > 0 {
		options = append(options, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_LINK_ATTRS.String(), *spanLinkAttrs))
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
