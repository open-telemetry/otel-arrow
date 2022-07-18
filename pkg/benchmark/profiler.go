package benchmark

import (
	"bytes"
	"fmt"
	"math"
	"strings"
	"time"
)

type Profiler struct {
	batchSizes []uint64
	benchmarks []ProfilerResult
}

func NewProfiler(batchSizes []uint64) *Profiler {
	return &Profiler{
		batchSizes: batchSizes,
		benchmarks: []ProfilerResult{},
	}
}

func (p *Profiler) Profile(profileable ProfileableSystem, maxIter uint64) error {
	tags := strings.Join(profileable.Tags()[:], "+")

	p.benchmarks = append(p.benchmarks, ProfilerResult{
		benchName: profileable.Name(),
		summaries: []BatchSummary{},
		tags:      tags,
	})

	for _, batchSize := range p.batchSizes {
		fmt.Printf("Profiling '%s' (compression=%s, batch-size=%d)\n", profileable.Name(), profileable.CompressionAlgorithm(), batchSize)

		uncompressedSize := NewMetric()
		compressedSize := NewMetric()
		batchCeation := NewMetric()
		processing := NewMetric()
		serialization := NewMetric()
		deserialization := NewMetric()
		compression := NewMetric()
		decompression := NewMetric()
		totalTime := NewMetric()
		processingResults := []string{}

		profileable.InitBatchSize(batchSize)

		for _i := uint64(0); _i < maxIter; _i++ {
			maxBatchCount := uint64(math.Ceil(float64(profileable.DatasetSize()) / float64(batchSize)))
			startAt := uint64(0)
			for _j := uint64(0); _j < maxBatchCount; _j++ {
				correctedBatchSize := min(profileable.DatasetSize()-startAt, batchSize)
				profileable.PrepareBatch(startAt, correctedBatchSize)

				start := time.Now()

				// Batch creation
				profileable.CreateBatch(startAt, correctedBatchSize)
				afterBatchCreation := time.Now()

				// Processing
				result := profileable.Process()
				afterProcessing := time.Now()
				processingResults = append(processingResults, result)

				// Serialization
				buffers, err := profileable.Serialize()
				if err != nil {
					return err
				}
				afterSerialization := time.Now()
				uncompressedSizeBytes := 0
				for _, buffer := range buffers {
					uncompressedSizeBytes += len(buffer)
				}
				uncompressedSize.Record(float64(uncompressedSizeBytes))

				// Compression
				var compressedBuffers [][]byte
				for _, buffer := range buffers {
					compressedBuffer, err := Compress(profileable.CompressionAlgorithm(), buffer)
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
				compressedSize.Record(float64(compressedSizeBytes))

				// Decompression
				var uncompressedBuffers [][]byte
				for _, buffer := range compressedBuffers {
					uncompressedBuffer, err := Decompress(profileable.CompressionAlgorithm(), buffer)
					if err != nil {
						return err
					}
					uncompressedBuffers = append(uncompressedBuffers, uncompressedBuffer)
				}
				afterDecompression := time.Now()
				if !bytesEqual(buffers, uncompressedBuffers) {
					return fmt.Errorf("Buffers are not equal after decompression")
				}

				// Deserialization
				profileable.Deserialize(buffers)
				afterDeserialization := time.Now()
				profileable.Clear()

				batchCeation.Record(float64(afterBatchCreation.Sub(start).Seconds()))
				processing.Record(float64(afterProcessing.Sub(afterBatchCreation).Seconds()))
				serialization.Record(float64(afterSerialization.Sub(afterProcessing).Seconds()))
				compression.Record(float64(afterCompression.Sub(afterSerialization).Seconds()))
				decompression.Record(float64(afterDecompression.Sub(afterCompression).Seconds()))
				deserialization.Record(float64(afterDeserialization.Sub(afterDecompression).Seconds()))

				totalTime.Record(
					float64(afterBatchCreation.Sub(start).Seconds()) +
						float64(afterProcessing.Sub(afterBatchCreation).Seconds()) +
						float64(afterSerialization.Sub(afterProcessing).Seconds()) +
						float64(afterCompression.Sub(afterSerialization).Seconds()) +
						float64(afterDecompression.Sub(afterCompression).Seconds()) +
						float64(afterDeserialization.Sub(afterDecompression).Seconds()),
				)
			}
		}

		profileable.ShowStats()
		currentBenchmark := &p.benchmarks[len(p.benchmarks)-1]
		currentBenchmark.summaries = append(currentBenchmark.summaries, BatchSummary{
			batchSize:            batchSize,
			uncompressedSizeByte: uncompressedSize.ComputeSummary(),
			compressedSizeByte:   compressedSize.ComputeSummary(),
			batchCreationSec:     batchCeation.ComputeSummary(),
			processingSec:        processing.ComputeSummary(),
			serializationSec:     serialization.ComputeSummary(),
			deserializationSec:   deserialization.ComputeSummary(),
			compressionSec:       compression.ComputeSummary(),
			decompressionSec:     decompression.ComputeSummary(),
			totalTimeSec:         totalTime.ComputeSummary(),
			processingResults:    processingResults,
		})
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
				refProcessingResults = benchmark.summaries[batchIdx].processingResults
			} else {
				if !stringsEqual(refProcessingResults, benchmark.summaries[batchIdx].processingResults) {
					panic("Processing results are not equal")
				}
			}
		}
	}
}

func (p *Profiler) PrintResults() {
	p.PrintStepsTiming()
	p.PrintCompressionRatio()
}

func (p *Profiler) PrintStepsTiming() {

}

func (p *Profiler) PrintCompressionRatio() {

}

func min(a, b uint64) uint64 {
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
