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

package benchmark

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"log"
	"math"
	"os"
	"path"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/dustin/go-humanize"
	"github.com/olekukonko/tablewriter"

	"github.com/f5/otel-arrow-adapter/pkg/benchmark/stats"
)

// Section identifiers used in the benchmark output.
var (
	OtlpArrowConversionSection = NewSectionConfig("otlp_arrow_conversion_sec", "OTLP -> OTLP Arrow conv.", false)
	OtlpConversionSection      = NewSectionConfig("otlp_conversion_sec", "OTLP Arrow -> OTLP conv.", false)
	SerializationSection       = NewSectionConfig("serialization_sec", "Protobuf serialization", false)
	CompressionSection         = NewSectionConfig("compression_sec", "Compression", false)
	DecompressionSection       = NewSectionConfig("decompression_sec", "Decompression", false)
	DeserializationSection     = NewSectionConfig("deserialization_sec", "Protobuf deserialization", false)
	TotalEncodingTimeSection   = NewSectionConfig("total_encoding_time_sec", "Sub total", false)
	TotalDecodingTimeSection   = NewSectionConfig("total_decoding_time_sec", "Sub total", false)
	Phase1TotalTimeSection     = NewSectionConfig("total_time_sec", "Total", true)
	Phase2TotalTimeSection     = NewSectionConfig("total_time_sec", "Total", false)
	ProcessingSection          = NewSectionConfig("processing_sec", "Batch processing", false)
	UncompressedSizeSection    = NewSectionConfig("uncompressed_size", "Uncompressed (bytes)", false)
	CompressedSizeSection      = NewSectionConfig("compressed_size", "Compressed (bytes)", false)
)

// Profiler is the main profiler object used to implement benchmarks.
type Profiler struct {
	batchSizes []int
	benchmarks []*stats.ProfilerResult
	writer     io.Writer
	outputDir  string
	warmUpIter uint64
}

// SectionConfig is the configuration for a section of the benchmark table output.
type SectionConfig struct {
	ID string

	Title string

	// Default column configuration (if not overridden by a custom column config).
	defaultColumnConfig *ColumnConfig

	// Custom column configuration per column identifier.
	columns map[string]*ColumnConfig

	// True if the section is a total.
	total bool
}

// ColumnConfig is the configuration for a column of the benchmark table output.
type ColumnConfig struct {
	subTitle         string
	metricApplicable bool
}

type ProfileOptions struct {
	UnaryRpcMode bool
}

func NewProfiler(batchSizes []int, logfile string, warmUpIter uint64) *Profiler {
	if _, err := os.Stat(logfile); os.IsNotExist(err) {
		err = os.MkdirAll(path.Dir(logfile), 0700)
		if err != nil {
			log.Fatal("error creating directory: ", err)
		}
	}

	file, _ := os.OpenFile(filepath.Clean(logfile), os.O_RDWR|os.O_CREATE|os.O_APPEND, 0600)
	mw := io.MultiWriter(os.Stdout, file)
	dt := time.Now()
	_, _ = fmt.Fprintln(mw, "\n================================================================================")
	_, _ = fmt.Fprintln(mw, "Benchmark started at: ", dt.String())
	_, _ = fmt.Fprintln(mw, "")

	return &Profiler{
		batchSizes: batchSizes,
		benchmarks: []*stats.ProfilerResult{},
		writer:     mw,
		outputDir:  path.Dir(logfile),
		warmUpIter: warmUpIter,
	}
}

func (p *Profiler) Printf(format string, a ...any) {
	_, _ = fmt.Fprintf(p.writer, format, a...)
}

