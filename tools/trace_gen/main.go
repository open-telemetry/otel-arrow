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
	"crypto/rand"
	"flag"
	"log"
	"math"
	"math/big"
	"os"
	"path"

	"github.com/f5/otel-arrow-adapter/pkg/datagen"

	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

var help = flag.Bool("help", false, "Show help")
var outputFile = "./data/otlp_traces.pb"
var batchSize = 50000

// This tool generates a trace dataset in the OpenTelemetry Protocol format from a fake traces generator.
func main() {
	// Define the flags.
	flag.StringVar(&outputFile, "output", outputFile, "Output file")
	flag.IntVar(&batchSize, "batchsize", batchSize, "Batch size")

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
	generator := datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())
	request := ptraceotlp.NewRequestFromTraces(generator.Generate(batchSize, 100))

	// Marshal the request to bytes.
	msg, err := request.MarshalProto()
	if err != nil {
		log.Fatal("marshaling error: ", err)
	}

	// Write protobuf to file
	if _, err := os.Stat(outputFile); os.IsNotExist(err) {
		err = os.MkdirAll(path.Dir(outputFile), 0700)
		if err != nil {
			log.Fatal("error creating directory: ", err)
		}
	}
	err = os.WriteFile(outputFile, msg, 0600)
	if err != nil {
		log.Fatal("write error: ", err)
	}
}
