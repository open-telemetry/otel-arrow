/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package stats

// Set of general statistics about the OTLP Arrow Producer.

import (
	"fmt"
	"sort"
	"sync"
	"sync/atomic"

	"github.com/HdrHistogram/hdrhistogram-go"
)

type (
	// ProducerStats is a struct that contains stats about the OTLP Arrow Producer.
	ProducerStats struct {
		MetricsBatchesProduced atomic.Uint64
		LogsBatchesProduced    atomic.Uint64
		TracesBatchesProduced  atomic.Uint64
		StreamProducersCreated atomic.Uint64
		StreamProducersClosed  atomic.Uint64
		RecordBuilderStats     RecordBuilderStats

		// SchemaStats is a flag that indicates whether to display schema stats.
		SchemaStats bool
		// SchemaUpdates is a flag that indicates whether to display schema updates.
		SchemaUpdates bool
		// RecordStats is a flag that indicates whether to display record stats.
		RecordStats bool
		// ProducerStats is a flag that indicates whether to display producer stats.
		ProducerStats bool
		// CompressionRatioStats is a flag that indicates whether to display compression ratio stats.
		CompressionRatioStats bool
		// DumpRecordRows specifies the number of rows to dump for each record.
		// If not defined or set to 0, no rows are dumped.
		DumpRecordRows map[string]int
	}

	RecordSizeStats struct {
		totalSize int64
		dist      *hdrhistogram.Histogram
	}

	RecordBuilderStats struct {
		SchemaUpdatesPerformed     atomic.Uint64
		DictionaryIndexTypeChanged atomic.Uint64
		DictionaryOverflowDetected atomic.Uint64

		recordSizeDistributionLock sync.Mutex
		recordSizeDistribution     map[string]*RecordSizeStats
	}
)

// NewProducerStats creates a new ProducerStats struct.
func NewProducerStats() *ProducerStats {
	return &ProducerStats{
		RecordBuilderStats: RecordBuilderStats{
			recordSizeDistribution: make(map[string]*RecordSizeStats),
		},
		SchemaStats:   false,
		SchemaUpdates: false,
	}
}

// GetAndReset returns the current stats and resets them to zero.
func (s *ProducerStats) GetAndReset() ProducerStats {
	stats := *s
	s.Reset()
	return stats
}

// Reset sets all stats to zero.
func (s *ProducerStats) Reset() {
	s.MetricsBatchesProduced.Store(0)
	s.LogsBatchesProduced.Store(0)
	s.TracesBatchesProduced.Store(0)
	s.StreamProducersCreated.Store(0)
	s.StreamProducersClosed.Store(0)
	s.RecordBuilderStats.Reset()
}

// Reset sets all stats to zero.
func (s *RecordBuilderStats) Reset() {
	s.SchemaUpdatesPerformed.Store(0)
	s.DictionaryIndexTypeChanged.Store(0)
	s.DictionaryOverflowDetected.Store(0)
}

// Show prints the stats to the console.
func (s *ProducerStats) Show(indent string) {
	fmt.Printf("%s- Metrics batches produced: %d\n", indent, s.MetricsBatchesProduced)
	fmt.Printf("%s- Logs batches produced: %d\n", indent, s.LogsBatchesProduced)
	fmt.Printf("%s- Traces batches produced: %d\n", indent, s.TracesBatchesProduced)
	fmt.Printf("%s- Stream producers created: %d\n", indent, s.StreamProducersCreated)
	fmt.Printf("%s- Stream producers closed: %d\n", indent, s.StreamProducersClosed)
	fmt.Printf("%s- RecordBuilder:\n", indent)
	s.RecordBuilderStats.Show(indent + "  ")
}

// RecordSizeStats returns statistics per record payload type.
func (s *ProducerStats) RecordSizeStats() map[string]*RecordSizeStats {
	return s.RecordBuilderStats.RecordSizeStats()
}

func (s *RecordBuilderStats) Observe(payloadType string, recordSize int64) {
	s.recordSizeDistributionLock.Lock()
	defer s.recordSizeDistributionLock.Unlock()

	recordSizeDist, ok := s.recordSizeDistribution[payloadType]
	if !ok {
		recordSizeDist = &RecordSizeStats{
			totalSize: 0,
			dist:      hdrhistogram.New(0, 1<<32, 2),
		}
		s.recordSizeDistribution[payloadType] = recordSizeDist
	}

	recordSizeDist.totalSize += recordSize
	_ = recordSizeDist.dist.RecordValue(recordSize)
}

func (s *RecordSizeStats) TotalSize() int64 {
	return s.totalSize
}