func (p *Profiler) Profile(profileable ProfileableSystem, maxIter uint64) error {
	runtime.GC()
	tags := strings.Join(profileable.Tags()[:], "+")

	p.benchmarks = append(p.benchmarks, &stats.ProfilerResult{
		BenchName: profileable.Name(),
		Summaries: []stats.BatchSummary{},
		Tags:      tags,
	})
	probe := stats.NewSystemProbe()

	for _, batchSize := range p.batchSizes {
		probe.Reset()
		profileable.StartProfiling(p.writer)
		batchStart := time.Now()
		_, _ = fmt.Fprintf(p.writer, "Profiling '%s' (parameters tags=[%v], batch-size=%d, dataset-size=%d)", profileable.Name(), strings.Join(profileable.Tags(), `,`), batchSize, profileable.DatasetSize())

		uncompressedSize := stats.NewMetric()
		compressedSize := stats.NewMetric()
		otlpArrowConversion := stats.NewMetric()
		otlpConversion := stats.NewMetric()
		processing := stats.NewMetric()
		serialization := stats.NewMetric()
		deserialization := stats.NewMetric()
		compression := stats.NewMetric()
		decompression := stats.NewMetric()
		totalTime := stats.NewMetric()
		var processingResults []string

		profileable.InitBatchSize(p.writer, batchSize)

		for _i := uint64(0); _i < maxIter; _i++ {
			maxBatchCount := uint64(math.Ceil(float64(profileable.DatasetSize()) / float64(batchSize)))
			startAt := 0

			for batchNum := uint64(0); batchNum < maxBatchCount; batchNum++ {
				correctedBatchSize := min(profileable.DatasetSize()-startAt, batchSize)
				profileable.PrepareBatch(p.writer, startAt, correctedBatchSize)

				start := time.Now()

				// OTLP -> OTLP Arrow conversion
				profileable.ConvertOtlpToOtlpArrow(p.writer, startAt, correctedBatchSize)
				afterOtlpArrowConversion := time.Now()

				// Processing
				result := profileable.Process(p.writer)
				afterProcessing := time.Now()

				processingResults = append(processingResults, result)

				// Serialization
				buffers, err := profileable.Serialize(p.writer)
				if err != nil {
					return err
				}
				afterSerialization := time.Now()
				uncompressedSizeBytes := 0

				for _, buffer := range buffers {
					uncompressedSizeBytes += len(buffer)
				}
				if batchNum >= p.warmUpIter {
					uncompressedSize.Record(float64(uncompressedSizeBytes))
				}

				// Compression
				var compressedBuffers [][]byte
				for _, buffer := range buffers {
					compressedBuffer, err := profileable.CompressionAlgorithm().Compress(buffer)
					if err != nil {
						return err
					}

					compressedBuffers = append(compressedBuffers, compressedBuffer)
				}
				afterCompression := time.Now()
				compressedSizeBytes := 0
				for _, buffer := range compressedBuffers {
					compressedSizeBytes += len(buffer)
				}
				if batchNum >= p.warmUpIter {
					compressedSize.Record(float64(compressedSizeBytes))
				}

				// Decompression
				var uncompressedBuffers [][]byte

				for _, buffer := range compressedBuffers {
					uncompressedBuffer, err := profileable.CompressionAlgorithm().Decompress(buffer)
					if err != nil {
						return err
					}

					uncompressedBuffers = append(uncompressedBuffers, uncompressedBuffer)
				}
				afterDecompression := time.Now()

				if !bytesEqual(buffers, uncompressedBuffers) {
					return fmt.Errorf("buffers are not equal after decompression")
				}

				// Deserialization
				profileable.Deserialize(p.writer, buffers)
				afterDeserialization := time.Now()

				// OTLP Arrow -> OTLP conversion
				profileable.ConvertOtlpArrowToOtlp(p.writer)
				afterOtlpConversion := time.Now()

				profileable.Clear()

				if batchNum >= p.warmUpIter {
					otlpArrowConversion.Record(afterOtlpArrowConversion.Sub(start).Seconds())
					processing.Record(afterProcessing.Sub(afterOtlpArrowConversion).Seconds())
					serialization.Record(afterSerialization.Sub(afterProcessing).Seconds())
					compression.Record(afterCompression.Sub(afterSerialization).Seconds())
					decompression.Record(afterDecompression.Sub(afterCompression).Seconds())
					deserialization.Record(afterDeserialization.Sub(afterDecompression).Seconds())
					otlpConversion.Record(afterOtlpConversion.Sub(afterDeserialization).Seconds())
				}

				totalTime.Record(
					afterOtlpArrowConversion.Sub(start).Seconds() +
						afterProcessing.Sub(afterOtlpArrowConversion).Seconds() +
						afterSerialization.Sub(afterProcessing).Seconds() +
						afterCompression.Sub(afterSerialization).Seconds() +
						afterDecompression.Sub(afterCompression).Seconds() +
						afterDeserialization.Sub(afterDecompression).Seconds() +
						afterOtlpConversion.Sub(afterDeserialization).Seconds(),
				)

				startAt += batchSize
			}
		}

		profileable.ShowStats()
		currentBenchmark := p.benchmarks[len(p.benchmarks)-1]
		currentBenchmark.Summaries = append(currentBenchmark.Summaries, stats.BatchSummary{
			BatchSize:              batchSize,
			UncompressedSizeByte:   uncompressedSize.ComputeSummary(),
			CompressedSizeByte:     compressedSize.ComputeSummary(),
			OtlpArrowConversionSec: otlpArrowConversion.ComputeSummary(),
			ProcessingSec:          processing.ComputeSummary(),
			SerializationSec:       serialization.ComputeSummary(),
			DeserializationSec:     deserialization.ComputeSummary(),
			CompressionSec:         compression.ComputeSummary(),
			DecompressionSec:       decompression.ComputeSummary(),
			TotalTimeSec:           totalTime.ComputeSummary(),
			ProcessingResults:      processingResults,
			CpuMemUsage:            probe.MeasureUsage(),
			OtlpConversionSec:      otlpConversion.ComputeSummary(),
		})

		profileable.EndProfiling(p.writer)
		fmt.Printf(", total duration=%fs\n", time.Now().Sub(batchStart).Seconds())
	}

	return nil
}

