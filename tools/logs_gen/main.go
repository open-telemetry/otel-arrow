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
	"crypto/rand"
	"encoding/json"
	"flag"
	"io"
	"log"
	"math"
	"math/big"
	"os"
	"path"

	"github.com/klauspost/compress/zstd"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"

	"github.com/open-telemetry/otel-arrow/pkg/datagen"
	"github.com/open-telemetry/otel-arrow/pkg/otel/arrow_record"
)

var help = flag.Bool("help", false, "Show help")
var outputFile = ""
var batchSize = 20
var format = "proto"
var otap = false

func writeJSON(file *os.File, batchsize int, useOtap bool, generator *datagen.LogsGenerator) {
	fw, err := zstd.NewWriter(file)
	if err != nil {
		log.Fatal("error creating compressed writer", err)
	}
	defer fw.Close()

	for i := 0; i < batchSize; i++ {
		var msg []byte

		if useOtap {
			producer := arrow_record.NewProducer()
			bar, err := producer.BatchArrowRecordsFromLogs(generator.Generate(1, 100))
			if err != nil {
				log.Fatal("error creating batch arrow records: ", err)
			}
			msg, err = json.Marshal(bar)
			if err != nil {
				log.Fatal("marshaling error: ", err)
			}
		} else {
			request := plogotlp.NewExportRequestFromLogs(generator.Generate(1, 100))
			var err error
			msg, err = request.MarshalJSON()
			if err != nil {
				log.Fatal("marshaling error: ", err)
			}
		}
		if _, err := fw.Write(msg); err != nil {
			log.Fatal("writing error: ", err)
		}
		if _, err := io.WriteString(fw, "\n"); err != nil {
			log.Fatal("writing newline error: ", err)
		}
	}

	fw.Flush()
}

func writeProto(file *os.File, batchsize int, useOtap bool, generator *datagen.LogsGenerator) {
	logs := generator.Generate(batchSize, 100)
	var msg []byte

	if useOtap {
		producer := arrow_record.NewProducer()
		bar, err := producer.BatchArrowRecordsFromLogs(logs)
		if err != nil {
			log.Fatal("error creating batch arrow records: ", err)
		}
		msg, err = json.Marshal(bar)
		if err != nil {
			log.Fatal("marshaling error: ", err)
		}
	} else {
		request := plogotlp.NewExportRequestFromLogs(generator.Generate(batchSize, 100))
		var err error
		msg, err = request.MarshalProto()
		if err != nil {
			log.Fatal("marshaling error: ", err)
		}
	}

	// Write protobuf to file
	_, err := file.Write(msg)
	if err != nil {
		log.Fatal("write error: ", err)
	}
}

func main() {
	// Define the flags.
	flag.StringVar(&outputFile, "output", outputFile, "Output file")
	flag.IntVar(&batchSize, "batchsize", batchSize, "Batch size")
	flag.StringVar(&format, "format", format, "file format")
	flag.BoolVar(&otap, "otap", otap, "Use OTAP format. If true, generated files will contain OTAP messages. Otherwise, they will contain OTLP messages. Default is false.")

	// Parse the flag
	flag.Parse()

	// Usage Demo
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	// Generate the dataset.
	v, err := rand.Int(rand.Reader, big.NewInt(math.MaxInt64))
	if err != nil {
		log.Fatalf("Failed to generate random number - %v", err)
	}

	entropy := datagen.NewTestEntropy(v.Int64())
	generator := datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	// set default output file name
	if outputFile == "" {
		outputFile = "./data/"
		if otap {
			outputFile += "otap_logs"
		} else {
			outputFile += "otlp_logs"
		}

		if format == "json" {
			outputFile += ".json.zst"
		} else {
			outputFile += ".pb"
		}
	}

	if _, err := os.Stat(outputFile); os.IsNotExist(err) {
		err = os.MkdirAll(path.Dir(outputFile), 0700)
		if err != nil {
			log.Fatal("error creating directory: ", err)
		}
	}
	f, err := os.OpenFile(outputFile, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0600)
	if err != nil {
		log.Fatal("failed to open file: ", err)
	}
	if format == "json" {
		writeJSON(f, batchSize, otap, generator)
	} else { // proto
		writeProto(f, batchSize, otap, generator)
	}

}
