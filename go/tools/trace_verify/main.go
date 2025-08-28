/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
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