func (p *Profiler) CheckProcessingResults() {
	for batchIdx := range p.batchSizes {
		if len(p.benchmarks) == 0 {
			continue
		}

		var refProcessingResults []string
		for _, benchmark := range p.benchmarks {
			if len(refProcessingResults) == 0 {
				refProcessingResults = benchmark.Summaries[batchIdx].ProcessingResults
			} else {
				if !stringsEqual(refProcessingResults, benchmark.Summaries[batchIdx].ProcessingResults) {
					panic("Processing results are not equal")
				}
			}
		}
	}
}

func (p *Profiler) PrintResults(maxIter uint64) {
	colorReset := "\033[0m"
	colorGreen := "\033[32m"

	// Message size and compression ratio
	println()
	println(colorGreen)
	println("======== MEASUREMENT OF MESSAGE SIZE AND COMPRESSION RATIO FOR EACH PROTOCOL CONFIGURATION ==========", colorReset)
	p.PrintCompressionRatio(maxIter)

	// Time spent in each step in phase 1
	println()
	println(colorGreen)
	println("======= PHASE 1: MEASUREMENT OF THE TIME SPENT ON THE DIFFERENT STEPS FOR EACH PROTOCOL CONFIGURATION ========", colorReset)
	p.PrintPhase1StepsTiming(maxIter)

	// Time spent in each step in phase 2
	println()
	println(colorGreen)
	println("======= PHASE 2: MEASUREMENT OF THE TIME SPENT ON THE DIFFERENT STEPS FOR EACH PROTOCOL CONFIGURATION ========", colorReset)
	p.PrintPhase2StepsTiming(maxIter)

	println()
}

