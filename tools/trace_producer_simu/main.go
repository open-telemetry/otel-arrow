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
	"math"
	"os"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/profileable/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/profileable/otlp"
	"github.com/open-telemetry/otel-arrow/pkg/config"
)

var help = flag.Bool("help", false, "Show help")

// This command simulates an OTel Arrow producer running for different
// configurations of batch size and stream duration.
func main() {
	// General flags
	batchSize := flag.Int("batch-size", 10000, "Batch size")
	maxBatchesPerStream := flag.Int("max-batches-per-stream", 10, "Maximum number of batches per stream")
	verbose := flag.Bool("verbose", false, "Verbose mode")

	// Statistics related flags (no statistics by default)
	schemaStats := flag.Bool("schema-stats", false, "Display Arrow schema statistics")
	recordStats := flag.Bool("record-stats", false, "Display Arrow record statistics")
	schemaUpdates := flag.Bool("schema-updates", false, "Display Arrow schema updates")
	producerStats := flag.Bool("producer-stats", false, "Display OTel Arrow producer statistics")
	all := flag.Bool("all", false, "Display all statistics and updates")

	// Number of rows to display per record type (0 by default)
	spans := flag.Int("spans", 0, "Number of spans to display per Arrow record")
	resourceAttrs := flag.Int("resource-attrs", 0, "Number of resource attributes to display per Arrow record")
	spanAttrs := flag.Int("span-attrs", 0, "Number of span attributes to display per Arrow record")
	spanEvents := flag.Int("span-events", 0, "Number of span events to display per Arrow record")
	spanLinks := flag.Int("span-links", 0, "Number of span links to display per Arrow record")
	spanEventAttrs := flag.Int("span-event-attrs", 0, "Number of span event attributes to display per Arrow record")
	spanLinkAttrs := flag.Int("span-link-attrs", 0, "Number of span link attributes to display per Arrow record")

	// Parse the flag
	flag.Parse()

	// Usage
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	// Define default input file
	inputFiles := flag.Args()
	if len(inputFiles) == 0 {
		panic("No input file specified")
	}

	var commonOptions []config.Option

	// Set flags
	if all != nil && *all {
		schemaStats = all
		recordStats = all
		schemaUpdates = all
		producerStats = all
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

	options := append([]config.Option{
		config.WithZstd(),
	}, commonOptions...)

	var otlpProfile *otlp.TracesProfileable
	var otelArrowProfile *arrow.TracesProfileable
	batchesPerStreamCount := 0

	for i := range inputFiles {
		ds := dataset.NewRealTraceDataset(inputFiles[i], benchmark.CompressionTypeZstd, "json", []string{"trace_id"})
		fmt.Printf("Dataset '%s' loaded %d spans\n", inputFiles[i], ds.Len())

		maxBatchCount := uint64(math.Ceil(float64(ds.Len()) / float64(*batchSize)))
		startAt := 0

		for batchNum := uint64(0); batchNum < maxBatchCount; batchNum++ {
			if batchesPerStreamCount >= *maxBatchesPerStream || otlpProfile == nil || otelArrowProfile == nil {
				otlpProfile = OtlpStream(otlpProfile, *batchSize)
				otelArrowProfile = OtelArrowStream(otelArrowProfile, *batchSize, options...)
				batchesPerStreamCount = 0
			}
			otlpProfile.SetDataset(ds)
			otelArrowProfile.SetDataset(ds)
			correctedBatchSize := Min(otelArrowProfile.DatasetSize()-startAt, *batchSize)

			// OTLP
			otlpUncompressed, otlpCompressed := ProcessBatch(otlpProfile, startAt, correctedBatchSize, *verbose)
			otlpProfile.Clear()

			// OTel Arrow Protocol
			otelArrowUncompressed, otelArrowCompressed := ProcessBatch(otelArrowProfile, startAt, correctedBatchSize, *verbose)
			otelArrowProfile.Clear()

			// Comparison OTLP vs OTel Arrow Protocol
			otlpCompressionImprovement := float64(otlpUncompressed) / float64(otlpCompressed)
			otelArrowCompressionImprovement := float64(otlpUncompressed) / float64(otelArrowCompressed)
			if *verbose {
				fmt.Printf("OTel_ARROW uncompressed message is %f smaller\n", float64(otlpUncompressed)/float64(otelArrowUncompressed))
				fmt.Printf("OTel_ARROW compressed message is   %f smaller\n", float64(otlpCompressed)/float64(otelArrowCompressed))
				if otelArrowCompressionImprovement > otlpCompressionImprovement {
					fmt.Printf("OTLP compression ratio=%5.2f vs OTel_ARROW compression ratio=%5.2f (batch: #%06d)\n", float64(otlpUncompressed)/float64(otlpCompressed), float64(otlpUncompressed)/float64(otelArrowCompressed), batchesPerStreamCount)
				} else {
					fmt.Printf(">>> OTLP compression ratio=%5.2f vs OTel_ARROW compression ratio=%5.2f (batch: #%06d)\n", float64(otlpUncompressed)/float64(otlpCompressed), float64(otlpUncompressed)/float64(otelArrowCompressed), batchesPerStreamCount)
				}
			}
			otelArrowImprovement := 100.0 - (otlpCompressionImprovement/otelArrowCompressionImprovement)*100.0
			fmt.Printf("OTel Arrow compression improvement=%5.2f%% (batch: #%06d)\n", otelArrowImprovement, batchesPerStreamCount)

			startAt += *batchSize
			batchesPerStreamCount++
		}
	}

	otlpProfile.EndProfiling(os.Stdout)
	otelArrowProfile.EndProfiling(os.Stdout)
}

func OtlpStream(prevStream *otlp.TracesProfileable, batchSize int) *otlp.TracesProfileable {
	if prevStream != nil {
		prevStream.EndProfiling(os.Stdout)
	}
	profile := otlp.New(benchmark.Zstd())
	profile.StartProfiling(os.Stdout)
	profile.InitBatchSize(os.Stdout, batchSize)
	return profile
}

func OtelArrowStream(prevStream *arrow.TracesProfileable, batchSize int, options ...config.Option) *arrow.TracesProfileable {
	if prevStream != nil {
		prevStream.EndProfiling(os.Stdout)
	}

	profile := arrow.WithOption([]string{"stream mode"}, options...)
	profile.StartProfiling(os.Stdout)
	profile.InitBatchSize(os.Stdout, batchSize)
	return profile
}

func ProcessBatch(profile benchmark.ProfileableSystem, startAt int, batchSize int, verbose bool) (uncompressed int, compressed int) {
	profile.PrepareBatch(os.Stdout, startAt, batchSize)
	profile.ConvertOtlpToOtlpArrow(os.Stdout, startAt, batchSize)
	buffers, err := profile.Serialize(os.Stdout)
	if err != nil {
		panic(err)
	}

	uncompressed = 0
	for _, buffer := range buffers {
		uncompressed += len(buffer)
	}

	var compressedBuffers [][]byte
	for _, buffer := range buffers {
		compressedBuffer, err := profile.CompressionAlgorithm().Compress(buffer)
		if err != nil {
			panic(err)
		}

		compressedBuffers = append(compressedBuffers, compressedBuffer)
	}
	compressed = 0
	for _, buffer := range compressedBuffers {
		compressed += len(buffer)
	}

	compressionRatio := float64(uncompressed) / float64(compressed)

	if verbose {
		fmt.Printf("%10s: uncompressed=%8d bytes, compressed=%8d bytes, compression ratio=%f\n", profile.Name(), uncompressed, compressed, compressionRatio)
	}

	return
}

func Min(a, b int) int {
	if a < b {
		return a
	}

	return b
}
