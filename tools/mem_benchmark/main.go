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
	"math"
	"runtime"

	"github.com/apache/arrow/go/v16/arrow"
	"github.com/apache/arrow/go/v16/arrow/memory"
	"github.com/dustin/go-humanize"

	"github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/builder"
	config "github.com/open-telemetry/otel-arrow/pkg/otel/common/schema/config"
	logs "github.com/open-telemetry/otel-arrow/pkg/otel/logs/arrow"
	metrics "github.com/open-telemetry/otel-arrow/pkg/otel/metrics/arrow"
	"github.com/open-telemetry/otel-arrow/pkg/otel/stats"
	traces "github.com/open-telemetry/otel-arrow/pkg/otel/traces/arrow"
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
	Report("METRICS", metrics.MetricsSchema)
	Report("LOGS", logs.LogsSchema)
	Report("TRACES", traces.TracesSchema)
}

var DictConfig = config.NewDictionary(math.MaxUint16, 0.0)

func Report(name string, schema *arrow.Schema) {
	pool := memory.NewGoAllocator()
	println("--------------------------------------------------")
	fmt.Printf("%s%s - Memory usage%s\n", ColorGreen, name, ColorReset)
	ReportMemUsageOf("NewRecordBuilderExt(schema)", func() {
		b := builder.NewRecordBuilderExt(pool, schema, DictConfig, stats.NewProducerStats(), nil)
		defer b.Release()
	})
	ReportMemUsageOf("NewRecordBuilder(...).NewRecord() - empty", func() {
		b := builder.NewRecordBuilderExt(pool, schema, DictConfig, stats.NewProducerStats(), nil)
		defer b.Release()
		record, err := b.NewRecord()
		if err != nil {
			panic(err)
		}
		defer record.Release()
	})

	b := builder.NewRecordBuilderExt(pool, schema, DictConfig, stats.NewProducerStats(), nil)
	defer b.Release()
	record, err := b.NewRecord()
	if err != nil {
		panic(err)
	}
	defer record.Release()
	ReportMemUsageOf("reusedRecordBuilder.NewRecord() - empty", func() {
		r, err := b.NewRecord()
		if err != nil {
			panic(err)
		}
		defer r.Release()
	})
	ReportMemUsageOf("builder.Analyze(...) + builder.UpdateSchema(...) if needed", func() {
		if b.IsSchemaUpToDate() {
			println("overflow detected")
			b.UpdateSchema()
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