func (p *Profiler) PrintPhase1StepsTiming(_ uint64) {
	_, _ = fmt.Fprintf(p.writer, "\n")
	headers := []string{"Proto msg step duration"}

	for _, benchmark := range p.benchmarks {
		headers = append(headers, fmt.Sprintf("%s %s - Mean", benchmark.BenchName, benchmark.Tags))
	}

	table := tablewriter.NewWriter(p.writer)
	table.SetHeader(headers)
	table.SetBorder(false)
	table.SetHeaderAlignment(tablewriter.ALIGN_LEFT)
	table.SetAutoWrapText(false)
	headerColors := []tablewriter.Colors{tablewriter.Color(tablewriter.Normal, tablewriter.FgGreenColor)}

	for i := 0; i < len(p.benchmarks); i++ {
		headerColors = append(headerColors, tablewriter.Color())
	}

	table.SetHeaderColor(headerColors...)

	values := make(map[string]*stats.Summary)

	for _, result := range p.benchmarks {
		for _, summary := range result.Summaries {
			key := fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, OtlpArrowConversionSection.ID)
			values[key] = summary.OtlpArrowConversionSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, ProcessingSection.ID)
			values[key] = summary.ProcessingSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, SerializationSection.ID)
			values[key] = summary.SerializationSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, CompressionSection.ID)
			values[key] = summary.CompressionSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, DecompressionSection.ID)
			values[key] = summary.DecompressionSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, DeserializationSection.ID)
			values[key] = summary.DeserializationSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, TotalEncodingTimeSection.ID)
			values[key] = stats.AddSummaries(summary.OtlpArrowConversionSec, summary.SerializationSec, summary.CompressionSec)
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, TotalDecodingTimeSection.ID)
			values[key] = stats.AddSummaries(summary.DeserializationSec, summary.DecompressionSec, summary.OtlpConversionSec)
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, Phase1TotalTimeSection.ID)
			values[key] = summary.TotalTimeSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, OtlpConversionSection.ID)
			values[key] = summary.OtlpConversionSec
		}
	}

	transform := func(value float64) float64 { return value * 1000.0 }
	greenTitle := tablewriter.Color(tablewriter.Normal, tablewriter.FgGreenColor)
	cyanTitle := tablewriter.Color(tablewriter.Normal, tablewriter.FgCyanColor)
	p.AddTitle(table, "Exporter steps", greenTitle)
	p.AddStep(OtlpArrowConversionSection, table, values, transform, greenTitle)
	// p.AddStep(ProcessingSection, table, values, transform, greenTitle)
	p.AddStep(SerializationSection, table, values, transform, greenTitle)
	p.AddStep(CompressionSection, table, values, transform, greenTitle)
	p.AddStep(TotalEncodingTimeSection, table, values, transform, cyanTitle)
	p.AddTitle(table, "Receiver steps", greenTitle)
	p.AddStep(DecompressionSection, table, values, transform, greenTitle)
	p.AddStep(DeserializationSection, table, values, transform, greenTitle)
	p.AddStep(OtlpConversionSection, table, values, transform, greenTitle)
	p.AddStep(TotalDecodingTimeSection, table, values, transform, cyanTitle)
	p.AddSeparator(table)
	p.AddTitle(table, "End-to-end", greenTitle)
	p.AddStep(Phase1TotalTimeSection, table, values, transform, cyanTitle)

	table.Render()
}

