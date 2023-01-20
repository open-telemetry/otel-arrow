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

package main

import (
	"fmt"
	"runtime"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/dustin/go-humanize"

	carrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	logs "github.com/f5/otel-arrow-adapter/pkg/otel/logs/arrow"
	metrics "github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	traces "github.com/f5/otel-arrow-adapter/pkg/otel/traces/arrow"
)

const (
	ColorReset  = "\033[0m"
	ColorGreen  = "\033[32m"
	ColorYellow = "\033[33m"
)

// This tool is used to measure the memory usage of the different OTLP Arrow constructs (benchmarking purpose only).
//
// Example of output:
// METRICS - Memory usage
//   arrow.NewAdaptiveSchema(schema)
//     - number of bytes allocated: 120 kB
//     - number of malloc operations: 849
//     - number of free operations: 1
//     - number of GC: 0
//     - heap object size: 1.5 MB
//     - heap object size after a forced GC: 1.4 MB
//  NewRecordBuilder(schema)
//     - number of bytes allocated: 180 kB
//     - number of malloc operations: 1564
//     - number of free operations: 0
//     - number of GC: 0
//     - heap object size: 1.6 MB
//     - heap object size after a forced GC: 1.4 MB
//  NewRecordBuilder(...).NewRecord() - empty
//     - number of bytes allocated: 1.3 MB
//     - number of malloc operations: 13523
//     - number of free operations: 1438
//     - number of GC: 0
//     - heap object size: 2.7 MB
//     - heap object size after a forced GC: 1.4 MB

func main() {
	Report("METRICS", metrics.Schema)
	Report("LOGS", logs.Schema)
	Report("TRACES", traces.Schema)
}

func Report(name string, schema *arrow.Schema) {
	pool := memory.NewGoAllocator()
	println("--------------------------------------------------")
	fmt.Printf("%s%s - Memory usage%s\n", ColorGreen, name, ColorReset)
	ReportMemUsageOf("arrow.NewAdaptiveSchema(schema)", func() {
		schema := carrow.NewAdaptiveSchema(pool, schema)
		defer schema.Release()
	})
	ReportMemUsageOf("NewRecordBuilder(schema)", func() {
		array.NewRecordBuilder(pool, schema)
	})
	ReportMemUsageOf("NewRecordBuilder(...).NewRecord() - empty", func() {
		builder := array.NewRecordBuilder(pool, schema)
		defer builder.Release()
		record := builder.NewRecord()
		defer record.Release()
	})

	adaptiveSchema := carrow.NewAdaptiveSchema(pool, schema)
	builder := array.NewRecordBuilder(pool, adaptiveSchema.Schema())
	defer builder.Release()
	record := builder.NewRecord()
	defer record.Release()
	ReportMemUsageOf("reusedRecordBuilder.NewRecord() - empty", func() {
		r := builder.NewRecord()
		defer r.Release()
	})
	ReportMemUsageOf("adaptiveSchema.Analyze(...) + adaptiveSchema.UpdateSchema(...) if needed", func() {
		overflowDetected, updates := adaptiveSchema.Analyze(record)
		if overflowDetected {
			println("overflow detected")
			adaptiveSchema.UpdateSchema(updates)
		}
	})
}

func ReportMemUsageOf(name string, fn func()) {
	runtime.GC()

	var before runtime.MemStats
	runtime.ReadMemStats(&before)

	fn()

	var after runtime.MemStats
	runtime.ReadMemStats(&after)

	fmt.Printf("  %s%s%s\n", ColorYellow, name, ColorReset)
	println("    - number of bytes allocated:", humanize.Bytes(after.TotalAlloc-before.TotalAlloc))
	println("    - number of malloc operations:", after.Mallocs-before.Mallocs)
	println("    - number of free operations:", after.Frees-before.Frees)
	println("    - number of GC:", after.NumGC-before.NumGC)
	println("    - heap object size:", humanize.Bytes(after.Alloc))
	runtime.GC()
	runtime.ReadMemStats(&after)
	println("    - heap object size after a forced GC:", humanize.Bytes(after.Alloc))
}
