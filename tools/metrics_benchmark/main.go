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
	"log"
	"os"

	"google.golang.org/protobuf/proto"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
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
		inputFiles = append(inputFiles, "./data/otlp_logs.pb")
	}

	// Compare the performance for each input file
	for i := range inputFiles {
		fmt.Printf("Benchmark '%s'\n", inputFiles[i])

		// Unmarshal the ExportTraceServiceRequest protobuf file.
		data, err := os.ReadFile(inputFiles[i])
		if err != nil {
			log.Fatal("read file:", err)
		}
		var otlp v1.ExportLogsServiceRequest
		if err := proto.Unmarshal(data, &otlp); err != nil {
			log.Fatal("unmarshal:", err)
		}

		// Compare the performance between the standard OTLP representation and the OTLP Arrow representation.
		// ToDo
	}
}