func (p *Profiler) PrintPhase2StepsTiming(_ uint64) {
	_, _ = fmt.Fprintf(p.writer, "\n")
	headers := []string{"Proto msg step duration"}

	for _, benchmark := range p.benchmarks {
		headers = append(headers, fmt.Sprintf("%s %s - Mean", benchmark.BenchName, benchmark.Tags))
	}

	table := tablewriter.NewWriter(p.writer)
	table.SetHeader(headers)
	table.SetBorder(false)
	table.SetHeaderAlignment(tablewriter.ALIGN_LEFT)
	table.SetAutoWrapText(false)
	headerColors := []tablewriter.Colors{tablewriter.Color(tablewriter.Normal, tablewriter.FgGreenColor)}

	for i := 0; i < len(p.benchmarks); i++ {
		headerColors = append(headerColors, tablewriter.Color())
	}

	table.SetHeaderColor(headerColors...)

	values := make(map[string]*stats.Summary)

	for _, result := range p.benchmarks {
		for _, summary := range result.Summaries {
			key := fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, ProcessingSection.ID)
			values[key] = summary.ProcessingSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, SerializationSection.ID)
			values[key] = summary.SerializationSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, CompressionSection.ID)
			values[key] = summary.CompressionSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, DecompressionSection.ID)
			values[key] = summary.DecompressionSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, DeserializationSection.ID)
			values[key] = summary.DeserializationSec
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, TotalEncodingTimeSection.ID)
			values[key] = stats.AddSummaries(summary.SerializationSec, summary.CompressionSec)
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, TotalDecodingTimeSection.ID)
			values[key] = stats.AddSummaries(summary.DeserializationSec, summary.DecompressionSec)
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, Phase1TotalTimeSection.ID)
			values[key] = stats.AddSummaries(summary.SerializationSec, summary.CompressionSec, summary.DeserializationSec, summary.DecompressionSec)
		}
	}

	transform := func(value float64) float64 { return value * 1000.0 }
	greenTitle := tablewriter.Color(tablewriter.Normal, tablewriter.FgGreenColor)
	cyanTitle := tablewriter.Color(tablewriter.Normal, tablewriter.FgCyanColor)
	p.AddTitle(table, "Exporter steps", greenTitle)
	p.AddStep(SerializationSection, table, values, transform, greenTitle)
	p.AddStep(CompressionSection, table, values, transform, greenTitle)
	p.AddStep(TotalEncodingTimeSection, table, values, transform, cyanTitle)
	p.AddTitle(table, "Receiver steps", greenTitle)
	p.AddStep(DecompressionSection, table, values, transform, greenTitle)
	p.AddStep(DeserializationSection, table, values, transform, greenTitle)
	p.AddStep(TotalDecodingTimeSection, table, values, transform, cyanTitle)
	p.AddSeparator(table)
	p.AddTitle(table, "End-to-end", greenTitle)
	p.AddStep(Phase2TotalTimeSection, table, values, transform, cyanTitle)

	table.Render()
}

func (p *Profiler) PrintCompressionRatio(maxIter uint64) {
	_, _ = fmt.Fprintf(p.writer, "\n")
	headers := []string{"Proto msg size"}
	for _, benchmark := range p.benchmarks {
		headers = append(headers, fmt.Sprintf("%s %s - Mean", benchmark.BenchName, benchmark.Tags))
	}

	table := tablewriter.NewWriter(p.writer)
	table.SetHeader(headers)
	table.SetBorder(false)
	table.SetHeaderAlignment(tablewriter.ALIGN_LEFT)
	table.SetAutoWrapText(false)
	headerColors := []tablewriter.Colors{tablewriter.Color(tablewriter.Normal, tablewriter.FgGreenColor)}

	for i := 0; i < len(p.benchmarks); i++ {
		headerColors = append(headerColors, tablewriter.Color())
	}

	table.SetHeaderColor(headerColors...)

	uncompressedTotal := make(map[string]int64)
	compressedTotal := make(map[string]int64)

	values := make(map[string]*stats.Summary)

	for _, result := range p.benchmarks {
		for _, summary := range result.Summaries {
			key := fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, CompressedSizeSection.ID)
			values[key] = summary.CompressedSizeByte
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, "total_compressed_size_byte")
			compressedTotal[key] = int64(summary.CompressedSizeByte.Total(maxIter))
			key = fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, summary.BatchSize, UncompressedSizeSection.ID)
			values[key] = summary.UncompressedSizeByte
			uncompressedTotal[key] = int64(summary.UncompressedSizeByte.Total(maxIter))
		}
	}

	transform := func(value float64) float64 { return math.Trunc(value) }
	p.AddSectionWithTotal(UncompressedSizeSection, table, values, transform, maxIter)
	p.AddSectionWithTotal(CompressedSizeSection, table, values, transform, maxIter)

	table.Render()
}

func (p *Profiler) AddTitle(table *tablewriter.Table, title string, titleColor []int) {
	titles := []string{title}
	colors := []tablewriter.Colors{titleColor}
	for i := 0; i < len(p.benchmarks); i++ {
		titles = append(titles, "")
		colors = append(colors, tablewriter.Color())
	}
	table.Rich(titles, colors)
}

