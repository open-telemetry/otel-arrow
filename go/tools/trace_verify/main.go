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
	"bufio"
	"encoding/json"
	"flag"
	"io"
	"log"
	"os"

	"github.com/open-telemetry/otel-arrow/go/pkg/otel/arrow_record"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/assert"

	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

func main() {
	flag.Parse()

	producer := arrow_record.NewProducer()
	consumer := arrow_record.NewConsumer()

	args := flag.Args()

	args = os.Args[1:]

	asserter := assert.NewStandaloneTest()

	for _, file := range args {
		f, err := os.Open(file)
		if err != nil {
			log.Fatalf("open: %s: %v", file, err)
			return
		}
		scanner := bufio.NewReader(f)
		for {
			line, err := scanner.ReadString('\n')
			if err == io.EOF {
				break
			} else if err != nil {
				log.Fatalf("read: %v", err)
			}
			var un ptrace.JSONUnmarshaler

			expected, err := un.UnmarshalTraces([]byte(line))
			if err != nil {
				log.Fatalf("parse: %v", err)
			}

			batch, err := producer.BatchArrowRecordsFromTraces(expected)
			if err != nil {
				log.Fatalf("produce arrow: %v", err)
			}

			received, err := consumer.TracesFrom(batch)
			if err != nil {
				log.Fatalf("consume arrow: %v", err)
			}
			if len(received) != 1 {
				log.Fatalf("expecting 1 traces: %d", len(received))
			}

			assert.Equiv(asserter, []json.Marshaler{
				ptraceotlp.NewExportRequestFromTraces(expected),
			}, []json.Marshaler{
				ptraceotlp.NewExportRequestFromTraces(received[0]),
			})

			log.Printf("Verified %d traces\n", expected.SpanCount())
		}
	}
}
