package main

import (
	"bytes"
	"fmt"
	"io"
	"log"
	"net/http"

	exportpb "go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/proto"
)

func otlpHandler(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var buf bytes.Buffer
	var pb exportpb.ExportTraceServiceRequest

	if _, err := io.Copy(&buf, r.Body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if err := proto.Unmarshal(buf.Bytes(), &pb); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	str := prototext.MarshalOptions{
		Multiline: true,
	}.Format(&pb)

	fmt.Println("Headers:", r.Header)
	fmt.Println("Content:", str)
}

func main() {
	http.HandleFunc("/v1/traces", otlpHandler)
	log.Fatal(http.ListenAndServe("127.0.0.1:5001", nil))
}