func (p *Profiler) AddSeparator(table *tablewriter.Table) {
	row := []string{"========================"}
	for i := 0; i < len(p.benchmarks); i++ {
		row = append(row, "")
	}
	table.Append(row)
}

func (p *Profiler) AddStep(
	section *SectionConfig,
	table *tablewriter.Table,
	values map[string]*stats.Summary,
	transform func(float64) float64,
	titleColor []int) {
	titles := []string{fmt.Sprintf("  %s", section.Title)}
	colors := []tablewriter.Colors{titleColor}
	for i := 0; i < len(p.benchmarks); i++ {
		result := p.benchmarks[i]
		titles = append(titles, section.SubTitle(fmt.Sprintf("%s:%s", result.BenchName, result.Tags)))
		colors = append(colors, tablewriter.Color())
	}
	table.Rich(titles, colors)

	for i, batchSize := range p.batchSizes {
		row := []string{fmt.Sprintf("  batch_size: %d", batchSize)}
		refImplName := ""
		for _, result := range p.benchmarks {
			metricNotApplicable := section.MetricNotApplicable(fmt.Sprintf("%s:%s", result.BenchName, result.Tags))
			key := fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, batchSize, section.ID)
			improvement := ""

			if refImplName == "" {
				refImplName = fmt.Sprintf("%s:%s", result.BenchName, result.Tags)
			} else {
				refKey := fmt.Sprintf("%s:%d:%s", refImplName, batchSize, section.ID)
				improvement = fmt.Sprintf(" (x%6.2f)", values[refKey].Mean/values[key].Mean)
			}

			value := transform(values[key].Mean)
			decoratedValue := "Not Applicable"
			if metricNotApplicable {
				if section.total {
					decoratedValue = fmt.Sprintf("%7.3fms/msg%s, %s", value, improvement, result.Summaries[i].CpuMemUsage.ToString())
				} else {
					decoratedValue = fmt.Sprintf("%7.3fms/msg%s", value, improvement)
				}
			}
			row = append(row, decoratedValue)
		}

		table.Append(row)
	}
}

func (p *Profiler) AddSectionWithTotal(section *SectionConfig, table *tablewriter.Table, values map[string]*stats.Summary, transform func(float64) float64, maxIter uint64) {
	labels := []string{section.Title}
	colors := []tablewriter.Colors{tablewriter.Color(tablewriter.Normal, tablewriter.FgGreenColor)}
	for i := 0; i < len(p.benchmarks); i++ {
		labels = append(labels, "")
		colors = append(colors, tablewriter.Color())
	}
	table.Rich(labels, colors)

	for _, batchSize := range p.batchSizes {
		row := []string{fmt.Sprintf("batch_size: %d", batchSize)}
		refImplName := ""
		for _, result := range p.benchmarks {
			key := fmt.Sprintf("%s:%s:%d:%s", result.BenchName, result.Tags, batchSize, section.ID)
			metricNotApplicable := section.MetricNotApplicable(fmt.Sprintf("%s:%s", result.BenchName, result.Tags))
			improvement := ""

			if refImplName == "" {
				refImplName = fmt.Sprintf("%s:%s", result.BenchName, result.Tags)
			} else {
				refKey := fmt.Sprintf("%s:%d:%s", refImplName, batchSize, section.ID)
				improvement = fmt.Sprintf("(x%6.2f)", values[refKey].Total(maxIter)/values[key].Total(maxIter))
			}

			value := transform(values[key].Mean)
			decoratedValue := "Not Applicable"
			if value == math.Trunc(value) {
				accumulatedSize := uint64(values[key].Total(maxIter))
				if metricNotApplicable {
					decoratedValue = fmt.Sprintf("%8d %s (total: %s)", int64(value), improvement, humanize.Bytes(accumulatedSize))
				}
				row = append(row, decoratedValue)
			} else {
				if value >= 1.0 {
					if metricNotApplicable {
						decoratedValue = fmt.Sprintf("%8.3f %s (total: %s)", value, improvement, humanize.Bytes(uint64(values[key].Total(maxIter))))
					}
					row = append(row, decoratedValue)
				} else {
					if metricNotApplicable {
						decoratedValue = fmt.Sprintf("%8.5f %s (total: %s)", value, improvement, humanize.Bytes(uint64(values[key].Total(maxIter))))
					}
					row = append(row, decoratedValue)
				}
			}
		}

		table.Append(row)
	}
}

