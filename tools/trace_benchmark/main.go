/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package main

import (
	"flag"
	"fmt"
	"os"

	"otel-arrow-adapter/pkg/air/config"
	"otel-arrow-adapter/pkg/benchmark"
	"otel-arrow-adapter/pkg/benchmark/profileable/otlp"
	"otel-arrow-adapter/pkg/benchmark/profileable/otlp_arrow"
)

var help = flag.Bool("help", false, "Show help")

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
		inputFiles = append(inputFiles, "./data/otlp_traces.pb")
	}

	// Compare the performance for each input file
	for i := range inputFiles {
		fmt.Printf("Dataset '%s'\n", inputFiles[i])

		// Compare the performance between the standard OTLP representation and the OTLP Arrow representation.
		profiler := benchmark.NewProfiler([]int{1000, 5000, 10000, 25000})
		compressionAlgo := benchmark.Zstd
		maxIter := uint64(3)
		dataset := benchmark.NewRealTraceDataset(inputFiles[i], []string{"trace_id"})
		otlpTraces := otlp.NewTraceProfileable(dataset, compressionAlgo)
		otlpArrowTracesWithoutDictionary := otlp_arrow.NewTraceProfileable([]string{"No dict"}, dataset, config.NewConfigWithoutDictionary(), compressionAlgo)
		otlpArrowTracesWithDictionary := otlp_arrow.NewTraceProfileable([]string{"With dict"}, dataset, config.NewDefaultConfig(), compressionAlgo)

		if err := profiler.Profile(otlpTraces, maxIter); err != nil {
			panic(fmt.Errorf("expected no error, got %v", err))
		}
		if err := profiler.Profile(otlpArrowTracesWithoutDictionary, maxIter); err != nil {
			panic(fmt.Errorf("expected no error, got %v", err))
		}
		if err := profiler.Profile(otlpArrowTracesWithDictionary, maxIter); err != nil {
			panic(fmt.Errorf("expected no error, got %v", err))
		}

		profiler.CheckProcessingResults()
		profiler.PrintResults()

		//profiler.ExportMetricsTimesCSV(fmt.Sprintf("%d_traces_benchmark_results", i))
		//profiler.ExportMetricsBytesCSV(fmt.Sprintf("%d_traces_benchmark_results", i))
	}
}
