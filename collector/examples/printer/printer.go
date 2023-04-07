package main

import (
	"bytes"
	"fmt"
	"io"
	"log"
	"net/http"

	metricpb "go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	tracepb "go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/proto"
)

func otlpTraces(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var buf bytes.Buffer
	var tpb tracepb.ExportTraceServiceRequest

	if _, err := io.Copy(&buf, r.Body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if err := proto.Unmarshal(buf.Bytes(), &tpb); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	fmt.Println("Trace Headers:", r.Header)
	fmt.Println("Content:", prototext.MarshalOptions{
		Multiline: true,
	}.Format(&tpb))
}

func otlpMetrics(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var buf bytes.Buffer
	var mpb metricpb.ExportMetricsServiceRequest

	if _, err := io.Copy(&buf, r.Body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if err := proto.Unmarshal(buf.Bytes(), &mpb); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	fmt.Println("Metrics Headers:", r.Header)
	fmt.Println("Content:", prototext.MarshalOptions{
		Multiline: true,
	}.Format(&mpb))
}

func main() {
	http.HandleFunc("/v1/traces", otlpTraces)
	http.HandleFunc("/v1/metrics", otlpMetrics)
	log.Fatal(http.ListenAndServe("127.0.0.1:5001", nil))
}