func (p *Profiler) ExportMetricsTimesCSV(filePrefix string) {
	filename := fmt.Sprintf("%s/%s_times.csv", p.outputDir, filePrefix)
	file, err := os.OpenFile(filepath.Clean(filename), os.O_CREATE|os.O_WRONLY, 0600)

	if err != nil {
		log.Fatalf("failed creating file: %s", err)
	}

	dataWriter := bufio.NewWriter(file)

	_, err = dataWriter.WriteString("batch_size,duration_ms,protocol,step\n")
	if err != nil {
		panic(fmt.Sprintf("failed writing to file: %s", err))
	}

	for batchIdx, batchSize := range p.batchSizes {
		if len(p.benchmarks) == 0 {
			continue
		}

		for _, result := range p.benchmarks {
			otlpArrowConversionMs := result.Summaries[batchIdx].OtlpArrowConversionSec.Mean
			serializationMs := result.Summaries[batchIdx].SerializationSec.Mean
			compressionMs := result.Summaries[batchIdx].CompressionSec.Mean
			decompressionMs := result.Summaries[batchIdx].DecompressionSec.Mean
			deserializationMs := result.Summaries[batchIdx].DeserializationSec.Mean
			otlpConversionMs := result.Summaries[batchIdx].OtlpConversionSec.Mean

			_, err = dataWriter.WriteString(fmt.Sprintf("%d,%f,%s [%s],0_OtlpArrowConversion\n", batchSize, otlpArrowConversionMs, result.BenchName, result.Tags))
			if err != nil {
				panic(fmt.Sprintf("failed writing to file: %s", err))
			}

			_, err = dataWriter.WriteString(fmt.Sprintf("%d,%f,%s [%s],1_Serialization\n", batchSize, serializationMs, result.BenchName, result.Tags))
			if err != nil {
				panic(fmt.Sprintf("failed writing to file: %s", err))
			}

			_, err = dataWriter.WriteString(fmt.Sprintf("%d,%f,%s [%s],2_Compression\n", batchSize, compressionMs, result.BenchName, result.Tags))
			if err != nil {
				panic(fmt.Sprintf("failed writing to file: %s", err))
			}

			_, err = dataWriter.WriteString(fmt.Sprintf("%d,%f,%s [%s],3_Decompression\n", batchSize, decompressionMs, result.BenchName, result.Tags))
			if err != nil {
				panic(fmt.Sprintf("failed writing to file: %s", err))
			}

			_, err = dataWriter.WriteString(fmt.Sprintf("%d,%f,%s [%s],4_Deserialization\n", batchSize, deserializationMs, result.BenchName, result.Tags))
			if err != nil {
				panic(fmt.Sprintf("failed writing to file: %s", err))
			}

			_, err = dataWriter.WriteString(fmt.Sprintf("%d,%f,%s [%s],5_OtlpConversion\n", batchSize, otlpConversionMs, result.BenchName, result.Tags))
			if err != nil {
				panic(fmt.Sprintf("failed writing to file: %s", err))
			}
		}
	}

	err = dataWriter.Flush()
	if err != nil {
		panic(fmt.Sprintf("failed flushing the file: %s", err))
	}

	err = file.Close()
	if err != nil {
		panic(fmt.Sprintf("failed closing the file: %s", err))
	}

	_, _ = fmt.Fprintf(p.writer, "Time meseasurements exported to %s\n", filename)
}

