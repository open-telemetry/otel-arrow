// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package main

import (
	"bytes"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"

	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

var verbose = flag.Bool("verbose", false, "print more information")

func otlpTraces(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var buf bytes.Buffer

	if _, err := io.Copy(&buf, r.Body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	var unm ptrace.ProtoUnmarshaler

	traces, err := unm.UnmarshalTraces(buf.Bytes())

	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	fmt.Println("Trace Headers:", r.Header)
	fmt.Println("Trace #Spans:", traces.SpanCount())
	if !*verbose {
		return
	}
	var mar ptrace.JSONMarshaler
	data, err := mar.MarshalTraces(traces)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	fmt.Println("Trace Content:", string(data))
}

func otlpMetrics(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var buf bytes.Buffer

	if _, err := io.Copy(&buf, r.Body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	var unm pmetric.ProtoUnmarshaler

	metrics, err := unm.UnmarshalMetrics(buf.Bytes())

	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	fmt.Println("Metrics Headers:", r.Header)
	fmt.Println("Metrics #Points:", metrics.DataPointCount())
	if !*verbose {
		return
	}
	var mar pmetric.JSONMarshaler
	data, err := mar.MarshalMetrics(metrics)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	fmt.Println("Metrics Content:", string(data))
}

func main() {
	http.HandleFunc("/v1/traces", otlpTraces)
	http.HandleFunc("/v1/metrics", otlpMetrics)
	log.Fatal(http.ListenAndServe("127.0.0.1:8101", nil))
}
