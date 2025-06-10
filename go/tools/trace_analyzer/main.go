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
	"fmt"
	"os"

	arrowpb "github.com/open-telemetry/otel-arrow/go/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/go/pkg/benchmark/dataset"
	"github.com/open-telemetry/otel-arrow/go/pkg/config"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/arrow_record"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/stats"
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

	testSorting := flag.Bool("test-sorting", false, "Test sorting")

	// Parse the flag
	flag.Parse()

	// Usage Demo
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	inputFiles := flag.Args()

	var commonOptions []config.Option

	// Set flags
	if all != nil && *all {
		schemaStats = all
		recordStats = all
		schemaUpdates = all
		producerStats = all
		compressionRatio = all
	}

	if schemaStats != nil && *schemaStats {
		commonOptions = append(commonOptions, config.WithSchemaStats())
	}
	if recordStats != nil && *recordStats {
		commonOptions = append(commonOptions, config.WithRecordStats())
	}
	if schemaUpdates != nil && *schemaUpdates {
		commonOptions = append(commonOptions, config.WithSchemaUpdates())
	}
	if producerStats != nil && *producerStats {
		commonOptions = append(commonOptions, config.WithProducerStats())
	}

	// Set number of rows to display (per payload type)
	if spans != nil && *spans > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPANS.String(), *spans))
	}
	if resourceAttrs != nil && *resourceAttrs > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_RESOURCE_ATTRS.String(), *resourceAttrs))
	}
	if spanAttrs != nil && *spanAttrs > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_ATTRS.String(), *spanAttrs))
	}
	if spanEvents != nil && *spanEvents > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_EVENTS.String(), *spanEvents))
	}
	if spanLinks != nil && *spanLinks > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_LINKS.String(), *spanLinks))
	}
	if spanEventAttrs != nil && *spanEventAttrs > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_EVENT_ATTRS.String(), *spanEventAttrs))
	}
	if spanLinkAttrs != nil && *spanLinkAttrs > 0 {
		commonOptions = append(commonOptions, config.WithDumpRecordRows(arrowpb.ArrowPayloadType_SPAN_LINK_ATTRS.String(), *spanLinkAttrs))
	}

	// Preload all datasets
	datasets := loadAllDatasets(inputFiles, format)

	if testSorting != nil && *testSorting {
		// Run all possible combinations of sorting options
		// One trial per combination
		results := make(map[string]int64)
		totalNumberOfTrials := len(config.OrderSpanByVariants) * len(config.OrderAttrs16ByVariants) * len(config.OrderAttrs32ByVariants)

		// Run all possible combinations of sorting options.
		// This is a brute force approach, but it's fine for now since we
		// don't have a lot of options (140 options).
		// If the number of options increases, we should consider a more
		// efficient approach such as black box optimization.
		for orderSpanByLabel, orderSpanBy := range config.OrderSpanByVariants {
			for orderAttrs16ByLabel, orderByAttrs16 := range config.OrderAttrs16ByVariants {
				for orderAttrs32ByLabel, orderByAttrs32 := range config.OrderAttrs32ByVariants {
					withoutCompressionOptions := []config.Option{
						config.WithNoZstd(),
						config.WithCompressionRatioStats(),
						config.WithOrderSpanBy(orderSpanBy),
						config.WithOrderAttrs16By(orderByAttrs16),
						config.WithOrderAttrs32By(orderByAttrs32),
					}
					withCompressionOptions := append([]config.Option{
						config.WithZstd(),
						config.WithCompressionRatioStats(),
						config.WithOrderSpanBy(orderSpanBy),
						config.WithOrderAttrs16By(orderByAttrs16),
						config.WithOrderAttrs32By(orderByAttrs32),
					}, commonOptions...)

					println("=====================================================================")
					fmt.Printf("Trial %d/%d with parameters => order spans with %q, order attrs16 by %q, order attrs32 by %q\n",
						len(results)+1, totalNumberOfTrials, orderSpanByLabel, orderAttrs16ByLabel, orderAttrs32ByLabel)

					totalCompressedSize := runTrial(datasets, defaultBatchSize, withoutCompressionOptions, withCompressionOptions)
					results[fmt.Sprintf("spans order by: %s - attrs 16 order by: %s - attrs 32 order by: %s", orderSpanByLabel, orderAttrs16ByLabel, orderAttrs32ByLabel)] = totalCompressedSize
				}
			}
		}
		minTotalSize := int64(0)
		minTotalLabel := ""
		for label, totalSize := range results {
			if minTotalSize == 0 || totalSize < minTotalSize {
				minTotalSize = totalSize
				minTotalLabel = label
			}
		}
		fmt.Printf("Min total size: %d bytes for %s\n", minTotalSize, minTotalLabel)
	} else {
		// Run a single trial with the default sorting options
		var withoutCompressionOptions []config.Option

		withCompressionOptions := append([]config.Option{
			config.WithZstd(),
		}, commonOptions...)

		if compressionRatio != nil && *compressionRatio {
			withoutCompressionOptions = []config.Option{
				config.WithNoZstd(),
				config.WithCompressionRatioStats(),
			}
			withCompressionOptions = append(withCompressionOptions, config.WithCompressionRatioStats())
		}

		runTrial(datasets, defaultBatchSize, withoutCompressionOptions, withCompressionOptions)
	}
}

// loadAllDatasets loads all datasets from the input files.
func loadAllDatasets(inputFiles []string, format *string) []*dataset.RealTraceDataset {
	var datasets []*dataset.RealTraceDataset

	for i := range inputFiles {
		ds := dataset.NewRealTraceDataset(inputFiles[i], benchmark.CompressionTypeZstd, *format, []string{"trace_id"})
		datasets = append(datasets, ds)
	}

	return datasets
}

// runTrial runs a single trial with the given options.
func runTrial(
	datasets []*dataset.RealTraceDataset,
	defaultBatchSize *int,
	withoutCompressionOptions []config.Option,
	withCompressionOptions []config.Option) int64 {
	var producerWithoutCompression *arrow_record.Producer

	if withoutCompressionOptions != nil && len(withoutCompressionOptions) > 0 {
		producerWithoutCompression = arrow_record.NewProducerWithOptions(withoutCompressionOptions...)
	}
	producerWithCompression := arrow_record.NewProducerWithOptions(withCompressionOptions...)

	// Analyze all datasets and report statistics
	for _, ds := range datasets {
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

	err := producerWithCompression.Close()
	if err != nil {
		panic(err)
	}
	producerWithCompression.ShowStats()
	withCompressionStats := producerWithCompression.RecordSizeStats()

	if producerWithoutCompression != nil {
		withoutCompressionStats := producerWithoutCompression.RecordSizeStats()
		println()
		stats.CompareRecordSizeStats(withCompressionStats, withoutCompressionStats)
	}

	totalSize := int64(0)
	for _, s := range withCompressionStats {
		totalSize += s.TotalSize
	}

	return totalSize
}

func min(a, b int) int {
	if a < b {
		return a
	}

	return b
}