func (p *Profiler) ExportMetricsBytesCSV(filePrefix string) {
	filename := fmt.Sprintf("%s/%s_bytes.csv", p.outputDir, filePrefix)
	file, err := os.OpenFile(filepath.Clean(filename), os.O_CREATE|os.O_WRONLY, 0600)

	if err != nil {
		log.Fatalf("failed creating file: %s", err)
	}

	dataWriter := bufio.NewWriter(file)

	_, err = dataWriter.WriteString("batch_size,iteration,compressed_size_byte,uncompressed_size_byte,Protocol\n")
	if err != nil {
		panic(fmt.Sprintf("failed writing to file: %s", err))
	}

	for batchIdx, batchSize := range p.batchSizes {
		if len(p.benchmarks) == 0 {
			continue
		}

		numSamples := len(p.benchmarks[0].Summaries[batchIdx].OtlpArrowConversionSec.Values)
		for sampleIdx := 0; sampleIdx < numSamples; sampleIdx++ {
			for _, result := range p.benchmarks {
				line := fmt.Sprintf("%d,%d", batchSize, sampleIdx)
				compressedSizeByte := result.Summaries[batchIdx].CompressedSizeByte.Values[sampleIdx]
				uncompressedSizeByte := result.Summaries[batchIdx].UncompressedSizeByte.Values[sampleIdx]

				line += fmt.Sprintf(",%f,%f,%s [%s]\n", compressedSizeByte, uncompressedSizeByte, result.BenchName, result.Tags)

				_, err = dataWriter.WriteString(line)
				if err != nil {
					panic(fmt.Sprintf("failed writing to file: %s", err))
				}
			}
		}
	}

	err = dataWriter.Flush()
	if err != nil {
		panic(fmt.Sprintf("failed flushing the file: %s", err))
	}
	err = file.Close()
	if err != nil {
		panic(fmt.Sprintf("failed closing the file: %s", err))
	}

	_, _ = fmt.Fprintf(p.writer, "Meseasurements of the message sizes exported to %s\n", filename)
}

func min(a, b int) int {
	if a < b {
		return a
	}

	return b
}

func bytesEqual(buffers1, buffers2 [][]byte) bool {
	if len(buffers1) != len(buffers2) {
		return false
	}

	for i := range buffers1 {
		if !bytes.Equal(buffers1[i], buffers2[i]) {
			return false
		}
	}

	return true
}

func stringsEqual(buffers1, buffers2 []string) bool {
	if len(buffers1) != len(buffers2) {
		return false
	}
	for i, v1 := range buffers1 {
		if v1 != buffers2[i] {
			return false
		}
	}
	return true
}

// DefaultColumnConfig creates a default column config.
func DefaultColumnConfig() *ColumnConfig {
	return &ColumnConfig{
		subTitle:         "",
		metricApplicable: true,
	}
}

// NewSectionConfig creates a new SectionConfig with default values.
func NewSectionConfig(sectionID string, title string, total bool) *SectionConfig {
	return &SectionConfig{
		ID:                  sectionID,
		Title:               title,
		defaultColumnConfig: DefaultColumnConfig(),
		columns:             make(map[string]*ColumnConfig),
		total:               total,
	}
}

func (sc *SectionConfig) CustomColumnFor(ps ProfileableSystem) *ColumnConfig {
	columnID := ProfileableSystemID(ps)
	column, ok := sc.columns[columnID]

	if !ok {
		column = DefaultColumnConfig()
		sc.columns[columnID] = column
	}

	return column
}

func (sc *SectionConfig) SubTitle(columnID string) string {
	column, ok := sc.columns[columnID]
	if !ok {
		return sc.defaultColumnConfig.subTitle
	}
	return column.subTitle
}

func (sc *SectionConfig) MetricNotApplicable(columnID string) bool {
	column, ok := sc.columns[columnID]
	if !ok {
		return sc.defaultColumnConfig.metricApplicable
	}
	return column.metricApplicable
}

func (c *ColumnConfig) SubTitle(subTitle string) *ColumnConfig {
	c.subTitle = subTitle
	return c
}

func (c *ColumnConfig) MetricNotApplicable() *ColumnConfig {
	c.metricApplicable = false
	return c
}