// Show prints the RecordBuilder stats to the console.
func (s *RecordBuilderStats) Show(indent string) {
	fmt.Printf("%s- Schema updates performed: %d\n", indent, s.SchemaUpdatesPerformed)
	fmt.Printf("%s- Dictionary index type changed: %d\n", indent, s.DictionaryIndexTypeChanged)
	fmt.Printf("%s- Dictionary overflow detected: %d\n", indent, s.DictionaryOverflowDetected)

	s.recordSizeDistributionLock.Lock()
	defer s.recordSizeDistributionLock.Unlock()
	if len(s.recordSizeDistribution) > 0 {
		type RecordSizeStats struct {
			PayloadType string
			TotalSize   int64
			Dist        *hdrhistogram.Snapshot
			Percent     float64
		}

		var recordSizeStats []RecordSizeStats
		totalSize := int64(0)

		for k, v := range s.recordSizeDistribution {
			recordSizeStats = append(recordSizeStats, RecordSizeStats{
				PayloadType: k,
				TotalSize:   v.totalSize,
				Dist:        v.dist.Export(),
				Percent:     0,
			})
			totalSize += v.totalSize
		}

		// Compute the percentage of each record size
		for i := range recordSizeStats {
			recordSizeStats[i].Percent = float64(recordSizeStats[i].TotalSize) / float64(totalSize) * 100.0
		}

		// Sort the record size stats by percentage (descending)
		sort.Slice(recordSizeStats, func(i, j int) bool {
			return recordSizeStats[i].TotalSize > recordSizeStats[j].TotalSize
		})

		fmt.Printf("%s- Record size distribution:\n", indent)
		for _, v := range recordSizeStats {
			dist := hdrhistogram.Import(v.Dist)
			fmt.Printf("%s  - %-18s: %8d bytes (%04.1f%%) (min: %7d, max: %7d, mean: %7.1f, stdev: %7.1f, p50: %7d, p99: %7d)\n", indent,
				v.PayloadType, v.TotalSize, v.Percent,
				dist.Min(), dist.Max(), dist.Mean(),
				dist.StdDev(),
				dist.ValueAtQuantile(50), dist.ValueAtQuantile(99),
			)
		}
	}
}

// RecordSizeStats returns statistics per record payload type.
func (s *RecordBuilderStats) RecordSizeStats() map[string]*RecordSizeStats {
	s.recordSizeDistributionLock.Lock()
	defer s.recordSizeDistributionLock.Lock()

	m := map[string]*RecordSizeStats{}

	for k, v := range s.recordSizeDistribution {
		rss := &RecordSizeStats{
			totalSize: v.totalSize,
			dist:      hdrhistogram.Import(v.dist.Export()),
		}
		rss.dist.Merge(v.dist)
		m[k] = rss

	}

	return s.recordSizeDistribution
}

// CompareRecordSizeStats compares the record size stats with and without compression
// and prints the results to the console.
func CompareRecordSizeStats(withCompression map[string]*RecordSizeStats, withNoCompression map[string]*RecordSizeStats) {
	type RecordSizeStats struct {
		PayloadType                string
		TotalSizeWithCompression   int64
		TotalSizeWithNoCompression int64
		Percent                    float64
	}

	var recordSizeStats []RecordSizeStats
	totalSize := int64(0)

	for payloadType, stats := range withCompression {
		statsWithNoCompression, ok := withNoCompression[payloadType]
		totalSizeWithNoCompression := int64(0)
		if ok {
			totalSizeWithNoCompression = statsWithNoCompression.totalSize
		}
		recordSizeStats = append(recordSizeStats, RecordSizeStats{
			PayloadType:                payloadType,
			TotalSizeWithCompression:   stats.totalSize,
			TotalSizeWithNoCompression: totalSizeWithNoCompression,
			Percent:                    0,
		})
		totalSize += stats.totalSize
	}

	// Compute the percentage of each record size (with compression)
	for i := range recordSizeStats {
		recordSizeStats[i].Percent = float64(recordSizeStats[i].TotalSizeWithCompression) / float64(totalSize) * 100.0
	}

	// Sort the record size stats by percentage (descending)
	sort.Slice(recordSizeStats, func(i, j int) bool {
		return recordSizeStats[i].TotalSizeWithCompression > recordSizeStats[j].TotalSizeWithCompression
	})

	totalCumulSize := 0
	fmt.Printf("Record cumulative size and compression ratio per payload type (sort by cumulative size):\n")
	for _, v := range recordSizeStats {
		fmt.Printf("- %-18s: %8d bytes (cumul), %04.1f%% of the total, compression ratio=%4.1fx\n",
			v.PayloadType, v.TotalSizeWithCompression, v.Percent,
			float64(v.TotalSizeWithNoCompression)/float64(v.TotalSizeWithCompression),
		)
		totalCumulSize += int(v.TotalSizeWithCompression)
	}
	fmt.Printf("Total cumulative size: %d bytes\n", totalCumulSize)
}
