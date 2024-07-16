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
	"strconv"
	"strings"
	"testing"

	"github.com/apache/arrow/go/v17/arrow"
	"github.com/apache/arrow/go/v17/arrow/memory"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/dataset"
	parrow "github.com/open-telemetry/otel-arrow/pkg/benchmark/profileable/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/benchmark/profileable/otlp"
	"github.com/open-telemetry/otel-arrow/pkg/config"
	"github.com/open-telemetry/otel-arrow/pkg/otel/observer"
	"github.com/open-telemetry/otel-arrow/pkg/record_message"
)

var help = flag.Bool("help", false, "Show help")

type SimObserver struct {
	BatchId uint64
	Label   string
}

func (o *SimObserver) OnRecord(arrow.Record, record_message.PayloadType) {}

// batch-id, event-type, record-name, field-path, comment
func (o *SimObserver) OnNewField(recordName string, fieldPath string) {
	fmt.Printf("OnNewField: %s.%s\n", recordName, fieldPath)
	if o.Label != "" {
		o.Label += "; "
	}
	o.Label = o.Label + fmt.Sprintf("Add %s.%s", recordName, fieldPath)
}

func (o *SimObserver) OnDictionaryUpgrade(recordName string, fieldPath string, prevIndexType, newIndexType arrow.DataType, card, total uint64) {
	fmt.Printf("OnDictionaryUpgrade: %s.%s (%s -> %s, card: %d, total: %d)\n", recordName, fieldPath, prevIndexType, newIndexType, card, total)
	if o.Label != "" {
		o.Label += "; "
	}
	o.Label = o.Label + fmt.Sprintf("DictUpgrade %s.%s", recordName, fieldPath)
}
func (o *SimObserver) OnDictionaryOverflow(recordName string, fieldPath string, card, total uint64) {
	fmt.Printf("OnDictionaryOverflow: %s.%s (card: %d, total: %d, ratio: %f)\n", recordName, fieldPath, card, total, float64(card)/float64(total))
	if o.Label != "" {
		o.Label += "; "
	}
	o.Label = o.Label + fmt.Sprintf("DictOverflow %s.%s", recordName, fieldPath)
}
func (o *SimObserver) OnSchemaUpdate(recordName string, old, new *arrow.Schema) {
	fmt.Printf("OnSchemaUpdate: %s\n", recordName)
	if o.Label != "" {
		o.Label += "; "
	}
	o.Label = o.Label + "Schema Update"
}
func (o *SimObserver) OnDictionaryReset(recordName string, fieldPath string, indexType arrow.DataType, card, total uint64) {
	fmt.Printf("OnDictionaryReset: %s.%s (type: %s, card: %d, total: %d, ratio: %f)\n", recordName, fieldPath, indexType, card, total, float64(card)/float64(total))
	if o.Label != "" {
		o.Label += "; "
	}
	o.Label = o.Label + fmt.Sprintf("DictReset %s.%s", recordName, fieldPath)
}
func (o *SimObserver) OnMetadataUpdate(recordName, metadataKey string) {
	fmt.Printf("OnMetadataUpdate: %s (metadata-key: %s)\n", recordName, metadataKey)
	if o.Label != "" {
		o.Label += "; "
	}
	o.Label = o.Label + fmt.Sprintf("MetadataUpdate %s key=%s", recordName, metadataKey)
}

// This command simulates an OTel Arrow producer running for different
// configurations of batch size and stream duration.
func main() {
	// General flags
	batchSizeList := flag.String("batch-size", "10000", "Batch size (use a comma separated list to specify multiple batch sizes)")
	maxBatchesPerStreamList := flag.String("max-batches-per-stream", "10", "Maximum number of batches per stream (use a comma separated list to specify multiple values)")
	verbose := flag.Bool("verbose", false, "Verbose mode")
	output := flag.String("output", "compression-efficiency-gain.csv", "Output file")
	dictResetThreshold := flag.Float64("dict-reset-threshold", 0.3, "Dictionary reset threshold (0.3 by default)")
	checkMemoryLeak := flag.Bool("check-memory-leak", false, "Check memory leak")

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

	batchSizeArray := ToIntList(batchSizeList, "batch size")
	maxBatchesPerStreamArray := ToIntList(maxBatchesPerStreamList, "max batches per stream")

	// Define default input file
	inputFiles := flag.Args()
	if len(inputFiles) == 0 {
		panic("No input file specified")
	}

	// Create the output file
	fOutput, err := os.Create(*output)
	if err != nil {
		panic(err)
	}
	defer func() { _ = fOutput.Close() }()
	if _, err = fOutput.WriteString("Batch ID, Number of spans, Compression Efficiency Gain (%), Batch size, Max batches per stream, Dictionary Reset Threshold, OTLP batch uncompressed size, OTLP batch compressed size, OTel Arrow batch uncompressed size, OTel Arrow batch compressed size, Schema Events\n"); err != nil {
		panic(err)
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

	maxSpanCount := 250 * 10000
	for _, batchSize := range batchSizeArray {
		for _, maxBatchesPerStream := range maxBatchesPerStreamArray {
			println("--------------------------------------------------")
			fmt.Printf("Batch size: %d, Max batches per stream: %d\n", batchSize, maxBatchesPerStream)
			if *checkMemoryLeak {
				// Create a closure having the signature of a test function.
				run := func(t *testing.T) {
					Run(t, maxSpanCount, *dictResetThreshold, commonOptions, inputFiles, batchSize, maxBatchesPerStream, *verbose, err, fOutput)
				}

				testing.Main(matchString, []testing.InternalTest{
					{"Test", run},
				}, nil, nil)
			} else {
				Run(nil, maxSpanCount, *dictResetThreshold, commonOptions, inputFiles, batchSize, maxBatchesPerStream, *verbose, err, fOutput)
			}
		}
	}
}

func ToIntList(value *string, elemType string) []int {
	var intArray []int
	elems := strings.Split(*value, ",")
	for _, elem := range elems {
		if elem != "" {
			intValue, err := strconv.Atoi(elem)
			if err != nil {
				panic(fmt.Sprintf("Each %s must be a valid integer: %s", elemType, err.Error()))
			}
			if intValue <= 0 {
				panic(fmt.Sprintf("Each %s must be greater than 0", elemType))
			}
			intArray = append(intArray, intValue)
		}
	}
	return intArray
}

func Run(t *testing.T, maxSpanCount int, dictResetThreshold float64, commonOptions []config.Option, inputFiles []string, batchSize int, maxBatchesPerStream int, verbose bool, err error, fOutput *os.File) {
	// Initialize the allocator based the memory leak check flag
	// If t is nil -> standard Go allocator
	// If t is not nil -> checked allocator
	var pool memory.Allocator = memory.NewGoAllocator()
	deferFunc := func() {}

	if t != nil {
		checkedPool := memory.NewCheckedAllocator(pool)
		deferFunc = func() {
			checkedPool.AssertSize(t, 0)
			println("No memory leak detected")
		}
		pool = checkedPool
		println("Memory leak check enabled")
	}
	defer deferFunc()

	simObserver := &SimObserver{}
	options := append([]config.Option{
		config.WithZstd(),
		config.WithObserver(simObserver),
		config.WithDictResetThreshold(dictResetThreshold),
		config.WithAllocator(pool),
	}, commonOptions...)

	var otlpProfile *otlp.TracesProfileable
	var otelArrowProfile *parrow.TracesProfileable
	batchesPerStreamCount := 0

	// Data structure to compute Compression Efficiency Gain (CEG) moving
	// average
	cegWindowSize := 50
	cegWindowData := make([]float64, cegWindowSize)
	currentCegIndex := 0
	cegCount := 0

	batchID := 0
	spanCount := 0

Loop:
	for i := range inputFiles {
		ds := dataset.NewRealTraceDataset(inputFiles[i], benchmark.CompressionTypeZstd, "json", []string{"trace_id"})
		fmt.Printf("Dataset '%s' loaded %d spans\n", inputFiles[i], ds.Len())

		maxBatchCount := uint64(math.Ceil(float64(ds.Len()) / float64(batchSize)))
		startAt := 0

		for batchNum := uint64(0); batchNum < maxBatchCount; batchNum++ {
			if batchesPerStreamCount >= maxBatchesPerStream || otlpProfile == nil || otelArrowProfile == nil {
				otlpProfile = OtlpStream(otlpProfile, batchSize)
				otelArrowProfile = OtelArrowStream(otelArrowProfile, batchSize, simObserver, options...)
				batchesPerStreamCount = 0
			}
			otlpProfile.SetDataset(ds)
			otelArrowProfile.SetDataset(ds)
			correctedBatchSize := Min(otelArrowProfile.DatasetSize()-startAt, batchSize)

			// OTLP
			otlpUncompressed, otlpCompressed := ProcessBatch(otlpProfile, startAt, correctedBatchSize, verbose)
			otlpProfile.Clear()

			// OTel Arrow Protocol
			otelArrowUncompressed, otelArrowCompressed := ProcessBatch(otelArrowProfile, startAt, correctedBatchSize, verbose)
			otelArrowProfile.Clear()

			// Comparison OTLP vs OTel Arrow Protocol
			otlpCompressionImprovement := float64(otlpUncompressed) / float64(otlpCompressed)
			otelArrowCompressionImprovement := float64(otlpUncompressed) / float64(otelArrowCompressed)
			if verbose {
				fmt.Printf("OTel_ARROW uncompressed message is %f smaller\n", float64(otlpUncompressed)/float64(otelArrowUncompressed))
				fmt.Printf("OTel_ARROW compressed message is   %f smaller\n", float64(otlpCompressed)/float64(otelArrowCompressed))
				if otelArrowCompressionImprovement > otlpCompressionImprovement {
					fmt.Printf("OTLP compression ratio=%5.2f vs OTel_ARROW compression ratio=%5.2f (batch: #%06d)\n", float64(otlpUncompressed)/float64(otlpCompressed), float64(otlpUncompressed)/float64(otelArrowCompressed), batchesPerStreamCount)
				} else {
					fmt.Printf(">>> OTLP compression ratio=%5.2f vs OTel_ARROW compression ratio=%5.2f (batch: #%06d)\n", float64(otlpUncompressed)/float64(otlpCompressed), float64(otlpUncompressed)/float64(otelArrowCompressed), batchesPerStreamCount)
				}
			}
			// Compute Compression Efficiency Gain (CEG)
			ceg := 100.0 - (otlpCompressionImprovement/otelArrowCompressionImprovement)*100.0
			cegMovingAvg := 0.0
			cegWindowData[currentCegIndex] = ceg
			currentCegIndex = (currentCegIndex + 1) % cegWindowSize
			if cegCount < cegWindowSize {
				cegCount++
				for i := 0; i < cegCount; i++ {
					cegMovingAvg += cegWindowData[i]
				}
				cegMovingAvg /= float64(cegCount)
			} else {
				for i := 0; i < cegWindowSize; i++ {
					cegMovingAvg += cegWindowData[i]
				}
				cegMovingAvg /= float64(cegWindowSize)
			}

			spanCount += correctedBatchSize

			fmt.Printf("OTel Arrow Compression Efficiency Gain(CEG)=%5.2f%%, CEG moving avg=%5.2f%% (batch: #%06d)\n", ceg, cegMovingAvg, batchID)
			if _, err = fOutput.WriteString(fmt.Sprintf("%d, %d, %f, %d, %d, %f, %d, %d, %d, %d, %q\n", batchID, spanCount, ceg, batchSize, maxBatchesPerStream, dictResetThreshold, otlpUncompressed, otlpCompressed, otelArrowUncompressed, otelArrowCompressed, simObserver.Label)); err != nil {
				panic(err)
			}
			simObserver.Label = ""
			startAt += batchSize
			batchesPerStreamCount++
			batchID++

			if spanCount >= maxSpanCount {
				break Loop
			}
		}
	}

	otlpProfile.EndProfiling(os.Stdout)
	otelArrowProfile.EndProfiling(os.Stdout)
}

func matchString(a, b string) (bool, error) {
	return a == b, nil
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

func OtelArrowStream(
	prevStream *parrow.TracesProfileable,
	batchSize int,
	observer observer.ProducerObserver,
	options ...config.Option,
) *parrow.TracesProfileable {
	if prevStream != nil {
		prevStream.EndProfiling(os.Stdout)
	}

	profile := parrow.WithOption([]string{"stream mode"}, options...)
	profile.SetObserver(observer)
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
